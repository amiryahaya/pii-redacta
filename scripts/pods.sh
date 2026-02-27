#!/bin/bash
# Podman helper script for PII Redacta

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if podman is installed
check_podman() {
    if ! command -v podman &> /dev/null; then
        log_error "Podman not found. Install with: brew install podman"
        exit 1
    fi
    log_success "Podman version: $(podman --version)"
}

# Load environment variables
load_env() {
    if [ -f .env ]; then
        export $(grep -v '^#' .env | xargs)
        log_info "Loaded environment from .env"
    else
        log_warn ".env file not found"
    fi
}

cmd_up() {
    check_podman
    log_info "Starting services with Podman Compose..."
    podman compose up -d
    log_success "Services started!"
    echo ""
    echo "  PostgreSQL: postgres://localhost:5432"
    echo "  Redis:      redis://localhost:6379"
    echo "  Adminer:    http://localhost:8081 (use --profile tools to enable)"
    echo ""
    log_info "Waiting for PostgreSQL to be ready..."
    sleep 3
    cmd_wait
}

cmd_down() {
    log_info "Stopping services..."
    podman compose down
    log_success "Services stopped"
}

cmd_stop() {
    log_info "Stopping services (keeping volumes)..."
    podman compose stop
}

cmd_restart() {
    log_info "Restarting services..."
    podman compose restart
}

cmd_status() {
    echo "=== Container Status ==="
    podman ps --filter name=pii-redacta --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
    echo ""
    echo "=== Podman Networks ==="
    podman network ls --filter name=pii-redacta
}

cmd_logs() {
    service=$1
    if [ -n "$service" ]; then
        podman compose logs -f "$service"
    else
        podman compose logs -f
    fi
}

cmd_wait() {
    log_info "Checking PostgreSQL health..."
    until podman exec pii-redacta-postgres pg_isready -U pii_redacta -d pii_redacta_dev > /dev/null 2>&1; do
        echo -n "."
        sleep 1
    done
    log_success "PostgreSQL is ready!"
    
    log_info "Checking Redis health..."
    until podman exec pii-redacta-redis redis-cli ping > /dev/null 2>&1; do
        echo -n "."
        sleep 1
    done
    log_success "Redis is ready!"
}

cmd_db() {
    load_env
    log_info "Connecting to database..."
    podman exec -it pii-redacta-postgres psql -U pii_redacta -d pii_redacta_dev
}

cmd_redis() {
    log_info "Connecting to Redis..."
    podman exec -it pii-redacta-redis redis-cli
}

cmd_migrate() {
    load_env
    check_podman
    log_info "Running database migrations..."
    
    # Wait for postgres to be ready
    cmd_wait
    
    # Run migrations using sqlx
    cd crates/pii_redacta_core
    sqlx migrate run
    cd ../..
    log_success "Migrations complete!"
}

cmd_reset() {
    log_warn "This will DESTROY all data in the database!"
    read -p "Are you sure? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log_info "Resetting database..."
        podman compose down -v
        cmd_up
        cmd_migrate
        log_success "Database reset complete!"
    else
        log_info "Cancelled"
    fi
}

cmd_clean() {
    log_warn "This will remove all containers, volumes, and images!"
    read -p "Are you sure? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log_info "Cleaning up..."
        podman compose down -v --rmi all
        podman system prune -f
        log_success "Cleanup complete!"
    else
        log_info "Cancelled"
    fi
}

cmd_build() {
    log_info "Building container image..."
    podman build -t pii-redacta:latest -f Containerfile .
    log_success "Build complete!"
}

cmd_help() {
    cat << HELP
Podman Helper for PII Redacta

Usage: $0 <command>

Services:
  up              Start all services (detached)
  down            Stop and remove containers
  stop            Stop containers (keep volumes)
  restart         Restart all services
  status          Show container status
  logs [service]  Show logs (optionally for specific service)

Database:
  migrate         Run SQLx migrations
  db              Open PostgreSQL console
  redis           Open Redis CLI
  reset           Destroy and recreate database (DANGER!)

Build:
  build           Build production container image
  clean           Remove all containers, volumes, images (DANGER!)

Examples:
  $0 up                    # Start services
  $0 migrate               # Run migrations
  $0 logs postgres         # Watch PostgreSQL logs
  $0 db                    # Open psql console

HELP
}

# Main
case "${1:-help}" in
    up) cmd_up ;;
    down) cmd_down ;;
    stop) cmd_stop ;;
    restart) cmd_restart ;;
    status) cmd_status ;;
    logs) cmd_logs "$2" ;;
    wait) cmd_wait ;;
    db) cmd_db ;;
    redis) cmd_redis ;;
    migrate) cmd_migrate ;;
    reset) cmd_reset ;;
    clean) cmd_clean ;;
    build) cmd_build ;;
    help|--help|-h) cmd_help ;;
    *) log_error "Unknown command: $1"; cmd_help; exit 1 ;;
esac
