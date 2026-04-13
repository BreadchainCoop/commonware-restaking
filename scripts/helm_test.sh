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

# Default values
CLUSTER_NAME="${CLUSTER_NAME:-commonware-test}"
HELM_RELEASE="${HELM_RELEASE:-commonware-avs}"
FORK_URL="${FORK_URL:-https://ethereum-holesky.publicnode.com}"
NODE_COUNT="${NODE_COUNT:-3}"
PRIVATE_KEY="${PRIVATE_KEY}"
FUNDED_KEY="${FUNDED_KEY}"
TEST_FAST_AGGREGATION="${TEST_FAST_AGGREGATION:-false}"
TEST_INGRESS="${TEST_INGRESS:-false}"

echo -e "${GREEN}Starting Helm-based Integration Test${NC}"
echo "Project root: $PROJECT_ROOT"
echo "Cluster name: $CLUSTER_NAME"
echo "Helm release: $HELM_RELEASE"

# Function to read counter value from contract
read_counter() {
    local counter_address=$1
    local response=$(curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d '{
          "jsonrpc":"2.0",
          "method":"eth_call",
          "params":[{
            "to":"'$counter_address'",
            "data":"0x8381f58a"
          }, "latest"],
          "id":1
        }')
    
    local hex_value=$(echo "$response" | jq -r '.result')
    
    if [ -z "$hex_value" ] || [ "$hex_value" = "null" ] || [ "$hex_value" = "0x" ]; then
        echo "0"
    else
        printf "%d\n" "$hex_value" 2>/dev/null || echo "0"
    fi
}

# Step 1: Check if kubectl is available
echo -e "${YELLOW}Step 1: Checking prerequisites...${NC}"
if ! command -v kubectl &> /dev/null; then
    echo -e "${RED}kubectl is not installed${NC}"
    exit 1
fi

if ! command -v helm &> /dev/null; then
    echo -e "${RED}helm is not installed${NC}"
    exit 1
fi

# Step 2: Verify cluster is accessible
echo -e "${YELLOW}Step 2: Verifying cluster access...${NC}"
if ! kubectl cluster-info &> /dev/null; then
    echo -e "${RED}Cannot access Kubernetes cluster${NC}"
    exit 1
fi

echo -e "${GREEN}Cluster is accessible${NC}"
kubectl get nodes

# Step 3: Build Docker images (if not already built)
echo -e "${YELLOW}Step 3: Building Docker images...${NC}"
cd "$PROJECT_ROOT"

if [[ "${SKIP_BUILD:-false}" != "true" ]]; then
    echo "Building router image..."
    docker build -f ./usecases/counter/router/Dockerfile -t commonware-avs-router:local .
    
    echo "Building node image..."
    docker build -f ./usecases/counter/node/Dockerfile -t commonware-avs-node:local .
    
    # Load images into cluster (kind-specific)
    if command -v kind &> /dev/null; then
        echo "Loading images into kind cluster..."
        kind load docker-image commonware-avs-router:local --name "$CLUSTER_NAME" || true
        kind load docker-image commonware-avs-node:local --name "$CLUSTER_NAME" || true
    fi
else
    echo "Skipping Docker build (SKIP_BUILD=true)"
fi

# Step 4: Install Helm chart
echo -e "${YELLOW}Step 4: Installing Helm chart...${NC}"

# Check if release already exists and uninstall it
if helm list -q | grep -q "^${HELM_RELEASE}$"; then
    echo "Release $HELM_RELEASE already exists. Uninstalling to ensure clean setup..."
    helm uninstall "$HELM_RELEASE" || true
    kubectl delete pvc -l app.kubernetes.io/instance="$HELM_RELEASE" || true
    echo "Waiting for resources to be cleaned up..."
    sleep 10
fi

# Run helm install
helm install "$HELM_RELEASE" ./helm/commonware-restaking \
    --set global.environment=LOCAL \
    --set global.nodeCount="$NODE_COUNT" \
    --set secrets.forkUrl="$FORK_URL" \
    --set secrets.privateKey="$PRIVATE_KEY" \
    --set secrets.fundedKey="$FUNDED_KEY" \
    --set node.image.repository=commonware-avs-node \
    --set node.image.tag=local \
    --set node.image.pullPolicy=Never \
    --set router.image.repository=commonware-avs-router \
    --set router.image.tag=local \
    --set router.image.pullPolicy=Never \
    --set sharedData.storageClass=""

echo -e "${GREEN}Helm chart installed successfully${NC}"

# Step 5: Wait for setup job
echo -e "${YELLOW}Step 5: Waiting for setup job to complete...${NC}"

# Find the actual setup job name (includes chart name in it)
SETUP_JOB=$(kubectl get jobs -o name | grep setup | head -1)

if [ -z "$SETUP_JOB" ]; then
    echo -e "${RED}Setup job not found!${NC}"
    echo "Available jobs:"
    kubectl get jobs
    exit 1
fi

echo "Found setup job: $SETUP_JOB"
kubectl wait --for=condition=complete "$SETUP_JOB" --timeout=300s

echo "Setup job completed:"
kubectl logs "$SETUP_JOB" --tail=20

# Step 6: Wait for pods to be ready
echo -e "${YELLOW}Step 6: Waiting for all pods to be ready...${NC}"

echo "Waiting for ethereum pod..."
kubectl wait --for=condition=ready pod -l app.kubernetes.io/component=ethereum --timeout=180s

echo "Waiting for signer pod..."
kubectl wait --for=condition=ready pod -l app.kubernetes.io/component=signer --timeout=180s

echo "Waiting for node pods..."
kubectl wait --for=condition=ready pod -l app.kubernetes.io/component=node --timeout=300s --all

echo "Waiting for router pod..."
kubectl wait --for=condition=ready pod -l app.kubernetes.io/component=router --timeout=300s

echo -e "${GREEN}All pods are ready!${NC}"

# Step 7: Setup port forwarding
echo -e "${YELLOW}Step 7: Setting up port forwarding...${NC}"

# Kill any existing port forwards
pkill -f "kubectl port-forward.*8545" 2>/dev/null || true
pkill -f "kubectl port-forward.*8080" 2>/dev/null || true
sleep 2

# Find actual service names (they include the chart name)
ETHEREUM_SERVICE=$(kubectl get services -o name | grep ethereum | head -1 | sed 's|service/||')
ROUTER_SERVICE=$(kubectl get services -o name | grep router | head -1 | sed 's|service/||')

if [ -z "$ETHEREUM_SERVICE" ] || [ -z "$ROUTER_SERVICE" ]; then
    echo -e "${RED}Required services not found!${NC}"
    kubectl get services
    exit 1
fi

echo "Ethereum service: $ETHEREUM_SERVICE"
echo "Router service: $ROUTER_SERVICE"

# Start new port forwards
kubectl port-forward service/$ETHEREUM_SERVICE 8545:8545 &
ETHEREUM_PF_PID=$!

kubectl port-forward service/$ROUTER_SERVICE 8080:8080 &
ROUTER_PF_PID=$!

# Wait for port forwards to be ready
sleep 10

# Cleanup function
cleanup_port_forwards() {
    echo -e "${YELLOW}Cleaning up port forwards...${NC}"
    kill $ETHEREUM_PF_PID 2>/dev/null || true
    kill $ROUTER_PF_PID 2>/dev/null || true
}

trap cleanup_port_forwards EXIT

echo -e "${GREEN}Port forwarding established${NC}"

# Step 8: Get counter contract address
echo -e "${YELLOW}Step 8: Retrieving contract address...${NC}"

ROUTER_POD=$(kubectl get pods -l app.kubernetes.io/component=router -o jsonpath='{.items[0].metadata.name}')
kubectl cp $ROUTER_POD:/app/.nodes/avs_deploy.json ./avs_deploy.json

if [ ! -f "./avs_deploy.json" ]; then
    echo -e "${RED}AVS deployment file not found${NC}"
    exit 1
fi

COUNTER_ADDRESS=$(cat ./avs_deploy.json | jq -r '.addresses.counter')
if [ -z "$COUNTER_ADDRESS" ] || [ "$COUNTER_ADDRESS" = "null" ]; then
    echo -e "${RED}Counter contract address not found${NC}"
    exit 1
fi

echo -e "${GREEN}Counter contract address: $COUNTER_ADDRESS${NC}"

# Step 9: Test basic counter increment
echo -e "${YELLOW}Step 9: Testing counter increment (default aggregation)...${NC}"

INITIAL_COUNT=$(read_counter "$COUNTER_ADDRESS")
echo "Initial counter value: $INITIAL_COUNT"

echo "Waiting for 5 aggregation cycles (150 seconds)..."
sleep 150

FINAL_COUNT=$(read_counter "$COUNTER_ADDRESS")
echo "Final counter value: $FINAL_COUNT"

if [ "$FINAL_COUNT" -gt "$INITIAL_COUNT" ]; then
    INCREMENTS=$((FINAL_COUNT - INITIAL_COUNT))
    echo -e "${GREEN}✓ Counter successfully incremented from $INITIAL_COUNT to $FINAL_COUNT ($INCREMENTS increments)${NC}"
    LAST_COUNT=$FINAL_COUNT
else
    echo -e "${RED}✗ Counter did not increment (still at $FINAL_COUNT)${NC}"
    echo "Router logs:"
    kubectl logs -l app.kubernetes.io/component=router --tail 50
    exit 1
fi

# Step 10: Test fast aggregation (optional)
if [ "$TEST_FAST_AGGREGATION" = "true" ]; then
    echo -e "${YELLOW}Step 10: Testing fast aggregation (0.5s frequency)...${NC}"
    
    # Find the actual router deployment name (it includes the chart name)
    ROUTER_DEPLOYMENT=$(kubectl get deployments -o name | grep router | head -1 | sed 's|deployment.apps/||')
    
    if [ -z "$ROUTER_DEPLOYMENT" ]; then
        echo -e "${RED}Router deployment not found!${NC}"
        kubectl get deployments
        exit 1
    fi
    
    echo "Router deployment: $ROUTER_DEPLOYMENT"
    
    # Update router deployment
    kubectl set env deployment/$ROUTER_DEPLOYMENT AGGREGATION_FREQUENCY=0.5
    kubectl rollout status deployment/$ROUTER_DEPLOYMENT --timeout=120s
    
    # Re-establish port forwarding
    cleanup_port_forwards
    sleep 2
    kubectl port-forward service/$ETHEREUM_SERVICE 8545:8545 &
    ETHEREUM_PF_PID=$!
    kubectl port-forward service/$ROUTER_SERVICE 8080:8080 &
    ROUTER_PF_PID=$!
    sleep 10
    
    START_COUNT=$LAST_COUNT
    echo "Starting counter value: $START_COUNT"
    
    echo "Waiting for 1 minute with fast aggregation..."
    sleep 60
    
    FAST_COUNT=$(read_counter "$COUNTER_ADDRESS")
    echo "Counter value after fast aggregation: $FAST_COUNT"
    
    if [ "$FAST_COUNT" -gt "$START_COUNT" ]; then
        INCREMENTS=$((FAST_COUNT - START_COUNT))
        echo -e "${GREEN}✓ Fast aggregation successful: $INCREMENTS increments in 1 minute${NC}"
        LAST_COUNT=$FAST_COUNT
    else
        echo -e "${RED}✗ Fast aggregation failed (counter still at $FAST_COUNT)${NC}"
        kubectl logs -l app.kubernetes.io/component=router --tail 50
        exit 1
    fi
fi

# Step 11: Test ingress (optional)
if [ "$TEST_INGRESS" = "true" ]; then
    echo -e "${YELLOW}Step 11: Testing ingress endpoint...${NC}"
    
    # Find the actual router deployment name (it includes the chart name)
    ROUTER_DEPLOYMENT=$(kubectl get deployments -o name | grep router | head -1 | sed 's|deployment.apps/||')
    
    if [ -z "$ROUTER_DEPLOYMENT" ]; then
        echo -e "${RED}Router deployment not found!${NC}"
        kubectl get deployments
        exit 1
    fi
    
    echo "Router deployment: $ROUTER_DEPLOYMENT"
    
    # Update router deployment
    kubectl set env deployment/$ROUTER_DEPLOYMENT INGRESS=true
    kubectl rollout status deployment/$ROUTER_DEPLOYMENT --timeout=120s
    
    # Re-establish port forwarding
    cleanup_port_forwards
    sleep 2
    kubectl port-forward service/$ETHEREUM_SERVICE 8545:8545 &
    ETHEREUM_PF_PID=$!
    kubectl port-forward service/$ROUTER_SERVICE 8080:8080 &
    ROUTER_PF_PID=$!
    sleep 15
    
    START_COUNT=$LAST_COUNT
    echo "Starting counter value: $START_COUNT"
    
    # Send ingress requests
    echo "Sending 5 ingress requests..."
    for i in {1..5}; do
        echo "Sending request $i..."
        curl -s -X POST http://localhost:8080/trigger \
            -H "Content-Type: application/json" \
            -d '{"body": {"metadata": {"request_id": "'$i'", "action": "increment"}}}' || true
        sleep 1
    done
    
    echo "Waiting for aggregation to process requests..."
    sleep 15
    
    INGRESS_COUNT=$(read_counter "$COUNTER_ADDRESS")
    echo "Counter value after ingress: $INGRESS_COUNT"
    
    if [ "$INGRESS_COUNT" -gt "$START_COUNT" ]; then
        INCREMENTS=$((INGRESS_COUNT - START_COUNT))
        echo -e "${GREEN}✓ Ingress test successful: $INCREMENTS increments${NC}"
    else
        echo -e "${RED}✗ Ingress test failed (counter still at $INGRESS_COUNT)${NC}"
        kubectl logs -l app.kubernetes.io/component=router --tail 50
        exit 1
    fi
fi

# Final summary
echo -e "${GREEN}=== Test Summary ===${NC}"
echo "Initial count: $INITIAL_COUNT"
echo "After default aggregation: $FINAL_COUNT"
[ "$TEST_FAST_AGGREGATION" = "true" ] && echo "After fast aggregation: $FAST_COUNT"
[ "$TEST_INGRESS" = "true" ] && echo "After ingress requests: $INGRESS_COUNT"
echo -e "${GREEN}✅ All tests passed!${NC}"

exit 0
