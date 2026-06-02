# AWS Lightsail Rust Sub-Agent (`benchmark-rust-aws`)

This directory contains the **AWS Lightsail Rust sub-agent** implementation for the Multi-Cloud Agent-to-Agent (A2A) Benchmark system. It exposes high-performance Mersenne prime calculation capabilities and agent status tracking over a standard JSON-RPC A2A interface.

## Table of Contents
- [Architecture & Design](#architecture--design)
- [Key Features](#key-features)
- [Project Structure](#project-structure)
- [Local Development](#local-development)
  - [Prerequisites](#prerequisites)
  - [Building](#building)
  - [Running Locally](#running-locally)
- [Testing](#testing)
  - [Unit Tests](#unit-tests)
  - [A2A Integration Tests](#a2a-integration-tests)
- [Deployment to AWS Lightsail](#deployment-to-aws-lightsail)
  - [Docker Build](#docker-build)
  - [AWS Lightsail Deployment](#aws-lightsail-deployment)
- [Agent Protocol Reference](#agent-protocol-reference)
  - [Agent Card](#agent-card)
  - [A2A message/send API](#a2a-messagesend-api)

---

## Architecture & Design

The agent is implemented in **Rust** using the **Axum** web framework and **Tokio** asynchronous runtime. Since Mersenne prime verification involves computing very large exponents ($2^p - 1$), the calculation code is computationally intensive. 

To prevent blocking the async runtime's reactor thread, CPU-bound calculation jobs are offloaded to OS threads using `tokio::task::spawn_blocking`. Additionally, a `CalculationGuard` ensures that only one heavy calculation runs at any given time, marking the agent's state as `not ready` (busy) when active.

---

## Key Features

- **Lucas-Lehmer Primality Test**: Highly optimized verification of Mersenne exponents using the `num-bigint` and `num-traits` crates for arbitrary-precision arithmetic.
- **Agent-to-Agent (A2A) Compatibility**: Fully implements the JSON-RPC A2A message transmission interface (`message/send`).
- **Self-Documenting Agent Card**: Exposes metadata about capabilities, versioning, and endpoint skills at `.well-known/agent.json` and `.well-known/agent-card.json`.
- **AWS Lightsail Integration**: Designed to be compiled into a minimal distroless Docker container and deployed serverlessly on Amazon Lightsail Container Services.

---

## Project Structure

The codebase is organized as follows:
* [src/main.rs](file:///home/xbill/a2a-multicloud/benchmark-rust-aws/src/main.rs): Core application entrypoint containing router setup, Lucas-Lehmer prime-checking algorithm, and A2A payload handlers.
* [Cargo.toml](file:///home/xbill/a2a-multicloud/benchmark-rust-aws/Cargo.toml): Dependency manifest configuring Axum, Tokio, Serde, and Num-BigInt.
* [Makefile](file:///home/xbill/a2a-multicloud/benchmark-rust-aws/Makefile): Automated shortcuts for development, testing, builds, and AWS deployment operations.
* [Dockerfile](file:///home/xbill/a2a-multicloud/benchmark-rust-aws/Dockerfile): Multi-stage container definition compiling on Rust build image and deploying on `distroless/cc-debian12`.
* [test-a2a.sh](file:///home/xbill/a2a-multicloud/benchmark-rust-aws/test-a2a.sh): Live or local compliance test suite utilizing `curl` and `jq` to validate A2A behaviors.

---

## Local Development

### Prerequisites
- [Rust Toolchain](https://www.rust-lang.org/tools/install) (configured for Edition 2024)
- [AWS CLI](https://aws.amazon.com/cli/) (if deploying/testing cloud components)
- `jq` and `curl` (for testing scripts)

### Building
Compile the project locally in debug mode:
```bash
make build
```
Or clean build outputs:
```bash
make clean
```

### Running Locally
Run the server on the default port:
```bash
make run
```

To run the agent as a background process locally:
```bash
make start
```
To stop the background agent:
```bash
make stop
```

---

## Testing

### Unit Tests
Run standard Rust unit tests:
```bash
make test
```

### A2A Integration Tests
To test the A2A messaging protocol conformance locally or against a deployed service, use the following:
```bash
# Run tests against the currently configured/deployed AWS Lightsail Container service
make test-a2a

# Or manually target a custom endpoint URL (e.g. localhost)
./test-a2a.sh http://localhost:8080
```

---

## Deployment to AWS Lightsail

### Docker Build
To build the Docker image locally:
```bash
make docker-build
```

### AWS Lightsail Deployment
Ensure you have configured your AWS CLI credentials or loaded `.aws_creds` (automatically handled by the Makefile if `.aws_creds` exists).

Build, push the container image to your Lightsail Container Service, and create a new deployment:
```bash
make deploy
# or
make deploy-lightsail
```

Once deployed, you can interact with the cloud service using:
```bash
# Get service status and health check
make status

# Query public endpoint URL
make endpoint

# Send a sample A2A status check message to the deployed agent
make a2a

# Clean up resources and delete the container service
make aws-destroy
```

---

## Agent Protocol Reference

### Agent Card
- **Endpoint**: `GET /.well-known/agent.json` or `GET /.well-known/agent-card.json`
- **Output**: Returns JSON indicating agent name, version, and the skills exposed (`find-mersenne-rust` and `check-status-rust`).

### A2A message/send API
- **Endpoint**: `POST /`
- **Headers**: `Content-Type: application/json`
- **JSON-RPC Request Format**:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "message/send",
    "params": {
      "message": {
        "kind": "message",
        "messageId": "unique-msg-id",
        "role": "user",
        "parts": [
          {
            "kind": "text",
            "text": "Calculate 5"
          }
        ],
        "contextId": "unique-context-id"
      }
    }
  }
  ```

- **Supported Inputs**:
  - **Status Query** (e.g., `"status"`, `"ready"`, `"active"`): Returns `"ready"` or `"not ready"` based on whether a CPU computation is running.
  - **Exponent Specifics** (e.g., `"exponent 31"`, `"exp:127"`, `"p=13"`): Runs the Lucas-Lehmer test on the given exponent. Returns the computed Mersenne prime if verified, or `"not prime"` if invalid.
  - **Generic Numbers** (e.g., `"5"`): Finds and outputs the first $n$ Mersenne primes, along with the elapsed time in milliseconds.
