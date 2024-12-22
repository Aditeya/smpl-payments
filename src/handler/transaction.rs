use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{utils::ValidateAuth, AppState};

#[derive(Debug, Deserialize)]
pub struct CreateTransaction {
    to_username: String,
    amount: BigDecimal,
}

pub async fn create_transaction(
    ValidateAuth(user_id): ValidateAuth,
    State(state): State<AppState>,
    Json(CreateTransaction { to_username, amount }): Json<CreateTransaction>,
) -> impl IntoResponse {
    match state.smpldb.insert_payment(user_id, &to_username, amount).await {
        Ok(transaction) => (StatusCode::CREATED, Json(transaction)).into_response(),
        Err(crate::db::Error::RollbackTransaction) => {
            (StatusCode::BAD_REQUEST, "Insufficient Funds").into_response()
        }
        Err(e) => {
            tracing::error!(?e, user_id, "Failed to create transaction for user");
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FormattedTransaction {
    from_username: String,
    to_username: String,
    amount: BigDecimal,
    created_at: DateTime<Utc>,
}

pub async fn get_transaction_by_id(
    ValidateAuth(user_id): ValidateAuth,
    State(state): State<AppState>,
    Path(transaction_id): Path<i32>,
) -> impl IntoResponse {
    match state.smpldb.get_transaction(user_id, transaction_id).await {
        Ok(Some((transaction, from_username, to_username))) => {
            let t = FormattedTransaction {
                from_username, to_username,
                amount: transaction.amount,
                created_at: transaction.created_at.unwrap_or_default()
            };
            (StatusCode::CREATED, Json(t)).into_response()
        },
        Ok(None) => (StatusCode::GONE, "Transaction Not Found").into_response(),
        Err(e) => {
            tracing::error!(?e, user_id, "Failed to get transaction for user");
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}

pub async fn list_transactions(
    ValidateAuth(user_id): ValidateAuth,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.smpldb.list_transactions(user_id).await {
        Ok(transactions) => (StatusCode::CREATED, Json(transactions)).into_response(),
        Err(e) => {
            tracing::error!(?e, user_id, "Failed to get all transactions for user");
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}
