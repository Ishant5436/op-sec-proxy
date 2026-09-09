use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SuperchainNetwork {
    OpMainnet,
    Base,
    Mode,
    Zora,
    Fraxtal,
    Custom(u64),
}

#[allow(dead_code)]
impl SuperchainNetwork {
    pub fn chain_id(&self) -> u64 {
        match self {
            Self::OpMainnet => 10,
            Self::Base => 8453,
            Self::Mode => 34443,
            Self::Zora => 7777777,
            Self::Fraxtal => 252,
            Self::Custom(id) => *id,
        }
    }

    pub fn default_rpc_url(&self) -> &'static str {
        match self {
            Self::OpMainnet => "https://mainnet.optimism.io",
            Self::Base => "https://mainnet.base.org",
            Self::Mode => "https://mainnet.mode.network",
            Self::Zora => "https://rpc.zora.energy",
            Self::Fraxtal => "https://rpc.frax.com",
            Self::Custom(_) => "https://mainnet.optimism.io",
        }
    }

    pub fn from_chain_id(id: u64) -> Self {
        match id {
            10 => Self::OpMainnet,
            8453 => Self::Base,
            34443 => Self::Mode,
            7777777 => Self::Zora,
            252 => Self::Fraxtal,
            other => Self::Custom(other),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub op_rpc_url: String,
    pub proxy_port: u16,
    #[allow(dead_code)]
    pub network: SuperchainNetwork,
    pub fail_open: bool,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok(); // Load .env file if it exists, ignore if not

        let op_rpc_url =
            env::var("OP_RPC_URL").unwrap_or_else(|_| "https://mainnet.optimism.io".to_string());

        let proxy_port_str = env::var("PROXY_PORT").unwrap_or_else(|_| "3000".to_string());

        let proxy_port = proxy_port_str
            .parse::<u16>()
            .expect("PROXY_PORT must be a valid u16 port number");

        let chain_id_str = env::var("CHAIN_ID").unwrap_or_else(|_| "10".to_string());
        let chain_id = chain_id_str.parse::<u64>().unwrap_or(10);
        let network = SuperchainNetwork::from_chain_id(chain_id);

        let fail_open = env::var("OP_SEC_FAIL_OPEN")
            .map(|v| v != "0" && v.to_lowercase() != "false")
            .unwrap_or(true);

        Self {
            op_rpc_url,
            proxy_port,
            network,
            fail_open,
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
    fn test_superchain_network_resolution() {
        assert_eq!(
            SuperchainNetwork::from_chain_id(10),
            SuperchainNetwork::OpMainnet
        );
        assert_eq!(
            SuperchainNetwork::from_chain_id(8453),
            SuperchainNetwork::Base
        );
        assert_eq!(
            SuperchainNetwork::from_chain_id(34443),
            SuperchainNetwork::Mode
        );
        assert_eq!(
            SuperchainNetwork::from_chain_id(7777777),
            SuperchainNetwork::Zora
        );
        assert_eq!(
            SuperchainNetwork::from_chain_id(252),
            SuperchainNetwork::Fraxtal
        );
        assert_eq!(
            SuperchainNetwork::from_chain_id(999),
            SuperchainNetwork::Custom(999)
        );

        assert_eq!(SuperchainNetwork::OpMainnet.chain_id(), 10);
        assert_eq!(SuperchainNetwork::Base.chain_id(), 8453);
        assert_eq!(
            SuperchainNetwork::OpMainnet.default_rpc_url(),
            "https://mainnet.optimism.io"
        );
        assert_eq!(
            SuperchainNetwork::Base.default_rpc_url(),
            "https://mainnet.base.org"
        );
    }
}
