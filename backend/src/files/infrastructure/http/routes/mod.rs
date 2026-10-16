mod files;
mod uploads;

pub use files::files_router;
pub use uploads::uploads_router;

use std::sync::Arc;

use axum::extract::State;

use crate::files::application::service::FilesService;

pub(crate) type Files<R, S> = State<Arc<FilesService<R, S>>>;
