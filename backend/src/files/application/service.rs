//! The files facade the use cases attach to; each use case lives in
//! `use_cases/` as its own `impl` block. Two ports only, so they stay as
//! two plain generic parameters instead of a bundle trait.

use super::ports::FileStorage;
use crate::files::domain::FileRepository;

#[derive(Clone)]
pub struct FilesService<R, S> {
    pub(super) repository: R,
    pub(super) storage: S,
    /// The largest upload allowed, in bytes.
    pub(super) max_size: u64,
}

impl<R: FileRepository, S: FileStorage> FilesService<R, S> {
    pub fn new(repository: R, storage: S, max_size: u64) -> Self {
        Self {
            repository,
            storage,
            max_size,
        }
    }
}
