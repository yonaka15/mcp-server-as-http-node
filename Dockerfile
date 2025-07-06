# MCP HTTP Server (Node.js Runtime) - Multi-platform Docker Build with uv
# Automatically detects platform and builds appropriate binary

# Stage 1: Platform-aware Rust builder
FROM --platform=${BUILDPLATFORM} rust:1.85-alpine AS rust-builder

# Install build dependencies
RUN apk add --no-cache \
  musl-dev \
  openssl-dev \
  openssl-libs-static \
  git \
  pkgconfig

# Build arguments for cross-compilation
ARG TARGETPLATFORM
ARG BUILDPLATFORM

# Set target based on platform
RUN case "${TARGETPLATFORM}" in \
  "linux/amd64") \
  echo "x86_64-unknown-linux-musl" > /target.txt && \
  rustup target add x86_64-unknown-linux-musl \
  ;; \
  "linux/arm64") \
  echo "aarch64-unknown-linux-musl" > /target.txt && \
  rustup target add aarch64-unknown-linux-musl \
  ;; \
  *) \
  echo "Unsupported platform: ${TARGETPLATFORM}" && exit 1 \
  ;; \
  esac

# Set environment for static linking
ENV RUSTFLAGS="-C target-feature=+crt-static" \
  PKG_CONFIG_ALL_STATIC=1 \
  PKG_CONFIG_ALL_DYNAMIC=0

WORKDIR /build

# Clone the repository
RUN git clone --branch v0.1.0 https://github.com/yonaka15/mcp-server-as-http-core.git .

# Build for the target platform
RUN RUST_TARGET=$(cat /target.txt) && \
  cargo build \
  --release \
  --target ${RUST_TARGET} \
  --config 'profile.release.lto = true' \
  --config 'profile.release.codegen-units = 1' \
  --config 'profile.release.panic = "abort"' \
  --config 'profile.release.strip = true' && \
  cp target/${RUST_TARGET}/release/mcp-server-as-http-core /mcp-http-server

# Stage 2: Runtime (Python with Node.js and uv)
FROM python:latest

# Install Node.js 18 and other system dependencies
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - && \
  apt-get update && \
  apt-get install -y --no-install-recommends \
  nodejs \
  curl \
  git \
  ca-certificates \
  build-essential \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

# Install uv by copying from official image (most reliable method)
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

# Verify installations
RUN python --version && node --version && npm --version && uv --version

# Create non-root user (Debian syntax)
RUN groupadd -g 1001 mcpuser && \
  useradd -r -u 1001 -g mcpuser -d /home/mcpuser -m mcpuser

WORKDIR /app

# Copy the binary from builder
COPY --from=rust-builder /mcp-http-server ./mcp-http-server

# Make executable and verify
RUN chmod +x ./mcp-http-server && \
  ./mcp-http-server --version || echo "Binary ready"

# Default port - can be overridden by environment variable
EXPOSE ${PORT:-3000}

# Copy configuration files
COPY *.config.json ./

# Setup directories
RUN mkdir -p /app/.npm-cache /app/.npm-config /tmp/mcp-servers /app/python-env && \
  chown -R mcpuser:mcpuser /app /tmp/mcp-servers

# Switch to non-root user
USER mcpuser

# Verify uv installation
RUN uv --version

CMD ["./mcp-http-server"]
