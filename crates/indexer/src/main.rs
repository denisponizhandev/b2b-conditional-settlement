use sqlx::postgres::PgPoolOptions;
use std::error::Error;

use chain::ChainClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL is not set (see configs/.env.example)")?;

    let _pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&database_url);

    let client = ChainClient::from_env().await?;

    eprintln!(
        "indexer scaffold ok: chain_id={}, factory={}",
        client.config().chain_id,
        client.config().escrow_factory
    );

    Ok(())
}