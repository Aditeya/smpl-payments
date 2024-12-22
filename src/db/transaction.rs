use bigdecimal::BigDecimal;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt, AsyncConnection};

use super::{handle_duplicate_error, models::Transaction, schema::{transaction, users, wallet}, Error, SmplDB};

impl SmplDB {
    pub async fn insert_payment(
        &self,
        from_user_id: i32,
        to_username: &str,
        amount: BigDecimal,
    ) -> Result<Transaction, Error> {
        let mut conn = self.get_conn().await?;
        conn.transaction(|conn| async move {
            // Lock the from_wallet and to_wallet rows for update
            let (from_wallet_id, from_wallet_balance): (i32, BigDecimal) = wallet::table
                .filter(wallet::user_id.eq(from_user_id))
                .select((wallet::id, wallet::balance))
                .for_update() // Lock the row for update
                .first(conn)
                .await?;

            // get to_user_id
            let to_user_id: i32 = users::table
                .filter(users::username.eq(to_username))
                .select(users::id)
                .first(conn)
                .await?;

            let (to_wallet_id, to_wallet_balance): (i32, BigDecimal) = wallet::table
                .filter(wallet::id.eq(to_user_id))
                .select((wallet::id, wallet::balance))
                .for_update() // Lock the row for update
                .first(conn)
                .await?;

            if from_wallet_balance < amount {
                return Err(diesel::result::Error::RollbackTransaction);
            }

            let now = Utc::now();
            // deduct from sender
            diesel::update(wallet::table.find(from_wallet_id))
                .set((
                    wallet::balance.eq(from_wallet_balance - &amount),
                    wallet::updated_at.eq(now),
                ))
                .execute(conn)
                .await?;

            // add to receiver
            diesel::update(wallet::table.find(to_wallet_id))
                .set((
                    wallet::balance.eq(to_wallet_balance + &amount),
                    wallet::updated_at.eq(now),
                ))
                .execute(conn)
                .await?;

            // make transaction
            diesel::insert_into(transaction::table)
                .values((
                    transaction::from_wallet.eq(from_wallet_id),
                    transaction::to_wallet.eq(to_wallet_id),
                    transaction::amount.eq(&amount),
                ))
                .returning(Transaction::as_returning())
                .get_result(conn)
                .await
        }.scope_boxed())
        .await
        .map_err(handle_duplicate_error)
    }

    pub async fn get_transaction(&self, user_id: i32, transaction_id: i32) -> Result<Option<(Transaction, String, String)>, Error> {
        let mut conn = self.get_conn().await?;

        let transaction: Transaction = transaction::table
            .select(Transaction::as_select())
            .filter(transaction::id.eq(transaction_id))
            .get_result(&mut conn)
            .await?;

        let from_user: (i32, Option<String>) = wallet::table
            .left_join(users::table.on(wallet::user_id.eq(users::id)))
            .select((wallet::user_id, users::username.nullable()))
            .filter(wallet::id.eq(transaction.from_wallet))
            .get_result(&mut conn)
            .await?;

        let to_user: (i32, Option<String>) = wallet::table
            .left_join(users::table.on(wallet::user_id.eq(users::id)))
            .select((wallet::user_id, users::username.nullable()))
            .filter(wallet::id.eq(transaction.to_wallet))
            .get_result(&mut conn)
            .await?;

        if from_user.0 != user_id && to_user.0 != user_id {
            return Ok(None);
        }

        Ok(from_user.1.zip(to_user.1).map(|(from_user, to_user)| (transaction, from_user, to_user)))
    }

    pub async fn list_transactions(&self, user_id: i32) -> Result<Vec<Transaction>, Error> {
        let mut conn = self.get_conn().await?;

        let wallet = self.get_wallet(user_id).await?;

        transaction::table
            .select(Transaction::as_select())
            .filter(
                transaction::from_wallet.eq(wallet.id)
                .or(transaction::to_wallet.eq(wallet.id))
            )
            .get_results(&mut conn)
            .await
            .map_err(handle_duplicate_error)
    }

}
