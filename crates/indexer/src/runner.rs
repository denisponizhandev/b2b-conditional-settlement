use std::time::Duration;

use chain::{ChainClient, ChainError};
use db::ChainEventRepository;

use crate::config::IndexerConfig;
use crate::error::IndexerError;
use crate::mapper;

pub struct IndexerRunner {
    client: ChainClient,
    config: IndexerConfig,
    events_repo: ChainEventRepository,
}

fn is_rate_limited(err: &ChainError) -> bool {
    match err {
        ChainError::Rpc(msg) => msg.contains("429") || msg.contains("Too Many Requests"),
        _ => false
    }
}

impl IndexerRunner {
    pub fn new(
        client: ChainClient, 
        config: IndexerConfig,
        events_repo: ChainEventRepository
    ) -> Self {
        Self {
            client,
            config, 
            events_repo 
        }
    }

    async fn persist_factory_logs(
        &self,
        chain_id: i64,
        logs: &[alloy::rpc::types::Log]
    ) -> Result<(u64, u64, u64), IndexerError> {
        let mut inserted = 0u64;
        let mut duplicate = 0u64;
        let mut decode_skipped = 0u64;

        for (log, decoded) in logs.iter().zip(ChainClient::decode_logs(logs)) {
            let event = match decoded {
                Ok(ev) => ev,
                Err(e) => {
                    eprintln!("indexer: skip undecodable log: {e}");
                    decode_skipped +=1;
                    continue;
                }
            };

            let row = mapper::to_new_chain_event(chain_id, log, &event)?;

            let is_new = self.events_repo.insert_if_new(&row).await?;
            if is_new {
                inserted += 1;
            } else {
                duplicate += 1;
            }
        }

        Ok((inserted, duplicate, decode_skipped))
    }

    pub async fn run_once(&self, mut cursor: u64) -> Result<u64, IndexerError> {
        let head = self.client.latest_block_number().await?;

        let chain_id = self.client.config().chain_id as i64;

        if cursor > head {
            eprintln!("indexer: cursor {cursor} > head {head}, nothing to poll");
            return Ok(cursor);
        }

        eprintln!("indexer: catch-up blocks {cursor}..={head}");

        let mut total_inserted = 0u64;
        let mut total_duplicate = 0u64;
        
        while cursor <= head {
            let chunk_end = cursor
                .saturating_add(self.config.max_chunk_blocks.saturating_sub(1))
                .min(head);

            let mut logs = None;
            for attempt in 0..=self.config.chunk_max_retries {
                match self.client.fetch_factory_logs(cursor, chunk_end).await {
                    Ok(v) => {
                        logs = Some(v);
                        break;
                    },
                    Err(e) if is_rate_limited(&e) && attempt < self.config.chunk_max_retries => {
                        // exponential backoff: 1s, 2s, 4s, 8s…
                        let wait = Duration::from_secs(1u64 << attempt);
                        eprintln!(
                            "indexer: rate limited (429), retry {}/{} in {}s",
                            attempt + 1,
                            self.config.chunk_max_retries,
                            wait.as_secs()
                        );
                        tokio::time::sleep(wait).await;
                    },
                    Err(e) => return Err(e.into())
                }
            }

            let logs = logs.expect("retries exhausted without error branch");

            let (inserted, duplicate, skipped) = self
                .persist_factory_logs(chain_id, &logs).await?;

            total_inserted += inserted;
            total_duplicate += duplicate;

            eprintln!(
               "indexer: factory logs blocks {cursor}..={chunk_end} -> {} raw, \
                inserted={inserted}, duplicate={duplicate}, decode_skipped={skipped}",
                logs.len()
            ); 

            cursor = chunk_end.saturating_add(1);

            if cursor + self.config.max_chunk_blocks <= head && self.config.chunk_delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.config.chunk_delay_ms)).await;
            }
        }

        eprintln!(
            "indexer: caught up, cursor now {cursor}, \
            total_inserted={total_inserted}, total_duplicate={total_duplicate}"
        );

        Ok(cursor)
    }

    pub async fn run_loop(&self, mut cursor: u64, interval: Duration) -> Result<(), IndexerError> {
        loop {
            cursor = self.run_once(cursor).await?;

            eprintln!(
                "indexer: sleeping {}s before next poll",
                interval.as_secs()
            );

            tokio::time::sleep(interval).await;
        }
    }
}