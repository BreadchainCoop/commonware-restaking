#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LOG_DIR="$PROJECT_ROOT/logs"

# Create logs directory
mkdir -p "$LOG_DIR"

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up Docker containers...${NC}"
    cd "$PROJECT_ROOT"
    docker compose down || true
    echo -e "${GREEN}Cleanup completed${NC}"
}

# Set trap for cleanup
trap cleanup EXIT INT TERM

echo -e "${GREEN}Starting BLS Signature Aggregation Integration Test${NC}"
echo "Project root: $PROJECT_ROOT"
echo "Logs directory: $LOG_DIR"

# Step 1: Build verification script
echo -e "${YELLOW}Step 1: Building verification script...${NC}"
cd "$PROJECT_ROOT/scripts"
cargo build --release --bin verify_increments --quiet
cd "$PROJECT_ROOT"

# Step 2: Set up environment files
echo -e "${YELLOW}Step 2: Setting up environment files...${NC}"

# Copy config template
cp config/config.example.json config/config.json

# Update .env for local mode
echo "Configuring .env for local mode..."
sed -i '' 's|^HTTP_RPC=.*|HTTP_RPC=http://localhost:8545|' .env
sed -i '' 's|^WS_RPC=.*|WS_RPC=ws://localhost:8545|' .env
sed -i '' 's|^RPC_URL=.*|RPC_URL=http://ethereum:8545|' .env
sed -i '' 's|^ENVIRONMENT=.*|ENVIRONMENT=LOCAL|' .env

# Use default Anvil private key for testing
DEFAULT_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
sed -i '' "s|^PRIVATE_KEY=.*|PRIVATE_KEY=$DEFAULT_PRIVATE_KEY|" .env
sed -i '' "s|^FUNDED_KEY=.*|FUNDED_KEY=$DEFAULT_PRIVATE_KEY|" .env

# Set Holesky contract addresses for LOCAL mode
sed -i '' 's|^#DELEGATION_MANAGER_ADDRESS=|DELEGATION_MANAGER_ADDRESS=|' .env
sed -i '' 's|^#STRATEGY_MANAGER_ADDRESS=|STRATEGY_MANAGER_ADDRESS=|' .env
sed -i '' 's|^#LST_CONTRACT_ADDRESS=|LST_CONTRACT_ADDRESS=|' .env
sed -i '' 's|^#LST_STRATEGY_ADDRESS=|LST_STRATEGY_ADDRESS=|' .env
sed -i '' 's|^#BLS_SIGNATURE_CHECKER_ADDRESS=|BLS_SIGNATURE_CHECKER_ADDRESS=|' .env
sed -i '' 's|^#OPERATOR_STATE_RETRIEVER_ADDRESS=|OPERATOR_STATE_RETRIEVER_ADDRESS=|' .env
sed -i '' 's|^#ALLOCATION_MANAGER_ADDRESS=|ALLOCATION_MANAGER_ADDRESS=|' .env

echo "Environment configuration complete"

# Step 3: Pull Docker images
echo -e "${YELLOW}Step 3: Pulling Docker images...${NC}"
docker compose pull

# Step 4: Start Docker Compose services
echo -e "${YELLOW}Step 4: Starting Docker Compose services...${NC}"
docker compose up -d

# Show running containers
docker compose ps

# Step 5: Wait for EigenLayer setup to complete
echo -e "${YELLOW}Step 5: Waiting for EigenLayer setup to complete...${NC}"
timeout=300
elapsed=0

while [ $elapsed -lt $timeout ]; do
    # Check if eigenlayer container has completed setup
    if docker compose logs eigenlayer 2>/dev/null | grep -q "Operator 3 weight in quorum" && [ -f config/.nodes/avs_deploy.json ]; then
        echo -e "${GREEN}EigenLayer setup completed successfully${NC}"
        break
    fi
    
    echo "Waiting for EigenLayer setup... ($elapsed/$timeout seconds)"
    sleep 10
    elapsed=$((elapsed + 10))
done

if [ $elapsed -ge $timeout ]; then
    echo -e "${RED}Timeout waiting for EigenLayer setup${NC}"
    echo "Eigenlayer logs:"
    docker compose logs eigenlayer
    exit 1
fi

# Give extra time for nodes to initialize
echo "Waiting for nodes to initialize..."
sleep 30

# Step 6: Check service health
echo -e "${YELLOW}Step 6: Checking service health...${NC}"
for service in node-1 node-2 node-3 router; do
    if docker compose ps | grep -q "$service.*Up"; then
        echo "Service $service is running"
    else
        echo -e "${YELLOW}Warning: Service $service might not be ready${NC}"
    fi
done

# Step 7: Wait for aggregation cycles
echo -e "${YELLOW}Step 7: Waiting for signature aggregation cycles...${NC}"
echo "This will take approximately 2-3 minutes..."
sleep 120

# Step 8: Verify increments
echo -e "${YELLOW}Step 8: Verifying counter increments...${NC}"
cd scripts

# Source environment and run verification
source ../.env
export AVS_DEPLOYMENT_PATH="../config/.nodes/avs_deploy.json"

if [ ! -f "$AVS_DEPLOYMENT_PATH" ]; then
    echo -e "${RED}Deployment file not found at $AVS_DEPLOYMENT_PATH${NC}"
    exit 1
fi

echo "Running verification..."
cargo run -p avs-scripts --release --bin verify_increments

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Integration test PASSED! Counter was incremented successfully.${NC}"
    
    # Show some logs for confirmation
    echo -e "${YELLOW}Recent router logs:${NC}"
    docker compose logs --tail=20 router
    
    exit 0
else
    echo -e "${RED}❌ Integration test FAILED! Counter was not incremented as expected.${NC}"
    
    # Print logs for debugging
    echo -e "${YELLOW}Router logs:${NC}"
    docker compose logs router
    
    echo -e "${YELLOW}Node logs:${NC}"
    docker compose logs node1 --tail=20
    docker compose logs node2 --tail=20
    docker compose logs node3 --tail=20
    
    echo -e "${YELLOW}EigenLayer logs:${NC}"
    docker compose logs eigenlayer --tail=50
    
    exit 1
fi