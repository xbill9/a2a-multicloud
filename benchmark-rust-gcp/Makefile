# Makefile for the Rust project

# Variables
SERVICE_NAME := bench-rust
APP_DIR := .
PORT ?= 8104
REGION := us-central1

.PHONY: all build release run start clean format check-fmt clippy lint test test-a2a deps docker-build deploy help status gcp-status card gcp-card a2a endpoint

# The default target
all: build

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

# Run the server on a TCP/IP port
start:
	@echo "Starting the server on port $(PORT)..."
	@PORT=$(PORT) cargo run

# Clean the project
clean:
	@echo "Cleaning the project..."
	@cargo clean

# Run tests
test:
	@echo "Running tests..."
	@cargo test

# Run A2A integration test
test-a2a:
	@echo "Running A2A integration test..."
	@./test-a2a.sh

status: gcp-status

gcp-status:
	@echo "Checking Google Cloud Run service status for $(SERVICE_NAME)..."
	@if gcloud run services describe $(SERVICE_NAME) --region $(REGION) >/dev/null 2>&1; then \
		gcloud run services describe $(SERVICE_NAME) --region $(REGION) --format 'table(metadata.name, status.conditions[0].status, status.url)'; \
		ENDPOINT_URL=$$(gcloud run services describe $(SERVICE_NAME) --region $(REGION) --format 'value(status.url)' 2>/dev/null); \
		if [ -z "$$ENDPOINT_URL" ] || [ "$$ENDPOINT_URL" = "null" ]; then \
			ENDPOINT_URL="https://bench-rust-289270257791.us-central1.run.app"; \
		fi; \
		if [ -n "$$ENDPOINT_URL" ] && [ "$$ENDPOINT_URL" != "None" ] && [ "$$ENDPOINT_URL" != "null" ]; then \
			ENDPOINT_URL=$${ENDPOINT_URL%/}; \
			echo "Checking deployed agent health at $$ENDPOINT_URL/.well-known/agent.json ..."; \
			if gcloud auth print-identity-token >/dev/null 2>&1; then \
				TOKEN=$$(gcloud auth print-identity-token 2>/dev/null); \
				HEALTH_RESP=$$(curl -s -H "Authorization: Bearer $$TOKEN" -w "%{http_code}" -o /dev/null "$$ENDPOINT_URL/.well-known/agent.json"); \
			else \
				HEALTH_RESP=$$(curl -s -w "%{http_code}" -o /dev/null "$$ENDPOINT_URL/.well-known/agent.json"); \
			fi; \
			if [ "$$HEALTH_RESP" = "200" ]; then \
				echo "Agent health check: PASSED (HTTP 200)"; \
			else \
				echo "Agent health check: FAILED (HTTP $$HEALTH_RESP)"; \
			fi; \
		fi; \
	else \
		echo "Cloud Run service $(SERVICE_NAME) not found."; \
	fi

# Target to get the public endpoint URL
endpoint:
	@URL=$$(gcloud run services describe $(SERVICE_NAME) --region $(REGION) --format 'value(status.url)' 2>/dev/null); \
	if [ -z "$$URL" ] || [ "$$URL" = "null" ] || [ "$$URL" = "None" ]; then \
		echo "$${GCP_AGENT_URL:-https://bench-rust-289270257791.us-central1.run.app}" | tr -d '"'; \
	else \
		echo "$$URL" | tr -d '"'; \
	fi


card: gcp-card

gcp-card:
	@ENDPOINT_URL=$${GCP_AGENT_URL}; \
	if [ -z "$$ENDPOINT_URL" ]; then \
		ENDPOINT_URL=$$(gcloud run services describe $(SERVICE_NAME) --region $(REGION) --format 'value(status.url)' 2>/dev/null); \
	fi; \
	if [ -z "$$ENDPOINT_URL" ] || [ "$$ENDPOINT_URL" = "null" ]; then \
		ENDPOINT_URL="https://bench-rust-289270257791.us-central1.run.app"; \
	fi; \
	ENDPOINT_URL=$${ENDPOINT_URL%/}; \
	echo "Fetching agent card from $$ENDPOINT_URL/.well-known/agent-card.json ..."; \
	HEADERS=""; \
	if gcloud auth print-identity-token >/dev/null 2>&1; then \
		TOKEN=$$(gcloud auth print-identity-token 2>/dev/null); \
		HEADERS="-H \"Authorization: Bearer $$TOKEN\""; \
	fi; \
	eval "curl -s -m 5 $$HEADERS \"$$ENDPOINT_URL/.well-known/agent-card.json\"" | jq . 2>/dev/null || eval "curl -s -m 5 $$HEADERS \"$$ENDPOINT_URL/.well-known/agent-card.json\"" || echo "Error: Failed to fetch card from $$ENDPOINT_URL"

# Target to send an A2A message to get the status of the GCP deployed agent
a2a:
	@ENDPOINT_URL=$${GCP_AGENT_URL}; \
	if [ -z "$$ENDPOINT_URL" ]; then \
		ENDPOINT_URL=$$(gcloud run services describe $(SERVICE_NAME) --region $(REGION) --format 'value(status.url)' 2>/dev/null); \
	fi; \
	if [ -z "$$ENDPOINT_URL" ] || [ "$$ENDPOINT_URL" = "null" ]; then \
		ENDPOINT_URL="https://bench-rust-289270257791.us-central1.run.app"; \
	fi; \
	ENDPOINT_URL=$${ENDPOINT_URL%/}; \
	echo "Sending A2A status request to $$ENDPOINT_URL/ ..."; \
	HEADERS=""; \
	if gcloud auth print-identity-token >/dev/null 2>&1; then \
		TOKEN=$$(gcloud auth print-identity-token 2>/dev/null); \
		HEADERS="-H \"Authorization: Bearer $$TOKEN\""; \
	fi; \
	PAYLOAD='{"jsonrpc": "2.0", "id": 1, "method": "message/send", "params": {"message": {"kind": "message", "messageId": "status-query-id", "role": "user", "parts": [{"kind": "text", "text": "status"}], "contextId": "status-context-id"}}}'; \
	eval "curl -s -m 5 -X POST -H \"Content-Type: application/json\" $$HEADERS -d '$$PAYLOAD' \"$$ENDPOINT_URL/\"" | jq . 2>/dev/null || eval "curl -s -m 5 -X POST -H \"Content-Type: application/json\" $$HEADERS -d '$$PAYLOAD' \"$$ENDPOINT_URL/\"" || echo "Error: Failed to fetch status from $$ENDPOINT_URL"


# Format the code
format:
	@echo "Formatting code..."
	@cargo fmt

# Check formatting
check-fmt:
	@echo "Checking formatting..."
	@cargo fmt -- --check

# Lint the code using clippy
clippy:
	@echo "Linting code..."
	@cargo clippy -- -D warnings

lint: clippy

# Install/update dependencies
deps:
	@echo "Checking cargo dependencies..."
	@cargo check

# Build the Docker image
docker-build:
	@echo "Building the Docker image..."
	@docker build -t gcr.io/$(shell gcloud config get-value project)/$(SERVICE_NAME):latest .

# Submit the build to Google Cloud Build
deploy:
	@echo "Submitting build to Google Cloud Build..."
	@gcloud builds submit . --config cloudbuild.yaml

help:
	@echo "Makefile for the Rust project"
	@echo ""
	@echo "Usage:"
	@echo "    make <target>"
	@echo ""
	@echo "Targets:"
	@echo "    all          (default) same as 'build'"
	@echo "    build        Build the project for development"
	@echo "    run          Run the project"
	@echo "    start        Start the server on port $(PORT)"
	@echo "    clean        Clean the project"
	@echo "    test         Run tests"
	@echo "    test-a2a     Run A2A integration tests against GCP deployment"
	@echo "    status       Get the GCP deployed agent status and health"
	@echo "    endpoint     Get the public endpoint URL"
	@echo "    card         Fetch the GCP deployed agent card"
	@echo "    a2a          Send an A2A status request to the GCP deployed agent"
	@echo "    format       Format the code"
	@echo "    check-fmt    Check formatting"
	@echo "    deps         Check dependencies"
	@echo "    docker-build Build the Docker image"
	@echo "    deploy       Submit the build to Google Cloud Build"
