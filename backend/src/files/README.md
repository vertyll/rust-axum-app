# files

Stores uploaded files and their metadata: the bytes on a storage, the metadata in PostgreSQL.

## Ports

| Port             | Adapter                | What it does                                                           |
|------------------|------------------------|------------------------------------------------------------------------|
| `FileRepository` | `ToastyFileRepository` | the metadata rows                                                      |
| `FileStorage`    | `Storages`             | signed links to the bytes, and their size and removal, on each storage |

`Storages` holds `S3FileStorage`, for a bucket on any S3 API such as Garage, and `LocalFileStorage`, for the
`FILES_UPLOAD_DIR` folder. A stored file records which storage holds it (`StorageKind`); `FILES_STORAGE` chooses the
storage of new uploads.

## Mechanisms

- [File storage](docs/mechanisms/file-storage.md) – How a file's bytes reach a storage and come back, with signed
  links, and what happens on soft delete and delete.
