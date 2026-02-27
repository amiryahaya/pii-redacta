# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

PII Redacta is a Rust-based PII (Personally Identifiable Information) detection and redaction service. It consists of a core library for pattern detection/tokenization and an Axum-based REST API, with a React/TypeScript portal UI.

## Build & Development Commands

```bash
# Build
cargo build                              # Debug build
cargo build --all-targets --all-features # Full build (matches CI)

# Run API server (requires PostgreSQL + Redis)
cargo run -p pii_redacta_api

# Tests
cargo test --all-features                # All tests
cargo test -p pii_redacta_core           # Core crate only
cargo test -p pii_redacta_api            # API crate only
cargo test test_name                     # Single test by name
cargo test test_name -- --nocapture      # With stdout output

# Lint & Format (CI enforces both)
cargo fmt --all -- --check               # Check formatting
cargo clippy --all-targets --all-features -- -D warnings  # Lint (warnings = errors)
cargo fmt --all                          # Auto-format

# Benchmarks
cargo bench -p pii_redacta_core

# Infrastructure (Podman)
./scripts/pods.sh up                     # Start PostgreSQL + Redis
./scripts/pods.sh down                   # Stop services
./scripts/pods.sh migrate                # Run SQLx migrations
./scripts/pods.sh db                     # Open psql console
./scripts/pods.sh redis                  # Open Redis CLI

# Portal (React UI)
cd portal && npm run dev                 # Dev server on :3000/:5173
cd portal && npm run build               # Production build
cd portal && npx playwright test         # E2E tests
```

## Architecture

### Workspace Structure

Two crates in a Cargo workspace (resolver v2, edition 2021, MSRV 1.75):

**`pii_redacta_core`** — Core library, no web dependencies:
- `detection/` — Regex-based PII pattern matching (email, phone, NRIC, passport, credit card, etc.). Patterns compiled once via `once_cell::Lazy<Regex>`.
- `tokenization/` — Deterministic SHA256-based PII redaction. Tokens are tenant-isolated and irreversible. Format: `<<PII_{TYPE}_{HASH}>>`.
- `extraction/` — File format text extraction (txt, pdf, docx, xlsx, csv).
- `types/` — `Entity` and `EntityType` enum shared across modules.
- `db/` — SQLx PostgreSQL layer: connection pool, migrations, `ApiKeyManager` (HMAC-SHA256), `TierManager` (subscription tiers with Redis caching).
- `error.rs` — `PiiError` with `thiserror`.

**`pii_redacta_api`** — Axum REST API server:
- `main.rs` — Entry point: loads config from env, connects DB, runs migrations, starts server with graceful shutdown.
- `lib.rs` — Two router constructors: `create_app()` (MVP, no auth) and `create_app_with_auth()` (full portal endpoints). Contains inline handler stubs for portal endpoints.
- `auth/` — JWT + API key authentication, rate limiting (Redis-based), custom Axum extractors.
- `handlers/` — Route handlers: `health`, `detection`, `upload`, `jobs`, `metrics`, `auth`, `api_keys`, `usage`, `subscription`.
- `config/` — `Config::from_env()` for server, database, JWT settings.
- `middleware/` — Security headers, body size limiting (10MB default).
- `jwt.rs` — JWT token generation/validation (HS256, 24h expiry).

### Key Patterns

- **Shared state**: `AppState` with `Arc<Database>` and `JwtConfig`, passed via Axum's `.with_state()`.
- **Test organization**: Unit tests in `*_test.rs` files next to implementation; integration tests in `tests/` directories.
- **API key format**: `pii_{env}_{prefix}_{secret}` — only HMAC hashes stored, never plaintext.
- **Migrations**: SQLx migrations in `crates/pii_redacta_core/migrations/`.

### Portal (`portal/`)

React 18 + TypeScript, Vite, Tailwind CSS, Zustand (state), TanStack Query (data fetching), React Router.

## Environment Variables

Required for running the API server (see `.env.example`):
- `DATABASE_URL` — PostgreSQL connection string
- `REDIS_URL` — Redis connection string
- `API_KEY_SECRET` / `SESSION_SECRET` — 32-byte secrets (generate with `openssl rand -base64 32`)
- `PORT` (default: 8080), `HOST` (default: 0.0.0.0)
- `RUST_LOG` (default: `info,pii_redacta_api=debug`)

## CI Pipeline

GitHub Actions on push to `main`/`develop`/`sprint/*` and PRs to `main`:
1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `cargo test --doc`
