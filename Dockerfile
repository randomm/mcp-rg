####################################
# STAGE 1: Build the binary
####################################
FROM rust:1.86-slim AS builder

# Install dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create a new empty project
WORKDIR /usr/src/app

# Copy only Cargo.toml first
COPY Cargo.toml ./

# Copy source code - needed to generate a proper Cargo.lock
COPY src ./src/

# Create a new Cargo.lock from scratch and build
RUN rustc --version && \
    # Force cargo to generate a new lock file that's compatible with this container's Rust version
    rm -f Cargo.lock && \
    # Build the project, which will create a compatible Cargo.lock
    cargo build --release

####################################
# STAGE 2: Create the runtime image
####################################
FROM debian:bookworm-slim

# Install ripgrep and minimal dependencies
RUN apt-get update && \
    apt-get install -y ripgrep ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN groupadd -r mcpuser && useradd -r -g mcpuser mcpuser

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/mcp-rg /usr/local/bin/

# Set working directory
WORKDIR /app

# Set default environment variables
ENV FILES_ROOT=/app/files
ENV LOG_LEVEL=info

# Create and own the files directory
RUN mkdir -p /app/files && \
    chown -R mcpuser:mcpuser /app

# Switch to non-root user
USER mcpuser

# Set entrypoint
ENTRYPOINT ["mcp-rg"]