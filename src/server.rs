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
    /// Country name or code (e.g. "Kenya", "United States")
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
    /// Resolution: day, week, month (default day)
    pub resolution: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NewsApiInput {
    /// Search query
    pub query: String,
    /// Max results (default 5)
    pub limit: Option<u32>,
    /// Sort by: relevancy, popularity, publishedAt
    pub sort_by: Option<String>,
}

#[derive(Clone)]
pub struct NewsServer {
    pub client: Client,
    pub newsapi_key: Option<String>,
}

#[tool_router(server_handler)]
impl NewsServer {
    #[tool(description = "Search global news articles by keyword, topic, or entity (free, via GDELT)")]
    async fn search_news(&self, Parameters(input): Parameters<SearchInput>) -> String {
        let limit = input.limit.unwrap_or(10);
        let url = format!(
            "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode=artlist&maxrecords={}&format=json&sort=datedesc",
            input.query.replace(' ', "+"), limit
        );
        match self.client.get(&url).send().await.and_then(|r| Ok(r)) {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => format_articles(&data),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get latest news for a specific country")]
    async fn get_country_news(&self, Parameters(input): Parameters<CountryNewsInput>) -> String {
        let limit = input.limit.unwrap_or(10);
        let topic = input.topic.as_deref().unwrap_or("");
        let query = if topic.is_empty() {
            input.country.clone()
        } else {
            format!("{} {}", input.country, topic)
        };
        let url = format!(
            "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode=artlist&maxrecords={}&format=json&sort=datedesc",
            query.replace(' ', "+"), limit
        );
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => format_articles(&data),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get trending news themes and topics globally or by country (via GDELT)")]
    async fn get_trending_topics(&self, Parameters(input): Parameters<TrendingInput>) -> String {
        let query = input.country.as_deref().unwrap_or("world");
        let url = format!(
            "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode=tonechart&format=json",
            query.replace(' ', "+")
        );
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => serde_json::to_string_pretty(&data).unwrap_or_default(),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get news volume timeline for a topic (how much coverage over time)")]
    async fn get_news_timeline(&self, Parameters(input): Parameters<TimelineInput>) -> String {
        let res = input.resolution.as_deref().unwrap_or("day");
        let mode = match res {
            "week" => "timelinevol",
            "month" => "timelinevol",
            _ => "timelinevol",
        };
        let url = format!(
            "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode={}&format=json&TIMERES={}",
            input.query.replace(' ', "+"), mode, res
        );
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => serde_json::to_string_pretty(&data).unwrap_or_default(),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get tone/sentiment analysis for news about a topic (via GDELT)")]
    async fn get_news_sentiment(&self, Parameters(input): Parameters<SearchInput>) -> String {
        let url = format!(
            "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode=tonechart&format=json",
            input.query.replace(' ', "+")
        );
        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => serde_json::to_string_pretty(&data).unwrap_or_default(),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Search news via NewsAPI (requires NEWSAPI_KEY, broader sources)")]
    async fn newsapi_search(&self, Parameters(input): Parameters<NewsApiInput>) -> String {
        match &self.newsapi_key {
            Some(key) => {
                let limit = input.limit.unwrap_or(5);
                let sort = input.sort_by.as_deref().unwrap_or("publishedAt");
                let url = format!(
                    "https://newsapi.org/v2/everything?q={}&pageSize={}&sortBy={}&apiKey={}",
                    input.query.replace(' ', "+"), limit, sort, key
                );
                match self.client.get(&url).send().await {
                    Ok(resp) => match resp.json::<Value>().await {
                        Ok(data) => {
                            let articles = data["articles"].as_array().unwrap_or(&vec![]).iter().map(|a| {
                                serde_json::json!({
                                    "title": a["title"],
                                    "source": a["source"]["name"],
                                    "url": a["url"],
                                    "published_at": a["publishedAt"],
                                    "description": a["description"]
                                })
                            }).collect::<Vec<_>>();
                            serde_json::to_string_pretty(&articles).unwrap_or_default()
                        }
                        Err(e) => format!("Error: {e}"),
                    },
                    Err(e) => format!("Error: {e}"),
                }
            }
            None => "NewsAPI not configured. Set NEWSAPI_KEY environment variable. Use search_news for free GDELT access.".into(),
        }
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
