use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use reqwest::Client;
use serde_json::Value;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchInput {
    /// Search query (keywords, topics, entities)
    pub query: String,
    /// Max results (default 10)
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CountryNewsInput {
    /// Country name or code
    pub country: String,
    /// Optional topic filter
    pub topic: Option<String>,
    /// Max results (default 10)
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TrendingInput {
    /// Country or region (optional, global if empty)
    pub country: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TimelineInput {
    /// Search query
    pub query: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NewsApiInput {
    /// Search query
    pub query: String,
    /// Max results (default 5)
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RssFeedInput {
    /// Region: global, china, india, australia, europe, middle-east, france, germany, uk
    pub region: String,
    /// Max articles (default 10)
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MultiRegionInput {
    /// Regions to fetch from
    pub regions: Vec<String>,
    /// Max articles per region (default 3)
    pub limit: Option<u32>,
}

#[derive(Clone)]
pub struct NewsServer {
    pub client: Client,
    pub newsapi_key: Option<String>,
    pub gnews_key: Option<String>,
}

#[tool_router(server_handler)]
impl NewsServer {
    #[tool(description = "Search global news articles by keyword (free, via GDELT)")]
    async fn search_news(&self, Parameters(input): Parameters<SearchInput>) -> String {
        let limit = input.limit.unwrap_or(10);
        let url = format!(
            "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode=artlist&maxrecords={}&format=json&sort=datedesc",
            input.query.replace(' ', "+"), limit
        );
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => match serde_json::from_str::<Value>(&text) {
                    Ok(data) => format_articles(&data),
                    Err(_) => text,
                },
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get latest news for a specific country (free, via GDELT)")]
    async fn get_country_news(&self, Parameters(input): Parameters<CountryNewsInput>) -> String {
        let limit = input.limit.unwrap_or(10);
        let query = match &input.topic {
            Some(t) => format!("{} {}", input.country, t),
            None => input.country.clone(),
        };
        let url = format!(
            "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode=artlist&maxrecords={}&format=json&sort=datedesc",
            query.replace(' ', "+"), limit
        );
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => match serde_json::from_str::<Value>(&text) {
                    Ok(data) => format_articles(&data),
                    Err(_) => text,
                },
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get trending news tone/themes globally or by country (via GDELT)")]
    async fn get_trending_topics(&self, Parameters(input): Parameters<TrendingInput>) -> String {
        let query = input.country.as_deref().unwrap_or("world");
        let url = format!(
            "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode=tonechart&format=json",
            query.replace(' ', "+")
        );
        fetch_gdelt_raw(&self.client, &url).await
    }

    #[tool(description = "Get news volume timeline for a topic (via GDELT)")]
    async fn get_news_timeline(&self, Parameters(input): Parameters<TimelineInput>) -> String {
        let url = format!(
            "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode=timelinevol&format=json",
            input.query.replace(' ', "+")
        );
        fetch_gdelt_raw(&self.client, &url).await
    }

    #[tool(description = "Get tone/sentiment analysis for news about a topic (via GDELT)")]
    async fn get_news_sentiment(&self, Parameters(input): Parameters<SearchInput>) -> String {
        let url = format!(
            "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode=tonechart&format=json",
            input.query.replace(' ', "+")
        );
        fetch_gdelt_raw(&self.client, &url).await
    }

    #[tool(description = "Search news via NewsAPI (requires NEWSAPI_KEY env var)")]
    async fn newsapi_search(&self, Parameters(input): Parameters<NewsApiInput>) -> String {
        match &self.newsapi_key {
            Some(key) => {
                let limit = input.limit.unwrap_or(5);
                let url = format!(
                    "https://newsapi.org/v2/everything?q={}&pageSize={}&sortBy=publishedAt&apiKey={}",
                    input.query.replace(' ', "+"), limit, key
                );
                match self.client.get(&url).send().await {
                    Ok(resp) => match resp.text().await {
                        Ok(text) => match serde_json::from_str::<Value>(&text) {
                            Ok(data) => {
                                let articles = data["articles"].as_array().unwrap_or(&vec![]).iter().map(|a| {
                                    serde_json::json!({
                                        "title": a["title"],
                                        "source": a["source"]["name"],
                                        "url": a["url"],
                                        "date": a["publishedAt"],
                                        "description": a["description"]
                                    })
                                }).collect::<Vec<_>>();
                                serde_json::to_string_pretty(&articles).unwrap_or_default()
                            }
                            Err(_) => text,
                        },
                        Err(e) => format!("Error: {e}"),
                    },
                    Err(e) => format!("Error: {e}"),
                }
            }
            None => "NewsAPI not configured. Set NEWSAPI_KEY env var.".into(),
        }
    }

    #[tool(description = "Get news from a specific region via RSS (free, no key). Regions: global, china, india, australia, europe, middle-east, france, germany, uk")]
    async fn get_regional_news(&self, Parameters(input): Parameters<RssFeedInput>) -> String {
        let limit = input.limit.unwrap_or(10) as usize;
        let urls = get_rss_urls(&input.region);
        if urls.is_empty() {
            return format!("Unknown region '{}'. Available: global, china, india, australia, europe, middle-east, france, germany, uk", input.region);
        }
        let mut all_articles = Vec::new();
        for url in urls {
            if let Ok(resp) = self.client.get(url).send().await {
                if let Ok(text) = resp.text().await {
                    all_articles.extend(parse_rss_items(&text, limit));
                }
            }
        }
        all_articles.truncate(limit);
        serde_json::to_string_pretty(&all_articles).unwrap_or_default()
    }

    #[tool(description = "Get news from multiple regions at once via RSS (free: BBC, CGTN, NDTV, ABC AU, DW, France24, Al Jazeera)")]
    async fn get_multi_region_news(&self, Parameters(input): Parameters<MultiRegionInput>) -> String {
        let limit = input.limit.unwrap_or(3) as usize;
        let mut result = serde_json::Map::new();
        for region in &input.regions {
            let urls = get_rss_urls(region);
            let mut articles = Vec::new();
            for url in urls {
                if let Ok(resp) = self.client.get(url).send().await {
                    if let Ok(text) = resp.text().await {
                        articles.extend(parse_rss_items(&text, limit));
                    }
                }
            }
            articles.truncate(limit);
            result.insert(region.clone(), serde_json::json!(articles));
        }
        serde_json::to_string_pretty(&serde_json::Value::Object(result)).unwrap_or_default()
    }

    #[tool(description = "Search news via GNews (high-quality sources, 100 req/day free). Requires GNEWS_API_KEY env var")]
    async fn gnews_search(&self, Parameters(input): Parameters<SearchInput>) -> String {
        match &self.gnews_key {
            Some(key) => {
                let limit = input.limit.unwrap_or(5);
                let url = format!(
                    "https://gnews.io/api/v4/search?q={}&lang=en&max={}&apikey={}",
                    input.query.replace(' ', "+"), limit, key
                );
                match self.client.get(&url).send().await {
                    Ok(resp) => match resp.text().await {
                        Ok(text) => match serde_json::from_str::<Value>(&text) {
                            Ok(data) => format_gnews(&data),
                            Err(_) => text,
                        },
                        Err(e) => format!("Error: {e}"),
                    },
                    Err(e) => format!("Error: {e}"),
                }
            }
            None => "GNews not configured. Set GNEWS_API_KEY environment variable.".into(),
        }
    }

    #[tool(description = "Get top headlines by country via GNews (country codes: us, gb, ke, in, au, cn, fr, de, ng, za, ae)")]
    async fn gnews_top_headlines(&self, Parameters(input): Parameters<CountryNewsInput>) -> String {
        match &self.gnews_key {
            Some(key) => {
                let limit = input.limit.unwrap_or(5);
                let country_code = country_to_gnews_code(&input.country);
                let url = format!(
                    "https://gnews.io/api/v4/top-headlines?country={}&max={}&apikey={}",
                    country_code, limit, key
                );
                match self.client.get(&url).send().await {
                    Ok(resp) => match resp.text().await {
                        Ok(text) => match serde_json::from_str::<Value>(&text) {
                            Ok(data) => format_gnews(&data),
                            Err(_) => text,
                        },
                        Err(e) => format!("Error: {e}"),
                    },
                    Err(e) => format!("Error: {e}"),
                }
            }
            None => "GNews not configured. Set GNEWS_API_KEY environment variable.".into(),
        }
    }
}

// --- Helper functions ---

async fn fetch_gdelt_raw(client: &Client, url: &str) -> String {
    match client.get(url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(data) => serde_json::to_string_pretty(&data).unwrap_or_default(),
                Err(_) => text,
            },
            Err(e) => format!("Error: {e}"),
        },
        Err(e) => format!("Error: {e}"),
    }
}

fn format_articles(data: &Value) -> String {
    let articles = data["articles"].as_array().unwrap_or(&vec![]).iter().map(|a| {
        serde_json::json!({
            "title": a["title"],
            "url": a["url"],
            "source": a["domain"],
            "date": a["seendate"],
            "language": a["language"],
            "source_country": a["sourcecountry"]
        })
    }).collect::<Vec<_>>();
    serde_json::to_string_pretty(&articles).unwrap_or_default()
}

fn get_rss_urls(region: &str) -> Vec<&'static str> {
    match region.to_lowercase().as_str() {
        "global" | "world" => vec!["https://feeds.bbci.co.uk/news/world/rss.xml"],
        "china" => vec!["https://www.cgtn.com/subscribe/rss/section/world.xml"],
        "india" => vec!["https://feeds.feedburner.com/ndtvnews-top-stories"],
        "australia" | "au" => vec!["https://www.abc.net.au/news/feed/2942460/rss.xml"],
        "europe" | "eu" => vec!["https://rss.dw.com/rdf/rss-en-all"],
        "france" | "fr" => vec!["https://www.france24.com/en/rss"],
        "germany" | "de" => vec!["https://rss.dw.com/rdf/rss-en-all"],
        "middle-east" | "mideast" => vec!["https://www.aljazeera.com/xml/rss/all.xml"],
        "africa" => vec!["https://www.aljazeera.com/xml/rss/all.xml"],
        "uk" => vec!["https://feeds.bbci.co.uk/news/rss.xml"],
        _ => vec![],
    }
}

fn parse_rss_items(xml: &str, limit: usize) -> Vec<serde_json::Value> {
    let mut items = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<item") {
        let end = match rest[start..].find("</item>") {
            Some(i) => start + i + 7,
            None => break,
        };
        let item_xml = &rest[start..end];
        let title = extract_tag(item_xml, "title").unwrap_or_default();
        let link = extract_tag(item_xml, "link").unwrap_or_default();
        let pub_date = extract_tag(item_xml, "pubDate").unwrap_or_default();
        let description = extract_tag(item_xml, "description")
            .map(|d| d.chars().take(200).collect::<String>())
            .unwrap_or_default();
        if !title.is_empty() {
            items.push(serde_json::json!({
                "title": title,
                "url": link,
                "date": pub_date,
                "description": strip_html(&description)
            }));
        }
        if items.len() >= limit { break; }
        rest = &rest[end..];
    }
    items
}

fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)?;
    let content_start = xml[start..].find('>')? + start + 1;
    let content = &xml[content_start..];
    let end = content.find(&close)?;
    let raw = &content[..end];
    let text = if raw.starts_with("<![CDATA[") {
        raw.trim_start_matches("<![CDATA[").trim_end_matches("]]>")
    } else {
        raw
    };
    Some(text.trim().to_string())
}

fn strip_html(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        if c == '<' { in_tag = true; }
        else if c == '>' { in_tag = false; }
        else if !in_tag { result.push(c); }
    }
    result.trim().to_string()
}

fn format_gnews(data: &Value) -> String {
    let articles = data["articles"].as_array().unwrap_or(&vec![]).iter().map(|a| {
        serde_json::json!({
            "title": a["title"],
            "url": a["url"],
            "source": a["source"]["name"],
            "date": a["publishedAt"],
            "description": a["description"],
            "image": a["image"]
        })
    }).collect::<Vec<_>>();
    serde_json::to_string_pretty(&articles).unwrap_or_default()
}

fn country_to_gnews_code(country: &str) -> String {
    match country.to_lowercase().as_str() {
        "kenya" | "ke" => "ke",
        "nigeria" | "ng" => "ng",
        "south africa" | "za" => "za",
        "india" | "in" => "in",
        "china" | "cn" => "cn",
        "australia" | "au" => "au",
        "united kingdom" | "uk" | "gb" => "gb",
        "united states" | "us" | "usa" => "us",
        "france" | "fr" => "fr",
        "germany" | "de" => "de",
        "uae" | "ae" => "ae",
        "japan" | "jp" => "jp",
        "brazil" | "br" => "br",
        "egypt" | "eg" => "eg",
        other => other,
    }.to_string()
}
