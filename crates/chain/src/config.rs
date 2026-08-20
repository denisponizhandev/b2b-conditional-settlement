use std::path::{Path, PathBuf};
use std::str::FromStr;

use alloy::primitives::Address;

use serde::Deserialize;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("environment variable {0} is not set")]
    MissingEnv(&'static str),

    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse chain config: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("invalid address for {field}: {value}")]
    InvalidAddress {
        field: &'static str,
        value: String
    }
}

#[derive(Debug, Deserialize)]
struct SepoliaTomlFile {
    chain_id: u64,
    escrow_factory: String,
    mock_token: String
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub rpc_url: String,
    pub escrow_factory: Address,
    pub mock_token: Address
}

impl ChainConfig {
    fn rpc_url_from_env() -> Result<String, ConfigError> {
        std::env::var("SEPOLIA_RPC_URL").map_err(|_| {
            ConfigError::MissingEnv("SEPOLIA_RPC_URL")
        })
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        let _ = dotenvy::dotenv();

        let rpc_url = Self::rpc_url_from_env()?;

        let config_path = std::env::var("CHAIN_CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../configs/chains/sepolia.toml")
            });

        Self::from_toml_file(&config_path, rpc_url)
    }

    pub fn from_toml_file(path: &Path, rpc_url: String) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)?;
        Self::from_toml(&contents, rpc_url)
    }

    pub fn from_toml(contents: &str, rpc_url: String) -> Result<Self, ConfigError> {
        let raw: SepoliaTomlFile = toml::from_str(contents)?;

        let escrow_factory = parse_address("escrow_factory", &raw.escrow_factory)?;
        let mock_token = parse_address("mock_token", &raw.mock_token)?;

        Ok(ChainConfig {
            chain_id: raw.chain_id,
            rpc_url,
            escrow_factory,
            mock_token
        })
    }
}

fn parse_address(field: &'static str, value: &str) -> Result<Address, ConfigError> {
    Address::from_str(value).map_err(|_| ConfigError::InvalidAddress {
        field,
        value: value.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TOML: &str = r#"
        chain_id = 11155111
        escrow_factory = "0xbaFB2BC76af8FFd41BEc1DB5B4EC82a3D088F317"
        mock_token = "0xA93D21E54933Ee2aed032d89c24003f38B22F2f9"
    "#;

    #[test]
    fn from_toml_parses_sepolia_addresses() {
        let cfg = ChainConfig::from_toml(&SAMPLE_TOML, "http://127.0.0.1:8545".into())
            .expect("sample toml should parse");

        assert_eq!(cfg.chain_id, 11155111);
        assert_eq!(cfg.rpc_url, "http://127.0.0.1:8545");

        let factory = Address::from_str("0xbaFB2BC76af8FFd41BEc1DB5B4EC82a3D088F317").unwrap();
        let token = Address::from_str("0xA93D21E54933Ee2aed032d89c24003f38B22F2f9").unwrap();

        assert_eq!(cfg.escrow_factory, factory);
        assert_eq!(cfg.mock_token, token);
    }

    #[test]
    fn from_toml_rejects_invalied_address() {
        let bad: &str = r#"
            chain_id = 11155111
            escrow_factory = "not-the-address"
            mock_token = "0xA93D21E54933Ee2aed032d89c24003f38B22F2f9"
        "#;

        let err = ChainConfig::from_toml(&bad, "http://127.0.0.1:8545".into()).unwrap_err();

        assert!(matches!(
            err,
            ConfigError::InvalidAddress { field, .. } if field == "escrow_factory"
        ));
    }

    #[test]
    fn from_env_requires_rpc_url() {
        let prev = std::env::var("SEPOLIA_RPC_URL").ok();

        unsafe { std::env::remove_var("SEPOLIA_RPC_URL") };

        let err = ChainConfig::rpc_url_from_env().unwrap_err();
        assert!(matches!(err, ConfigError::MissingEnv("SEPOLIA_RPC_URL")));

        if let Some(v) = prev {
            unsafe { std::env::set_var("SEPOLIA_RPC_URL", v) }
        }
    }

    #[test]
    fn from_toml_file_reads_repo_config() {
        let path = Path::new("configs/chains/sepolia.toml");

        if !path.exists() {
            return;
        }

        let cfg = ChainConfig::from_toml_file(path,  "http://127.0.0.1:8545".into())
            .expect("epo sepolia.toml should load");

        assert_eq!(cfg.chain_id, 11155111);
    }
}