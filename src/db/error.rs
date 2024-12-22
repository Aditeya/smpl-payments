use diesel_async::pooled_connection::deadpool::PoolError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to get connection from diesel connection pool: {0}")]
    ConnectionAcqusitionFailure(#[from] PoolError),
    #[error("Failed to run diesel query: {0}")]
    DieselFailure(#[from] diesel::result::Error),
    #[error("Unique Violation in DB")]
    Duplicate,
    #[error("Transaction was rolled back")]
    RollbackTransaction
}
