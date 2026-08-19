use alloy::primitives::Address;

pub mod config;

pub use config::{ChainConfig, ConfigError};

const _: () = {
    let _ = Address::ZERO;
};