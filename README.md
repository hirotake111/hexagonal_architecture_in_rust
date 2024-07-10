# hexagonal_architecture_in_rust

Example of hexagonal architecture in Rust

```bash
# Start PostgreSQL
docker-compose up -d
# Database migration
psql -h localhost -U postgres -f ./database.sql

# Start dev server
PORT=8080 RUST_LOG=debug cargo run
```
