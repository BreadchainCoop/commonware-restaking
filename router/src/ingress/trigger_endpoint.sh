#!/bin/bash

# Configuration
ENDPOINT_URL="http://localhost:8080/trigger"  # TODO: dynamically get from env
INTERVAL=30  # seconds

# Sample data - adjust these values as needed
# Counter for tracking requests
counter=1

echo "Starting to send requests to $ENDPOINT_URL every $INTERVAL seconds..."
echo "Press Ctrl+C to stop"
echo ""

while true; do
    current_datetime=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$current_datetime] Sending request #$counter"
    
    # Create JSON payload
    json_data=$(cat <<EOF
{
    "body": {
        "var1": "$current_datetime",
        "var2": "value2_$counter",
        "var3": "value3_$(date +%s)"
    }
}
EOF
)

    # Send POST request
    response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$json_data" \
        "$ENDPOINT_URL")

    # Extract HTTP status code
    http_status=$(echo "$response" | grep "HTTP_STATUS:" | cut -d':' -f2)
    response_body=$(echo "$response" | sed '/HTTP_STATUS:/d')

    # Display results
    if [ "$http_status" = "200" ]; then
        echo "✅ Success (HTTP $http_status): $response_body"
    else
        echo "❌ Error (HTTP $http_status): $response_body"
    fi
    
    echo ""
    
    # Increment counter
    ((counter++))
    
    # Wait for next interval
    sleep $INTERVAL
done 