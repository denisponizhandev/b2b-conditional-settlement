use thiserror::Error;
use crate::types::{DealStatus};
yt
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {

    #[error("milestone {index} not found")]
    MilestoneNotFound { index: u16 },

    //EscrowDeal: sender should be payer address
    //EscrowDeal: funding is only possible in the Draft state"
    //EscrowDeal: amount should be equal to the total milestones amount

    #[error("release not allowed in status {0:?}")]
    InvalidStatus(DealStatus),

    #[error("milestone {index} already released")]
    MilestoneAlreadyReleased { index: u16 },

    #[error("previous milestone {index} is not released")]
    PreviousMilestoneNotReleased { index: u16 }

    //EscrowDeal: not enough token balance
}