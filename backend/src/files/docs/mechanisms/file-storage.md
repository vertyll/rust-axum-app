# File storage

What happens to the bytes and the metadata row on upload, soft delete and delete.

- **Upload** writes the bytes under a new UUID name, keeping the original name only as metadata, so two uploads never
  collide and a name from the client never becomes a path.
- **Soft delete** marks the row deleted, recording who did it and when, and keeps the bytes; listing and reading no
  longer return it.
- **Delete** removes the bytes and the row.

Listing, reading and downloading need `FILES_VIEW`; uploading, changing metadata and both kinds of deletion need
`FILES_MANAGE` ([Authorization](../../../../../docs/authorization.md)). The stored files are served from `/uploads`
behind the same session and `FILES_VIEW`.
