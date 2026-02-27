#!/bin/bash
#
# Podman-based Test Runner
# Works without podman-compose

set -e

PG_USER="pii_redacta"
PG_PASS="pii_redacta_dev"
PG_DB="pii_redacta_test"

cleanup() {
    echo "Cleaning up..."
    podman rm -f pii-redacta-test-db 2>/dev/null || true
    podman rm -f pii-redacta-test-redis 2>/dev/null || true
}

start_postgres() {
    echo "Starting PostgreSQL..."
    podman run -d \
        --name pii-redacta-test-db \
        -e POSTGRES_USER=$PG_USER \
        -e POSTGRES_PASSWORD=$PG_PASS \
        -e POSTGRES_DB=$PG_DB \
        -p 5432:5432 \
        -v $(pwd)/crates/pii_redacta_core/migrations:/docker-entrypoint-initdb.d:Z \
        docker.io/postgres:16-alpine
    
    echo -n "Waiting for PostgreSQL"
    for i in {1..30}; do
        if podman exec pii-redacta-test-db pg_isready -U $PG_USER -d $PG_DB >/dev/null 2>&1; then
            echo " ✓"
            return 0
        fi
        echo -n "."
        sleep 1
    done
    echo ""
    echo "PostgreSQL failed to start"
    exit 1
}

start_redis() {
    echo "Starting Redis..."
    podman run -d \
        --name pii-redacta-test-redis \
        -p 6379:6379 \
        docker.io/redis:7-alpine
    
    echo -n "Waiting for Redis"
    for i in {1..30}; do
        if podman exec pii-redacta-test-redis redis-cli ping >/dev/null 2>&1; then
            echo " ✓"
            return 0
        fi
        echo -n "."
        sleep 1
    done
    echo ""
    echo "Redis failed to start"
    exit 1
}

run_tests() {
    echo ""
    echo "========================================"
    echo "Running Tests"
    echo "========================================"
    
    # Set environment for tests
    export TEST_DATABASE_URL="postgres://$PG_USER:$PG_PASS@localhost:5432/$PG_DB"
    
    echo ""
    echo "1. Running unit tests..."
    cargo test --lib --bins 2>&1 | tail -20
    
    echo ""
    echo "2. Running integration tests..."
    cargo test --test auth_api_test 2>&1 | tail -20
    cargo test --test api_keys_integration_test 2>&1 | tail -20
    cargo test --test e2e_test 2>&1 | tail -20
    cargo test --test auth_integration_test 2>&1 | tail -20
    
    echo ""
    echo "3. Running core tests..."
    cargo test -p pii_redacta_core 2>&1 | tail -20
    
    echo ""
    echo "========================================"
    echo "All tests complete!"
    echo "========================================"
}

# Main
trap cleanup EXIT

case "${1:-all}" in
    start)
        start_postgres
        start_redis
        echo ""
        echo "Infrastructure ready!"
        echo "PostgreSQL: localhost:5432"
        echo "Redis: localhost:6379"
        ;;
    stop)
        cleanup
        echo "Infrastructure stopped"
        ;;
    test)
        run_tests
        ;;
    all)
        cleanup
        start_postgres
        start_redis
        run_tests
        ;;
    *)
        echo "Usage: $0 [start|stop|test|all]"
        exit 1
        ;;
esac
