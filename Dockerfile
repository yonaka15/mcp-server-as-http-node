# Multi-stage build for mcp-http-server
# Stage 1: Build stage with Rust and Node.js
FROM rust:1.85-slim-bookworm as builder

# Install Node.js and build dependencies
RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev \
  curl \
  && curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
  && apt-get install -y nodejs \
  && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src target/release/deps/mcp*

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Stage 2: Runtime stage
FROM node:18-slim

# Install only runtime dependencies
RUN apt-get update && apt-get install -y \
  ca-certificates \
  curl \
  && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN groupadd -r mcpuser && useradd -r -g mcpuser mcpuser

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/mcp-http-server .

# Copy configuration files
COPY mcp_servers.config.json .
COPY .env.example .env

# npm/npxが使用するキャッシュディレクトリと設定ディレクトリを/app配下に作成
# これらのディレクトリの所有権をmcpuserに変更
RUN mkdir -p /app/.npm-cache /app/.npm-config && \
  chown -R mcpuser:mcpuser /app/.npm-cache /app/.npm-config

# アプリケーションの他のファイルもmcpuser所有にする
# (この行は既に存在しますが、新しいディレクトリ作成後に効果があるように配置を確認)
RUN chown -R mcpuser:mcpuser /app

# Switch to non-root user
USER mcpuser

# npm/npxがこれらのディレクトリを使用するように環境変数を設定
ENV NPM_CONFIG_CACHE=/app/.npm-cache
# XDG_CONFIG_HOMEは、npxが内部で利用する設定ファイルのパスのヒントになることがあります
ENV XDG_CONFIG_HOME=/app/.npm-config

# Expose port
EXPOSE 3000

# Set environment variables with defaults
ENV MCP_CONFIG_FILE=mcp_servers.config.json
ENV MCP_SERVER_KEY=brave-search
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/api/v1 -X POST \
  -H "Content-Type: application/json" \
  -d '{"command":"test"}' || exit 1

# Run the application
CMD ["./mcp-http-server"]
