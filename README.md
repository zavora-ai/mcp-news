# mcp-news

[![Crates.io](https://img.shields.io/crates/v/mcp-news.svg)](https://crates.io/crates/mcp-news)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Global news and market intelligence MCP server — **20 tools** covering breaking news, tech (Hacker News), finance (Yahoo Finance), crypto (CoinGecko), science (ArXiv), cybersecurity (CVE/CISA), sports (BBC), and climate events (NASA). Most sources are free with no API key required.

## Installation

```bash
cargo install mcp-news
```

## Quick Start

```bash
# Free sources only (GDELT, HN, RSS, ArXiv, CVE, Sports, NASA, CoinGecko)
mcp-news

# With premium sources
GNEWS_API_KEY=your_key NEWSAPI_KEY=your_key mcp-news
```

## Sources (11)

| Source | Category | Free? | Key needed? |
|--------|----------|:-----:|:-----------:|
| **GDELT** | Global news, sentiment, timelines | ✅ | No |
| **RSS Feeds** | BBC, CGTN, NDTV, ABC AU, DW, France24, Al Jazeera | ✅ | No |
| **Hacker News** | Tech, startups, programming | ✅ | No |
| **Yahoo Finance** | Stock/crypto price charts | ✅ | No |
| **CoinGecko** | Crypto prices, trending coins | ✅ | No |
| **ArXiv** | Scientific papers (AI, ML, physics) | ✅ | No |
| **NIST NVD** | CVE vulnerability database | ✅ | No |
| **CISA KEV** | Actively exploited vulnerabilities | ✅ | No |
| **BBC Sport** | Football, cricket, tennis, F1, golf | ✅ | No |
| **NASA EONET** | Wildfires, storms, volcanoes | ✅ | No |
| **GNews** | Quality headlines by country | ✅ | Yes (free tier) |
| **NewsAPI** | Premium global sources | ❌ | Yes |

## Tools (20)

### General News (5)
| Tool | Description |
|------|-------------|
| `search_news` | Search articles by keyword globally (GDELT) |
| `get_country_news` | Latest news for a specific country |
| `get_trending_topics` | Trending themes/tone globally or by country |
| `get_news_timeline` | News volume over time for a topic |
| `get_news_sentiment` | Tone/sentiment analysis for a topic |

### Regional News (2)
| Tool | Description |
|------|-------------|
| `get_regional_news` | News from a region: china, india, australia, europe, middle-east, uk, france, germany |
| `get_multi_region_news` | Multiple regions at once |

### Tech (2)
| Tool | Description |
|------|-------------|
| `hn_stories` | Hacker News top/new/best/ask/show/job stories |
| `hn_search` | Full-text search across all HN history (Algolia) |

### Finance & Crypto (3)
| Tool | Description |
|------|-------------|
| `yfinance_chart` | Stock/crypto price chart (AAPL, MSFT, BTC-USD, ETH-USD) |
| `crypto_prices` | Live crypto prices with 24h change and market cap |
| `crypto_trending` | Trending cryptocurrencies |

### Science (1)
| Tool | Description |
|------|-------------|
| `arxiv_search` | Search scientific papers (cs.AI, cs.CL, cs.LG, physics, math) |

### Cybersecurity (2)
| Tool | Description |
|------|-------------|
| `cve_search` | Search CVE vulnerabilities by keyword |
| `cisa_exploited_vulns` | Latest actively exploited vulnerabilities (CISA KEV) |

### Sports (1)
| Tool | Description |
|------|-------------|
| `sports_news` | BBC Sport: football, cricket, tennis, formula1, golf |

### Climate & Events (1)
| Tool | Description |
|------|-------------|
| `nasa_events` | Active natural events (wildfires, storms, volcanoes) |

### Premium (3)
| Tool | Description |
|------|-------------|
| `gnews_search` | Search via GNews (requires GNEWS_API_KEY) |
| `gnews_top_headlines` | Headlines by country via GNews |
| `newsapi_search` | Search via NewsAPI (requires NEWSAPI_KEY) |

## MCP Client Configuration

### Claude Desktop / Cursor

```json
{
  "mcpServers": {
    "news": {
      "command": "mcp-news",
      "env": {
        "GNEWS_API_KEY": "optional",
        "NEWSAPI_KEY": "optional"
      }
    }
  }
}
```

## Usage Examples

### "What's happening in Kenya today?"
```
→ get_country_news(country="Kenya", limit=5)
→ gnews_top_headlines(country="Kenya", limit=5)
```

### "Show me the top Hacker News stories"
```
→ hn_stories(story_type="top", limit=10)
```

### "What's Bitcoin at?"
```
→ crypto_prices(coins="bitcoin,ethereum,solana")

BTC: $75,850 (-2.15%), MCap $1.52T
ETH: $2,067 (-2.87%), MCap $249B
SOL: $...
```

### "Any critical vulnerabilities this week?"
```
→ cisa_exploited_vulns()
→ cve_search(query="remote code execution", limit=5)
```

### "Latest AI research papers"
```
→ arxiv_search(query="large language model", category="cs.CL", limit=5)
```

### "Football transfer news"
```
→ sports_news(sport="football", limit=10)
```

### "News from China, India, and Europe"
```
→ get_multi_region_news(regions=["china", "india", "europe"], limit=3)
```

### "Apple stock this month"
```
→ yfinance_chart(symbol="AAPL", range="1mo")
```

## Environment Variables

| Variable | Required | Description |
|----------|:--------:|-------------|
| `GNEWS_API_KEY` | No | GNews API key (free: 100 req/day at gnews.io) |
| `NEWSAPI_KEY` | No | NewsAPI key (newsapi.org) |

## Rate Limits

| Source | Limit | Notes |
|--------|-------|-------|
| GDELT | 1 req/5s | Free, no key |
| GNews | 100 req/day | Free tier |
| NewsAPI | 100 req/day | Free tier |
| CoinGecko | 10-30 req/min | Free, needs User-Agent |
| ArXiv | 1 req/3s | Free |
| NVD/CISA | Unlimited | Free |
| HN/Algolia | Generous | Free |
| Yahoo Finance | Generous | Free |
| RSS feeds | Unlimited | Free |
| NASA EONET | Unlimited | Free |

## License

Apache-2.0
