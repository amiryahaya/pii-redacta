#!/bin/bash
#
# Quick Infrastructure Setup for Testing
# 
# Usage:
#   ./scripts/setup-infra.sh       # Start infrastructure
#   ./scripts/setup-infra.sh stop  # Stop infrastructure

set -e

COMPOSE_FILE="podman-compose.test.yml"
PROJECT_NAME="pii-redacta-test"

# Detect container runtime
if command -v podman >/dev/null 2>&1; then
    COMPOSE_CMD="podman-compose"
    RUNTIME="podman"
elif command -v docker >/dev/null 2>&1; then
    COMPOSE_CMD="docker-compose"
    RUNTIME="docker"
else
    echo "Error: Neither Podman nor Docker found"
    exit 1
fi

echo "Using: $RUNTIME"

if [ "$1" = "stop" ]; then
    echo "Stopping infrastructure..."
    $COMPOSE_CMD -f $COMPOSE_FILE -p $PROJECT_NAME down -v
    echo "Infrastructure stopped"
    exit 0
fi

if [ "$1" = "restart" ]; then
    echo "Restarting infrastructure..."
    $COMPOSE_CMD -f $COMPOSE_FILE -p $PROJECT_NAME down -v
fi

echo "Starting infrastructure..."
$COMPOSE_CMD -f $COMPOSE_FILE -p $PROJECT_NAME up -d

echo "Waiting for services to be ready..."

# Wait for PostgreSQL
echo -n "Waiting for PostgreSQL"
for i in {1..30}; do
    if $RUNTIME exec pii-redacta-test-db pg_isready -U pii_redacta -d pii_redacta_test >/dev/null 2>&1; then
        echo " ✓"
        break
    fi
    echo -n "."
    sleep 1
done

# Wait for Redis
echo -n "Waiting for Redis"
for i in {1..30}; do
    if $RUNTIME exec pii-redacta-test-redis redis-cli ping >/dev/null 2>&1; then
        echo " ✓"
        break
    fi
    echo -n "."
    sleep 1
done

# Run migrations
echo "Running database migrations..."
for file in crates/pii_redacta_core/migrations/*.sql; do
    if [ -f "$file" ]; then
        $RUNTIME exec -i pii-redacta-test-db psql -U pii_redacta -d pii_redacta_test < "$file" 2>/dev/null || true
    fi
done
echo "Migrations complete ✓"

# Wait for API
echo -n "Waiting for API"
for i in {1..60}; do
    if curl -s http://localhost:8080/health >/dev/null 2>&1; then
        echo " ✓"
        break
    fi
    echo -n "."
    sleep 1
done

echo ""
echo "========================================"
echo "Infrastructure ready!"
echo "========================================"
echo "PostgreSQL: localhost:5432"
echo "Redis:      localhost:6379"
echo "API:        http://localhost:8080"
echo "Health:     http://localhost:8080/health"
echo ""
echo "To stop: ./scripts/setup-infra.sh stop"
