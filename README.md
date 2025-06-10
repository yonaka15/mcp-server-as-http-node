# MCP HTTP Server

A HTTP server that provides a REST API interface to Model Context Protocol (MCP) servers.

> **⚠️ IMPORTANT NOTE**: This repository contains a general-purpose implementation that works with various MCP servers via `npx` commands. For production use and better performance, we recommend using language-specific repositories:
>
> - **Node.js/TypeScript MCP servers**: Use `mcp-server-as-http-node` (planned)
> - **Python MCP servers**: Use `mcp-server-as-http-python` (planned)
> - **Docker-based MCP servers**: Use `mcp-server-as-http-docker` (planned)
>
> Each specialized repository provides optimized Docker images, better dependency management, and improved performance for specific MCP server types.

## Features

- **REST API Interface**: Convert MCP protocol to HTTP REST API
- **Authentication**: Bearer token authentication support
- **Configuration**: JSON-based MCP server configuration
- **Docker Support**: Full Docker containerization with multi-stage builds
- **Health Checks**: Built-in health checking capabilities
- **Logging**: Comprehensive debug logging

## Quick Start with Docker

### 1. Build and Run (Recommended)

```bash
# Make the build script executable
chmod +x docker-build.sh

# Build and start the container
./docker-build.sh

# Or with custom options
./docker-build.sh --port 8080 --tag v1.0.0
```

### 2. Using Docker Compose

```bash
# Start the service
docker-compose up -d

# View logs
docker-compose logs -f

# Stop the service
docker-compose down
```

### 3. Manual Docker Commands

```bash
# Build the image
docker build -t mcp-http-server .

# Run the container
docker run -d \
  --name mcp-http-server \
  -p 3000:3000 \
  --env-file .env \
  mcp-http-server
```

## Configuration

### Environment Variables

Create a `.env` file (copy from `.env.example`):

```bash
# HTTP Server Authentication
HTTP_API_KEY=your-secret-api-key-here
DISABLE_AUTH=false

# MCP Server Configuration
MCP_CONFIG_FILE=mcp_servers.config.json
MCP_SERVER_NAME=redmine
```

### MCP Server Configuration

Edit `mcp_servers.config.json` to configure MCP servers:

```json
{
  "redmine": {
    "repository": "https://github.com/yonaka15/mcp-server-redmine",
    "build_command": "npm install && npm run build",
    "command": "node",
    "args": ["dist/index.js"],
    "env": {
      "REDMINE_URL": "https://your-redmine.example.com",
      "REDMINE_API_KEY": "your-api-key"
    }
  },
  "brave-search": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-brave-search"]
  }
}
```

## API Usage

### Authentication

Include Bearer token in Authorization header:

```bash
curl -X POST http://localhost:3000/api/v1 \
  -H "Authorization: Bearer your-secret-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{"command": "your-mcp-command"}'
```

Example for `tools/list`:
```bash
 curl -X POST http://localhost:3000/api/v1 \
  -H "Authorization: Bearer your-secret-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{"command": "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"tools/list\", \"params\": {}}"}'
```

### Without Authentication

Set `DISABLE_AUTH=true` in your `.env` file:

```bash
curl -X POST http://localhost:3000/api/v1 \
  -H "Content-Type: application/json" \
  -d '{"command": "your-mcp-command"}'
```

## Development

### Local Development

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Git for repository cloning
# Node.js dependencies will be automatically installed during build

# Build and run
cargo build --release
./target/release/mcp-http-server
```

### Docker Development

```bash
# Build only (no run)
./docker-build.sh --build-only

# Build without cache
./docker-build.sh --no-cache

# Custom port
./docker-build.sh --port 8080
```

## Architecture

### Multi-stage Docker Build

- **Stage 1 (Builder)**: Rust + Node.js environment for building
- **Stage 2 (Runtime)**: Minimal Node.js runtime with compiled binary

### Security Features

- Non-root user execution
- Minimal runtime dependencies
- Optional Bearer token authentication
- Health check endpoints

### Dependencies

- **Runtime**: Node.js (for npx and MCP servers)
- **Build**: Rust toolchain
- **MCP Servers**: Various npm packages (installed dynamically)

## Monitoring

### Health Check

```bash
# Check container health
docker ps

# Manual health check
curl -f http://localhost:3000/api/v1 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"command":"test"}'
```

### Logs

```bash
# Docker logs
docker logs -f mcp-http-server

# Docker Compose logs
docker-compose logs -f
```

## Troubleshooting

### Common Issues

1. **Node.js/npx not found**: Ensure Node.js is installed in the container
2. **MCP server startup failure**: Check network connectivity for npm package downloads
3. **Permission denied**: Verify file permissions and user configuration
4. **Port conflicts**: Change the port mapping in Docker commands

### Debug Mode

Enable debug logging:

```bash
# Set environment variable
export RUST_LOG=debug

# Or in .env file
RUST_LOG=debug
```

## Repository Strategy

### Current Repository (`mcp-server-as-http`)

This repository serves as:
- **Proof of Concept**: Demonstrates HTTP-to-MCP protocol conversion
- **General Purpose Tool**: Works with any MCP server accessible via `npx`
- **Development Base**: Foundation for specialized implementations

### Planned Specialized Repositories

For production use, we're developing language-specific repositories with optimized Docker images:

#### 1. `mcp-server-as-http-node` (Node.js/TypeScript)
- **Base Image**: `node:18-alpine`
- **Features**: 
  - Direct GitHub repository cloning and building
  - Optimized for npm/yarn-based MCP servers
  - TypeScript compilation support
  - Node.js-specific optimizations
- **Use Cases**: Most MCP servers from @modelcontextprotocol org

#### 2. `mcp-server-as-http-python` (Python)
- **Base Image**: `python:3.11-alpine`
- **Features**:
  - Poetry/pip dependency management
  - Python virtual environment isolation
  - Support for Python-based MCP servers
  - Optimized Python runtime
- **Use Cases**: Python-based MCP servers and custom implementations

#### 3. `mcp-server-as-http-docker` (Docker-in-Docker)
- **Base Image**: `docker:dind`
- **Features**:
  - Run MCP servers that require Docker
  - Full containerization support
  - Multi-service orchestration
  - Advanced networking capabilities
- **Use Cases**: Complex MCP servers requiring isolated environments

### Migration Strategy

1. **Current Phase**: Use this repository for development and testing
2. **Production Phase**: Migrate to appropriate specialized repository
3. **Configuration**: Same JSON configuration format across all repositories
4. **API Compatibility**: Identical REST API interface

### Benefits of Repository Separation

- **Optimized Images**: Smaller, faster Docker images with only necessary dependencies
- **Better Performance**: Language-specific optimizations and caching
- **Simplified Maintenance**: Focused codebase per language ecosystem
- **Security**: Minimal attack surface with language-specific base images
- **CI/CD Efficiency**: Faster builds and testing cycles

### Configuration Compatibility

All repositories will support the same configuration format:

```json
{
  "server-name": {
    "repository": "https://github.com/org/mcp-server",
    "build_command": "npm install && npm run build",
    "command": "node",
    "args": ["dist/index.js"],
    "env": {
      "CUSTOM_VAR": "value"
    }
  }
}
```

## License

This project is open source. Please refer to the LICENSE file for details.
