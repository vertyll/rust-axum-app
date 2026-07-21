use super::fake_storage::FakeStorage;
use super::in_memory_files::InMemoryFiles;
use crate::files::application::commands::UploadFile;
use crate::files::application::service::FilesService;
use crate::files::domain::StorageKind;

type Harness = (FilesService<InMemoryFiles, FakeStorage>, InMemoryFiles, FakeStorage);

pub(crate) fn files_harness() -> Harness {
	let repository = InMemoryFiles::default();
	let storage = FakeStorage::default();
	(FilesService::new(repository.clone(), storage.clone()), repository, storage)
}

pub(crate) fn upload_cmd(name: &str, bytes: &[u8]) -> UploadFile {
	UploadFile {
		data: bytes.to_vec(),
		original_name: name.to_string(),
		mime_type: "text/plain".to_string(),
		storage: StorageKind::Local,
	}
}
