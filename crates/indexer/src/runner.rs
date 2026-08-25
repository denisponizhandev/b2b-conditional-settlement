use std::time::Duration;

use chain::{ChainClient, ChainError};

use crate::config::IndexerConfig;

pub struct IndexerRunner {
    client: ChainClient,
    config: IndexerConfig
}

fn is_rate_limited(err: &ChainError) -> bool {
    match err {
        ChainError::Rpc(msg) => msg.contains("429") || msg.contains("Too Many Requests"),
        _ => false
    }
}

impl IndexerRunner {
    pub fn new(client: ChainClient, config: IndexerConfig) -> Self {
        Self { client, config }
    }

    pub async fn run_once(&self, mut cursor: u64) -> Result<u64, ChainError> {
        let head = self.client.latest_block_number().await?;

        if cursor > head {
            eprintln!("indexer: cursor {cursor} > head {head}, nothing to poll");
            return Ok(cursor);
        }

        eprintln!("indexer: catch-up blocks {cursor}..={head}");
        
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
                    Err(e) => return Err(e)
                }
            }

            let logs = logs.expect("retries exhausted without error branch");

            eprintln!(
                "indexer: factory logs blocks {cursor}..={chunk_end} -> {} log(s)",
                logs.len()
            ); 

            cursor = chunk_end.saturating_add(1);

            if cursor + self.config.max_chunk_blocks <= head && self.config.chunk_delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.config.chunk_delay_ms)).await;
            }
        }

        eprintln!("indexer: caught up, cursor now {cursor}");
        Ok(cursor)
    }

    pub async fn run_loop(&self, mut cursor: u64, interval: Duration) -> Result<(), ChainError> {
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