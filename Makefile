.PHONY: help build test run clean docker-up docker-down docker-logs check fmt clippy

# Default target
help:
	@echo "Available targets:"
	@echo "  build       - Build all services"
	@echo "  test        - Run all tests"
	@echo "  run         - Run the API gateway service"
	@echo "  clean       - Clean build artifacts"
	@echo "  docker-up   - Start all services with Docker Compose"
	@echo "  docker-down - Stop all Docker services"
	@echo "  docker-logs - Show Docker logs"
	@echo "  check       - Run cargo check"
	@echo "  fmt         - Format code"
	@echo "  clippy      - Run clippy linter"

# Build all services
build:
	cargo build --workspace

# Build in release mode
build-release:
	cargo build --workspace --release

# Run tests
test:
	cargo test --workspace

# Run the API gateway service
run:
	ENVIRONMENT=development cargo run --bin elementa-api-gateway

# Clean build artifacts
clean:
	cargo clean

# Start all services with Docker Compose
docker-up:
	docker-compose up -d

# Stop all Docker services
docker-down:
	docker-compose down

# Show Docker logs
docker-logs:
	docker-compose logs -f

# Run cargo check
check:
	cargo check --workspace

# Format code
fmt:
	cargo fmt --all

# Run clippy linter
clippy:
	cargo clippy --workspace -- -D warnings

# Setup development environment
setup-dev:
	@echo "Setting up development environment..."
	@if [ ! -f .env ]; then cp .env.example .env; echo "Created .env file from template"; fi
	@echo "Starting databases..."
	docker-compose up -d postgres mongodb redis
	@echo "Waiting for databases to be ready..."
	@sleep 10
	@echo "Development environment ready!"

# Run database migrations
migrate:
	cargo run --bin elementa-api-gateway -- migrate

# Reset database (WARNING: This will delete all data)
reset-db:
	docker-compose down -v
	docker-compose up -d postgres mongodb redis
	@sleep 10
	$(MAKE) migrate