pub mod config;
pub mod bindings;
pub mod events;

pub use config::{ChainConfig, ConfigError};
pub use bindings::{
    DealCreated, DealState, Funded, IERC20, IEscrowDeal, IEscrowFactory, MilestoneReleased,
};

pub use events::{decode_log, ChainEvent, EventDecodeError};