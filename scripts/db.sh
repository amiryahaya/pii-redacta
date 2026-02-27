#!/bin/bash
# Database helper script for PII Redacta
# Works with both containerized (Podman) and local PostgreSQL

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Load environment variables
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

# Detect if using container or local PostgreSQL
detect_postgres() {
    # Check if Podman container is running
    if command -v podman &> /dev/null && podman ps --format "{{.Names}}" | grep -q "pii-redacta-postgres"; then
        echo "container"
    else
        echo "local"
    fi
}

POSTGRES_TYPE=$(detect_postgres)

# Set PostgreSQL path for macOS local install
if [ "$POSTGRES_TYPE" = "local" ] && [ -d "/opt/homebrew/opt/postgresql@15/bin" ]; then
    export PATH="/opt/homebrew/opt/postgresql@15/bin:$PATH"
fi

run_sqlx_cmd() {
    cd crates/pii_redacta_core
    if [ "$POSTGRES_TYPE" = "container" ]; then
        log_info "Using containerized PostgreSQL"
    else
        log_info "Using local PostgreSQL"
    fi
    sqlx migrate "$1"
    cd ../..
}

cmd_migrate() {
    log_info "Running migrations..."
    run_sqlx_cmd run
    log_success "Migrations complete!"
}

cmd_rollback() {
    log_info "Rolling back last migration..."
    run_sqlx_cmd revert
    log_success "Rollback complete!"
}

cmd_reset() {
    log_warn "This will DESTROY all data!"
    read -p "Are you sure? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if [ "$POSTGRES_TYPE" = "container" ]; then
            log_info "Resetting containerized database..."
            podman exec pii-redacta-postgres psql -U pii_redacta -d pii_redacta_dev -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
            run_sqlx_cmd run
        else
            log_info "Resetting local database..."
            dropdb pii_redacta_dev 2>/dev/null || true
            createdb pii_redacta_dev
            run_sqlx_cmd run
        fi
        log_success "Database reset complete!"
    else
        log_info "Cancelled"
    fi
}

cmd_console() {
    if [ "$POSTGRES_TYPE" = "container" ]; then
        podman exec -it pii-redacta-postgres psql -U pii_redacta -d pii_redacta_dev
    else
        psql "$DATABASE_URL"
    fi
}

cmd_status() {
    if [ "$POSTGRES_TYPE" = "container" ]; then
        log_info "PostgreSQL: Container (pii-redacta-postgres)"
        podman ps --filter name=pii-redacta-postgres --format "table {{.Names}}\t{{.Status}}"
    else
        if pg_isready -q 2>/dev/null; then
            log_info "PostgreSQL: Local (running)"
        else
            log_warn "PostgreSQL: Local (not running)"
        fi
    fi
}

cmd_help() {
    cat << HELP
Database Helper for PII Redacta

Auto-detects container vs local PostgreSQL.

Usage: $0 <command>

Commands:
  migrate   Run pending migrations
  rollback  Revert last migration
  reset     Drop and recreate database (DANGER!)
  console   Open psql console
  status    Show database status
  help      Show this help

Examples:
  $0 migrate
  $0 console
  $0 status

HELP
}

# Main
case "${1:-help}" in
    migrate) cmd_migrate ;;
    rollback) cmd_rollback ;;
    reset) cmd_reset ;;
    console|psql) cmd_console ;;
    status) cmd_status ;;
    help|--help|-h) cmd_help ;;
    *) log_error "Unknown command: $1"; cmd_help; exit 1 ;;
esac
