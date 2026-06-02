# Makefile for Rust A2A Multi-Cloud Benchmark Setup

-include .env

.PHONY: all build release start lint test test-aws test-gcp test-azure status git-status card card-gcp card-aws a2a a2a-aws endpoint clean

all: build

# Target to start the rust MCP/A2A server
start:
	@echo "Starting the Rust MCP/A2A server..."
	@bash ./a2a-master-rust.sh

# Target to build the application
build:
	@echo "Building Rust Master..."
	@$(MAKE) -C rust-master build
	@echo "Building Rust Benchmark Agent..."
	@$(MAKE) -C benchmark-rust build

# Target to build the application for release
release:
	@echo "Building Rust Master in release mode..."
	@$(MAKE) -C rust-master release
	@echo "Building Rust Benchmark Agent in release mode..."
	@$(MAKE) -C benchmark-rust release

# Target to lint the code
lint:
	@echo "Linting Rust Master..."
	@$(MAKE) -C rust-master lint
	@echo "Linting Rust Benchmark Agent..."
	@$(MAKE) -C benchmark-rust lint

# Target to run tests
test:
	@echo "Running tests for Rust Master..."
	@$(MAKE) -C rust-master test
	@echo "Running tests for Rust Benchmark Agent..."
	@$(MAKE) -C benchmark-rust test

test-aws:
	@echo "Running A2A tests for AWS..."
	@$(MAKE) -C benchmark-rust-aws test-a2a

test-gcp:
	@echo "Running A2A tests for GCP..."
	@$(MAKE) -C benchmark-rust-gcp test-a2a

card-gcp:
	@echo "Fetching GCP Agent Card..."
	@$(MAKE) -C benchmark-rust-gcp card

card-aws:
	@echo "Fetching AWS Agent Card..."
	@$(MAKE) -C benchmark-rust-aws card

a2a-aws:
	@echo "Sending A2A status request to AWS Agent..."
	@$(MAKE) -C benchmark-rust-aws a2a

test-azure:
	@echo "Running A2A tests for Azure..."
	@$(MAKE) -C benchmark-rust-azure test-a2a

# Target to show git status
git-status:
	@echo "Showing git status..."
	@git status

# Target to check status of all deployed agents (AWS, GCP, Azure)
status:
	@echo "========================================="
	@echo "Checking AWS Deployed Agent Status..."
	@echo "========================================="
	-@$(MAKE) -C benchmark-rust-aws status
	@echo ""
	@echo "========================================="
	@echo "Checking GCP Deployed Agent Status..."
	@echo "========================================="
	-@$(MAKE) -C benchmark-rust-gcp status
	@echo ""
	@echo "========================================="
	@echo "Checking Azure Deployed Agent Status..."
	@echo "========================================="
	-@AZ_RESOURCE_GROUP=a2a-rg-westus2 AZ_ACA_NAME=a2a-app-penguin AZ_ACR_NAME=a2aacrpenguinv2 AZ_ACA_ENV_NAME=a2a-env-penguin $(MAKE) -C benchmark-rust-azure status

# Target to fetch the agent card (options: AGENT=master|local|aws|gcp|azure|all|<url>)
card:
	@AGENT=$${AGENT:-master}; \
	if [ "$$AGENT" = "all" ]; then \
		echo "========================================="; \
		echo "Fetching Agent Card: Master Agent (local)..."; \
		echo "========================================="; \
		$(MAKE) card AGENT=master; \
		echo ""; \
		echo "========================================="; \
		echo "Fetching Agent Card: Local Benchmark Agent..."; \
		echo "========================================="; \
		$(MAKE) card AGENT=local; \
		echo ""; \
		echo "========================================="; \
		echo "Fetching Agent Card: AWS (Remote)..."; \
		echo "========================================="; \
		$(MAKE) card AGENT=aws; \
		echo ""; \
		echo "========================================="; \
		echo "Fetching Agent Card: GCP (Remote)..."; \
		echo "========================================="; \
		$(MAKE) card AGENT=gcp; \
		echo ""; \
		echo "========================================="; \
		echo "Fetching Agent Card: Azure (Remote)..."; \
		echo "========================================="; \
		$(MAKE) card AGENT=azure; \
	else \
		if [ "$$AGENT" = "master" ]; then \
			URL="http://localhost:8100"; \
		elif [ "$$AGENT" = "local" ]; then \
			URL="http://localhost:8104"; \
		elif [ "$$AGENT" = "aws" ]; then \
			URL="$${AWS_AGENT_URL:-https://a2a-lightsail-rust-aws.6wpv8vensby5c.us-east-1.cs.amazonlightsail.com}"; \
		elif [ "$$AGENT" = "gcp" ]; then \
			URL="$${GCP_AGENT_URL:-https://bench-rust-289270257791.us-central1.run.app}"; \
		elif [ "$$AGENT" = "azure" ]; then \
			URL="$${AZURE_AGENT_URL:-https://a2a-app-penguin.icyplant-a768d75c.westus2.azurecontainerapps.io}"; \
		else \
			URL="$$AGENT"; \
		fi; \
		URL=$${URL%/}; \
		echo "Fetching agent card from $$URL/.well-known/agent-card.json ..."; \
		HEADERS=""; \
		if [ "$$AGENT" = "gcp" ] || echo "$$URL" | grep -q "run.app"; then \
			if gcloud auth print-identity-token >/dev/null 2>&1; then \
				TOKEN=$$(gcloud auth print-identity-token 2>/dev/null); \
				HEADERS="-H \"Authorization: Bearer $$TOKEN\""; \
			fi; \
		fi; \
		eval "curl -s -m 5 $$HEADERS \"$$URL/.well-known/agent-card.json\"" | jq . 2>/dev/null || eval "curl -s -m 5 $$HEADERS \"$$URL/.well-known/agent-card.json\"" || echo "Error: Failed to fetch card from $$URL"; \
	fi

# Target to send an A2A message to get the status (options: AGENT=master|local|aws|gcp|azure|all|<url>)
a2a:
	@AGENT=$${AGENT:-local}; \
	if [ "$$AGENT" = "all" ]; then \
		echo "========================================="; \
		echo "Fetching Status: Master Agent (local)..."; \
		echo "========================================="; \
		$(MAKE) a2a AGENT=master; \
		echo ""; \
		echo "========================================="; \
		echo "Fetching Status: Local Benchmark Agent..."; \
		echo "========================================="; \
		$(MAKE) a2a AGENT=local; \
		echo ""; \
		echo "========================================="; \
		echo "Fetching Status: AWS (Remote)..."; \
		echo "========================================="; \
		$(MAKE) a2a AGENT=aws; \
		echo ""; \
		echo "========================================="; \
		echo "Fetching Status: GCP (Remote)..."; \
		echo "========================================="; \
		$(MAKE) a2a AGENT=gcp; \
		echo ""; \
		echo "========================================="; \
		echo "Fetching Status: Azure (Remote)..."; \
		echo "========================================="; \
		$(MAKE) a2a AGENT=azure; \
	else \
		if [ "$$AGENT" = "master" ]; then \
			URL="http://localhost:8100"; \
		elif [ "$$AGENT" = "local" ]; then \
			URL="http://localhost:8104"; \
		elif [ "$$AGENT" = "aws" ]; then \
			URL="$${AWS_AGENT_URL:-https://a2a-lightsail-rust-aws.6wpv8vensby5c.us-east-1.cs.amazonlightsail.com}"; \
		elif [ "$$AGENT" = "gcp" ]; then \
			URL="$${GCP_AGENT_URL:-https://bench-rust-289270257791.us-central1.run.app}"; \
		elif [ "$$AGENT" = "azure" ]; then \
			URL="$${AZURE_AGENT_URL:-https://a2a-app-penguin.icyplant-a768d75c.westus2.azurecontainerapps.io}"; \
		else \
			URL="$$AGENT"; \
		fi; \
		URL=$${URL%/}; \
		echo "Sending A2A status request to $$URL/ ..."; \
		HEADERS=""; \
		if [ "$$AGENT" = "gcp" ] || echo "$$URL" | grep -q "run.app"; then \
			if gcloud auth print-identity-token >/dev/null 2>&1; then \
				TOKEN=$$(gcloud auth print-identity-token 2>/dev/null); \
				HEADERS="-H \"Authorization: Bearer $$TOKEN\""; \
			fi; \
		fi; \
		PAYLOAD='{"jsonrpc": "2.0", "id": 1, "method": "message/send", "params": {"message": {"kind": "message", "messageId": "status-query-id", "role": "user", "parts": [{"kind": "text", "text": "status"}], "contextId": "status-context-id"}}}'; \
		eval "curl -s -m 5 -X POST -H \"Content-Type: application/json\" $$HEADERS -d '$$PAYLOAD' \"$$URL/\"" | jq . 2>/dev/null || eval "curl -s -m 5 -X POST -H \"Content-Type: application/json\" $$HEADERS -d '$$PAYLOAD' \"$$URL/\"" || echo "Error: Failed to fetch status from $$URL"; \
	fi

# Target to get the public endpoint URL (options: AGENT=master|local|aws|gcp|azure|all, defaults to all)
endpoint:
	@AGENT=$${AGENT:-all}; \
	if [ "$$AGENT" = "all" ]; then \
		MASTER_URL=$$( $(MAKE) --no-print-directory endpoint AGENT=master ); \
		LOCAL_URL=$$( $(MAKE) --no-print-directory endpoint AGENT=local ); \
		AWS_URL=$$( $(MAKE) --no-print-directory -C benchmark-rust-aws endpoint ); \
		GCP_URL=$$( $(MAKE) --no-print-directory -C benchmark-rust-gcp endpoint ); \
		AZURE_URL=$$( AZ_RESOURCE_GROUP=a2a-rg-westus2 AZ_ACA_NAME=a2a-app-penguin AZ_ACR_NAME=a2aacrpenguinv2 AZ_ACA_ENV_NAME=a2a-env-penguin $(MAKE) --no-print-directory -C benchmark-rust-azure endpoint ); \
		echo "========================================="; \
		echo "Endpoints of all agents:"; \
		echo "========================================="; \
		printf "Master Agent (local):     %s\n" "$$MASTER_URL"; \
		printf "Local Benchmark Agent:    %s\n" "$$LOCAL_URL"; \
		printf "AWS (Remote):             %s\n" "$$AWS_URL"; \
		printf "GCP (Remote):             %s\n" "$$GCP_URL"; \
		printf "Azure (Remote):           %s\n" "$$AZURE_URL"; \
	elif [ "$$AGENT" = "master" ]; then \
		$(MAKE) --no-print-directory -C rust-master endpoint; \
	elif [ "$$AGENT" = "local" ]; then \
		$(MAKE) --no-print-directory -C benchmark-rust endpoint; \
	elif [ "$$AGENT" = "aws" ]; then \
		$(MAKE) --no-print-directory -C benchmark-rust-aws endpoint; \
	elif [ "$$AGENT" = "gcp" ]; then \
		$(MAKE) --no-print-directory -C benchmark-rust-gcp endpoint; \
	elif [ "$$AGENT" = "azure" ]; then \
		AZ_RESOURCE_GROUP=a2a-rg-westus2 AZ_ACA_NAME=a2a-app-penguin AZ_ACR_NAME=a2aacrpenguinv2 AZ_ACA_ENV_NAME=a2a-env-penguin $(MAKE) --no-print-directory -C benchmark-rust-azure endpoint; \
	else \
		echo "Unknown agent: $$AGENT"; \
	fi


clean:
	@echo "Cleaning up..."
	@$(MAKE) -C rust-master clean
	@$(MAKE) -C benchmark-rust clean
