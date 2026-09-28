# Taytay security boundary

- Lunsaran access tokens are read from a protected local file and are never included in artifact metadata, logs, or source requests.
- Taytay receives only organization/project-scoped upload-session data. It never receives or persists provider credentials.
- Signed URLs, bearer tokens, camera credentials, source passwords, and media bytes are not valid diagnostic output.
- Source metadata is treated as sensitive by default; operators should use opaque source IDs and avoid filenames when names reveal customer information.
- The service runs unprivileged with a read-only system filesystem and a narrowly scoped writable spool.
- Idempotency keys use the durable artifact ID, preventing retries from creating a second upload session for the same publication.

