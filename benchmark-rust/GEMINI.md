# Gemini Workspace for `benchmark-rust`

This workspace contains the local Rust sub-agent implementation for prime calculation within the A2A Multi-Cloud Benchmark system. It is designed as an autonomous agent that coordinates and runs Mersenne prime checking using standard A2A protocols.

## Developer & Agent Guidelines

- **Rust Best Practices**: Follow idiomatic Rust. Use Rust edition 2024.
- **Asynchronous Safety**: Ensure that no CPU-heavy operations (e.g., calculations or loops) block the async Tokio runtime. Always delegate CPU-bound computations to blocking tasks:
  ```rust
  tokio::task::spawn_blocking(move || {
      // CPU-intensive prime calculation goes here
  })
  .await;
  ```
- **Error Handling**: Use structured JSON-RPC responses for API communication. Errors must return valid JSON-RPC 2.0 error payloads with appropriate error codes (e.g., `-32601` for method not found, `-32602` for invalid params).
- **Concurrency**: State management (such as checking if a calculation is active) must use atomic variables (`AtomicBool` with `Ordering::SeqCst` memory ordering) or thread-safe sync primitives to avoid data races.
- **Documentation**: Maintain code readability by leaving inline comments explaining the Lucas-Lehmer test logic and regex parsing logic.

## Key Code References

- [src/main.rs](file:///home/xbill/a2a-multicloud/benchmark-rust/src/main.rs): Core application entrypoint, routes configuration, A2A JSON-RPC handlers, and mathematics backend (Lucas-Lehmer algorithm).
- [Cargo.toml](file:///home/xbill/a2a-multicloud/benchmark-rust/Cargo.toml): Dependency manager specifying `axum`, `tokio`, `num-bigint`, and parsing libraries.
- [Makefile](file:///home/xbill/a2a-multicloud/benchmark-rust/Makefile): Automation targets for building, linting, testing, and managing background agent processes.

## Verification & Workflow

Before finishing tasks or committing changes:

1. **Verify Formatting and Lints**:
   Run `make lint` to format the code and run Clippy checks. Ensure there are no warnings.
2. **Execute Tests**:
   Run `make test` to ensure unit tests run successfully.
3. **Verify API / Health**:
   Start the agent (`make start`) and run `make status`, `make card`, and `make a2a` to verify that the health checks, agent card metadata discovery, and A2A status JSON-RPC are fully functional. Remember to run `make stop` afterwards to release the bound port.
