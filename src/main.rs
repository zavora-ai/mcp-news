mod server;

use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manifest = adk_mcp_sdk::ServerManifest::from_file(std::path::Path::new("mcp-server.toml"))?;
    let errors = manifest.validate();
    if !errors.is_empty() {
        for e in &errors { eprintln!("  - {e}"); }
    }

    let newsapi_key = std::env::var("NEWSAPI_KEY").ok();
    let server = server::NewsServer { client: reqwest::Client::new(), newsapi_key };
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
