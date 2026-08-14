use uuid::{Uuid};
use crate::errors::{DomainError};
use crate::types::{DealStatus};

#[derive(Debug, Clone)]
pub struct Milestone {
    index: u16,
    amount: u64,
    released: bool
}

impl Milestone {
    pub fn new(index: u16, amount: u64, released: bool) -> Self {
        Milestone {
            index,
            amount,
            released
        }
    }

    pub fn index(&self) -> u16 { self.index }
    pub fn amount(&self) -> u64 { self.amount }
    pub fn is_released(&self) -> bool { self.released }

    pub fn try_mark_released(&mut self) -> Result<(), DomainError> {
        match self.released {
            true => Err(DomainError::MilestoneAlreadyReleased { index: self.index }),
            false => {
                self.released = true;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Deal {
    id: Uuid,
    status: DealStatus,
    milestones: Vec<Milestone>,
    payer_org_id: Uuid,
    payee_org_id: Uuid,
    chain_address: Option<String>
}

impl Deal {
    pub fn new(
        id: Uuid, 
        status: DealStatus, 
        milestones: Vec<Milestone>,
        payer_org_id: Uuid,
        payee_org_id: Uuid,
        chain_address: Option<String>
    ) -> Self {
        Deal {
            id,
            status,
            milestones,
            payer_org_id,
            payee_org_id,
            chain_address
        }
    }

    pub fn id(&self) -> Uuid { self.id }
    pub fn status(&self) -> DealStatus { self.status }
    pub fn milestones(&self) -> &[Milestone] { &self.milestones }
    pub fn payer_org_id(&self) -> Uuid { self.payer_org_id }
    pub fn payee_org_id(&self) -> Uuid { self.payee_org_id }
    pub fn chain_address(&self) -> &Option<String> { &self.chain_address }

    pub fn ensure_can_approve_release(&self, index: u16) -> Result<(), DomainError> {
        if !self.status.can_approve_release() {
            return Err(DomainError::InvalidStatus(self.status));
        }

        let milestone = self.milestones.iter()
            .find(|m| m.index() == index)
            .ok_or(DomainError::MilestoneNotFound { index })?;

        if milestone.is_released() {
            return Err(DomainError::MilestoneAlreadyReleased { index });
        }

        let blocker = self.milestones.iter()
            .filter(|m| m.index() < index)
            .find(|m| !m.is_released());

        if let Some(m) = blocker {
            return Err(DomainError::PreviousMilestoneNotReleased { index: m.index() });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_milestone_created() {
        let index: u16 = 0;
        let amount: u64 = 100000;
        let released: bool = false;

        let milestone: Milestone = Milestone::new(index, amount, released);

        assert_eq!(milestone.index(), index);
        assert_eq!(milestone.amount(), amount);
        assert_eq!(milestone.is_released(), released);
    }

    #[test]
    fn milestone_releases_when_not_released() {
        let index: u16 = 0;
        let amount: u64 = 100000;
        let released: bool = false;

        let mut milestone: Milestone = Milestone::new(index, amount, released);
        let res = milestone.try_mark_released();
        
        assert!(res.is_ok());
        assert_eq!(milestone.is_released(), true);
    }

    #[test]
    fn milestone_fails_when_already_released() {
        let index: u16 = 0;
        let amount: u64 = 100000;
        let released: bool = true;

        let mut milestone: Milestone = Milestone::new(index, amount, released);
        let res = milestone.try_mark_released();

        assert_eq!(res, Err(DomainError::MilestoneAlreadyReleased { index: 0 }));
        assert_eq!(milestone.is_released(), true);
    }

    #[test]
    fn funded_deal_allows_release_approval() {
        assert!(DealStatus::Funded.can_approve_release());
        assert!(!DealStatus::Draft.can_approve_release());
    }

    fn test_deal(status: DealStatus, milestones: Vec<Milestone>) -> Deal {
        let deal_id = Uuid::new_v4();
        let payer_id = Uuid::new_v4();
        let payee_id = Uuid::new_v4();

        Deal::new(
            deal_id, 
            status, 
            milestones,
            payer_id,
            payee_id,
            Some(String::from("chain_addr"))
        )
    }

    #[test]
    fn funded_deal_approves_release() {     
        let status = DealStatus::Funded;
        let milestones = vec![
            Milestone::new(0, 10, false),
            Milestone::new(1, 20, false),
            Milestone::new(2, 30, false)
        ];

        let deal = test_deal(status, milestones);
        let res = deal.ensure_can_approve_release(0);

        assert!(res.is_ok());
    }

    #[test]
    fn in_progress_deal_approves_release() {     
        let status = DealStatus::InProgress;
        let milestones = vec![
            Milestone::new(0, 10, false),
            Milestone::new(1, 20, false),
            Milestone::new(2, 30, false)
        ];

        let deal = test_deal(status, milestones);
        let res = deal.ensure_can_approve_release(0);

        assert!(res.is_ok());
    }

    #[test]
    fn draft_deal_does_not_approve_release() {
        let status = DealStatus::Draft;

        let milestones = vec![
            Milestone::new(0, 10, false),
            Milestone::new(1, 20, false),
            Milestone::new(2, 30, false)
        ];

        let deal = test_deal(status, milestones);
        let res = deal.ensure_can_approve_release(0);

        assert_eq!(res, Err(DomainError::InvalidStatus(DealStatus::Draft)));
    }

    #[test]
    fn released_milestone_does_not_approve_release() {
        let status = DealStatus::Funded;

        let milestones = vec![
            Milestone::new(0, 10, true),
            Milestone::new(1, 20, false),
            Milestone::new(2, 30, false)
        ];

        let deal = test_deal(status, milestones);
        let res = deal.ensure_can_approve_release(0);

        assert_eq!(res, Err(DomainError::MilestoneAlreadyReleased {index: 0} ));
    }

    #[test]
    fn absent_milestone_does_not_approve_release() {
        let status = DealStatus::Funded;

        let milestones = vec![
            Milestone::new(0, 10, false),
            Milestone::new(1, 20, false),
            Milestone::new(2, 30, false)
        ];

        let deal = test_deal(status, milestones);
        let res = deal.ensure_can_approve_release(4);

        assert_eq!(res, Err(DomainError::MilestoneNotFound {index: 4} ));
    }

    #[test]
    fn prev_not_released_milestone_does_not_approve_release() {
        let status = DealStatus::Funded;

        let milestones = vec![
            Milestone::new(0, 10, true),
            Milestone::new(1, 20, false),
            Milestone::new(2, 30, false)
        ];

        let deal = test_deal(status, milestones);
        let res = deal.ensure_can_approve_release(2);

        assert_eq!(res, Err(DomainError::PreviousMilestoneNotReleased {index: 1} ));
    }

}
