# files

Stores uploaded files and their metadata: the bytes on a storage, the metadata in PostgreSQL.

## Ports

| Port             | Adapter                | What it does                     |
|------------------|------------------------|----------------------------------|
| `FileRepository` | `ToastyFileRepository` | the metadata rows                |
| `FileStorage`    | `LocalFileStorage`     | the bytes, in `FILES_UPLOAD_DIR` |

A stored file records which storage holds it (`StorageKind`). Only the local disk exists today; another storage is a
new variant and a new `FileStorage` adapter, nothing else.

## Mechanisms

- [File storage](docs/mechanisms/file-storage.md) – What happens to the bytes and the metadata row on upload, soft
  delete and delete.
