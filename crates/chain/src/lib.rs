pub mod bindings;
pub mod client;
pub mod config;
pub mod error;
pub mod events;

pub use bindings::{
    DealCreated, DealState, Funded, IERC20, IEscrowDeal, IEscrowFactory, MilestoneReleased,
};
pub use client::ChainClient;
pub use config::ChainConfig;
pub use error::ChainError;
pub use events::{decode_log, ChainEvent};

pub use config::ConfigError;
pub use events::EventDecodeError;