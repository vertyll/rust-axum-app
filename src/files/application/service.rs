//! The files facade the use cases attach to; each use case lives in
//! `use_cases/` as its own `impl` block. Two ports only, so they stay as
//! two plain generic parameters instead of a bundle trait.

use super::ports::FileStorage;
use crate::files::domain::FileRepository;

#[derive(Clone)]
pub struct FilesService<R, S> {
	pub(super) repository: R,
	pub(super) storage: S,
}

impl<R: FileRepository, S: FileStorage> FilesService<R, S> {
	pub fn new(repository: R, storage: S) -> Self {
		Self { repository, storage }
	}
}
