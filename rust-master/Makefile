# Makefile for the Rust project

# Variables
SERVICE_NAME := bench-rust
APP_DIR := .

.PHONY: all build run start clean format check-fmt lint test deps help

# The default target
all: build

# Build the project for development
build:
	@echo "Building the Rust project..."
	@cargo build

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
