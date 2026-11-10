//! A time limit on a request, answered with a problem document when it runs
//! out.

use std::time::Duration;

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use super::problem::{Problem, TIMEOUT};

/// Middleware, for `from_fn_with_state` with the limit as its state: a
/// request that takes longer is answered `503` with `errors.timeout`, and its
/// handler is dropped at its next `.await`.
pub async fn within(State(limit): State<Duration>, request: Request, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    tokio::time::timeout(limit, next.run(request))
        .await
        .unwrap_or_else(|_| {
            tracing::warn!(path, limit = ?limit, "request timed out");
            Problem::new(StatusCode::SERVICE_UNAVAILABLE, TIMEOUT).into_response()
        })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::middleware::from_fn_with_state;
    use axum::routing::get;
    use tower::ServiceExt;

    use super::within;

    fn app(limit: Duration) -> Router {
        Router::new()
            .route("/fast", get(|| async { "done" }))
            .route(
                "/slow",
                get(|| async {
                    tokio::time::sleep(Duration::from_secs(60)).await;
                    "done"
                }),
            )
            .layer(from_fn_with_state(limit, within))
    }

    async fn get_status(app: Router, path: &str) -> (StatusCode, serde_json::Value) {
        let response = app
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&body).unwrap_or_default())
    }

    #[tokio::test]
    async fn a_request_within_the_limit_is_answered_by_its_handler() {
        let (status, _) = get_status(app(Duration::from_secs(5)), "/fast").await;

        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn a_request_past_the_limit_is_answered_with_a_timeout_document() {
        let (status, body) = get_status(app(Duration::from_millis(50)), "/slow").await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["code"], "errors.timeout");
    }
}
