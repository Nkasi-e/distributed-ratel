use axum::body::Body;
use axum::http::Request;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::TraceLayer;
use tracing::info_span;


/// Request logging / tracing (middleware).
pub fn request_trace_layer(
) -> TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    impl Clone + Fn(&Request<Body>) -> tracing::Span,
> {
    TraceLayer::new_for_http().make_span_with(|req: &Request<Body>| {
        let method = req.method().clone();
        let uri = req.uri().clone();
        info_span!("http_request", %method, %uri)
    })
}