# files

Stores uploaded files and their metadata: the bytes on a storage, the metadata in PostgreSQL.

## Ports

| Port             | Adapter                | What it does                     |
|------------------|------------------------|----------------------------------|
| `FileRepository` | `ToastyFileRepository` | the metadata rows                |
| `FileStorage`    | `LocalFileStorage`     | the bytes, in `FILES_UPLOAD_DIR` |

A stored file records which storage holds it (`StorageKind`). Only the local disk exists today; another storage is a
new variant and a new `FileStorage` adapter, nothing else.

## Behavior

- **Upload** writes the bytes under a new UUID name, keeping the original name only as metadata, so two uploads never
  collide and a name from the client never becomes a path.
- **Soft delete** marks the row deleted, recording who did it and when, and keeps the bytes; listing and reading no
  longer return it.
- **Delete** removes the bytes and the row.

Signed-in users may list, read and upload; changing metadata and both kinds of deletion need `ADMIN`. The stored files
are served publicly from `/uploads`.
