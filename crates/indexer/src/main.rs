mod config;
mod error;
mod mapper;
mod runner;
mod projector;

use std::error::Error;
use std::time::Duration;

use chain::ChainClient;
use db::{ChainEventRepository, DealRepository, IndexerCursorRepository};
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

    let events_repo = ChainEventRepository::new(pool.clone());
    let deals_repo = DealRepository::new(pool.clone());
    let cursor_repo = IndexerCursorRepository::new(pool.clone());

    let indexer_config = IndexerConfig::from_env()?;
    let start_block = indexer_config.start_block;
    let poll_secs = indexer_config.poll_interval_secs;

    let client = ChainClient::from_env().await?;
    let chain_id = client.config().chain_id as i64;

    let cursor = match cursor_repo.get_next_block(chain_id).await? {
        Some(saved) => {
            eprintln!("indexer: resume from DB next_block={saved}");
            saved
        }
        None => {
            eprintln!("indexer: no cursor row, bootstrap next_block={start_block}");
            start_block
        }
    };

    let runner = IndexerRunner::new(
        client, 
        indexer_config, 
        pool, 
        events_repo, 
        deals_repo,
        cursor_repo
    );

    eprintln!("indexer: poll_secs={poll_secs:?}");

    match poll_secs {
        Some(secs) if secs > 0 => {
            runner
                .run_loop(cursor, Duration::from_secs(secs))
                .await?;
        }
        _ => {
            let _cursor = runner.run_once(cursor).await?;
        }
    }

    Ok(())
}