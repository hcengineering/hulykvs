# Hulykvs

Hulykvs is a simple key-value store service implemented in Rust. It uses cockroachdb as the backend and provides a simple http api for storing and retrieving key-value pairs.

## API
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
Pre-build docker images is available at: ghcr.io/hcengineering/hulykvs. You can use the following command to run the image locally:
```bash
docker run -p 8086:8086 -it --rm ghcr.io/hcengineering/hulykvs:0.1.0"
```

If you want to run the image as a part of local huly development environment use the following command:
```bash
 export KVS_DB_CONNECTION="host=cockroach user=root port=26257"
 docker run --rm -it --network dev_default -p 8086:8086 ghcr.io/hcengineering/hulykvs:0.1.0
```
This will run Hulykvs in the same network as the rest of huly services, and set the coackroach connection string to the one matching the local dev cockroach instance. 

You can then access hulykvs at http://localhost:8086.

## Authetication
Hulykvs uses bearer JWT token authetication. At the moment, it will accept any token signed by the hulykvs secret. The secret is set in the environment variable KVS_TOKEN_SECRET variable. 

## Configuration
The following environment variables are used to configure hulykvs:
   - ```KVS_DB_CONNECTION```: cockroachdb (postgres) connection string (default: host=localhost user=root port=26257)
   - ```KVS_DB_NAMESPACE```: database schema for the key-value store (default: hulykvs)
   - ```KVS_TOKEN_SECRET```: secret used to sign JWT tokens (default: secret)
   - ```KVS_BIND_HOST```: host to bind the server to (default: 0.0.0.0)
   - ```KVS_BIND_PORT```: port to bind the server to (default: 8086)

## Databse DDL
Database schema is created automatically on startup. Database objects are also created or migrated automatically on startup. 

## Todo (in no particular order)
- [ ] Optional value encryption
- [ ] HEAD request
- [ ] Conditional update (optimistic locking)
- [ ] Support for open telemetry
- [ ] Concurrency control for database migration (several instances of hulykvs are updated at the same time)
- [ ] TLS support
- [ ] Namespacee based access control

## Contributing
Contributions are welcome! Please open an issue or a pull request if you have any suggestions or improvements.

## License
This project is licensed under EPL-2.0






