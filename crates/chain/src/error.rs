use alloy::transports::TransportError;

use thiserror::Error;

use crate::config::ConfigError;
use crate::events::EventDecodeError;

#[derive(Debug, Error)]
pub enum ChainError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error("RPC call failed: {0}")]
    Rpc(String),

    #[error("RPC transport error: {0}")]
    Transport(#[from] TransportError),

    #[error("chain id mismatch: config {expected}, node returned {actual}")]
    ChainIdMismatch { expected: u64, actual: u64 },

    #[error(transparent)]
    Decode(#[from] EventDecodeError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    use alloy::primitives::Address;

    use crate::config::ChainConfig;

    const SAMPLE_TOML: &str = r#"
        chain_id = 11155111
        escrow_factory = "not-an-address"
        mock_token = "0xA93D21E54933Ee2aed032d89c24003f38B22F2f9"
    "#;

    #[test]
    fn config_error_converts_to_chain_error() {
        let err: ChainError = ChainConfig::from_toml(SAMPLE_TOML, "http://127.0.0.1:8545".into())
            .unwrap_err()
            .into();

        assert!(matches!(err, ChainError::Config(ConfigError::InvalidAddress { .. })));
    }

    #[test]
    fn chain_id_mismatch_is_chain_error() {
        let err = ChainError::ChainIdMismatch {
            expected: 11155111,
            actual: 1,
        };

        assert!(err.to_string().contains("11155111"));
    }
}