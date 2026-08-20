pub mod config;
pub mod bindings;
pub mod events;
pub mod client;

pub use config::{ChainConfig, ConfigError};
pub use bindings::{
    DealCreated, DealState, Funded, IERC20, IEscrowDeal, IEscrowFactory, MilestoneReleased,
};

pub use events::{decode_log, ChainEvent, EventDecodeError};
pub use client::{ChainClient, ClientError};