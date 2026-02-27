# PII Redacta - Development Guide

## Quick Start (Podman)

```bash
# 1. Start services (PostgreSQL 17 + Redis)
./scripts/pods.sh up

# 2. Run migrations
./scripts/pods.sh migrate

# 3. Run tests
cargo test

# 4. Start the API server
cargo run -p pii_redacta_api
```

## Architecture Overview

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Portal UI     │────▶│   API Server    │────▶│   PostgreSQL    │
│   (React)       │     │   (Axum)        │     │     (Data)      │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                               │
                               ▼
                        ┌─────────────────┐
                        │     Redis       │
                        │    (Cache)      │
                        └─────────────────┘
```

### Security Model
- **API Keys**: Stored as HMAC-SHA256 hashes (server secret required)
- **Passwords**: Argon2id hashing
- **API Key Format**: `pii_{live|test}_{prefix}_{secret}`

## Portal UI (React + Vite)

The Portal UI is located in the `portal/` directory.

```bash
cd portal

# Install dependencies
npm install

# Start development server (port 3000)
npm run dev

# Build for production
npm run build
```

### Portal Features
- **Authentication**: Login, register, password reset
- **Dashboard**: Usage overview, plan status, quick actions
- **API Keys**: Create, view, revoke API keys
- **Usage Analytics**: Daily breakdown, monthly limits
- **Settings**: Profile, billing, notifications

## Podman Commands

| Command | Description |
|---------|-------------|
| `./scripts/pods.sh up` | Start PostgreSQL and Redis containers |
| `./scripts/pods.sh down` | Stop and remove containers |
| `./scripts/pods.sh status` | Show container status |
| `./scripts/pods.sh migrate` | Run SQLx database migrations |
| `./scripts/pods.sh db` | Open PostgreSQL console (psql) |
| `./scripts/pods.sh redis` | Open Redis CLI |
| `./scripts/pods.sh logs postgres` | Watch PostgreSQL logs |
| `./scripts/pods.sh reset` | **DESTROY** and recreate database |
| `./scripts/pods.sh clean` | **DESTROY** all containers, volumes, images |

## Environment Configuration

Copy `.env.example` to `.env` and update:

```bash
# Database (PostgreSQL 17 in container)
DATABASE_URL=postgres://pii_redacta:pii_redacta_dev@localhost:5432/pii_redacta_dev

# Redis (container)
REDIS_URL=redis://127.0.0.1:6379

# Secrets (auto-generated)
API_KEY_SECRET=...
SESSION_SECRET=...
```

## Services

| Service | URL/Port | Description |
|---------|----------|-------------|
| PostgreSQL | `localhost:5432` | Database (user: `pii_redacta`) |
| Redis | `localhost:6379` | Cache |
| Adminer | `localhost:8081` | DB GUI (optional) |

To enable Adminer:
```bash
podman compose --profile tools up -d
```

## Database Schema

### Tables
- `tiers` - Configurable pricing plans
- `users` - User accounts
- `subscriptions` - User subscriptions
- `api_keys` - HMAC-hashed API keys
- `usage_logs` - Analytics and limits
- `ip_blocks` - Security blocks

### Migrations

Located in `crates/pii_redacta_core/migrations/`.

Create new migration:
```bash
cd crates/pii_redacta_core
sqlx migrate add <description>
```

Run migrations:
```bash
./scripts/pods.sh migrate
```

## Building Container Image

```bash
./scripts/pods.sh build
```

This creates a `pii-redacta:latest` image using the multi-stage `Containerfile`.

## Troubleshooting

### "Cannot connect to database"
```bash
# Check if containers are running
./scripts/pods.sh status

# Restart services
./scripts/pods.sh restart
```

### "Permission denied" on migration files
```bash
# SELinux/permissions issue - relabel volumes
podman compose down
podman compose up -d
```

### Port already in use
```bash
# Check what's using port 5432
lsof -i :5432

# Kill or reconfigure
```

## Why Podman?

- **Daemonless**: No background process running as root
- **Rootless**: Containers run as your user by default
- **Docker-compatible**: Same CLI, compose files work
- **Open source**: No licensing restrictions
