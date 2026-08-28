use sqlx::PgPool;

use crate::error::DbError;

pub struct IndexerCursorRepository {
    pool: PgPool
}

impl IndexerCursorRepository {
    pub fn new(pool: PgPool) -> Self {
        IndexerCursorRepository { pool }
    }

    pub async fn get_next_block(
        &self,
        chain_id: i64,
    ) -> Result<Option<u64>, DbError> {
        let row = sqlx::query_scalar::<_, i64>(
            "SELECT next_block FROM indexer_cursors WHERE chain_id = $1"
        )
        .bind(chain_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|b| b as u64))
    }

    pub async fn save_next_block(
        &self,
        chain_id: i64,
        next_block: u64
    ) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO indexer_cursors (chain_id, next_block, updated_at) \
            VALUES ($1, $2, now()) \
            ON CONFLICT (chain_id) DO UPDATE \
            SET next_block = EXCLUDED.next_block, \
                updated_at = now()",
        )
        .bind(chain_id)
        .bind(next_block as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}