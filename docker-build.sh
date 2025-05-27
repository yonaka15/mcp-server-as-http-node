#!/bin/bash
# Build and run script for mcp-http-server

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    print_error "Docker is not running. Please start Docker and try again."
    exit 1
fi

# Default values
IMAGE_NAME="mcp-http-server"
TAG="latest"
PORT="3000"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --build-only)
            BUILD_ONLY=true
            shift
            ;;
        --no-cache)
            NO_CACHE="--no-cache"
            shift
            ;;
        --port)
            PORT="$2"
            shift 2
            ;;
        --tag)
            TAG="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --build-only    Only build the image, don't run"
            echo "  --no-cache      Build without using cache"
            echo "  --port PORT     Port to expose (default: 3000)"
            echo "  --tag TAG       Docker image tag (default: latest)"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option $1"
            exit 1
            ;;
    esac
done

# Check if .env file exists, if not copy from example
if [[ ! -f .env ]]; then
    print_warning ".env file not found. Creating from .env.example"
    if [[ -f .env.example ]]; then
        cp .env.example .env
        print_status "Created .env file from .env.example"
        print_warning "Please edit .env file with your actual configuration"
    else
        print_error ".env.example file not found"
        exit 1
    fi
fi

# Build the Docker image
print_status "Building Docker image: ${IMAGE_NAME}:${TAG}"
if docker build ${NO_CACHE} -t "${IMAGE_NAME}:${TAG}" .; then
    print_status "Docker image built successfully"
else
    print_error "Failed to build Docker image"
    exit 1
fi

# Exit if build-only flag is set
if [[ "${BUILD_ONLY}" == "true" ]]; then
    print_status "Build completed. Use 'docker run' or 'docker-compose up' to start the container."
    exit 0
fi

# Stop and remove existing container if it exists
if docker ps -a --format 'table {{.Names}}' | grep -q "^${IMAGE_NAME}$"; then
    print_status "Stopping and removing existing container"
    docker stop "${IMAGE_NAME}" >/dev/null 2>&1 || true
    docker rm "${IMAGE_NAME}" >/dev/null 2>&1 || true
fi

# Run the container
print_status "Starting container on port ${PORT}"
docker run -d \
    --name "${IMAGE_NAME}" \
    -p "${PORT}:3000" \
    --env-file .env \
    --restart unless-stopped \
    "${IMAGE_NAME}:${TAG}"

# Wait a moment for the container to start
sleep 3

# Check if container is running
if docker ps --format 'table {{.Names}}' | grep -q "^${IMAGE_NAME}$"; then
    print_status "Container started successfully"
    print_status "Server is running at: http://localhost:${PORT}"
    print_status "API endpoint: http://localhost:${PORT}/api/v1"
    
    # Show container logs
    echo ""
    print_status "Container logs:"
    docker logs "${IMAGE_NAME}" --tail 10
    
    echo ""
    print_status "To view live logs: docker logs -f ${IMAGE_NAME}"
    print_status "To stop container: docker stop ${IMAGE_NAME}"
else
    print_error "Container failed to start"
    print_error "Check logs with: docker logs ${IMAGE_NAME}"
    exit 1
fi
