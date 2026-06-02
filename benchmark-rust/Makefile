# Makefile for the Rust project

# Variables
SERVICE_NAME := bench-rust
APP_DIR := .
PORT ?= 8104
PID_FILE := local_agent.pid
LOG_FILE := local_agent.log
GET_PID = (lsof -t -i TCP:$(PORT) -sTCP:LISTEN 2>/dev/null || ss -tlnp 2>/dev/null | grep -E "[:\s]$(PORT)\s" | grep -o -E "pid=[0-9]+" | cut -d= -f2) | head -n 1

.PHONY: all build release run start stop status clean format check-fmt lint test deps help card a2a endpoint

# Target to get the local agent endpoint URL
endpoint:
	@echo "http://localhost:$${PORT:-$(PORT)}"

# The default target
all: build

# Target to fetch the agent card
card:
	@PORT=$${PORT:-$(PORT)}; \
	echo "Fetching local benchmark agent card from http://localhost:$$PORT..."; \
	curl -s http://localhost:$$PORT/.well-known/agent-card.json | jq . 2>/dev/null || curl -s http://localhost:$$PORT/.well-known/agent-card.json

# Target to send an A2A message to get the status of the local agent
a2a:
	@PORT=$${PORT:-$(PORT)}; \
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

# Run the server on a TCP/IP port in the background
start: build
	@PID=$$( $(GET_PID) ); \
	if [ -n "$$PID" ]; then \
		echo "Local agent is already running (PID: $$PID on port $(PORT))"; \
		if [ ! -f $(PID_FILE) ]; then \
			echo $$PID > $(PID_FILE); \
		fi \
	else \
		echo "Starting local agent on port $(PORT) in the background..."; \
		PORT=$(PORT) setsid ./target/debug/benchmark-rust > $(LOG_FILE) 2>&1 & echo $$! > $(PID_FILE); \
		sleep 1; \
		if [ -f $(PID_FILE) ] && kill -0 $$(cat $(PID_FILE)) 2>/dev/null; then \
			echo "Local agent started successfully (PID: $$(cat $(PID_FILE)))."; \
		else \
			echo "Failed to start local agent. Check $(LOG_FILE) for details."; \
			rm -f $(PID_FILE); \
		fi \
	fi

# Stop the running local agent
stop:
	@if [ -f $(PID_FILE) ]; then \
		PID=$$(cat $(PID_FILE)); \
		if kill -0 $$PID 2>/dev/null; then \
			echo "Stopping local agent (PID: $$PID)..."; \
			kill $$PID; \
			sleep 1; \
			if kill -0 $$PID 2>/dev/null; then \
				echo "Force killing local agent (PID: $$PID)..."; \
				kill -9 $$PID; \
			fi; \
			echo "Local agent stopped."; \
		else \
			echo "Local agent (PID: $$PID) is not running."; \
		fi; \
		rm -f $(PID_FILE); \
	else \
		PID=$$( $(GET_PID) ); \
		if [ -n "$$PID" ]; then \
			echo "Stopping local agent running on port $(PORT) (PID: $$PID)..."; \
			kill $$PID; \
			sleep 1; \
			if kill -0 $$PID 2>/dev/null; then \
				echo "Force killing local agent (PID: $$PID)..."; \
				kill -9 $$PID; \
			fi; \
			echo "Local agent stopped."; \
		else \
			echo "No running agent detected on port $(PORT)."; \
		fi \
	fi

# Get the status of the local agent
status:
	@if [ -f $(PID_FILE) ]; then \
		PID=$$(cat $(PID_FILE)); \
		if kill -0 $$PID 2>/dev/null; then \
			echo "Local agent is running (PID: $$PID)."; \
			echo "Listening on port: $(PORT)"; \
			if command -v curl >/dev/null 2>&1; then \
				echo "Testing endpoint health..."; \
				curl -s http://localhost:$(PORT)/health || echo "Health check failed."; \
				echo ""; \
			fi \
		else \
			echo "Local agent is not running, but stale PID file exists (PID: $$PID)."; \
		fi \
	else \
		PID=$$( $(GET_PID) ); \
		if [ -n "$$PID" ]; then \
			echo "Local agent is running (PID: $$PID, detected on port $(PORT))."; \
			if command -v curl >/dev/null 2>&1; then \
				echo "Testing endpoint health..."; \
				curl -s http://localhost:$(PORT)/health || echo "Health check failed."; \
				echo ""; \
			fi \
		else \
			echo "Local agent is not running (no PID file or active port)."; \
		fi \
	fi

# Clean the project
clean:
	@echo "Cleaning the project..."
	@cargo clean
	@rm -f $(PID_FILE) $(LOG_FILE)

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
