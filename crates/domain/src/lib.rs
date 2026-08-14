pub mod deal;
pub mod org;
pub mod errors;
pub mod types;

pub use types::{DealStatus};
pub use deal::{Deal, Milestone};
pub use org::{Organization, WalletRole};
pub use errors::DomainError;