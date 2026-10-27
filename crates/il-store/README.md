# il-store

`SqliteStore::open(path)` creates a WAL-mode SQLite database, registers the statically linked sqlite-vec extension, and applies the embedded `migrations/*.sql` files in name order. Calls on the `Store` implementation run on Tokio's blocking pool. `active_flows` and `recoverable_runs` support startup restoration.

Credentials are encrypted with AES-256-GCM. Set `IL_MASTER_KEY` to the base64 encoding of exactly 32 bytes before storing credentials; a missing key leaves non-credential storage usable. API keys and webhook tokens are stored as SHA-256 digests. The server should generate high-entropy raw tokens and reveal them once.

`DiskFileStore::new(store, root)` writes file contents to `root` and stores their metadata in the same database. It returns `/api/files/<id>` preview URLs for the server to serve.
