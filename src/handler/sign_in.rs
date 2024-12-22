use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{handler::validate_n_hash_password, utils::issue_new_jwt, AppState};

use super::validate_email;

#[derive(Debug, Deserialize)]
pub struct SignIn {
    email: String,
    password: String,
}

pub async fn sign_in(
    State(state): State<AppState>,
    Json(SignIn { email, password }): Json<SignIn>,
) -> impl IntoResponse {
    // validate email
    if let Some(r) = validate_email(&email) { return r; };

    let password = match validate_n_hash_password(&password) {
        Ok(h) => h,
        Err(r) => return r,
    };

    // check db
    let user = match state.smpldb.get_user(&email).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::GONE, "Incorrect email or password").into_response(),
        Err(e) => {
            tracing::error!(?e, "Failed to get user");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        },
    };

    if pwhash::bcrypt::verify(password, &user.password) {
        tracing::info!(email, "Failed password attempt");
        return (StatusCode::UNAUTHORIZED, "Incorrect email or Password").into_response()
    }

    match issue_new_jwt(user.id) {
        Ok(token) => (StatusCode::OK, token).into_response(),
        Err(r) => r
    }
}
