use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha384;

/// Contains User ID
pub struct ValidateAuth(pub i32);

#[async_trait]
impl<S> FromRequestParts<S> for ValidateAuth
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Some(value) = parts.headers.get(AUTHORIZATION) else {
            return Err((StatusCode::BAD_REQUEST, "`Authorization` header is missing"));
        };
        let Some(unvalidated_token) = value.to_str().ok().and_then(|k| k.strip_prefix("Bearer "))
        else {
            return Err((StatusCode::BAD_REQUEST, "Invalid Token"));
        };

        validate_jwt(&unvalidated_token).map(|id| ValidateAuth(id))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct AuthToken {
    id: i32,
    issued_at: i64,
}
impl AuthToken {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            issued_at: chrono::Utc::now().timestamp(),
        }
    }
}

fn get_key() -> Hmac<Sha384> {
    const KEY: [u8; 10] = *b"secret-key";
    Hmac::new_from_slice(&KEY).expect("Should not fail")
}

pub fn issue_new_jwt(id: i32) -> Result<String, Response> {
    let key = get_key();
    let header = Header {
        algorithm: AlgorithmType::Hs384,
        ..Default::default()
    };
    let claims = AuthToken::new(id);

    match Token::new(header, claims).sign_with_key(&key) {
        Ok(t) => Ok(t.as_str().to_string()),
        Err(e) => {
            tracing::error!(?e, "Failed to sign token with key");
            return Err(
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            );
        }
    }
}

fn validate_jwt(token: &str) -> Result<i32, (StatusCode, &'static str)> {
    let key = get_key();
    let token: Token<Header, AuthToken, _> = match token.verify_with_key(&key) {
        Ok(t) => t,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Malformed Token")),
    };
    let claims = token.claims();

    let Some(start_time) = DateTime::from_timestamp(claims.issued_at, 0) else {
        return Err((StatusCode::BAD_REQUEST, "Malformed Token"));
    };
    let end_time = Utc::now();
    let duration = end_time.signed_duration_since(start_time);
    let hours = duration.num_hours();

    if hours > 1 {
        return Err((StatusCode::UNAUTHORIZED, "Token Expired"));
    }
    Ok(claims.id)
}
