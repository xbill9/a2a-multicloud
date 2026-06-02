# Makefile for the Rust project

# Variables
SERVICE_NAME := bench-rust
APP_DIR := .

.PHONY: all build release run start clean format check-fmt lint test deps help card a2a endpoint

# Target to get the master agent endpoint URL
endpoint:
	@echo "http://localhost:$${PORT:-8100}"

# Target to fetch the agent card
card:
	@PORT=$${PORT:-8100}; \
	echo "Fetching master agent card from http://localhost:$$PORT..."; \
	curl -s http://localhost:$$PORT/.well-known/agent-card.json | jq . 2>/dev/null || curl -s http://localhost:$$PORT/.well-known/agent-card.json

# Target to send an A2A message to get the status of the master agent
a2a:
	@PORT=$${PORT:-8100}; \
	echo "Sending A2A status request to http://localhost:$$PORT/..."; \
	PAYLOAD='{"jsonrpc": "2.0", "id": 1, "method": "message/send", "params": {"message": {"kind": "message", "messageId": "status-query-id", "role": "user", "parts": [{"kind": "text", "text": "status"}], "contextId": "status-context-id"}}}'; \
	curl -s -m 5 -X POST -H "Content-Type: application/json" -d "$$PAYLOAD" http://localhost:$$PORT/ | jq . 2>/dev/null || curl -s -m 5 -X POST -H "Content-Type: application/json" -d "$$PAYLOAD" http://localhost:$$PORT/ || echo "Error: Failed to fetch status from http://localhost:$$PORT"


# Build the project for development
build:
	@echo "Building the Rust project..."
	@cargo build

# Build the project for release
release:
	@echo "Building the Rust project in release mode..."
	@cargo build --release

# Run the project
run:
	@echo "Running the Rust project..."
	@cargo run

# Start the Rust MCP/A2A server
start:
	@echo "Starting the Rust MCP/A2A server..."
	@cargo run

# Clean the project
clean:
	@echo "Cleaning the project..."
	@cargo clean

# Run tests
test:
	@echo "Running tests..."
	@cargo test

# Format the code
format:
	@echo "Formatting code..."
	@cargo fmt

# Check formatting
check-fmt:
	@echo "Checking formatting..."
	@cargo fmt -- --check

# Lint the code using clippy
lint:
	@echo "Linting the Rust project..."
	@cargo clippy --all-targets --all-features -- -D warnings
	@cargo fmt -- --check

# Install/update dependencies
deps:
	@echo "Checking cargo dependencies..."
	@cargo check
