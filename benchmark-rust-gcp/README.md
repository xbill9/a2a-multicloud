# Google Cloud Run Rust Sub-Agent (`benchmark-rust-gcp`)

This directory contains the **Google Cloud Run Rust sub-agent** implementation for the Multi-Cloud Agent-to-Agent (A2A) Benchmark system. It exposes high-performance Mersenne prime calculation capabilities and agent status tracking over a standard JSON-RPC A2A interface.

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
- [Deployment to Google Cloud Run](#deployment-to-google-cloud-run)
  - [Docker Build](#docker-build)
  - [Cloud Run Deployment](#cloud-run-deployment)
- [Agent Protocol Reference](#agent-protocol-reference)
  - [Agent Card](#agent-card)
  - [A2A message/send API](#a2a-messagesend-api)

---

## Architecture & Design

The agent is implemented in **Rust (2024 Edition)** using the **Axum** web framework and **Tokio** asynchronous runtime. Since Mersenne prime verification involves computing very large exponents ($2^p - 1$), the calculation code is computationally intensive. 

To prevent blocking the async runtime's reactor thread, CPU-bound calculation jobs are offloaded to OS threads using `tokio::task::spawn_blocking`. Additionally, a `CalculationGuard` ensures that only one heavy calculation runs at any given time, marking the agent's state as `not ready` (busy) when active.

---

## Key Features

- **Lucas-Lehmer Primality Test**: Highly optimized verification of Mersenne exponents using the `num-bigint` and `num-traits` crates for arbitrary-precision arithmetic.
- **Agent-to-Agent (A2A) Compatibility**: Fully implements the JSON-RPC A2A message transmission interface (`message/send`).
- **Self-Documenting Agent Card**: Exposes metadata about capabilities, versioning, and endpoint skills at `.well-known/agent.json` and `.well-known/agent-card.json`.
- **Google Cloud Run Integration**: Designed to be compiled into a minimal distroless Docker container and deployed serverlessly on GCP with zero cold-start state.

---

## Project Structure

The codebase is organized as follows:
* [src/main.rs](file:///home/xbill/a2a-multicloud/benchmark-rust-gcp/src/main.rs): Core application entrypoint containing router setup, Lucas-Lehmer prime-checking algorithm, and A2A payload handlers.
* [Cargo.toml](file:///home/xbill/a2a-multicloud/benchmark-rust-gcp/Cargo.toml): Dependency manifest configuring Axum, Tokio, Serde, and Num-BigInt.
* [Makefile](file:///home/xbill/a2a-multicloud/benchmark-rust-gcp/Makefile): Automated shortcuts for development, testing, builds, and Google Cloud operations.
* [Dockerfile](file:///home/xbill/a2a-multicloud/benchmark-rust-gcp/Dockerfile): Multi-stage container definition compiling on Rust build image and deploying on `distroless/cc-debian12`.
* [cloudbuild.yaml](file:///home/xbill/a2a-multicloud/benchmark-rust-gcp/cloudbuild.yaml): Continuous integration and deployment configuration for Google Cloud Build.
* [test-a2a.sh](file:///home/xbill/a2a-multicloud/benchmark-rust-gcp/test-a2a.sh): Live or local compliance test suite utilizing `curl` and `jq` to validate A2A behaviors.

---

## Local Development

### Prerequisites
- [Rust Toolchain](https://www.rust-lang.org/tools/install) (latest stable supporting 2024 edition)
- [gcloud CLI](https://cloud.google.com/sdk/docs/install) (if deploying/testing cloud components)
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
Run the server on the default port (`8104`):
```bash
make run
```

To bind to a specific port (e.g., `8080`):
```bash
make start PORT=8080
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
# Run tests against the currently configured/deployed cloud run instance
make test-a2a

# Or manually target a custom endpoint URL (e.g. localhost)
./test-a2a.sh http://localhost:8104
```

---

## Deployment to Google Cloud Run

### Docker Build
To build the Docker image locally:
```bash
make docker-build
```

### Cloud Run Deployment
Submit the directory to **Google Cloud Build** to automatically build and deploy it as a serverless container on Cloud Run:
```bash
make deploy
```

Once deployed, you can interact with the cloud service using:
```bash
# Get service status and pass/fail health check
make status

# Query public endpoint URL
make endpoint

# Fetch the remote agent capabilities card
make card

# Send a sample A2A status check message to the deployed agent
make a2a
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
