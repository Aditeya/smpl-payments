use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{db, utils::ValidateAuth, AppState};

pub async fn get_profile(
    ValidateAuth(user_id): ValidateAuth,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.smpldb.get_user_by_id(user_id).await {
        Ok(Some(user)) => (StatusCode::OK, Json(user)).into_response(),
        Ok(None) => {
            tracing::error!(user_id, "Failed to find user with id");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL SERVER ERROR: user not found",
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(?e, "Failed to get user");
            (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL SERVER ERROR").into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfile {
    username: String,
}

pub async fn update_profile(
    ValidateAuth(user_id): ValidateAuth,
    State(state): State<AppState>,
    Json(UpdateProfile { username }): Json<UpdateProfile>,
) -> impl IntoResponse {
    if username.is_empty() {
        return (StatusCode::BAD_REQUEST, "Username cannot be empty").into_response();
    }

    match state.smpldb.update_username(user_id, &username).await {
        Ok(Some(user)) => (StatusCode::OK, Json(user)).into_response(),
        Ok(None) => {
            tracing::error!(user_id, "Failed to find user with id");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL SERVER ERROR: user not found",
            )
                .into_response()
        }
        Err(db::Error::Duplicate) => (StatusCode::BAD_REQUEST, "Username Taken").into_response(),
        Err(e) => {
            tracing::error!(?e, "Failed to get user");
            (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL SERVER ERROR").into_response()
        }
    }
}
