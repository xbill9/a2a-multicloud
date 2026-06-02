# Gemini Workspace for `benchmark-rust-aws`

Welcome! This guide is designed for AI coding assistants working in the `benchmark-rust-aws` sub-project of the Multi-Cloud A2A (Agent-to-Agent) Prime Calculation Benchmark system.

---

## 1. Role and Context
You are a Rust developer working within a multi-cloud agent structure. This directory hosts the **AWS Lightsail** agent. 

### Multi-Cloud Coordinator Flow
```mermaid
graph TD
    Master[Rust Master Coordinator] -->|A2A POST /| Local[Local Sub-Agent]
    Master -->|A2A POST /| AWS[AWS Lightsail Sub-Agent]
    Master -->|A2A POST /| GCP[GCP Cloud Run Sub-Agent]
    Master -->|A2A POST /| Azure[Azure Container Apps Sub-Agent]
```

The coordinator (`rust-master`) calls this sub-agent to:
1. Query its status / readiness before distributing tasks.
2. Delegate exponent validation (specifically Lucas-Lehmer primality tests).
3. Record benchmarks for distributed calculations.

---

## 2. Key Codebase Components

### Entrypoint and Routes: [src/main.rs](file:///home/xbill/a2a-multicloud/benchmark-rust-aws/src/main.rs)
- **HTTP Server**: Uses Axum and Tokio listener.
- **Routes**:
  - `GET /health`: Basic ping returning `OK`.
  - `GET /.well-known/agent.json` and `GET /.well-known/agent-card.json`: Returns capability information (skills list) using the [AgentCard](file:///home/xbill/a2a-multicloud/benchmark-rust-aws/src/main.rs#L38-L51) struct.
  - `POST /`: JSON-RPC endpoint. Handles method `message/send` (e.g., status check and primality test requests).
- **State management**:
  - Uses `AtomicBool` `CALCULATION_ACTIVE` to signal if calculation is running.
  - [CalculationGuard](file:///home/xbill/a2a-multicloud/benchmark-rust-aws/src/main.rs#L14-L27) uses RAII to automatically toggle the `CALCULATION_ACTIVE` state off when the calculation finishes or panics.

### Dependency Configuration: [Cargo.toml](file:///home/xbill/a2a-multicloud/benchmark-rust-aws/Cargo.toml)
- Set to **Rust 2024 edition** (as configured in the Cargo manifest).
- Crucial dependencies:
  - `num-bigint`: Arbitrary-precision integer arithmetic for $2^p - 1$ calculations used in [is_mersenne_prime](file:///home/xbill/a2a-multicloud/benchmark-rust-aws/src/main.rs#L126-L145).
  - `axum`: Modern, lightweight web framework built on top of Hyper and Tokio.
  - `tokio`: Async engine, configured with `full` features.

### Containerization: [Dockerfile](file:///home/xbill/a2a-multicloud/benchmark-rust-aws/Dockerfile)
- Multi-stage Docker builder setup to compile binaries in Rust (`rust:1.95-bookworm`) and run them on a small `cc-debian12` distroless runner.
- Builds target `x86_64-unknown-linux-gnu`.
- Deploys automatically via AWS CLI toolchain to Amazon Lightsail Container Services.

---

## 3. Architecture & Development Guidelines

### Async vs. CPU-Heavy Tasks
Mersenne prime check is a CPU-heavy mathematical task.
> [!IMPORTANT]
> Never run blocking math functions directly inside Axum handlers, as it blocks the event loop. Always use `tokio::task::spawn_blocking`:
> ```rust
> let result = tokio::task::spawn_blocking(move || {
>     // computationally heavy code here
> }).await;
> ```

### Modifying the A2A Parser
The input parsing in `POST /` handles status checks and exponent checks using regex matchers.
- **Status matcher**: matches case-insensitive patterns of `status`, `ready`, `active`.
- **Exponent matcher**: matches case-insensitive strings like `exponent 31`, `exp:31`, or `p=31`, as well as general integers.
- If you extend or modify the message parser, ensure the regex changes do not break simple integer requests (e.g., standard benchmark query `"5"` which triggers finding the first 5 Mersenne primes).

### Keeping State Safe
- Because Lightsail Container Services are stateless and can scale or spin up multiple container instances, the atomic state tracker `CALCULATION_ACTIVE` is localized to each instance.
- Avoid using file-based storage or mutable global states that assume single-concurrency globally.

---

## 4. Key Development Commands

All commands can be invoked from the [Makefile](file:///home/xbill/a2a-multicloud/benchmark-rust-aws/Makefile).

| Command | Action |
|---------|--------|
| `make build` | Local debug compilation |
| `make run` | Run the sub-agent project locally in foreground |
| `make start` | Spin up a local server in the background (logs to `server.log`, PID in `server.pid`) |
| `make stop` | Stop the local background server |
| `make status` | Query health status of local server and Lightsail container service |
| `make test` | Run Rust unit tests |
| `make test-a2a` | Run A2A integration test script ([test-a2a.sh](file:///home/xbill/a2a-multicloud/benchmark-rust-aws/test-a2a.sh)) |
| `make docker-build` | Build local Docker image |
| `make deploy-lightsail` | Build, push to Amazon Lightsail, and deploy |
| `make a2a` | Test deployed endpoint with a JSON-RPC status check payload |
| `make lightsail-status`| Query AWS Lightsail container service status directly |
| `make endpoint` | Get the public endpoint URL |
| `make aws-destroy` | Delete the container service on AWS Lightsail |
