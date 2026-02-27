#!/bin/bash
#
# Test Runner Script for PII Redacta
# 
# This script sets up the test infrastructure using Podman and runs all tests.
#
# Usage:
#   ./scripts/run-tests.sh           # Run all tests
#   ./scripts/run-tests.sh backend   # Run only backend tests
#   ./scripts/run-tests.sh e2e       # Run only E2E tests
#   ./scripts/run-tests.sh unit      # Run only unit tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_TYPE="${1:-all}"
COMPOSE_FILE="podman-compose.test.yml"
PROJECT_NAME="pii-redacta-test"

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check for Podman
if command_exists podman; then
    CONTAINER_RUNTIME="podman"
    COMPOSE_CMD="podman-compose"
elif command_exists docker; then
    print_warning "Podman not found, falling back to Docker"
    CONTAINER_RUNTIME="docker"
    COMPOSE_CMD="docker-compose"
else
    print_error "Neither Podman nor Docker found. Please install Podman."
    exit 1
fi

print_info "Using container runtime: $CONTAINER_RUNTIME"

# Function to cleanup infrastructure
cleanup() {
    print_info "Cleaning up test infrastructure..."
    $COMPOSE_CMD -f $COMPOSE_FILE -p $PROJECT_NAME down -v 2>/dev/null || true
    
    # Remove any lingering containers
    $CONTAINER_RUNTIME rm -f pii-redacta-test-db pii-redacta-test-redis pii-redacta-test-api 2>/dev/null || true
}

# Function to setup infrastructure
setup_infrastructure() {
    print_info "Setting up test infrastructure..."
    
    # Start services
    $COMPOSE_CMD -f $COMPOSE_FILE -p $PROJECT_NAME up -d
    
    # Wait for PostgreSQL
    print_info "Waiting for PostgreSQL to be ready..."
    for i in {1..30}; do
        if $CONTAINER_RUNTIME exec pii-redacta-test-db pg_isready -U pii_redacta -d pii_redacta_test >/dev/null 2>&1; then
            print_success "PostgreSQL is ready"
            break
        fi
        sleep 1
    done
    
    # Wait for Redis
    print_info "Waiting for Redis to be ready..."
    for i in {1..30}; do
        if $CONTAINER_RUNTIME exec pii-redacta-test-redis redis-cli ping >/dev/null 2>&1; then
            print_success "Redis is ready"
            break
        fi
        sleep 1
    done
    
    # Run migrations
    print_info "Running database migrations..."
    for file in crates/pii_redacta_core/migrations/*.sql; do
        if [ -f "$file" ]; then
            print_info "Applying migration: $(basename $file)"
            $CONTAINER_RUNTIME exec -i pii-redacta-test-db psql -U pii_redacta -d pii_redacta_test < "$file"
        fi
    done
    print_success "Migrations complete"
    
    # Wait for API
    print_info "Waiting for API to be ready..."
    for i in {1..60}; do
        if curl -s http://localhost:8080/health >/dev/null 2>&1; then
            print_success "API is ready"
            break
        fi
        sleep 1
    done
}

# Function to run unit tests
run_unit_tests() {
    print_info "Running unit tests..."
    cargo test --lib --bins -- --nocapture
    print_success "Unit tests complete"
}

# Function to run backend integration tests
run_backend_tests() {
    print_info "Running backend integration tests..."
    
    export TEST_DATABASE_URL="postgres://pii_redacta:pii_redacta_dev@localhost:5432/pii_redacta_test"
    
    # Run tests
    cargo test --test auth_api_test -- --nocapture
    cargo test --test api_keys_integration_test -- --nocapture
    cargo test --test e2e_test -- --nocapture
    cargo test --test auth_integration_test -- --nocapture
    
    print_success "Backend integration tests complete"
}

# Function to run E2E tests
run_e2e_tests() {
    print_info "Running E2E tests with Playwright..."
    
    cd portal
    
    # Check if node_modules exists
    if [ ! -d "node_modules" ]; then
        print_info "Installing npm dependencies..."
        npm ci
    fi
    
    # Check if Playwright browsers are installed
    if [ ! -d "$HOME/.cache/ms-playwright" ]; then
        print_info "Installing Playwright browsers..."
        npx playwright install chromium
    fi
    
    # Set environment variables
    export API_BASE_URL="http://localhost:8080"
    export PLAYWRIGHT_BASE_URL="http://localhost:5173"
    export SKIP_WEBSERVER="true"
    
    # Run tests
    npx playwright test --project=chromium
    
    cd ..
    print_success "E2E tests complete"
}

# Function to show test results
show_results() {
    print_success "========================================="
    print_success "All tests completed successfully!"
    print_success "========================================="
    
    if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "e2e" ]; then
        print_info "E2E Report: portal/playwright-report/index.html"
    fi
}

# Main execution
trap cleanup EXIT

print_info "Starting PII Redacta Test Suite"
print_info "Test type: $TEST_TYPE"

case $TEST_TYPE in
    unit)
        run_unit_tests
        ;;
    backend)
        setup_infrastructure
        run_backend_tests
        ;;
    e2e)
        setup_infrastructure
        run_e2e_tests
        ;;
    all)
        run_unit_tests
        setup_infrastructure
        run_backend_tests
        run_e2e_tests
        show_results
        ;;
    *)
        print_error "Unknown test type: $TEST_TYPE"
        print_info "Usage: $0 [unit|backend|e2e|all]"
        exit 1
        ;;
esac

print_success "Test run complete!"
