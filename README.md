# Hulykvs

Hulykvs is a simple key-value store service implemented in Rust. It supports PostgreSQL 15+ and CockroachDB as backends and provides a simple HTTP API for storing and retrieving key-value pairs.

## API v2
Create a key-value pair api

```PUT /api2/{workspace}/{namespace}/{key}```
Stores request payload as the value for the given key in the given namespace. Existing keys will be overwritten. Returns 204 (NoContent) on success.

**Optional headers:**

- `If-Match: *` — update only if the key exists
- `If-Match: <md5>` — update only if current value's MD5 matches
- `If-None-Match: *` — insert only if the key does not exist

Returns:
- `204` on successful insert or update
- `201` if inserted with `If-None-Match: *`
- `412` if the condition is not met
- `400` if headers are invalid


```GET /api2/{workspace}/{namespace}/{key}```
Retrieves the value for the given key in the given namespace. Returns 404 if the key does not exist.


```DELETE /api2/{workspace}/{namespace}/{key}```
Deletes the key-value pair for the given key in the given namespace. Returns 404 if the key does not exist, 204 (NoContent) on success, 404 if the key does not exist.


```GET /api2/{workspace}/{namespace}?[prefix=<prefix>]```
Retrieves all key-value pairs in the given namespace. Optionally, a prefix can be provided to filter the results. The following structure is returned:
```json
{
  "workspace": "workspace",
  "namespace": "namespace",
  "count": 3,
  "keys": ["key1", "key2", "keyN"]
}
```
## API (old)
workspace = "defaultspace"

Create a key-value pair

```POST /api/{namespace}/{key}```
Stores request payload as the value for the given key in the given namespace. Existing keys will be overwritten. Returs 204 (NoContent) on sucesss.

```GET /api/{namespace}/{key}```
Retrieves the value for the given key in the given namespace. Returns 404 if the key does not exist.

```DELETE /api/{namespace}/{key}```
Deletes the key-value pair for the given key in the given namespace. Returns 404 if the key does not exist, 204 (NoContent) on success, 404 if the key does not exist.

```GET /api/{namespace}?[prefix=<prefix>]```
Retrieves all key-value pairs in the given namespace. Optionally, a prefix can be provided to filter the results. The following structure is returned:
```json
{
  "namespace": "namespace",
  "count": 3,
  "keys": ["key1", "key2", "keyN"]
}
```

## Running
Pre-build docker images is available at: hardcoreeng/service_hulykvs:{tag}.

You can use the following command to run the image locally:
```bash
docker run -p 8094:8094 -it --rm hardcoreeng/service_hulykvs:{tag}"
```

Run from source locally:
```bash
cd hulykvs_server
export HULY_DB_CONNECTION="postgresql://postgres:postgres@localhost:5432/postgres?sslmode=disable"
export HULY_TOKEN_SECRET="secret"
cargo run --bin hulykvs
```
Service endpoint: `http://localhost:8094`

For local CockroachDB instead of PostgreSQL:
```bash
export HULY_DB_CONNECTION="postgresql://root@huly.local:26257/defaultdb?sslmode=disable"
```

## Testing
Run API test script from the repository root after starting `hulykvs` on `http://localhost:8094`:
```bash
cd scripts
./tests.sh
```

Prerequisites:
- `jwt` CLI installed (used by `scripts/token.sh` to sign test tokens)
- service started with `HULY_TOKEN_SECRET=secret` (or export your value before running tests: `export HULY_TOKEN_SECRET=...`)
- `scripts/token.sh` prefers `HULY_TOKEN_SECRET` and otherwise reads `token_secret` from `hulykvs_server/src/config/default.toml`


If you want to run the service as a part of local huly development environment use the following command:
```bash
 export HULY_DB_CONNECTION="postgresql://root@huly.local:26257/defaultdb?sslmode=disable"
 docker run --rm -it --network dev_default -p 8094:8094 hardcoreeng/service_hulykvs:{tag}
```
This will run Hulykvs in the same network as the rest of huly services, and set the connection string to the local CockroachDB instance used in local Huly development.

You can then access hulykvs at http://localhost:8094.

## Authetication
Hulykvs uses bearer JWT token authetication. At the moment, it will accept any token signed by the hulykvs secret. The secret is set in the environment variable HULY_TOKEN_SECRET variable. 

## Configuration
The following environment variables are used to configure hulykvs:
   - ```HULY_DB_CONNECTION```: PostgreSQL-compatible connection string (PostgreSQL 15+ or CockroachDB). Default: `postgresql://root@huly.local:26257/defaultdb?sslmode=disable`
   - ```HULY_DB_SCHEME```: database schema for the key-value store (default: hulykvs)
   - ```HULY_TOKEN_SECRET```: secret used to sign JWT tokens (default: secret)
   - ```HULY_BIND_HOST```: host to bind the server to (default: 0.0.0.0)
   - ```HULY_BIND_PORT```: port to bind the server to (default: 8094)
   - ```HULY_PAYLOAD_SIZE_LIMIT```: maximum size of the payload (default: 2Mb)
   - ```HULY_DEFAULT_WORKSPACE_UUID```: default workspace uuid (for old API and DB migration only)

## Databse DDL
Database schema is created automatically on startup. Database objects are also created or migrated automatically on startup. 

## Todo (in no particular order)
- [ ] Optional value encryption
- [ ] Support for open telemetry
- [ ] TLS support
- [ ] Liveness/readiness probe endpoint
    + Namespacee based access control
    + Concurrency control for database migration (several instances of hulykvs are updated at the same time)
    + HEAD request
    + Conditional update (optimistic locking)

## Contributing
Contributions are welcome! Please open an issue or a pull request if you have any suggestions or improvements.

## License
This project is licensed under EPL-2.0

