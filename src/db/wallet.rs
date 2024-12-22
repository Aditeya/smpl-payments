use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::Utc;
use diesel::{prelude::AsChangeset, ExpressionMethods, Insertable, QueryDsl, SelectableHelper};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};

use super::{handle_duplicate_error, models::Wallet, schema::wallet, Error, SmplDB};

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = wallet)]
pub struct NewWallet {
    user_id: i32,
    balance: BigDecimal,
    status: bool,
}

impl SmplDB {
    pub async fn create_wallet(&self, user_id: i32) -> Result<Wallet, Error> {
        let wallet = NewWallet {
            user_id,
            balance: BigDecimal::from_u8(0).expect("couldn't create BigDecimal 0"),
            status: true,
        };

        let mut conn = self.get_conn().await?;
        diesel::insert_into(wallet::table)
            .values(&wallet)
            .returning(Wallet::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(handle_duplicate_error)
    }

    pub async fn deposit(&self, user_id: i32, amount: BigDecimal) -> Result<Wallet, Error> {
        let mut conn = self.get_conn().await?;
        conn.transaction(|conn| {
            async move {
                // Lock the wallet row for update
                let (id, balance): (i32, BigDecimal) = wallet::table
                    .filter(wallet::user_id.eq(user_id))
                    .select((wallet::id, wallet::balance))
                    .for_update() // Lock the row
                    .first(conn)
                    .await?;

                // Update the balance
                let now = Utc::now();
                Ok(diesel::update(wallet::table.find(id))
                    .set((
                        wallet::balance.eq(balance + amount),
                        wallet::updated_at.eq(now),
                    ))
                    .returning(Wallet::as_returning())
                    .get_result(conn)
                    .await?)
            }
            .scope_boxed()
        })
        .await
    }

    pub async fn withdraw(&self, user_id: i32, amount: BigDecimal) -> Result<Wallet, Error> {
        let mut conn = self.get_conn().await?;
        conn.transaction(|conn| {
            async move {
                // Lock the wallet row for update
                let (id, balance): (i32, BigDecimal) = wallet::table
                    .filter(wallet::user_id.eq(user_id))
                    .select((wallet::id, wallet::balance))
                    .for_update() // Lock the row
                    .first(conn)
                    .await?;

                if balance < amount {
                    return Err(diesel::result::Error::RollbackTransaction);
                }

                // Update the balance
                let now = Utc::now();
                diesel::update(wallet::table.find(id))
                    .set((
                        wallet::balance.eq(balance - amount),
                        wallet::updated_at.eq(now),
                    ))
                    .returning(Wallet::as_returning())
                    .get_result(conn)
                    .await
            }
            .scope_boxed()
        })
        .await
        .map_err(handle_duplicate_error)
    }

    pub async fn get_wallet(&self, user_id: i32) -> Result<Wallet, Error> {
        let mut conn = self.get_conn().await?;
        conn.transaction(|conn| {
            async move {
                // Lock the wallet row for reading
                Ok(wallet::table
                    .filter(wallet::user_id.eq(user_id))
                    .select(Wallet::as_select())
                    .for_no_key_update() // Lock the row for reading
                    .first(conn)
                    .await?)
            }
            .scope_boxed()
        })
        .await
    }
}
