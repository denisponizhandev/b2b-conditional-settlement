use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;

use db::{ChainEventType, NewChainEvent, ChainEventRepository};

async fn pool_from_env() -> PgPool {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    PgPool::connect(&url).await.expect("failed to connect to PostgresSQL")
}

fn sample_event(tx_hash: String) -> NewChainEvent {
    NewChainEvent {
        id: Uuid::new_v4(),
        chain_id: 11155111,
        block_number: 1,
        tx_hash: tx_hash.clone(),
        log_index: 1,
        event_type: ChainEventType::Funded,
        deal_id: format!("0x{}", Uuid::new_v4().simple()),
        contract_address:  String::from("0x12345"),
        payload: Value::Object(Default::default()),
    }
}

#[tokio::test]
async fn insert_chain_event() {
    let pool = pool_from_env().await;
    let repo = ChainEventRepository::new(pool.clone());

    let event = sample_event(format!("0x{}", Uuid::new_v4().simple()));
    
    let res = repo.insert_if_new(&pool, &event).await.expect("insert_if_new");

    assert!(res);
}

#[tokio::test]
async fn insert_if_new_duplicate_returns_false() {
    let pool = pool_from_env().await;
    let repo = ChainEventRepository::new(pool.clone());

    let event = sample_event(format!("0x{}", Uuid::new_v4().simple()));

    let res = repo.insert_if_new(&pool, &event).await.expect("insert_if_new");
    assert!(res);

    let res = repo.insert_if_new(&pool, &event).await.expect("insert_if_new");
    assert_eq!(res, false);
}