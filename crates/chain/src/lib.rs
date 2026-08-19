pub mod config;
pub mod bindings;

pub use config::{ChainConfig, ConfigError};
pub use bindings::{
    DealCreated, DealState, Funded, IERC20, IEscrowDeal, IEscrowFactory, MilestoneReleased,
};