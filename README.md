# Rust A2A Multi-Cloud Benchmark Setup

This repository contains a Rust-only implementation of a multi-agent system using the Agent-to-Agent (A2A) protocol and Model Context Protocol (MCP). It features a central **Rust Master Agent** that coordinates prime calculations and benchmarks by delegating tasks to dedicated Rust sub-agents running locally or deployed across AWS, GCP, and Azure.

## Project Structure

- [rust-master/](file:///home/xbill/a2a-multicloud/rust-master/): The master coordinator agent. It runs an MCP/A2A server that exposes tools to calculate Mersenne primes and check exponent primality by calling sub-agents. See [rust-master/src/main.rs](file:///home/xbill/a2a-multicloud/rust-master/src/main.rs) for implementation details.
- [benchmark-rust/](file:///home/xbill/a2a-multicloud/benchmark-rust/): The local Rust sub-agent implementation for prime calculation. See [benchmark-rust/src/main.rs](file:///home/xbill/a2a-multicloud/benchmark-rust/src/main.rs) for implementation details.
- [benchmark-rust-aws/](file:///home/xbill/a2a-multicloud/benchmark-rust-aws/): The AWS Lightsail deployment configuration and integration tests for the Rust prime agent.
- [benchmark-rust-gcp/](file:///home/xbill/a2a-multicloud/benchmark-rust-gcp/): The Google Cloud Run deployment configuration and integration tests for the Rust prime agent.
- [benchmark-rust-azure/](file:///home/xbill/a2a-multicloud/benchmark-rust-azure/): The Azure Container Apps deployment configuration and integration tests for the Rust prime agent.

## Execution and Test Scripts

- [a2a-master-rust.sh](file:///home/xbill/a2a-multicloud/a2a-master-rust.sh): Starts the local Rust master coordinator agent.
- [set_env.sh](file:///home/xbill/a2a-multicloud/set_env.sh): Sourcing script to configure standard GCP environment variables and connection URLs for deployed cloud agents.
- [set_adc.sh](file:///home/xbill/a2a-multicloud/set_adc.sh): Helper script to configure Google Application Default Credentials.
- [Makefile](file:///home/xbill/a2a-multicloud/Makefile): Defines standard development commands.

## Commands

### Sourcing Environment Setup

Prior to running make targets, prepare the cloud credentials and endpoint definitions:
```bash
source ./set_env.sh
```

### Build

To compile the master and local agent:
```bash
make build
```

### Start

To run the Rust master agent locally (starts on http://0.0.0.0:8100):
```bash
make start
```

### Status

To check the deployment and health status of all Rust sub-agents (AWS, GCP, Azure) and local servers:
```bash
make status
```

### Test

To run unit tests across Rust modules:
```bash
make test
```

To run cloud integration tests specifically:
```bash
make test-aws
make test-gcp
make test-azure
```

## How It Works

1. **A2A Communication**:
   - The master coordinator calls sub-agents via standard A2A JSON-RPC payload format, invoking the `message/send` method.
   - The GCP sub-agent requires authentication using GCP ID tokens, which the master generates dynamically via Application Default Credentials (ADC) if available.

2. **Benchmarking Exponents**:
   - Calling the `calculate_mersenne_prime` tool (via the MCP interface) initiates a benchmark run from exponent 1 to `n`.
   - The master agent cycles through ready cloud and local sub-agents to verify primality using the Lucas-Lehmer test.
   - A detailed breakdown table is output, showing the assigned agent, ready check latency, calculation duration, and primality result.
   - The benchmark run metadata is logged to `benchmark_results.json` and archived timestamped copies.
