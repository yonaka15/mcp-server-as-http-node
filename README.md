# MCP HTTP Server - Node.js Runtime

A **Docker-optimized** HTTP server for Node.js/TypeScript Model Context Protocol (MCP) servers, using the [`mcp-server-as-http-core`](https://github.com/yonaka15/mcp-server-as-http-core) binary.

> **🎯 Node.js Focused**: This implementation is specifically optimized for Node.js and TypeScript MCP servers with automatic dependency management, TypeScript compilation, and npm/yarn/pnpm support.

> **🐳 Docker-Only**: This version uses the pre-built core binary and focuses purely on Docker deployment and Node.js environment optimization.

## 🏗️ Architecture

```
mcp-server-as-http-node/
├── Dockerfile              # Node.js optimized container
├── docker-compose.yml      # Docker Compose configuration  
├── docker-build.sh         # Build and deployment script
├── mcp_servers.config.json # Node.js server examples
├── .env.example            # Environment template
└── README.md               # This file
```

**Core Components:**
- **Pre-built Binary**: Uses `mcp-server-as-http-core` binary (built during Docker image creation)
- **Node.js Environment**: Node.js 18+ with npm, npx, and Git
- **Docker Optimization**: Multi-stage build for minimal runtime image
- **Volume Management**: Persistent npm cache and MCP server storage

## 🚀 Quick Start

### Prerequisites

- **Docker** and **Docker Compose**
- **Git** (for repository cloning)

### Option 1: Docker Compose (Recommended)

```bash
# Clone repository
git clone https://github.com/your-repo/mcp-server-as-http-node.git
cd mcp-server-as-http-node

# Copy and configure environment
cp .env.example .env
# Edit .env with your settings

# Start with Docker Compose
docker-compose up -d

# View logs
docker-compose logs -f
```

### Option 2: Build Script

```bash
# Clone repository
git clone https://github.com/your-repo/mcp-server-as-http-node.git
cd mcp-server-as-http-node

# Copy and configure environment  
cp .env.example .env
# Edit .env with your settings

# Build and run
./docker-build.sh

# Or use docker-compose via script
./docker-build.sh --use-compose
```

### Option 3: Manual Docker

```bash
# Build image
docker build -t mcp-http-server-node .

# Run container
docker run -d \
  --name mcp-http-server-node \
  -p 3000:3000 \
  --env-file .env \
  -v mcp_cache:/tmp/mcp-servers \
  -v npm_cache:/app/.npm-cache \
  mcp-http-server-node
```

## ⚙️ Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `MCP_CONFIG_FILE` | `mcp_servers.config.json` | Path to MCP servers configuration |
| `MCP_SERVER_NAME` | `redmine` | Server name to use from config |
| `PORT` | `3000` | HTTP server port |
| `HTTP_API_KEY` | - | Bearer token for authentication |
| `DISABLE_AUTH` | `false` | Disable authentication |
| `NODE_PACKAGE_MANAGER` | `npm` | Package manager (npm/yarn/pnpm) |
| `ENABLE_TYPESCRIPT` | `true` | Enable TypeScript compilation |
| `AUTO_INSTALL_DEPS` | `true` | Auto-install dependencies |
| `WORK_DIR` | `/tmp/mcp-servers` | Working directory for MCP servers |

### Node.js MCP Server Configuration

Create or update `mcp_servers.config.json`:

```json
{
  "version": "1.0",
  "servers": {
    "redmine": {
      "repository": "https://github.com/yonaka15/mcp-server-redmine",
      "build_command": "npm install && npm run build",
      "command": "node",
      "args": ["dist/index.js"],
      "runtime_config": {
        "node": {
          "version": ">=18.0.0",
          "package_manager": "npm"
        }
      }
    }
  }
}
```

### Additional Server Examples

If you want to add more servers, here are some examples:

```json
{
  "version": "1.0",
  "servers": {
    "redmine": {
      "repository": "https://github.com/yonaka15/mcp-server-redmine",
      "build_command": "npm install && npm run build",
      "command": "node",
      "args": ["dist/index.js"],
      "runtime_config": {
        "node": {
          "version": ">=18.0.0",
          "package_manager": "npm"
        }
      }
    },
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "your-token"
      }
    },
    "brave-search": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-brave-search"],
      "env": {
        "BRAVE_API_KEY": "your-brave-api-key"
      }
    }
  }
}
```

## 🛠️ API Usage

### With Authentication

```bash
curl -X POST http://localhost:3000/api/v1 \
  -H "Authorization: Bearer your-secret-api-key" \
  -H "Content-Type: application/json" \
  -d '{"command": "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"tools/list\", \"params\": {}}"}'
```

### Without Authentication

Set `DISABLE_AUTH=true` in `.env`:

```bash
curl -X POST http://localhost:3000/api/v1 \
  -H "Content-Type: application/json" \
  -d '{"command": "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"tools/list\", \"params\": {}}"}'
```

### Health Check

```bash
curl -f http://localhost:3000/health
```

## 🐳 Docker Features

### Multi-Stage Build
- **Stage 1**: Builds the core binary from source during image creation
- **Stage 2**: Node.js 18 slim runtime with the pre-built binary

### Node.js Optimizations
- Automatic npm cache management
- TypeScript compilation support
- Package manager detection (npm/yarn/pnpm)
- Git repository cloning and building
- Non-root user execution

### Volume Management
- **mcp_cache**: Persistent storage for MCP server files
- **npm_cache**: NPM package cache for faster builds

### Security
- Non-root user execution (`mcpuser`)
- Minimal runtime dependencies
- Health check endpoints

## 📊 Monitoring and Management

### View Logs

```bash
# Docker Compose
docker-compose logs -f

# Docker
docker logs -f mcp-http-server-node

# Build script helper
docker logs -f mcp-http-server-node
```

### Container Management

```bash
# Docker Compose
docker-compose up -d      # Start
docker-compose down       # Stop
docker-compose restart    # Restart

# Docker
docker start mcp-http-server-node
docker stop mcp-http-server-node
docker restart mcp-http-server-node
```

### Health Monitoring

```bash
# Check container health
docker inspect mcp-http-server-node | grep -A 10 Health

# Manual health check
curl -f http://localhost:3000/health

# Check running processes
docker exec mcp-http-server-node ps aux
```

## 🔧 Development and Debugging

### Build Options

```bash
# Build without cache
./docker-build.sh --no-cache

# Build only (don't run)
./docker-build.sh --build-only

# Custom port
./docker-build.sh --port 8080

# Use docker-compose
./docker-build.sh --use-compose
```

### Debugging

```bash
# Execute shell in container
docker exec -it mcp-http-server-node /bin/bash

# Check Node.js environment
docker exec mcp-http-server-node node --version
docker exec mcp-http-server-node npm --version

# Check Git availability
docker exec mcp-http-server-node git --version

# View environment variables
docker exec mcp-http-server-node env | grep MCP
```

### Troubleshooting

```bash
# Check container status
docker ps -a | grep mcp-http-server-node

# Inspect container configuration
docker inspect mcp-http-server-node

# Check volume mounts
docker volume ls | grep mcp
docker volume inspect mcp_cache
```

## 🧩 Node.js Specific Features

### Automatic Dependency Management
- Detects `package.json` and runs appropriate package manager
- Supports npm, yarn, and pnpm
- Handles TypeScript compilation automatically

### Repository Support  
- Clones Git repositories automatically
- Executes build commands before starting MCP servers
- Manages working directories in isolated containers

### Environment Validation
- Validates Node.js version compatibility
- Checks npm/npx availability
- Reports Git status for repository operations

## 📦 Pre-built Binary

This Docker image builds the `mcp-server-as-http-core` binary from source during image creation. The binary provides:

- High-performance HTTP server (Rust/Axum)
- Authentication middleware
- Process management for MCP servers
- Configuration management
- Runtime abstraction

## 🔗 Related Projects

- **[mcp-server-as-http-core](https://github.com/yonaka15/mcp-server-as-http-core)**: Core library and binary
- **mcp-server-as-http-python**: Python runtime (planned)
- **mcp-server-as-http-docker**: Docker-in-Docker runtime (planned)

## 🤝 Contributing

1. Fork the repository
2. Clone your fork: `git clone <your-fork>`
3. Create a feature branch
4. Make your changes (focus on Docker, configuration, and documentation)
5. Test with `./docker-build.sh --build-only`
6. Submit a pull request

### Core Changes

For changes to the HTTP server functionality, please contribute to the [`mcp-server-as-http-core`](https://github.com/yonaka15/mcp-server-as-http-core) repository.

This repository focuses on:
- Docker optimization for Node.js environments
- Configuration management
- Build scripts and deployment tools
- Documentation and examples

## 📄 License

This project is open source under the MIT License. See the LICENSE file for details.

## ⚠️ Important Notes

- **Docker Required**: This implementation is Docker-only and requires Docker to run
- **No Local Rust**: No Rust toolchain needed locally - binary is built in Docker
- **Node.js Focus**: Optimized specifically for Node.js/TypeScript MCP servers
- **Core Dependency**: Uses the latest `mcp-server-as-http-core` built from source

## 🔄 Updates

To update to the latest core server:

```bash
# Rebuild with latest core
./docker-build.sh --no-cache

# Or with docker-compose
docker-compose build --no-cache
```

The Docker build process automatically pulls and builds the latest version of the core server.

## 🆘 Support

- **Issues**: Report bugs and request features in this repository
- **Core Issues**: For HTTP server core functionality, use the [core repository](https://github.com/yonaka15/mcp-server-as-http-core)
- **Discussions**: Use GitHub Discussions for questions and community support

---

**Built with ❤️ for the MCP (Model Context Protocol) community**

**🐳 Docker-optimized • 🚀 Node.js-focused • ⚡ Ready to deploy**
