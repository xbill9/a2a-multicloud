#!/bin/bash
# Test script for Azure A2A agent

# Configuration
AZ_RESOURCE_GROUP=${AZ_RESOURCE_GROUP:-mcp-rg-westus2}
AZ_ACA_NAME=${AZ_ACA_NAME:-mcp-app-$(hostname | tr -cd '[:alnum:]' | tr '[:upper:]' '[:lower:]' | cut -c1-10)}

# Determine Endpoint URL
if [ -n "$1" ]; then
    ENDPOINT_URL="$1"
else
    echo "Querying Azure Container App for public endpoint FQDN..."
    FQDN=$(az containerapp show \
        --name "$AZ_ACA_NAME" \
        --resource-group "$AZ_RESOURCE_GROUP" \
        --query "properties.configuration.ingress.fqdn" \
        --output tsv 2>/dev/null)
    if [ -n "$FQDN" ] && [ "$FQDN" != "null" ]; then
        ENDPOINT_URL="https://$FQDN"
    fi
fi

if [ -z "$ENDPOINT_URL" ] || [ "$ENDPOINT_URL" == "None" ] || [ "$ENDPOINT_URL" == "null" ]; then
    # Fallback to local port if azure app not found
    echo "Azure deployment not found. Falling back to localhost:8104 for local testing..."
    ENDPOINT_URL="http://localhost:8104"
fi

# Clean up trailing slashes
ENDPOINT_URL="${ENDPOINT_URL%/}"

echo "Testing A2A endpoint at: $ENDPOINT_URL"

# Test agent-card endpoint for health check
echo "Checking health via agent card..."
HEALTH_RESP=$(curl -s -w "%{http_code}" -o /dev/null "$ENDPOINT_URL/.well-known/agent.json")
if [ "$HEALTH_RESP" != "200" ]; then
    echo "Error: Health check failed with status $HEALTH_RESP"
    exit 1
fi
echo "Health check passed!"

# Send message/send request
echo "Sending test A2A JSON-RPC message..."
PAYLOAD='{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "message/send",
  "params": {
    "message": {
      "kind": "message",
      "messageId": "test-msg-azure-id",
      "role": "user",
      "parts": [
        {
          "kind": "text",
          "text": "Calculate 5 Mersenne primes"
        }
      ],
      "contextId": "test-ctx-azure-id"
    }
  }
}'

RESPONSE=$(curl -s -X POST \
  -H "Content-Type: application/json" \
  -d "$PAYLOAD" \
  "$ENDPOINT_URL/")

echo "Received Response:"
echo "$RESPONSE" | jq . 2>/dev/null || echo "$RESPONSE"

# Verify response content
if echo "$RESPONSE" | grep -q "Mersenne primes"; then
    echo "SUCCESS: A2A agent test passed!"
    exit 0
else
    echo "ERROR: Response did not contain expected content."
    exit 1
fi
