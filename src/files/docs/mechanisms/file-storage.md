# File storage

What happens to the bytes and the metadata row on upload, soft delete and delete.

- **Upload** writes the bytes under a new UUID name, keeping the original name only as metadata, so two uploads never
  collide and a name from the client never becomes a path.
- **Soft delete** marks the row deleted, recording who did it and when, and keeps the bytes; listing and reading no
  longer return it.
- **Delete** removes the bytes and the row.

Signed-in users may list, read and upload; changing metadata and both kinds of deletion need `ADMIN`. The stored files
are served publicly from `/uploads`.
