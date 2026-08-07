use dotenvy::dotenv;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub op_rpc_url: String,
    pub proxy_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok(); // Load .env file if it exists, ignore if not

        let op_rpc_url = env::var("OP_RPC_URL")
            .expect("OP_RPC_URL environment variable must be set");

        let proxy_port_str = env::var("PROXY_PORT")
            .unwrap_or_else(|_| "3000".to_string());
        
        let proxy_port = proxy_port_str
            .parse::<u16>()
            .expect("PROXY_PORT must be a valid u16 port number");

        Self {
            op_rpc_url,
            proxy_port,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_parsing_success() {
        unsafe {
            env::set_var("OP_RPC_URL", "http://localhost:8545");
            env::set_var("PROXY_PORT", "8080");
        }

        let config = Config::from_env();
        assert_eq!(config.op_rpc_url, "http://localhost:8545");
        assert_eq!(config.proxy_port, 8080);
        
        // Clean up
        unsafe {
            env::remove_var("OP_RPC_URL");
            env::remove_var("PROXY_PORT");
        }
    }

    #[test]
    #[ignore] // This test is unreliable when .env file is present (dotenv reloads the var).
    #[should_panic(expected = "OP_RPC_URL environment variable must be set")]
    fn test_config_missing_rpc_url_panics() {
        // Ensure it's not set
        unsafe {
            env::remove_var("OP_RPC_URL");
        }
        Config::from_env();
    }
}
