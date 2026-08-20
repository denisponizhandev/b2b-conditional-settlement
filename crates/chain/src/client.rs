use alloy::eips::BlockNumberOrTag;
use alloy::network::Ethereum;
use alloy::primitives::{Address, B256};
use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use alloy::rpc::types::{Filter, Log as RpcLog};
use alloy::sol_types::SolEvent;
use alloy::transports::TransportError;

use thiserror::Error;

use crate::bindings::{DealCreated, IEscrowDeal};
use crate::config::{ChainConfig, ConfigError};
use crate::events::{decode_log, ChainEvent, EventDecodeError};

#[derive(Debug, Error)]
pub enum ClientError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    
    #[error("RPC call failed: {0}")]
    Rpc(String),

    #[error("invalid RPC URL: {0}")]
    InvalidRpcUrl(String),

    #[error("RPC transport error: {0}")]
    Transport(#[from] TransportError),

    #[error("chain id mismatch: config: {expected}, node returned {actual}")]
    ChainIdMismatch{ expected: u64, actual: u64},

    #[error(transparent)]
    EventDecode(#[from] EventDecodeError)
}

pub struct ChainClient {
    config: ChainConfig,
    provider: alloy::providers::RootProvider<Ethereum>
}

impl ChainClient {
    pub async fn connect(config: ChainConfig) -> Result<Self, ClientError> { 
        let provider = RootProvider::<Ethereum>::connect(&config.rpc_url)
            .await
            .map_err(ClientError::Transport)?;

        let actual = provider
            .get_chain_id()
            .await
            .map_err(|e| ClientError::Rpc(e.to_string()))?;

        if actual != config.chain_id {
            return Err(ClientError::ChainIdMismatch{
                expected: config.chain_id,
                actual
            });
        }

        Ok(Self { config, provider })
    }

    pub async fn from_env() -> Result<Self, ClientError> {
        let config = ChainConfig::from_env()?;
        Self::connect(config).await
    }

    pub fn config(&self) -> &ChainConfig {
        &self.config
    }

    pub async fn get_logs(&self, filter: &Filter) -> Result<Vec<RpcLog>, ClientError> {
        let logs = self
            .provider
            .get_logs(filter)
            .await
            .map_err(|e| ClientError::Rpc(e.to_string()))?;

        Ok(logs)
    } 

    pub async fn fetch_factory_logs(
        &self,
        from_block: u64,
        to_block: u64
    ) -> Result <Vec<RpcLog>, ClientError> {
        let filter = Filter::new()
            .address(self.config.escrow_factory)
            .from_block(BlockNumberOrTag::Number(from_block))
            .to_block(BlockNumberOrTag::Number(to_block));

        self.get_logs(&filter).await
    }

    pub async fn fetch_deal_logs(
        &self,
        deal_address: Address,
        from_block: u64,
        to_block: u64
    ) -> Result <Vec<RpcLog>, ClientError> {
        let filter = Filter::new()
            .address(deal_address)
            .from_block(BlockNumberOrTag::Number(from_block))
            .to_block(BlockNumberOrTag::Number(to_block));

        self.get_logs(&filter).await
    }

    pub fn decode_logs(logs: &[RpcLog]) -> Vec<Result<ChainEvent, EventDecodeError>> {
        logs.iter().map(|log| decode_log(log)).collect()
    }

    pub async fn read_deal_id(&self, deal_address: Address) -> Result<B256, ClientError> {
        let contract = IEscrowDeal::new(deal_address, &self.provider);
        let deal_id = contract
            .dealId()
            .call()
            .await
            .map_err(|e| ClientError::Rpc(e.to_string()))?;

        Ok(deal_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires SEPOLIA_RPC_URL and network"]
    async fn from_evn_connects() {
        let client = ChainClient::from_env().await.expect("connect");
        assert_eq!(client.config().chain_id, 11155111);
    }

    #[tokio::test]
    #[ignore = "requires SEPOLIA_RPC_URL"]
    async fn fetch_factory_logs_smoke() {
        let client = ChainClient::from_env().await.expect("connect");
        let factory = client.config().escrow_factory;

        let head: u64 = client
            .provider
            .get_block_number()
            .await
            .expect("get_block_number");

        let from = head.saturating_sub(1_000);

        let logs = client
            .fetch_factory_logs(from, head)
            .await
            .expect("get_logs");

        for log in &logs {
            assert_eq!(log.address(), factory);
        }
    }
}
