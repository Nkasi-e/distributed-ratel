use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;


use crate::application::error::AppError;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Domain(String),
    Redis(String),
}


impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            ApiError::Domain(m) => (StatusCode::BAD_REQUEST, m.clone()),
            ApiError::Redis(m) => (StatusCode::BAD_GATEWAY, m.clone()),
        };
        let body = Json(json!({"error": msg}));
        (status, body).into_response()
    }
}

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        match value {
            AppError::Domain(e) => ApiError::Domain(e.to_string()),
            AppError::Redis(e) => ApiError::Redis(e.to_string()),
        }
    }
}