use axum::Router;
use axum::middleware::from_fn;
use tower_http::services::ServeDir;

use crate::files::application::permissions::FilesView;
use crate::identity::require;

/// The stored bytes, for a caller whose roles grant `FILES_VIEW`; the
/// composition root puts it behind the auth layer.
pub fn uploads_router(upload_dir: &str) -> Router {
    Router::new()
        .fallback_service(ServeDir::new(upload_dir))
        .layer(from_fn(require::<FilesView>))
}
