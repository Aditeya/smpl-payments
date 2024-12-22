use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bigdecimal::{BigDecimal, FromPrimitive};
use serde::Deserialize;

use crate::{utils::ValidateAuth, AppState};

pub async fn get_wallet(
    ValidateAuth(user_id): ValidateAuth,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.smpldb.get_wallet(user_id).await {
        Ok(wallet) => (StatusCode::OK, Json(wallet)).into_response(),
        Err(e) => {
            tracing::error!(?e, user_id, "Failed to get wallet for user");
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum UpdateWalletType {
    Deposit,
    Withdraw,
}
#[derive(Debug, Deserialize)]
pub struct UpdateWallet {
    action: UpdateWalletType,
    amount: BigDecimal,
}

pub async fn update_wallet(
    ValidateAuth(user_id): ValidateAuth,
    State(state): State<AppState>,
    Json(UpdateWallet { action, amount }): Json<UpdateWallet>,
) -> impl IntoResponse {
    if amount <= BigDecimal::from_u8(0).expect("shouldn't fail") {
        return (StatusCode::BAD_REQUEST, "Amount cannot be zero").into_response();
    };

    match action {
        UpdateWalletType::Deposit => match state.smpldb.deposit(user_id, amount).await {
            Ok(wallet) => (StatusCode::OK, Json(wallet)).into_response(),
            Err(e) => {
                tracing::error!(?e, user_id, "Failed to deposit funds wallet for user");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
        },
        UpdateWalletType::Withdraw => match state.smpldb.withdraw(user_id, amount).await {
            Ok(wallet) => (StatusCode::OK, Json(wallet)).into_response(),
            Err(crate::db::Error::RollbackTransaction) => {
                (StatusCode::BAD_REQUEST, "Insufficient Funds").into_response()
            }
            Err(e) => {
                tracing::error!(?e, user_id, "Failed to withdraw funds from wallet for user");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
        },
    }
}
