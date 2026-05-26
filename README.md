# mcp-news

[![Crates.io](https://img.shields.io/crates/v/mcp-news.svg)](https://crates.io/crates/mcp-news)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Global news intelligence MCP server — real-time articles, country news, trending topics, sentiment analysis, and volume timelines. **6 tools** powered by GDELT (free, no API key) with optional NewsAPI for broader sources.

## Quick Start

```bash
cargo install mcp-news
mcp-news  # Free GDELT access, no key needed

# Optional: NewsAPI for premium sources
NEWSAPI_KEY=your_key mcp-news
```

## Tools (6)

| Tool | Description | Source |
|------|-------------|--------|
| `search_news` | Search articles by keyword/topic/entity | GDELT (free) |
| `get_country_news` | Latest news for a specific country | GDELT (free) |
| `get_trending_topics` | Trending themes globally or by country | GDELT (free) |
| `get_news_timeline` | News volume over time for a topic | GDELT (free) |
| `get_news_sentiment` | Tone/sentiment analysis for a topic | GDELT (free) |
| `newsapi_search` | Search via NewsAPI (broader sources) | NewsAPI (key) |

## Configuration

```json
{
  "mcpServers": {
    "news": {
      "command": "mcp-news",
      "env": { "NEWSAPI_KEY": "optional_key" }
    }
  }
}
```

## License

Apache-2.0
