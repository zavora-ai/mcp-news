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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HnInput {
    /// Type: top, new, best, ask, show, job
    pub story_type: Option<String>,
    /// Max stories (default 10)
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HnSearchInput {
    /// Search query
    pub query: String,
    /// Max results (default 10)
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TickerInput {
    /// Stock/crypto ticker symbol (e.g. AAPL, MSFT, BTC-USD, ETH-USD)
    pub symbol: String,
    /// Range: 1d, 5d, 1mo, 3mo, 6mo, 1y, 5y (default 5d)
    pub range: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CryptoInput {
    /// Coin IDs comma-separated (e.g. "bitcoin,ethereum,solana")
    pub coins: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ArxivInput {
    /// Search query (e.g. "large language model", "transformer")
    pub query: String,
    /// Category filter: cs.AI, cs.CL, cs.LG, cs.CV, stat.ML (optional)
    pub category: Option<String>,
    /// Max results (default 5)
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CveInput {
    /// Keyword search (e.g. "remote code execution", "apache")
    pub query: String,
    /// Max results (default 5)
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SportInput {
    /// Sport: football, cricket, tennis, formula1, golf (default: all)
    pub sport: Option<String>,
    /// Max articles (default 10)
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EmptyInput {}

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

    #[tool(description = "Get Hacker News top/new/best stories (free, no key). Types: top, new, best, ask, show, job")]
    async fn hn_stories(&self, Parameters(input): Parameters<HnInput>) -> String {
        let story_type = input.story_type.as_deref().unwrap_or("top");
        let limit = input.limit.unwrap_or(10) as usize;
        let endpoint = match story_type {
            "new" => "newstories",
            "best" => "beststories",
            "ask" => "askstories",
            "show" => "showstories",
            "job" => "jobstories",
            _ => "topstories",
        };
        let url = format!("https://hacker-news.firebaseio.com/v0/{}.json", endpoint);
        let ids: Vec<u64> = match self.client.get(&url).send().await {
            Ok(resp) => resp.json().await.unwrap_or_default(),
            Err(e) => return format!("Error: {e}"),
        };
        let mut stories = Vec::new();
        for id in ids.iter().take(limit) {
            let item_url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
            if let Ok(resp) = self.client.get(&item_url).send().await {
                if let Ok(item) = resp.json::<Value>().await {
                    stories.push(serde_json::json!({
                        "title": item["title"],
                        "url": item["url"],
                        "score": item["score"],
                        "comments": item["descendants"],
                        "by": item["by"],
                        "id": id,
                        "hn_url": format!("https://news.ycombinator.com/item?id={}", id)
                    }));
                }
            }
        }
        serde_json::to_string_pretty(&stories).unwrap_or_default()
    }

    #[tool(description = "Search Hacker News articles by keyword (free, via Algolia HN Search API)")]
    async fn hn_search(&self, Parameters(input): Parameters<HnSearchInput>) -> String {
        let limit = input.limit.unwrap_or(10);
        let url = format!(
            "https://hn.algolia.com/api/v1/search?query={}&tags=story&hitsPerPage={}",
            input.query.replace(' ', "+"), limit
        );
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    let hits = data["hits"].as_array().unwrap_or(&vec![]).iter().map(|h| {
                        serde_json::json!({
                            "title": h["title"],
                            "url": h["url"],
                            "score": h["points"],
                            "comments": h["num_comments"],
                            "by": h["author"],
                            "date": h["created_at"],
                            "hn_url": format!("https://news.ycombinator.com/item?id={}", h["objectID"].as_str().unwrap_or(""))
                        })
                    }).collect::<Vec<_>>();
                    serde_json::to_string_pretty(&hits).unwrap_or_default()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get stock/crypto price chart from Yahoo Finance (free, no key). Symbols: AAPL, MSFT, GOOGL, BTC-USD, ETH-USD")]
    async fn yfinance_chart(&self, Parameters(input): Parameters<TickerInput>) -> String {
        let range = input.range.as_deref().unwrap_or("5d");
        let url = format!(
            "https://query2.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range={}",
            input.symbol, range
        );
        match self.client.get(&url).header("User-Agent", "mcp-news/1.0").send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    let result = &data["chart"]["result"][0];
                    let meta = &result["meta"];
                    let timestamps = result["timestamp"].as_array();
                    let closes = result["indicators"]["quote"][0]["close"].as_array();
                    let mut chart_data = serde_json::json!({
                        "symbol": meta["symbol"],
                        "price": meta["regularMarketPrice"],
                        "previous_close": meta["chartPreviousClose"],
                        "currency": meta["currency"],
                        "exchange": meta["exchangeName"],
                        "range": range
                    });
                    if let (Some(ts), Some(cl)) = (timestamps, closes) {
                        let points: Vec<Value> = ts.iter().zip(cl.iter()).map(|(t, c)| {
                            serde_json::json!({"timestamp": t, "close": c})
                        }).collect();
                        chart_data["data_points"] = serde_json::json!(points.len());
                        chart_data["prices"] = serde_json::json!(points);
                    }
                    serde_json::to_string_pretty(&chart_data).unwrap_or_default()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get crypto prices and 24h changes from CoinGecko (free, no key). Coins: bitcoin, ethereum, solana, cardano, etc.")]
    async fn crypto_prices(&self, Parameters(input): Parameters<CryptoInput>) -> String {
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true&include_market_cap=true&include_24hr_vol=true",
            input.coins
        );
        match self.client.get(&url).header("User-Agent", "mcp-news/1.0").send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => serde_json::to_string_pretty(&data).unwrap_or_default(),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get trending cryptocurrencies from CoinGecko (free, no key)")]
    async fn crypto_trending(&self, Parameters(_input): Parameters<SearchInput>) -> String {
        let url = "https://api.coingecko.com/api/v3/search/trending";
        match self.client.get(url).header("User-Agent", "mcp-news/1.0").send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    let coins = data["coins"].as_array().unwrap_or(&vec![]).iter().map(|c| {
                        let item = &c["item"];
                        serde_json::json!({
                            "name": item["name"],
                            "symbol": item["symbol"],
                            "market_cap_rank": item["market_cap_rank"],
                            "price_btc": item["price_btc"],
                            "score": item["score"]
                        })
                    }).collect::<Vec<_>>();
                    serde_json::to_string_pretty(&coins).unwrap_or_default()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Search ArXiv for scientific papers (AI, ML, NLP, CV, physics, math). Free, no key")]
    async fn arxiv_search(&self, Parameters(input): Parameters<ArxivInput>) -> String {
        let limit = input.limit.unwrap_or(5);
        let query = match &input.category {
            Some(cat) => format!("cat:{}+AND+all:{}", cat, input.query.replace(' ', "+")),
            None => format!("all:{}", input.query.replace(' ', "+")),
        };
        let url = format!(
            "https://export.arxiv.org/api/query?search_query={}&max_results={}&sortBy=submittedDate&sortOrder=descending",
            query, limit
        );
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(xml) => parse_arxiv(&xml),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Search CVE vulnerabilities from NIST NVD (free, no key). Find security issues by keyword")]
    async fn cve_search(&self, Parameters(input): Parameters<CveInput>) -> String {
        let limit = input.limit.unwrap_or(5);
        let url = format!(
            "https://services.nvd.nist.gov/rest/json/cves/2.0?resultsPerPage={}&keywordSearch={}",
            limit, input.query.replace(' ', "+")
        );
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    let vulns = data["vulnerabilities"].as_array().unwrap_or(&vec![]).iter().map(|v| {
                        let cve = &v["cve"];
                        let metrics = &cve["metrics"];
                        let score = metrics["cvssMetricV31"].as_array()
                            .and_then(|a| a.first())
                            .and_then(|m| m["cvssData"]["baseScore"].as_f64())
                            .or_else(|| metrics["cvssMetricV2"].as_array()
                                .and_then(|a| a.first())
                                .and_then(|m| m["cvssData"]["baseScore"].as_f64()));
                        serde_json::json!({
                            "id": cve["id"],
                            "description": cve["descriptions"][0]["value"],
                            "severity": score,
                            "published": cve["published"],
                            "modified": cve["lastModified"]
                        })
                    }).collect::<Vec<_>>();
                    serde_json::to_string_pretty(&vulns).unwrap_or_default()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get CISA Known Exploited Vulnerabilities (actively exploited in the wild). Free, no key")]
    async fn cisa_exploited_vulns(&self, Parameters(_input): Parameters<EmptyInput>) -> String {
        let url = "https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json";
        match self.client.get(url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    let vulns = data["vulnerabilities"].as_array().cloned().unwrap_or_default();
                    let recent: Vec<Value> = vulns.iter().rev().take(10).map(|v| {
                        serde_json::json!({
                            "cve_id": v["cveID"],
                            "vendor": v["vendorProject"],
                            "product": v["product"],
                            "name": v["vulnerabilityName"],
                            "date_added": v["dateAdded"],
                            "due_date": v["dueDate"],
                            "action": v["requiredAction"]
                        })
                    }).collect();
                    serde_json::to_string_pretty(&recent).unwrap_or_default()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get latest sports news from BBC Sport (free RSS). Sports: football, cricket, tennis, formula1, golf")]
    async fn sports_news(&self, Parameters(input): Parameters<SportInput>) -> String {
        let limit = input.limit.unwrap_or(10) as usize;
        let feed = match input.sport.as_deref() {
            Some("football") | Some("soccer") => "https://feeds.bbci.co.uk/sport/football/rss.xml",
            Some("cricket") => "https://feeds.bbci.co.uk/sport/cricket/rss.xml",
            Some("tennis") => "https://feeds.bbci.co.uk/sport/tennis/rss.xml",
            Some("formula1") | Some("f1") => "https://feeds.bbci.co.uk/sport/formula1/rss.xml",
            Some("golf") => "https://feeds.bbci.co.uk/sport/golf/rss.xml",
            _ => "https://feeds.bbci.co.uk/sport/rss.xml",
        };
        match self.client.get(feed).send().await {
            Ok(resp) => match resp.text().await {
                Ok(xml) => serde_json::to_string_pretty(&parse_rss_items(&xml, limit)).unwrap_or_default(),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get active natural events from NASA EONET (wildfires, storms, volcanoes, earthquakes). Free, no key")]
    async fn nasa_events(&self, Parameters(_input): Parameters<EmptyInput>) -> String {
        let url = "https://eonet.gsfc.nasa.gov/api/v3/events?limit=10&status=open";
        match self.client.get(url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    let events = data["events"].as_array().unwrap_or(&vec![]).iter().map(|e| {
                        serde_json::json!({
                            "title": e["title"],
                            "category": e["categories"][0]["title"],
                            "date": e["geometry"].as_array().and_then(|a| a.last()).and_then(|g| g["date"].as_str()),
                            "coordinates": e["geometry"].as_array().and_then(|a| a.last()).and_then(|g| g["coordinates"].as_array()),
                            "source": e["sources"][0]["url"]
                        })
                    }).collect::<Vec<_>>();
                    serde_json::to_string_pretty(&events).unwrap_or_default()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
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

fn parse_arxiv(xml: &str) -> String {
    let mut papers = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<entry>") {
        let end = match rest[start..].find("</entry>") {
            Some(i) => start + i + 8,
            None => break,
        };
        let entry = &rest[start..end];
        let title = extract_tag(entry, "title").unwrap_or_default().replace('\n', " ");
        let id = extract_tag(entry, "id").unwrap_or_default();
        let summary = extract_tag(entry, "summary").unwrap_or_default().replace('\n', " ");
        let published = extract_tag(entry, "published").unwrap_or_default();
        if !title.is_empty() {
            papers.push(serde_json::json!({
                "title": title.trim(),
                "url": id,
                "abstract": summary.trim().chars().take(300).collect::<String>(),
                "published": published
            }));
        }
        rest = &rest[end..];
    }
    if papers.is_empty() && xml.contains("Rate exceeded") {
        return "ArXiv rate limited. Please wait 3 seconds between requests.".into();
    }
    serde_json::to_string_pretty(&papers).unwrap_or_default()
}
