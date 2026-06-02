# Mersenne Prime Sub-Agent (Rust)

This subdirectory contains the local Rust sub-agent implementation for prime calculation. It is designed to run as an independent web service that implements the Agent-to-Agent (A2A) protocol.

The agent is responsible for executing heavy prime-checking computations (specifically checking exponents for Mersenne primes using the Lucas-Lehmer test) and returning the results back to the master coordinator.

## Table of Contents
1. [Architecture & Features](#architecture--features)
2. [API Endpoints](#api-endpoints)
3. [A2A JSON-RPC Protocol](#a2a-json-rpc-protocol)
4. [Development Commands](#development-commands)
5. [Configuration](#configuration)

---

## Architecture & Features

- **Asynchronous Axum Server**: Built with the `axum` web framework and run on `tokio` for efficient, asynchronous network handling.
- **Offloaded Calculations**: Prime-checking calculations are offloaded to Tokio's blocking thread pool (`tokio::task::spawn_blocking`) to keep the HTTP server responsive under heavy computing loads.
- **Lucas-Lehmer Test**: Performs primality tests on exponents using arbitrary-precision arithmetic provided by the `num-bigint` crate.
- **Agent Card Auto-Discovery**: Serves the `.well-known/agent-card.json` metadata file, allowing coordinator agents to auto-discover capabilities.
- **State Check**: Tracks calculation concurrency using atomic flags to prevent concurrent compute runs.

---

## API Endpoints

The agent exposes the following HTTP endpoints:

| Endpoint | Method | Description |
|---|---|---|
| `/health` | `GET` | Simple health check endpoint that returns `OK`. |
| `/.well-known/agent-card.json` | `GET` | Returns agent metadata, skills, and configuration information. |
| `/.well-known/agent.json` | `GET` | Alias to the agent card endpoint. |
| `/` | `POST` | The entry point for A2A JSON-RPC messages. |

---

## A2A JSON-RPC Protocol

Sub-agents communicate via standard JSON-RPC 2.0. The master agent requests calculations or health checks by posting to the root (`/`) path using the `message/send` method.

### 1. Calculation Request
To request a primality check on an exponent, send a payload containing the exponent number.
* **Payload**:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "message/send",
    "params": {
      "message": {
        "kind": "message",
        "messageId": "unique-id-123",
        "role": "user",
        "parts": [
          {
            "kind": "text",
            "text": "exponent: 31"
          }
        ],
        "contextId": "context-456"
      }
    }
  }
  ```
* **Success Response (Mersenne Prime Found)**:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
      "kind": "message",
      "messageId": "generated-uuid",
      "role": "agent",
      "parts": [
        {
          "kind": "text",
          "text": "2147483647"
        }
      ],
      "contextId": "context-456"
    }
  }
  ```
* **Success Response (Not Prime)**:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
      "kind": "message",
      "messageId": "generated-uuid",
      "role": "agent",
      "parts": [
        {
          "kind": "text",
          "text": "not prime"
        }
      ],
      "contextId": "context-456"
    }
  }
  ```

### 2. Status Request
Used by the master coordinator to verify if the agent is idle or currently busy with another calculation.
* **Payload**:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 2,
    "method": "message/send",
    "params": {
      "message": {
        "kind": "message",
        "messageId": "status-query-id",
        "role": "user",
        "parts": [
          {
            "kind": "text",
            "text": "status"
          }
        ],
        "contextId": "status-context"
      }
    }
  }
  ```
* **Response**:
  Returns `"ready"` if idle, or `"not ready"` if a computation is active.

---

## Development Commands

All standard development actions can be performed using the local `Makefile`:

```bash
# Build the sub-agent project
make build

# Run the agent in the foreground (Ctrl+C to quit)
make run

# Start the agent in the background (logs to local_agent.log, PID to local_agent.pid)
make start

# Stop the background agent
make stop

# Check status of the agent process and query its health endpoint
make status

# Fetch the Agent Card (well-known metadata)
make card

# Send an A2A JSON-RPC status check query to the running agent
make a2a

# Run tests
make test

# Format the code base
make format

# Lint the codebase (Clippy & format check)
make lint

# Clean build files and background process artifacts
make clean
```

---

## Configuration

The agent behavior is configured via environment variables:

- `PORT`: The port on which the server listens (defaults to `8104`).
- `MODEL_NAME`: Optional name of the underlying model / agent executor.
- `PUBLIC_URL`: Public-facing address used when generating the agent card metadata.
