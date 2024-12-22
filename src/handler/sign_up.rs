use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use serde::Deserialize;

use crate::{handler::validate_email, AppState};

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// creates a new user
pub async fn sign_up(
    State(state): State<AppState>,
    Json(CreateUser {
        username,
        email,
        mut password,
    }): Json<CreateUser>,
) -> impl IntoResponse {
    // validate email
    if let Some(r) = validate_email(&email) { return r; };

    // validate username
    if username.is_empty() {
        return (StatusCode::BAD_REQUEST, "Empty username not allowed").into_response();
    }

    // validate password
    if password.is_empty() {
        return (StatusCode::BAD_REQUEST, "Empty password not allowed").into_response();
    }

    // hash password
    password = match pwhash::bcrypt::hash(password) {
        Ok(p) => p,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    };

    tracing::info!(username, email, "Creating user");
    // insert to db
    match state.smpldb.sign_up_user(&username, &email, &password).await {
        Ok(u) => {
            tracing::info!(username, email, "Created user");
            (StatusCode::CREATED, Json(u)).into_response()
        },
        Err(e) => {
            tracing::error!(?e,username, email, "Error creating user");
            (StatusCode::BAD_REQUEST, format!("{e:#?}")).into_response()
        },
    }
}
