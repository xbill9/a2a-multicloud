# Use a Rust image for building the application
FROM rust:1.95-bookworm as builder

WORKDIR /app

# Copy Cargo.toml and Cargo.lock to leverage Docker cache for dependencies
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application
RUN cargo build --release --locked --target x86_64-unknown-linux-gnu --jobs $(nproc)

# Use a minimal base image for the final stage
FROM gcr.io/distroless/cc-debian12

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/mcp-aca-rust-azure .

# Expose the port the application listens on
EXPOSE 8080

# Run the application
CMD ["./mcp-aca-rust-azure"]
