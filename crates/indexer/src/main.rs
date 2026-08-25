mod config;
mod runner;

use std::error::Error;
use std::time::Duration;

use chain::ChainClient;

use crate::config::IndexerConfig;
use crate::runner::IndexerRunner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

   let indexer_config = IndexerConfig::from_env()?;
   let start_block = indexer_config.start_block;
   let poll_secs = indexer_config.poll_interval_secs;

    let client = ChainClient::from_env().await?;
    let runner = IndexerRunner::new(client, indexer_config);

    eprintln!("indexer 4.3: start_block={start_block}, poll_secs={poll_secs:?}");

    match poll_secs {
        Some(secs) if secs > 0 => {
            runner
                .run_loop(start_block, Duration::from_secs(secs))
                .await?;
        },
        _ => {
            let _cursor = runner.run_once(start_block).await?;
        }
    }

    Ok(())
}