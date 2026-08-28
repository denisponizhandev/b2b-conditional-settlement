use chain::ChainEvent;
use db::{DealRepository, DbError};
use alloy::primitives::{B256, U256, Address};
use sqlx::Postgres;

pub enum ProjectionOutcome {
    Applied,
    Unlinked
}

fn intent_id_hex(deal_id: &B256) -> String {
    format!("{deal_id:#x}")
}

fn milestone_index_from_u256(value: U256) -> Result<u16, DbError> {
    let n: u64 = value
        .try_into()
        .map_err(|_| DbError::InvalidMilestoneIndex(value.to_string()))?;
    
    u16::try_from(n).map_err(|_| DbError::InvalidMilestoneIndex(value.to_string()))
}

fn address_hex(value: Address) -> String {
    format!("{value:#x}")
}

pub async fn apply(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    deals: &DealRepository,
    event: &ChainEvent,
    contract_address: &str
) -> Result<ProjectionOutcome, DbError> {
    match event {
        ChainEvent::DealCreated(e) => {
            let intent_id = intent_id_hex(&e.dealId);
            let chain_address = address_hex(e.dealAddress);

            if deals.link_chain_deal(tx, &intent_id, &chain_address).await? {
                Ok(ProjectionOutcome::Applied)
            } else {
                Ok(ProjectionOutcome::Unlinked)
            }
        }

        ChainEvent::Funded(e) => {
            let intent_id = intent_id_hex(&e.dealId);

            if deals
                .mark_funded(tx, &intent_id, contract_address)
                .await?
            {
                Ok(ProjectionOutcome::Applied)
            } else {
                Ok(ProjectionOutcome::Unlinked)
            }
        }

        ChainEvent::MilestoneReleased(e) => {
            let intent_id = intent_id_hex(&e.dealId);
            let milestone_index = milestone_index_from_u256(e.milestoneIndex)?;

            if deals
                .mark_milestone_released(
                    tx,
                    &intent_id,
                    contract_address,
                    milestone_index
                )
                .await?
            {
                Ok(ProjectionOutcome::Applied)
            } else {
                Ok(ProjectionOutcome::Unlinked)
            }
        }
    }
}