use alloy::primitives::{Address, B256, Log as PrimitiveLog, LogData};
use alloy::rpc::types::Log as RpcLog;
use alloy::sol_types::SolEvent;

use thiserror::Error;

use crate::bindings::{DealCreated, Funded, MilestoneReleased};

#[derive(Clone)]
pub enum ChainEvent {
    DealCreated(DealCreated),
    Funded(Funded),
    MilestoneReleased(MilestoneReleased)
}

#[derive(Debug, Error)]
pub enum EventDecodeError {
    #[error("log has no topics (not a standard event)")]
    EmptyTopics,

    #[error("unknow event topic0: {0}")]
    UnknownTopic(B256),

    #[error("failed to ABI-decode log body: {0}")]
    Decode(#[from] alloy::sol_types::Error)
}

pub fn decode_log(log: &RpcLog) -> Result<ChainEvent, EventDecodeError> {
    let topics = log.topics();

    if topics.is_empty() {
        return Err(EventDecodeError::EmptyTopics);
    }

    let topic0 = topics[0];
    let log_data: &LogData = log.data();

    if topic0 == DealCreated::SIGNATURE_HASH {
        let event = DealCreated::decode_log_data(log_data)?;
        return Ok(ChainEvent::DealCreated(event));
    }

    if topic0 == Funded::SIGNATURE_HASH {
        let event = Funded::decode_log_data(log_data)?;
        return Ok(ChainEvent::Funded(event));
    }

    if topic0 == MilestoneReleased::SIGNATURE_HASH {
        let event = MilestoneReleased::decode_log_data(log_data)?;
        return Ok(ChainEvent::MilestoneReleased(event));
    }

    return Err(EventDecodeError::UnknownTopic(topic0));
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{B256, U256};
    use alloy::sol_types::SolEvent;

    fn rpc_log_from_encoded(address: Address, log_data: LogData) -> RpcLog {
        let inner = PrimitiveLog {address, data: log_data };

        RpcLog {
            inner,
            block_hash: None,
            block_number: None,
            block_timestamp: None,
            transaction_hash: None,
            transaction_index: None,
            log_index: None,
            removed: false
        }
    }

    fn rpc_log_from_event<E: SolEvent> (address: Address, event: &E) -> RpcLog {
        let log_data = event.encode_log_data();
        rpc_log_from_encoded(address, log_data)
    }

    #[test]
    fn decode_funded_roundtrip() {
        use crate::bindings::Funded;

        let deal_id = B256::repeat_byte(0xAB);
        let amount = U256::from(1_000_000u64);

        let event = Funded {
            dealId: deal_id,
            amount
        };

        let log = rpc_log_from_event(Address::ZERO, &event);

        let decoded = decode_log(&log).expect("Funded log should decode");

        match decoded {
            ChainEvent::Funded(f) => {
                assert_eq!(f.dealId, deal_id);
                assert_eq!(f.amount, amount);
            },
            other => panic!("expected Funded, got other")
        }
    }

    #[test]
    fn decode_deal_created_roundtrip() {
        use crate::bindings::DealCreated;

        let factory = Address::repeat_byte(0x11);
        let deal_addr = Address::repeat_byte(0x22);
        
        let event = DealCreated {
            dealId: B256::repeat_byte(0x33),
            dealAddress: deal_addr,
            dealNonce: U256::from(1u64),
            admin: factory,
            approver: Address::repeat_byte(0x44),
            payer: Address::repeat_byte(0x55),
            payee: Address::repeat_byte(0x66),
            token: Address::repeat_byte(0x77),
            retentionBps: U256::from(500u64),
        };
        
        let log = rpc_log_from_event(factory, &event);
        let decoded = decode_log(&log).expect("DealCreated should decode");
        
        match decoded {
            ChainEvent::DealCreated(d) => {
                assert_eq!(d.dealAddress, deal_addr);
                assert_eq!(d.dealNonce, U256::from(1u64));
            }
            other => panic!("expected DealCreated, got other"),
        }
    }

    #[test]
    fn unknown_topic_returns_err() {
        let fake_topic = B256::ZERO;
        let log_data = LogData::new_unchecked(vec![fake_topic], Default::default());
        let log = rpc_log_from_encoded(Address::ZERO, log_data);

        let res = decode_log(&log);
        assert!(matches!(
            res, 
            Err(EventDecodeError::UnknownTopic(t)) if t == fake_topic
        ));
    }

    #[test]
    fn empty_topics_returns_err() {
        let log_data = LogData::new_unchecked(vec![], Default::default());
        let log = rpc_log_from_encoded(Address::ZERO, log_data);

        let res = decode_log(&log);
        assert!(matches!(
            res,
            Err(EventDecodeError::EmptyTopics)
        ));
    }
}