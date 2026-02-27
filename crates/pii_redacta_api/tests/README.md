# PII Redacta API Integration Tests

This directory contains integration and end-to-end tests for the PII Redacta API.

## Prerequisites

- PostgreSQL running locally or accessible via network
- Rust toolchain installed

## Database Setup

By default, tests use the following database URL:
```
postgres://pii_redacta:pii_redacta_dev@localhost:5432/pii_redacta_test
```

You can override this by setting the `TEST_DATABASE_URL` environment variable:
```bash
export TEST_DATABASE_URL="postgres://user:password@host:port/database"
```

### Creating the Test Database

```bash
# Create database
createdb pii_redacta_test

# Create user (if needed)
createuser -P pii_redacta  # Set password to 'pii_redacta_dev'

# Grant permissions
psql -c "GRANT ALL PRIVILEGES ON DATABASE pii_redacta_test TO pii_redacta;"
```

### Running Migrations

Tests assume the database schema is already set up. Run migrations before testing:

```bash
cd /path/to/pii-redacta
cargo run --bin pii-redacta-api -- migrate
```

Or manually with SQLx:
```bash
sqlx migrate run --database-url postgres://pii_redacta:pii_redacta_dev@localhost:5432/pii_redacta_test
```

## Running Tests

### Run all tests
```bash
cargo test --test auth_api_test
cargo test --test api_keys_integration_test
cargo test --test e2e_test
```

### Run specific test
```bash
cargo test --test auth_api_test test_register_user_success
```

### Run with output
```bash
cargo test --test auth_api_test -- --nocapture
```

### Run with specific database URL
```bash
TEST_DATABASE_URL="postgres://localhost/test_db" cargo test --test auth_api_test
```

## Test Structure

- `common/` - Shared test utilities and fixtures
  - `mod.rs` - Test context, database setup, HTTP client
  - `fixtures.rs` - Factory functions for test data
- `auth_api_test.rs` - Authentication endpoint tests
- `api_keys_integration_test.rs` - API key management tests
- `e2e_test.rs` - End-to-end user flow tests
- `auth_integration_test.rs` - JWT and password hashing tests

## Test Isolation

Tests use the following strategies for isolation:

1. **Unique identifiers** - Each test generates unique emails/user IDs using UUIDs
2. **Cleanup** - Tests clean up created data after execution
3. **TestContext** - Helper struct tracks created resources and cleans up on drop
4. **Transactions** - Some tests could use database transactions (future improvement)

## Troubleshooting

### "Failed to connect to database"
- Ensure PostgreSQL is running
- Check database URL is correct
- Verify user has permissions

### "Failed to run database migrations"
- Run migrations manually before tests
- Check that `sqlx-cli` is installed: `cargo install sqlx-cli`

### Test timeouts
- Some tests make multiple HTTP requests
- Increase timeout with: `cargo test -- --timeout 60`

### Port conflicts
- Tests use an in-memory Axum router (no TCP port binding)
- Database conflicts are avoided via unique identifiers
