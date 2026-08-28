use std::str::FromStr;
use std::time::Duration;

use alloy::primitives::Address;
use chain::{ChainClient, ChainError};
use db::{ChainEventRepository, DealRepository, DbError, IndexerCursorRepository};
use sqlx::PgPool;

use crate::config::IndexerConfig;
use crate::error::IndexerError;
use crate::mapper;
use crate::projector::{self, ProjectionOutcome};

pub struct IndexerRunner {
    client: ChainClient,
    config: IndexerConfig,
    pool: PgPool,
    events_repo: ChainEventRepository,
    deals_repo: DealRepository,
    cursor_repo: IndexerCursorRepository
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
        pool: PgPool,
        events_repo: ChainEventRepository,
        deals_repo: DealRepository,
        cursor_repo: IndexerCursorRepository
    ) -> Self {
        Self {
            client,
            config, 
            pool,
            events_repo,
            deals_repo,
            cursor_repo
        }
    }

    async fn persist_logs(
        &self,
        chain_id: i64,
        logs: &[alloy::rpc::types::Log]
    ) -> Result<(u64, u64, u64, u64), IndexerError> {
        let mut inserted = 0u64;
        let mut duplicate = 0u64;
        let mut decode_skipped = 0u64;
        let mut projected = 0u64;

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

            let mut tx = self
                .pool
                .begin()
                .await
                .map_err(DbError::from)?;
            
            let is_new = self
                .events_repo
                .insert_if_new(&mut *tx, &row)
                .await?;
                
            if is_new {
                inserted += 1;

                match projector::apply(
                    &mut tx,
                    &self.deals_repo,
                    &event,
                    &row.contract_address
                )
                .await?
                {
                    ProjectionOutcome::Applied => projected += 1,
                    ProjectionOutcome::Unlinked => {
                        eprintln!(
                            "indexer: unlinked deal_id={} event={:?}",
                            row.deal_id,
                            row.event_type
                        );
                    }
                }
            } else {
                duplicate += 1;
            }

            tx.commit().await.map_err(DbError::from)?;
        }

        Ok((inserted, duplicate, decode_skipped, projected))
    }

    async fn fetch_logs_with_retry<F, Fut>(
        &self,
        fetch: F
    ) -> Result<Vec<alloy::rpc::types::Log>, IndexerError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<Vec<alloy::rpc::types::Log>, ChainError>> 
    {
        for attempt in 0..=self.config.chunk_max_retries {
            match fetch().await {
                Ok(v) => return Ok(v),
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
        unreachable!("retries exhausted without return")
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
        let mut total_projected = 0u64;
        
        while cursor <= head {
            let chunk_end = cursor
                .saturating_add(self.config.max_chunk_blocks.saturating_sub(1))
                .min(head);

            // 1
            let factory_logs = self
                .fetch_logs_with_retry(|| {
                    self.client.fetch_factory_logs(cursor, chunk_end)
                })
                .await?;

            let (ins, dup, skip, proj) = self
                .persist_logs(chain_id, &factory_logs)
                .await?;

            total_inserted += ins;
            total_duplicate += dup;
            total_projected += proj;

            eprintln!(
                "indexer: factory blocks {cursor}..={chunk_end} -> {} raw, \
                inserted={ins}, duplicate={dup}, decode_skipped={skip}, projected={proj}",
                factory_logs.len()
            );

            // 2
            let deal_addresses = self.events_repo.known_deal_addresses(chain_id).await?;

            for addr_str in deal_addresses {
                let deal_address = match Address::from_str(&addr_str) {
                    Ok(a) => a,
                    Err(_) => {
                        eprintln!("indexer: skip invalid deal address: {addr_str}");
                        continue;
                    }
                };

                let deal_logs = self
                    .fetch_logs_with_retry(|| {
                        self.client.fetch_deal_logs(deal_address, cursor, chunk_end)
                    })
                    .await?;

                let (ins, dup, skip, proj) = self
                    .persist_logs(chain_id, &deal_logs)
                    .await?;

                total_inserted += ins;
                total_duplicate += dup;
                total_projected += proj;

                if !deal_logs.is_empty() {
                    eprintln!(
                        "indexer: deal {addr_str} blocks {cursor}..={chunk_end} -> {} raw, \
                        insterted={ins}, duplicate={dup}, projected={proj}",
                        deal_logs.len()
                    );
                }
            }
            //
            
            cursor = chunk_end.saturating_add(1);

            if cursor + self.config.max_chunk_blocks <= head && self.config.chunk_delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.config.chunk_delay_ms)).await;
            }
        }

        eprintln!(
            "indexer: caught up, cursor now {cursor}, \
            total_inserted={total_inserted}, total_duplicate={total_duplicate}, \
            total_projected={total_projected}"
        );

        self.cursor_repo
            .save_next_block(chain_id, cursor)
            .await?;

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