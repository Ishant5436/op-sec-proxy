mod config;
mod server;
mod rpc_client;
mod interceptor;
mod decoder;
mod fork_db;
mod simulator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cfg = config::Config::from_env();
    println!("Starting OP Security Proxy...");
    println!("Upstream RPC: {}", cfg.op_rpc_url);
    
    server::run_server(cfg.proxy_port, cfg.op_rpc_url).await?;
    
    Ok(())
}
