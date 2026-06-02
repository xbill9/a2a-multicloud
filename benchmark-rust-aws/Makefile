# Makefile for the Rust project

# Include and export AWS credentials if they exist
-include .aws_creds
export

export AWS_PAGER :=

# Variables
SERVICE_NAME := mcp-lightsail-rust-aws

# AWS Lightsail Configuration
AWS_REGION ?= us-east-1
LIGHTSAIL_SERVICE_NAME ?= a2a-lightsail-rust-aws
IMAGE_NAME ?= mcp-server-image

.PHONY: all build run clean release test test-a2a fmt clippy check docker-build deploy-lightsail start stop status help lightsail-status endpoint push-lightsail create-deployment aws-destroy lightsail deploy a2a card aws-card

# The default target
all: build

# Build the project for development
build:
	@echo "Building the Rust project..."
	@cargo build

# Run the project
run:
	@echo "Running the Rust project..."
	@cargo run --release

# Start the server in the background
start: release
	@echo "Starting the MCP server..."
	@nohup ./target/release/$(SERVICE_NAME) > server.log 2>&1 & echo $$! > server.pid
	@echo "Server started with PID $$(cat server.pid)"

# Stop the background server
stop:
	@if [ -f server.pid ]; then \
		PID=$$(cat server.pid); \
		echo "Stopping $(SERVICE_NAME) (PID $$PID)..."; \
		kill $$PID || true; \
		rm server.pid; \
	else \
		echo "$(SERVICE_NAME) is not running."; \
	fi

# Get the status of the Lightsail deployed agent
status: lightsail-status
	@if URL=$$(aws lightsail get-container-services --service-name $(LIGHTSAIL_SERVICE_NAME) --region $(AWS_REGION) --query 'containerServices[0].url' --output text 2>/dev/null); then \
		if [ -n "$$URL" ] && [ "$$URL" != "None" ] && [ "$$URL" != "null" ]; then \
			URL=$${URL%/}; \
			echo "Checking endpoint health at $$URL/health..."; \
			HEALTH_RESP=$$(curl -s -m 5 -w "%{http_code}" -o /dev/null "$$URL/health"); \
			if [ "$$HEALTH_RESP" = "200" ]; then \
				echo "Health status: HEALTHY (HTTP 200)"; \
			else \
				echo "Health status: UNHEALTHY (HTTP $$HEALTH_RESP)"; \
			fi; \
		else \
			echo "No public endpoint URL available to health check."; \
		fi; \
	else \
		echo "Skipping endpoint health check due to AWS query failure."; \
	fi

# Clean the project
clean:
	@echo "Cleaning the project..."
	@cargo clean
	@rm -f server.pid server.log

# Build the project for release
release:
	@echo "Building Release..."
	@cargo build --release

# Run tests
test:
	@echo "Running tests..."
	@cargo test

test-a2a:
	@echo "Running A2A integration test..."
	@./test-a2a.sh

# Format the code
fmt:
	@echo "Formatting code..."
	@cargo fmt --all -- --check

# Lint the code
clippy:
	@echo "Linting code..."
	@cargo clippy -- -D warnings

# Lint the code
lint:
	@echo "Linting code..."
	@cargo clippy -- -D warnings

# Check the code
check:
	@echo "Checking the code..."
	@cargo check

# Build the Docker image
docker-build:
	@echo "Building the Docker image..."
	@docker build -t $(IMAGE_NAME) .

# Deploy to Amazon Lightsail
deploy-lightsail: docker-build push-lightsail create-deployment
lightsail: deploy-lightsail
deploy: lightsail

# Target to show AWS Lightsail status
lightsail-status:
	@echo "Checking AWS Lightsail service status for $(LIGHTSAIL_SERVICE_NAME)..."
	@aws lightsail get-container-services --service-name $(LIGHTSAIL_SERVICE_NAME) --region $(AWS_REGION) \
		--query 'containerServices[0].{State:state, Power:power, Deployment:currentDeployment.state, URL:url}' \
		--output table 2>/dev/null || echo "Unable to query Lightsail service status (AWS credentials may be expired or unconfigured)."

# Target to get the public endpoint URL
endpoint:
	@URL=$$(aws lightsail get-container-services --service-name $(LIGHTSAIL_SERVICE_NAME) --region $(AWS_REGION) --query 'containerServices[0].url' --output text 2>/dev/null); \
	if [ -z "$$URL" ] || [ "$$URL" = "None" ] || [ "$$URL" = "null" ]; then \
		echo "$${AWS_AGENT_URL:-https://a2a-lightsail-rust-aws.6wpv8vensby5c.us-east-1.cs.amazonlightsail.com}" | tr -d '"'; \
	else \
		echo "$$URL" | tr -d '"'; \
	fi


push-lightsail:
	@echo "Pushing the Docker image to Lightsail..."
	@aws lightsail push-container-image --region $(AWS_REGION) --service-name $(LIGHTSAIL_SERVICE_NAME) --label $(IMAGE_NAME) --image $(IMAGE_NAME)

create-deployment:
	@echo "Creating a new deployment for Lightsail Container Service..."
	@IMAGE_IDENTIFIER=$$(aws lightsail get-container-images --service-name $(LIGHTSAIL_SERVICE_NAME) --query "containerImages[0].image" --output text) && \
	URL=$$(aws lightsail get-container-services --service-name $(LIGHTSAIL_SERVICE_NAME) --region $(AWS_REGION) --query 'containerServices[0].url' --output text | sed 's|https://||;s|/||') && \
	aws lightsail create-container-service-deployment --region $(AWS_REGION) \
		--service-name $(LIGHTSAIL_SERVICE_NAME) \
		--containers "{\"$(LIGHTSAIL_SERVICE_NAME)\":{\"image\":\"$$IMAGE_IDENTIFIER\",\"ports\":{\"8080\":\"HTTP\"},\"environment\":{\"ALLOWED_HOSTS\":\"*,$$URL,0.0.0.0,localhost,127.0.0.1\"}}}" \
		--public-endpoint "{\"containerName\":\"$(LIGHTSAIL_SERVICE_NAME)\",\"containerPort\":8080,\"healthCheck\":{\"path\":\"/health\"}}"

# Target to send an A2A message to get the status of the AWS deployed agent
a2a:
	@URL=$$(aws lightsail get-container-services --service-name $(LIGHTSAIL_SERVICE_NAME) --region $(AWS_REGION) --query 'containerServices[0].url' --output text 2>/dev/null); \
	if [ -z "$$URL" ] || [ "$$URL" = "None" ] || [ "$$URL" = "null" ]; then \
		URL="$${AWS_AGENT_URL:-https://a2a-lightsail-rust-aws.6wpv8vensby5c.us-east-1.cs.amazonlightsail.com}"; \
	fi; \
	URL=$${URL%/}; \
	echo "Sending A2A status request to $$URL/..."; \
	PAYLOAD='{"jsonrpc": "2.0", "id": 1, "method": "message/send", "params": {"message": {"kind": "message", "messageId": "status-query-id", "role": "user", "parts": [{"kind": "text", "text": "status"}], "contextId": "status-context-id"}}}'; \
	curl -s -m 5 -X POST -H "Content-Type: application/json" -d "$$PAYLOAD" "$$URL/" | jq . 2>/dev/null || curl -s -m 5 -X POST -H "Content-Type: application/json" -d "$$PAYLOAD" "$$URL/" || echo "Error: Failed to fetch status from $$URL"

card: aws-card

aws-card:
	@URL=$$(aws lightsail get-container-services --service-name $(LIGHTSAIL_SERVICE_NAME) --region $(AWS_REGION) --query 'containerServices[0].url' --output text 2>/dev/null); \
	if [ -z "$$URL" ] || [ "$$URL" = "None" ] || [ "$$URL" = "null" ]; then \
		URL="$${AWS_AGENT_URL:-https://a2a-lightsail-rust-aws.6wpv8vensby5c.us-east-1.cs.amazonlightsail.com}"; \
	fi; \
	URL=$${URL%/}; \
	echo "Fetching agent card from $$URL/.well-known/agent-card.json ..."; \
	curl -s -m 5 "$$URL/.well-known/agent-card.json" | jq . 2>/dev/null || curl -s -m 5 "$$URL/.well-known/agent-card.json" || echo "Error: Failed to fetch card from $$URL"

aws-destroy:
	@echo "Destroying AWS resources for $(LIGHTSAIL_SERVICE_NAME)..."
	@aws lightsail delete-container-service --service-name $(LIGHTSAIL_SERVICE_NAME) --region $(AWS_REGION) 2>/dev/null || true
	@echo "AWS resources destroyed."

help:
	@echo "Makefile for the Rust project"
	@echo ""
	@echo "Usage:"
	@echo "    make <target>"
	@echo ""
	@echo "Targets:"
	@echo "    all              (default) same as 'build'"
	@echo "    build            Build the project for development"
	@echo "    run              Run the project"
	@echo "    start            Start the server in the background"
	@echo "    stop             Stop the background server"
	@echo "    status           Get the status and health of the Lightsail deployed agent"
	@echo "    a2a              Send an A2A status request to the AWS deployed agent"
	@echo "    card             Fetch the AWS deployed agent card"
	@echo "    clean            Clean the project"
	@echo "    release          Build the project for release"
	@echo "    test             Run tests"
	@echo "    test-a2a         Run A2A integration tests against AWS deployment"
	@echo "    fmt              Check formatting"
	@echo "    clippy           Lint the code"
	@echo "    check            Check the code"
	@echo "    docker-build     Build the Docker image"
	@echo "    deploy-lightsail Deploy to Amazon Lightsail"
	@echo "    lightsail-status Check AWS Lightsail service status"
	@echo "    endpoint         Get the public endpoint URL"
	@echo "    aws-destroy      Destroy AWS Lightsail resources"
