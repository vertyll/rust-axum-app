use super::fake_storage::FakeStorage;
use super::in_memory_files::InMemoryFiles;
use crate::files::application::commands::RequestUpload;
use crate::files::application::service::FilesService;
use crate::files::domain::StoredFile;

type Harness = (FilesService<InMemoryFiles, FakeStorage>, InMemoryFiles, FakeStorage);

/// The largest upload the harness allows, in bytes.
const MAX_SIZE: u64 = 100;

pub(crate) fn files_harness() -> Harness {
    let repository = InMemoryFiles::default();
    let storage = FakeStorage::default();
    (
        FilesService::new(repository.clone(), storage.clone(), MAX_SIZE),
        repository,
        storage,
    )
}

pub(crate) fn request(name: &str, size: u64) -> RequestUpload {
    RequestUpload {
        original_name: name.to_string(),
        mime_type: "text/plain".to_string(),
        size,
    }
}

/// A file uploaded the whole way: allowed, sent and confirmed.
pub(crate) async fn uploaded(service: &FilesService<InMemoryFiles, FakeStorage>, name: &str) -> StoredFile {
    let (file, _) = service.request_upload(request(name, 5)).await.unwrap();
    service.storage.put(&file.object_key, 5);
    service.confirm_upload(file.id).await.unwrap()
}
