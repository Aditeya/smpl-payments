mod error;
pub mod models;
mod schema;
mod users;
mod wallet;
mod transaction;
use diesel::{result::DatabaseErrorKind, Connection, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub use error::Error;

use anyhow::Context;
use diesel_async::{
    pooled_connection::{
        deadpool::{Object, Pool, PoolError},
        AsyncDieselConnectionManager,
    },
    AsyncPgConnection,
};

fn handle_duplicate_error(e: diesel::result::Error) -> Error {
    match e {
        diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => Error::Duplicate,
        diesel::result::Error::RollbackTransaction=>Error::RollbackTransaction,
        e => Error::DieselFailure(e)
    }
}

pub struct SmplDB {
    pool: Pool<AsyncPgConnection>,
}

impl SmplDB {
    pub fn new(db_url: &str) -> anyhow::Result<Self> {
        if let Err(e) = Self::run_migrations(db_url) {
            eprintln!("Failed to run DB migrations: {e:#?}");
        };
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
        let pool = Pool::builder(config)
            .build()
            .context("Failed to created SmplDB Diesel Async Pool")?;

        Ok(Self { pool })
    }

    async fn get_conn(&self) -> Result<Object<AsyncPgConnection>, PoolError> {
        self.pool.get().await
    }

    fn run_migrations(
        db_url: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut conn = PgConnection::establish(db_url)?;
        pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
        conn.run_pending_migrations(MIGRATIONS)?;
        Ok(())
    }
}
