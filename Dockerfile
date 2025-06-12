# Multi-stage Docker build for MCP HTTP Server (Node.js Runtime)
# Optimized for Node.js/TypeScript MCP servers

# Stage 1: Build stage with Rust and Node.js
FROM rust:1.85-slim-bookworm as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    git \
    && curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy project files including git submodules
COPY . .

# Initialize git submodules
RUN git submodule update --init --recursive || echo "No submodules or git not available"

# Build dependencies first (for caching)
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src target/release/deps/mcp_http_server_node*

# Build the actual application
COPY src ./src
RUN cargo build --release

# Stage 2: Runtime stage optimized for Node.js
FROM node:18-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r mcpuser && useradd -r -g mcpuser mcpuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/mcp-http-server-node ./mcp-http-server

# Copy configuration files
COPY mcp_servers.config.json .env.example ./

# Setup npm cache and config directories
RUN mkdir -p /app/.npm-cache /app/.npm-config /tmp/mcp-servers && \
    chown -R mcpuser:mcpuser /app /tmp/mcp-servers

# Switch to non-root user
USER mcpuser

# Configure npm to use app-local directories
ENV NPM_CONFIG_CACHE=/app/.npm-cache
ENV XDG_CONFIG_HOME=/app/.npm-config

# Expose port
EXPOSE 3000

# Environment variables for Node.js optimization
ENV MCP_CONFIG_FILE=mcp_servers.config.json
ENV MCP_SERVER_NAME=redmine
ENV NODE_PACKAGE_MANAGER=npm
ENV ENABLE_TYPESCRIPT=true
ENV AUTO_INSTALL_DEPS=true
ENV WORK_DIR=/tmp/mcp-servers
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/v1 -X POST \
        -H "Content-Type: application/json" \
        -d '{"command": "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"tools/list\", \"params\": {}}"}' || exit 1

# Run the application
CMD ["./mcp-http-server"]
