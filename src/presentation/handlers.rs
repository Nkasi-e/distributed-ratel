use std::time::{SystemTime, UNIX_EPOCH};

use axum::{extract::State, http::{HeaderMap, HeaderName, HeaderValue, StatusCode}, routing::{get, post}, Json, Router};
use serde_json::json;

use crate::domain::key::RateLimitKey;

use super::dto::{limit_from_header, reset_offset_secs, AllowRequest, AllowResponse};
use super::error::ApiError;
use super::state::AppState;



pub fn routes() -> Router<AppState> {
    Router::new().route("/health",  get(health_check)).route("/allow", post(post_allow))
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}

fn static_header(name: &'static str, value: &str) -> Result<HeaderValue, ApiError> {
    HeaderValue::from_str(value).map_err(|_| ApiError::BadRequest("invalid header value".into()))
}

/// POST /allow — JSON in/out + rate limit headers (partial until store returns usage).
pub async fn post_allow(State(state): State<AppState>, Json(body): Json<AllowRequest>,) -> Result<(StatusCode, HeaderMap, Json<AllowResponse>), ApiError> {
    let key: RateLimitKey = body.key.try_into().map_err(ApiError::BadRequest)?;
    let kind = key.kind();
    let resolved = state.policy.resolve(kind);
    let limit = limit_from_header(&resolved);

    let allowed = state.limiter.allow(&key, body.cost).await?;


    // Heuristic remaining (TODO: replace with real counter from engine).
    let remaining = if allowed {
        limit.saturating_sub(body.cost as u64)
    } else {
        0
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let reset_unix = now.saturating_add(reset_offset_secs(&resolved));

    let mut headers = HeaderMap::new();

    headers.insert(HeaderName::from_static("x-ratelimit-limit"), static_header("x-ratelimit-limit", &limit.to_string())?);

    headers.insert(HeaderName::from_static("x-ratelimit-remaining"), static_header("x-ratelimit-remaining", &remaining.to_string())?);

    headers.insert(HeaderName::from_static("x-ratelimit-reset"), static_header("x-ratelimit-reset", &reset_unix.to_string())?);

    let status = if allowed {
        StatusCode::OK
    } else {
        StatusCode::TOO_MANY_REQUESTS
    };

    let body = AllowResponse {
        allowed,
        cost: body.cost as u64,
        remaining,
        reset_unix

    };

    Ok((status, headers, Json(body)))
}
