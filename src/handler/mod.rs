use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use email_address::EmailAddress;

pub mod sign_in;
pub mod sign_up;
pub mod profile;
pub mod wallet;
pub mod transaction;

fn validate_email(email: &str) -> Option<Response> {
    if !EmailAddress::is_valid(email) {
        return Some((StatusCode::BAD_REQUEST, "Invalid Email Address").into_response());
    }
    None
}

fn validate_n_hash_password(password: &str) -> Result<String, Response> {
    // validate password
    if password.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Empty password not allowed").into_response());
    }

    // hash password
    match pwhash::bcrypt::hash(password) {
        Ok(p) => Ok(p),
        Err(e) => {
            tracing::error!(?e, "Error hashing password");
            return Err(
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            );
        }
    }
}
