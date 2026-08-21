use sqlx::{PgPool};
use uuid::Uuid;
use crate::error::{DbError};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainEventType {
    DealCreated,
    Funded,
    MilestoneReleased
}

impl ChainEventType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DealCreated => "deal_created",
            Self::Funded => "funded",
            Self::MilestoneReleased => "milestone_released"
        }
    }
}

pub struct NewChainEvent {
    pub id: Uuid,
    pub chain_id: i64,
    pub block_number: i64,
    pub tx_hash: String,
    pub log_index: i32,
    pub event_type: ChainEventType,
    pub deal_id: String,
    pub contract_address: String,
    pub payload: Value,
}

pub struct ChainEventRepository {
    pool: PgPool
}

impl ChainEventRepository {
    pub fn new(pool: PgPool) -> Self {
        ChainEventRepository {
            pool
        }
    }

    // idempotent, true = new row
    pub async fn insert_if_new(&self, new_event: &NewChainEvent) -> Result<bool, DbError> {
        let result = sqlx::query(
            "INSERT INTO chain_events(id, chain_id, block_number, tx_hash, log_index, event_type, deal_id, contract_address, payload) \
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
            ON CONFLICT (chain_id, tx_hash, log_index) \
            DO NOTHING",
        )
        .bind(new_event.id)
        .bind(new_event.chain_id)
        .bind(new_event.block_number)
        .bind(new_event.tx_hash.clone())
        .bind(new_event.log_index)
        .bind(new_event.event_type.as_str())
        .bind(new_event.deal_id.clone())
        .bind(new_event.contract_address.clone())
        .bind(new_event.payload.clone())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }
}