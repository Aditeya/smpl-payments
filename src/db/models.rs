use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = super::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip)]
    pub password: String,
    pub status: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = super::schema::wallet)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Wallet {
    pub id: i32,
    pub user_id: i32,
    pub balance: BigDecimal,
    pub status: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = super::schema::transaction)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub id: i32,
    pub from_wallet: i32,
    pub to_wallet: i32,
    pub amount: BigDecimal,
    pub created_at: Option<DateTime<Utc>>,
}
