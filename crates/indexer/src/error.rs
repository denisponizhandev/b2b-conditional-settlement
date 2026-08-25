use chain::ChainError;
use db::DbError;

#[derive(Debug, thiserror::Error)]
pub enum IndexerError {
    #[error(transparent)]
    Chain(#[from] ChainError),

    #[error(transparent)]
    Db(#[from] DbError),

    #[error("RPC log missing required field: {0}")]
    MissingLogField(&'static str),
}