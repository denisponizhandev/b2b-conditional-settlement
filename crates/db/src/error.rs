use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DbError {

    #[error("deal not found: {0}")]
    NotFound(Uuid),

    #[error("unknown deal status in database: {0}")]
    InvalidStatus(String),

    #[error("milestone amount out of range")]
    InvalidAmount,

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error)
}