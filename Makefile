# Makefile for Rust A2A Multi-Cloud Benchmark Setup

.PHONY: all build start test test-aws test-gcp test-azure status git-status clean

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

clean:
	@echo "Cleaning up..."
	@$(MAKE) -C rust-master clean
	@$(MAKE) -C benchmark-rust clean
