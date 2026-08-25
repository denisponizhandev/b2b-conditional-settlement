mod config;
mod error;
mod mapper;
mod runner;

use std::error::Error;
use std::time::Duration;

use chain::ChainClient;
use db::ChainEventRepository;
use sqlx::postgres::PgPoolOptions;

use crate::config::IndexerConfig;
use crate::runner::IndexerRunner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL is not set (see configs/.env.example)")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let events_repo = ChainEventRepository::new(pool);

    let indexer_config = IndexerConfig::from_env()?;
    let start_block = indexer_config.start_block;
    let poll_secs = indexer_config.poll_interval_secs;

    let client = ChainClient::from_env().await?;
    let runner = IndexerRunner::new(client, indexer_config, events_repo);

    eprintln!("indexer 4.4: start_block={start_block}, poll_secs={poll_secs:?}");

    match poll_secs {
        Some(secs) if secs > 0 => {
            runner
                .run_loop(start_block, Duration::from_secs(secs))
                .await?;
        }
        _ => {
            let _cursor = runner.run_once(start_block).await?;
        }
    }

    Ok(())
}