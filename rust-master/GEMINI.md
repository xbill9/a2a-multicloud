# Gemini Developer Workspace Guide - `rust-master`

Welcome! You are a Rust developer working on the **Master Coordinator Agent** (`benchmark-rust-master`). This guide outlines key constraints, standards, and files you must consider when modifying the coordinator agent.

---

## 📌 Development Context

- **Language & Edition:** Rust (Edition 2024).
- **Core Role:** The coordinator acts as both an **A2A Agent Server** and an **MCP Server**. It calls remote and local sub-agents to compute Mersenne prime exponents.
- **Primary Source:** **[src/main.rs](file:///home/xbill/a2a-multicloud/rust-master/src/main.rs)** is the single entrypoint containing all logic (servers, LLM loop, benchmark driver).

---

## ⚠️ Critical Development Constraints

When modifying the codebase, adhere strictly to the following rules:

### 1. Maintain Stdio Integrity for MCP
- When running in MCP mode (`--stdio`), `stdout` is used exclusively for protocol JSON-RPC exchanges.
- **NEVER** write debug statements, logging, or print commands directly to standard output (`stdout` / `println!`).
- All tracing/logging configuration must direct outputs to `stderr` (e.g., `.with_writer(std::io::stderr)`).

### 2. A2A & MCP Protocol Standard Compliance
- **A2A Protocol**: Requires responses to conform to JSON-RPC 2.0. The method `message/send` expects params containing a `Message` object. Capabilities are retrieved from `/.well-known/agent-card.json`.
- **MCP Protocol**: Adheres to the `2024-11-05` spec. Ensure tools declared in `tools/list` have strict JSON-schemas matched exactly in `tools/call`.

### 3. GCP Authentication (OIDC ID Token)
- Requests to the GCP Cloud Run sub-agent (`gcp_agent`) must include a Bearer token (OIDC Identity Token) in the `Authorization` header.
- Use `get_gcp_id_token()` to fetch it from system environments or local credentials, and append it dynamically in `call_sub_agent`.

### 4. Code Quality & Format
- Keep code clean, format using `cargo fmt`, and run clippy.
- Ensure modifications don't break unit tests. Use the `Makefile` to run validations.

---

## 🛠️ Key Developer Workflow

Use the target commands in the local **[Makefile](file:///home/xbill/a2a-multicloud/rust-master/Makefile)** to inspect and test:

```bash
# Build binary
make build

# Launch the coordinator server locally (defaults to port 8100)
make start

# Run tests
make test

# Format the workspace
make format

# Run formatting check and Clippy linter
make lint
```

---

## 🔗 Useful File Links
- Master main implementation: [main.rs](file:///home/xbill/a2a-multicloud/rust-master/src/main.rs)
- Master dependencies: [Cargo.toml](file:///home/xbill/a2a-multicloud/rust-master/Cargo.toml)
- Master Makefile: [Makefile](file:///home/xbill/a2a-multicloud/rust-master/Makefile)
- Latest benchmark results: [benchmark_results.json](file:///home/xbill/a2a-multicloud/rust-master/benchmark_results.json)
