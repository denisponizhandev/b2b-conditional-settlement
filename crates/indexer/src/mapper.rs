use alloy::rpc::types::Log as RpcLog;
use chain::ChainEvent;
use db::{ChainEventType, NewChainEvent};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::IndexerError;

fn b256_hex(value: alloy::primitives::B256) -> String {
    format!("{value:#x}")
}

fn address_hex(value: alloy::primitives::Address) -> String {
    format!("{value:#x}")
}

fn u256_dec(value: alloy::primitives::U256) -> String {
    value.to_string()
}

fn deal_created_payload(event: &chain::DealCreated) -> Value {
    json!({
        "deal_address": address_hex(event.dealAddress),
        "deal_nonce": u256_dec(event.dealNonce),
        "admin": address_hex(event.admin),
        "approver": address_hex(event.approver),
        "payer": address_hex(event.payer),
        "payee": address_hex(event.payee),
        "token": address_hex(event.token.into()),
        "retention_bps": u256_dec(event.retentionBps),
    })
}

fn funded_payload(event: &chain::Funded) -> Value {
    json!({
        "amount": u256_dec(event.amount),
    })
}

fn milestone_released_payload(event: &chain::MilestoneReleased) -> Value {
    json!({
        "milestone_index": u256_dec(event.milestoneIndex),
        "amount": u256_dec(event.amount),
        "retention_held": u256_dec(event.retentionHeld),
    })
}

pub fn to_new_chain_event(
    chain_id: i64,
    log: &RpcLog,
    event: &ChainEvent
) -> Result<NewChainEvent, IndexerError> {
    let block_number = log
        .block_number
        .ok_or(IndexerError::MissingLogField("block_number"))? as i64;

    let tx_hash_raw = log
        .transaction_hash
        .ok_or(IndexerError::MissingLogField("transaction_hash"))?;

    let tx_hash = format!("{tx_hash_raw:#x}");

    let log_index_raw = log
        .log_index
        .ok_or(IndexerError::MissingLogField("log_index"))?;

    let log_index = i32::try_from(log_index_raw)
        .map_err(|_| IndexerError::MissingLogField("log_index (overflow i32)"))?;

    let contract_address = address_hex(log.address());

    let (event_type, deal_id, payload) = match event {
        ChainEvent::DealCreated(e) => (
            ChainEventType::DealCreated,
            b256_hex(e.dealId),
            deal_created_payload(e),
        ),
        ChainEvent::Funded(e) => (
            ChainEventType::Funded,
            b256_hex(e.dealId),
            funded_payload(e),
        ),
        ChainEvent::MilestoneReleased(e) => (
            ChainEventType::MilestoneReleased,
            b256_hex(e.dealId),
            milestone_released_payload(e),
        ),
    };

    Ok(NewChainEvent {
        id: Uuid::new_v4(),
        chain_id,
        block_number,
        tx_hash,
        log_index,
        event_type,
        deal_id, 
        contract_address,
        payload
    })
}