# Gemini Workspace for `a2a-multicloud`

You are a Rust Developer working with Google Cloud, AWS, and Azure Multi-Cloud Agent environments.
You should follow Rust Best practices.
The recommended edition is 2024.

You will use the Agent-to-Agent (A2A) protocol and Model Context Protocol (MCP).

## Project Structure

This project is a multi-agent system demonstration using A2A and MCP, showcasing distributed calculations (specifically finding and verifying Mersenne primes) across multiple cloud providers.

The project is organized into several directories:
- [rust-master/](file:///home/xbill/a2a-multicloud/rust-master/): The master coordinator agent. It runs an MCP and A2A server that exposes tools to calculate Mersenne primes and check exponent primality by calling sub-agents. See [rust-master/src/main.rs](file:///home/xbill/a2a-multicloud/rust-master/src/main.rs).
- [benchmark-rust/](file:///home/xbill/a2a-multicloud/benchmark-rust/): The local Rust sub-agent implementation for prime calculation. See [benchmark-rust/src/main.rs](file:///home/xbill/a2a-multicloud/benchmark-rust/src/main.rs).
- [benchmark-rust-aws/](file:///home/xbill/a2a-multicloud/benchmark-rust-aws/): The AWS Lightsail deployment configuration and integration tests for the Rust prime agent.
- [benchmark-rust-gcp/](file:///home/xbill/a2a-multicloud/benchmark-rust-gcp/): The Google Cloud Run deployment configuration and integration tests for the Rust prime agent.
- [benchmark-rust-azure/](file:///home/xbill/a2a-multicloud/benchmark-rust-azure/): The Azure Container Apps deployment configuration and integration tests for the Rust prime agent.

## Key Development Commands

- `source set_env.sh`: Initialize environment variables and cloud agent endpoints, and generate the `.env` file.
- `source set_adc.sh`: Initialize Google Application Default Credentials.
- `make build`: Build both the master and local agent.
- `make release`: Build the master, local, and sub-agents in release mode.
- `make start`: Start the local Rust master agent.
- `make status`: Query the status and health of all sub-agents (AWS, GCP, Azure, and local).
- `make endpoint`: Query the public/local endpoint URL of all sub-agents or a specific one (options: AGENT=master|local|aws|gcp|azure|all, defaults to all).
- `make card`: Fetch the agent capability card from an agent (options: AGENT=master|local|aws|gcp|azure|all|<url>, defaults to master).
- `make a2a`: Query the calculation status of an agent via A2A protocol (options: AGENT=master|local|aws|gcp|azure|all|<url>, defaults to local).
- `make test`: Run unit tests for the master and local agent.
- `make test-aws` / `make test-gcp` / `make test-azure`: Run cloud A2A integration tests.

## A2A and MCP Protocols
- **A2A Protocol**: Sub-agents communicate using the standard A2A JSON-RPC interface. They implement a GET `/.well-known/agent.json` or `/.well-known/agent-card.json` for agent capabilities, and a POST `/` with a `message/send` method.
- **MCP Server**: The [rust-master](file:///home/xbill/a2a-multicloud/rust-master/) agent acts as an MCP server. When running with `--stdio`, it supports standard MCP JSON-RPC protocol over stdin/stdout. It exposes three tools:
  - `ask_master_agent(query: String)`: Ask the master agent a question (uses Gemini under the hood).
  - `calculate_mersenne_prime(n: i64)`: Runs a distributed benchmark from exponent 1 to n, delegating work to ready sub-agents across GCP, AWS, Azure, and Local. Saves the run metrics to [benchmark_results.json](file:///home/xbill/a2a-multicloud/benchmark_results.json).
  - `check_agents_status()`: Queries the status/health of all connected agents.
