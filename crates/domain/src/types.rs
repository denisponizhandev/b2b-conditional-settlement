#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DealStatus {
    Draft,
    Funded,
    InProgress,
    Released,
    Disputed,
    PendingChainConfirm,
    Closed
}

impl DealStatus {
    pub fn can_approve_release(self) -> bool {
        matches!(self, DealStatus::Funded | DealStatus::InProgress)
    }
}

