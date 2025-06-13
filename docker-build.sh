#!/bin/bash
# Build and run script for MCP HTTP Server (Node.js Runtime)
# Multi-platform Docker implementation with build error handling

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}[MCP HTTP Server - Node.js Runtime]${NC} $1"
}

print_feature() {
    echo -e "${PURPLE}[FEATURE]${NC} $1"
}

# Detect platform
PLATFORM=$(uname -m)
case $PLATFORM in
    x86_64)
        ARCH_INFO="x86_64 (Intel/AMD)"
        ;;
    aarch64|arm64)
        ARCH_INFO="ARM64 (Apple Silicon/ARM)"
        ;;
    *)
        ARCH_INFO="$PLATFORM (Other)"
        ;;
esac

# Check Docker
if ! docker info >/dev/null 2>&1; then
    print_error "Docker is not running. Please start Docker and try again."
    exit 1
fi

# Default values
IMAGE_NAME="mcp-http-server-node"
TAG="latest"
PORT="3000"
DOCKERFILE="Dockerfile"  # Default to main Dockerfile
FALLBACK_TO_SIMPLE=false

# Parse arguments
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
        --use-compose)
            USE_COMPOSE=true
            shift
            ;;
        --multiplatform)
            DOCKERFILE="Dockerfile"
            IMAGE_NAME="mcp-http-server-node-multi"
            print_feature "Using multi-platform build (may be slower)"
            shift
            ;;
        --native)
            DOCKERFILE="Dockerfile.native"
            IMAGE_NAME="mcp-http-server-node-native"
            print_feature "Using native platform build (recommended)"
            shift
            ;;
        --distroless)
            DOCKERFILE="Dockerfile.distroless"
            IMAGE_NAME="mcp-http-server-node-distroless"
            print_feature "Using ultra-lightweight distroless build"
            shift
            ;;
        --simple)
            DOCKERFILE="Dockerfile.simple"
            IMAGE_NAME="mcp-http-server-node-simple"
            print_feature "Using simple build with fallback"
            shift
            ;;
        --fallback)
            FALLBACK_TO_SIMPLE=true
            shift
            ;;
        --size-comparison)
            SHOW_SIZE_COMPARISON=true
            shift
            ;;
        --help)
            echo ""
            print_header "Build and run MCP HTTP Server optimized for Node.js"
            echo ""
            echo "Platform: $ARCH_INFO"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Build Options:"
            echo "  --native          Use native platform build (default, fastest)"
            echo "  --multiplatform   Use multi-platform build (slower, cross-platform)"
            echo "  --distroless      Use Google's distroless image (smallest)"
            echo "  --simple          Use simple build with error handling"
            echo "  --fallback        Auto-fallback to simple build on error"
            echo "  --size-comparison Show image size comparison"
            echo ""
            echo "Standard Options:"
            echo "  --build-only      Only build the image, don't run"
            echo "  --no-cache        Build without using cache"
            echo "  --port PORT       Port to expose (default: 3000)"
            echo "  --tag TAG         Docker image tag (default: latest)"
            echo "  --use-compose     Use docker-compose instead of docker run"
            echo "  --help            Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                        # Build native version (recommended)"
            echo "  $0 --multiplatform        # Build with cross-platform support"
            echo "  $0 --simple               # Use simple build method"
            echo "  $0 --fallback             # Auto-fallback on build errors"
            echo "  $0 --size-comparison      # Compare all image sizes"
            echo ""
            echo "Troubleshooting:"
            echo "  For cross-compilation errors on Apple Silicon:"
            echo "  $0 --native               # Build for current platform only"
            echo "  $0 --simple               # Use simplest build method"
            echo ""
            exit 0
            ;;
        *)
            print_error "Unknown option $1"
            exit 1
            ;;
    esac
done

print_header "Starting MCP HTTP Server (Node.js Runtime) setup..."
print_status "Platform: $ARCH_INFO"

# Configuration check
if [[ ! -f .env ]]; then
    print_warning ".env file not found. Creating from .env.example"
    if [[ -f .env.example ]]; then
        cp .env.example .env
        print_status "Created .env file from .env.example"
        print_warning "Please edit .env with your configuration!"
    else
        print_error ".env.example file not found"
        exit 1
    fi
fi

if [[ ! -f mcp_servers.config.json ]]; then
    print_error "mcp_servers.config.json not found"
    exit 1
fi

print_status "Configuration files found ✓"

# Size comparison mode
if [[ "${SHOW_SIZE_COMPARISON}" == "true" ]]; then
    print_header "Building all variants for size comparison..."
    
    declare -A builds
    builds["native"]="Dockerfile.native"
    builds["multiplatform"]="Dockerfile"
    builds["distroless"]="Dockerfile.distroless"
    builds["simple"]="Dockerfile.simple"
    
    echo ""
    print_header "📊 Image Size Comparison"
    echo "----------------------------------------"
    printf "%-15s | %-10s | %s\n" "Image Type" "Status" "Size"
    echo "----------------------------------------"
    
    for variant in "${!builds[@]}"; do
        dockerfile="${builds[$variant]}"
        print_status "Testing ${variant} build..."
        if timeout 300 docker build -f "$dockerfile" -t "mcp-${variant}:comparison" . >/dev/null 2>&1; then
            size=$(docker images "mcp-${variant}:comparison" --format "table {{.Size}}" | tail -n +2)
            printf "%-15s | %-10s | %s\n" "$variant" "✓ SUCCESS" "$size"
            docker rmi "mcp-${variant}:comparison" >/dev/null 2>&1
        else
            printf "%-15s | %-10s | %s\n" "$variant" "✗ FAILED" "N/A"
        fi
    done
    echo "----------------------------------------"
    
    echo ""
    print_status "Recommendation for your platform ($PLATFORM):"
    case $PLATFORM in
        aarch64|arm64)
            print_status "  1. --native (fastest, most reliable)"
            print_status "  2. --simple (fallback if native fails)"
            ;;
        x86_64)
            print_status "  1. --native (fastest)"
            print_status "  2. --multiplatform (cross-platform support)"
            ;;
        *)
            print_status "  1. --simple (safest for uncommon platforms)"
            ;;
    esac
    
    exit 0
fi

# Build function with error handling
build_with_fallback() {
    local primary_dockerfile="$1"
    local fallback_dockerfile="$2"
    local image_name="$3"
    local tag="$4"
    
    print_status "Building image: ${image_name}:${tag}"
    print_status "Using: ${primary_dockerfile}"
    print_status "Platform: $ARCH_INFO"
    
    BUILD_START=$(date +%s)
    
    # Try primary build
    if docker build ${NO_CACHE} -f "${primary_dockerfile}" -t "${image_name}:${tag}" .; then
        BUILD_END=$(date +%s)
        BUILD_TIME=$((BUILD_END - BUILD_START))
        
        print_status "Build completed in ${BUILD_TIME}s ✓"
        
        IMAGE_SIZE=$(docker images "${image_name}:${tag}" --format "table {{.Size}}" | tail -n +2)
        print_status "Image size: ${IMAGE_SIZE}"
        return 0
    else
        print_warning "Primary build failed with ${primary_dockerfile}"
        
        if [[ -n "$fallback_dockerfile" ]] && [[ "$FALLBACK_TO_SIMPLE" == "true" ]]; then
            print_status "Attempting fallback build with ${fallback_dockerfile}..."
            
            if docker build ${NO_CACHE} -f "${fallback_dockerfile}" -t "${image_name}:${tag}" .; then
                BUILD_END=$(date +%s)
                BUILD_TIME=$((BUILD_END - BUILD_START))
                
                print_status "Fallback build completed in ${BUILD_TIME}s ✓"
                
                IMAGE_SIZE=$(docker images "${image_name}:${tag}" --format "table {{.Size}}" | tail -n +2)
                print_status "Image size: ${IMAGE_SIZE}"
                return 0
            else
                print_error "Both primary and fallback builds failed"
                print_error "Try: $0 --simple (for maximum compatibility)"
                return 1
            fi
        else
            print_error "Build failed. Try one of these options:"
            print_error "  $0 --simple      # Use simple build method"
            print_error "  $0 --fallback    # Auto-retry with simple method"
            if [[ "$DOCKERFILE" != "Dockerfile.native" ]]; then
                print_error "  $0 --native      # Use native platform build"
            fi
            return 1
        fi
    fi
}

# Docker Compose mode
if [[ "${USE_COMPOSE}" == "true" ]]; then
    print_status "Using docker-compose for build and deployment"
    
    export DOCKERFILE=$DOCKERFILE
    
    if [[ "${DOCKERFILE}" == "Dockerfile.distroless" ]]; then
        COMPOSE_PROFILES="--profile distroless"
        print_feature "Using distroless profile"
    fi
    
    if [[ "${BUILD_ONLY}" == "true" ]]; then
        print_status "Building with docker-compose..."
        if docker-compose ${COMPOSE_PROFILES} build ${NO_CACHE}; then
            print_status "Build completed ✓"
        else
            if [[ "$FALLBACK_TO_SIMPLE" == "true" ]]; then
                print_warning "Trying fallback build..."
                export DOCKERFILE="Dockerfile.simple"
                docker-compose build ${NO_CACHE}
            else
                exit 1
            fi
        fi
    else
        print_status "Building and starting with docker-compose..."
        if docker-compose ${COMPOSE_PROFILES} up -d --build ${NO_CACHE}; then
            sleep 5
            if docker-compose ps | grep -q "Up"; then
                print_status "Service started successfully ✓"
                print_status "Server: http://localhost:${PORT}"
                print_status "API: http://localhost:${PORT}/api/v1"
            else
                print_error "Service failed to start"
                exit 1
            fi
        else
            if [[ "$FALLBACK_TO_SIMPLE" == "true" ]]; then
                print_warning "Trying fallback build..."
                export DOCKERFILE="Dockerfile.simple"
                docker-compose up -d --build ${NO_CACHE}
            else
                exit 1
            fi
        fi
    fi
    exit 0
fi

# Standard Docker build with fallback
FALLBACK_DOCKERFILE=""
if [[ "$FALLBACK_TO_SIMPLE" == "true" ]] && [[ "$DOCKERFILE" != "Dockerfile.simple" ]]; then
    FALLBACK_DOCKERFILE="Dockerfile.simple"
fi

if ! build_with_fallback "$DOCKERFILE" "$FALLBACK_DOCKERFILE" "$IMAGE_NAME" "$TAG"; then
    exit 1
fi

if [[ "${BUILD_ONLY}" == "true" ]]; then
    print_status "Build completed. Run with:"
    echo "docker run -d --name ${IMAGE_NAME} -p ${PORT}:\${PORT} --env-file .env ${IMAGE_NAME}:${TAG}"
    exit 0
fi

# Run container
if docker ps -a --format 'table {{.Names}}' | grep -q "^${IMAGE_NAME}$"; then
    print_status "Stopping existing container..."
    docker stop "${IMAGE_NAME}" >/dev/null 2>&1 || true
    docker rm "${IMAGE_NAME}" >/dev/null 2>&1 || true
fi

print_status "Starting container on port ${PORT}..."

# Read PORT from .env file if it exists
if [[ -f .env ]]; then
    ENV_PORT=$(grep "^PORT=" .env | cut -d'=' -f2)
    if [[ -n "$ENV_PORT" ]]; then
        CONTAINER_PORT="$ENV_PORT"
    else
        CONTAINER_PORT="${PORT}"
    fi
else
    CONTAINER_PORT="${PORT}"
fi

docker run -d \
    --name "${IMAGE_NAME}" \
    -p "${PORT}:${CONTAINER_PORT}" \
    --env-file .env \
    -v mcp_cache:/tmp/mcp-servers \
    -v npm_cache:/app/.npm-cache \
    --memory=512m \
    --cpus=0.5 \
    --restart unless-stopped \
    "${IMAGE_NAME}:${TAG}"

sleep 5

if docker ps --format 'table {{.Names}}' | grep -q "^${IMAGE_NAME}$"; then
    print_status "Container started successfully ✓"
    print_status "Server: http://localhost:${PORT}"
    print_status "API: http://localhost:${PORT}/api/v1"
    
    echo ""
    print_status "Management commands:"
    echo "  Logs:    docker logs -f ${IMAGE_NAME}"
    echo "  Stop:    docker stop ${IMAGE_NAME}"
    echo "  Restart: docker restart ${IMAGE_NAME}"
    echo "  Shell:   docker exec -it ${IMAGE_NAME} /bin/sh"
    
    # Health check
    sleep 2
    if curl -s -f "http://localhost:${PORT}/health" >/dev/null 2>&1; then
        print_status "Health check: ✓ PASS"
    else
        print_warning "Health check: ⚠ PENDING (normal during startup)"
    fi
else
    print_error "Container failed to start"
    print_error "Check logs: docker logs ${IMAGE_NAME}"
    exit 1
fi

print_header "🚀 Deployment successful!"
IMAGE_SIZE=$(docker images "${IMAGE_NAME}:${TAG}" --format "table {{.Size}}" | tail -n +2)
print_feature "Built with ${DOCKERFILE} • Size: ${IMAGE_SIZE} • Port: ${PORT} • Platform: $ARCH_INFO"
