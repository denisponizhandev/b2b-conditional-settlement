use sqlx::{PgPool};
use uuid::Uuid;
use domain::deal::{Deal};
use crate::error::{DbError};
use crate::mapper::{row_to_deal_fields, row_to_milestone, status_to_str};

pub struct DealRepository {
    pool: PgPool
}

impl DealRepository {
    pub fn new(pool: PgPool) -> Self {
        DealRepository { pool }
    }

    pub async fn get_deal_by_id(&self, id: Uuid) -> Result<Deal, DbError> {
        let deal_row = sqlx::query(
            "SELECT id, payer_org_id, payee_org_id, status, intent_id, chain_address \
            FROM deals WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DbError::NotFound(id))?;

        let (id, status, intent_id, payer_org_id, payee_org_id, chain_address) = 
            row_to_deal_fields(&deal_row)?;
       
        let milestones_rows = sqlx::query(
            "SELECT milestone_index, amount, released \
            FROM milestones WHERE deal_id = $1 ORDER BY milestone_index"
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        let milestones = milestones_rows
            .iter()
            .map(row_to_milestone)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Deal::new(
            id, 
            status,
            intent_id,
            milestones,
            payer_org_id, 
            payee_org_id,
            chain_address
        ))
    }

    pub async fn insert_deal(&self, deal: &Deal) -> Result<(), DbError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "INSERT INTO deals(id, payer_org_id, payee_org_id, status, intent_id, chain_address) \
            VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(deal.id())
        .bind(deal.payer_org_id())
        .bind(deal.payee_org_id())
        .bind(status_to_str(deal.status()))
        .bind(deal.intent_id())
        .bind(deal.chain_address())
        .execute(&mut *tx)
        .await?;

        for m in deal.milestones() {
            sqlx::query(
                "INSERT INTO milestones (deal_id, milestone_index, amount, released) \
                VALUES ($1, $2, $3, $4)"
            )
            .bind(deal.id())
            .bind(m.index() as i16)
            .bind(m.amount() as i64)
            .bind(m.is_released())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}