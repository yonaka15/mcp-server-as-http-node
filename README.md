# MCP HTTP Server - Node.js Runtime

A specialized HTTP server for Node.js/TypeScript Model Context Protocol (MCP) servers, built on top of `mcp-server-as-http-core`.

> **🎯 Node.js Optimized**: This implementation is specifically optimized for Node.js and TypeScript MCP servers with automatic dependency management, TypeScript compilation, and npm/yarn/pnpm support.

## 🚀 Features

- **Node.js Native**: Optimized for Node.js/TypeScript MCP servers
- **Automatic Setup**: Automatic npm install, TypeScript compilation
- **Multi Package Manager**: Support for npm, yarn, and pnpm
- **Docker First**: Containerized deployment with multi-stage builds
- **Git Integration**: Automatic repository cloning and building
- **Authentication**: Bearer token authentication support
- **Health Checks**: Built-in health monitoring

## 📦 Quick Start with Docker

### 1. Using Docker Compose (Recommended)

```bash
# Copy environment configuration
cp .env.example .env

# Edit your configuration
vim .env

# Start the service
docker-compose up -d

# View logs
docker-compose logs -f mcp-http-server-node
```

### 2. Using Docker Build Script

```bash
# Build and run
chmod +x docker-build.sh
./docker-build.sh --port 3000 --tag node-v1.0.0
```

### 3. Manual Docker Commands

```bash
# Build the image
docker build -t mcp-http-server-node .

# Run the container
docker run -d \
  --name mcp-http-server-node \
  -p 3000:3000 \
  --env-file .env \
  mcp-http-server-node
```

## ⚙️ Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `MCP_CONFIG_FILE` | `mcp_servers.config.json` | Path to MCP servers configuration |
| `MCP_SERVER_NAME` | `redmine` | Server name to use from config |
| `PORT` | `3000` | HTTP server port |
| `HOST` | `0.0.0.0` | HTTP server host |
| `HTTP_API_KEY` | - | Bearer token for authentication |
| `DISABLE_AUTH` | `false` | Disable authentication |
| `NODE_PACKAGE_MANAGER` | `npm` | Package manager (npm/yarn/pnpm) |
| `ENABLE_TYPESCRIPT` | `true` | Enable TypeScript support |
| `AUTO_INSTALL_DEPS` | `true` | Auto install dependencies |
| `WORK_DIR` | `/tmp/mcp-servers` | Working directory for repositories |

### MCP Server Configuration

Create `mcp_servers.config.json`:

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
  "github": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-github"],
    "env": {
      "GITHUB_PERSONAL_ACCESS_TOKEN": "your-github-token"
    }
  }
}
```

## 🛠️ API Usage

### Authentication

Include Bearer token in Authorization header:

```bash
curl -X POST http://localhost:3000/api/v1 \
  -H "Authorization: Bearer your-secret-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{"command": "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"tools/list\", \"params\": {}}"}'
```

### Without Authentication

Set `DISABLE_AUTH=true`:

```bash
curl -X POST http://localhost:3000/api/v1 \
  -H "Content-Type: application/json" \
  -d '{"command": "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"tools/list\", \"params\": {}}"}'
```

## 🏗️ Architecture

### Core Components

- **mcp-server-as-http-core**: Core library (git submodule)
- **Node.js Runtime**: Specialized Node.js/TypeScript support
- **Docker Container**: Multi-stage build with Node.js 18
- **Authentication**: Bearer token middleware
- **Process Management**: Async MCP server communication

### Node.js Optimizations

- **Automatic TypeScript Detection**: Compiles TypeScript projects automatically
- **Package Manager Support**: Works with npm, yarn, and pnpm
- **Dependency Auto-Install**: Automatically runs `npm install` for repositories
- **Version Checking**: Validates Node.js version compatibility
- **npm Cache**: Optimized npm caching in Docker

## 📊 Monitoring

### Health Check

```bash
# Container health status
docker ps

# Manual health check
curl -f http://localhost:3000/api/v1 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"command":"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\",\"params\":{}}"}'
```

### Logs

```bash
# Docker Compose logs
docker-compose logs -f mcp-http-server-node

# Docker logs
docker logs -f mcp-http-server-node
```

## 🔧 Development

### Local Development

```bash
# Clone with submodules
git clone --recurse-submodules https://github.com/your-repo/mcp-server-as-http-node.git

# Update submodules
git submodule update --init --recursive

# Build
cargo build --release

# Run
cargo run
```

### Submodule Management

```bash
# Update core library
git submodule update --remote core

# Pull latest changes
git submodule foreach git pull origin main
```

## 🔗 Related Projects

- **[mcp-server-as-http-core](https://github.com/yonaka15/mcp-server-as-http-core)**: Core library
- **mcp-server-as-http-python**: Python runtime (planned)
- **mcp-server-as-http-docker**: Docker-in-Docker runtime (planned)

## 📄 License

This project is open source. Please refer to the LICENSE file for details.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Update submodules if needed
4. Write tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

---

**Note**: This implementation uses `mcp-server-as-http-core` as a git submodule. Make sure to initialize submodules when cloning.
