mod config;
mod server;
mod rpc_client;
mod interceptor;
mod decoder;
mod fork_db;
mod simulator;

fn mask_rpc_url(url: &str) -> String {
    if let Some((scheme_host, _)) = url.split_once("/v2/") {
        format!("{}/v2/[REDACTED_KEY]", scheme_host)
    } else if let Some((base, query)) = url.split_once('?') {
        if !query.is_empty() {
            format!("{}?[REDACTED_PARAMS]", base)
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cfg = config::Config::from_env();
    println!("Starting OP Security Proxy...");
    println!("Upstream RPC: {}", mask_rpc_url(&cfg.op_rpc_url));
    
    server::run_server(cfg.proxy_port, cfg.op_rpc_url).await?;
    
    Ok(())
}
