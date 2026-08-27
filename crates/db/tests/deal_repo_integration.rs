use sqlx::PgPool;
use uuid::Uuid;

use db::DealRepository;
use domain::{Deal, DealStatus, DomainError, Milestone};

async fn pool_from_env() -> PgPool {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    PgPool::connect(&url).await.expect("failed to connect to PostgresSQL")
}

async fn insert_org(pool: &PgPool, id: Uuid, name: &str) {
    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(id)
        .bind(name)
        .execute(pool)
        .await
        .expect("insert organozation");
}

fn random_intent_id() -> String {
    format!("0x{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

fn sample_deal(deal_id: Uuid, payer_org_id: Uuid, payee_org_id: Uuid) -> Deal {
    Deal::new(
        deal_id,
        DealStatus::Funded,
        random_intent_id(),
        vec![
            Milestone::new(0, 400_000, false),
            Milestone::new(1, 600_000, false)
        ],
        payer_org_id,
        payee_org_id,
        Some("0xabc123".into())
    )
}

#[tokio::test]
async fn insert_and_load_deal_roundtrip() {
    let pool = pool_from_env().await;
    let repo = DealRepository::new(pool.clone());

    let payer_org_id = Uuid::new_v4();
    let payee_org_id = Uuid::new_v4();
    let deal_id = Uuid::new_v4();

    insert_org(&pool, payer_org_id, "Payer Org").await;
    insert_org(&pool, payee_org_id, "Payee Org").await;

    let deal = sample_deal(deal_id, payer_org_id, payee_org_id);
    let expected_intent = deal.intent_id().clone();

    repo.insert_deal(&deal).await.expect("inster_deal");

    let loaded = repo.get_deal_by_id(deal_id).await.expect("get_deal_by_id");

    assert_eq!(loaded.intent_id(), &expected_intent);
    assert_eq!(loaded.id(), deal_id);
    assert_eq!(loaded.status(), DealStatus::Funded);
    assert_eq!(loaded.payer_org_id(), payer_org_id);
    assert_eq!(loaded.payee_org_id(), payee_org_id);
    assert_eq!(loaded.chain_address(), &Some("0xabc123".into()));
    assert_eq!(loaded.milestones().len(), 2);
    assert_eq!(loaded.milestones()[0].amount(), 400_000);
    assert_eq!(loaded.milestones()[1].amount(), 600_000);
    assert!(!loaded.milestones()[0].is_released());
}

#[tokio::test]
async fn loaded_deal_allows_approve_on_first_milestone() {
    let pool = pool_from_env().await;
    let repo = DealRepository::new(pool.clone());

    let payer_org_id = Uuid::new_v4();
    let payee_org_id = Uuid::new_v4();
    let deal_id = Uuid::new_v4();

    insert_org(&pool, payer_org_id, "Payer Org").await;
    insert_org(&pool, payee_org_id, "Payee Org").await;

    let deal = sample_deal(deal_id, payer_org_id, payee_org_id);
    repo.insert_deal(&deal).await.expect("insert_deal");
    
    let loaded = repo.get_deal_by_id(deal_id).await.expect("get_deal_by_id");
    
    assert!(loaded.ensure_can_approve_release(0).is_ok());
}

#[tokio::test]
async fn loaded_deal_rejects_approve_when_previous_not_released() {
    let pool = pool_from_env().await;
    let repo = DealRepository::new(pool.clone());

    let payer_org_id = Uuid::new_v4();
    let payee_org_id = Uuid::new_v4();
    let deal_id = Uuid::new_v4();

    insert_org(&pool, payer_org_id, "Payer Org").await;
    insert_org(&pool, payee_org_id, "Payee Org").await;
    
    let deal = sample_deal(deal_id, payer_org_id, payee_org_id);
    repo.insert_deal(&deal).await.expect("insert_deal");
    
    let loaded = repo.get_deal_by_id(deal_id).await.expect("get_deal_by_id");
    
    let err = loaded
        .ensure_can_approve_release(1)
        .expect_err("milestone 1 should be blocked");
    
    assert_eq!(
        err,
        DomainError::PreviousMilestoneNotReleased { index: 0 }
    );
}