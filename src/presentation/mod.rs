
mod state;
mod dto;
mod handlers;
mod error;
mod middleware;

pub use state::AppState;

use axum::Router;

pub fn build_router(state: AppState) -> Router {
    Router::new().merge(handlers::routes()).layer(middleware::request_trace_layer()).with_state(state)
}