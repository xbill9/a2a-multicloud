#!/bin/bash
# Test script for GCP A2A agent

# Configuration
SERVICE_NAME=${SERVICE_NAME:-bench-rust}
REGION=${REGION:-us-central1}

# Determine Endpoint URL
if [ -n "$1" ]; then
    ENDPOINT_URL="$1"
else
    echo "Querying Google Cloud Run for public endpoint URL..."
    ENDPOINT_URL=$(gcloud run services describe "$SERVICE_NAME" \
        --region "$REGION" \
        --format 'value(status.url)' 2>/dev/null)
fi

if [ -z "$ENDPOINT_URL" ] || [ "$ENDPOINT_URL" == "None" ] || [ "$ENDPOINT_URL" == "null" ]; then
    # Fallback to the known public URL
    ENDPOINT_URL="https://bench-rust-289270257791.us-central1.run.app"
fi

# Clean up trailing slashes
ENDPOINT_URL="${ENDPOINT_URL%/}"

# Determine Authorization Header (if authenticated with gcloud)
AUTH_HEADER=""
if gcloud auth print-identity-token &>/dev/null; then
    TOKEN=$(gcloud auth print-identity-token 2>/dev/null)
    if [ -n "$TOKEN" ]; then
        AUTH_HEADER="Authorization: Bearer $TOKEN"
        echo "Authenticated using gcloud identity token."
    fi
fi

echo "Testing A2A endpoint at: $ENDPOINT_URL"

# Test agent-card endpoint for health check
echo "Checking health via agent card..."
if [ -n "$AUTH_HEADER" ]; then
    HEALTH_RESP=$(curl -s -H "$AUTH_HEADER" -w "%{http_code}" -o /dev/null "$ENDPOINT_URL/.well-known/agent.json")
else
    HEALTH_RESP=$(curl -s -w "%{http_code}" -o /dev/null "$ENDPOINT_URL/.well-known/agent.json")
fi

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
      "messageId": "test-msg-gcp-id",
      "role": "user",
      "parts": [
        {
          "kind": "text",
          "text": "Calculate 5 Mersenne primes"
        }
      ],
      "contextId": "test-ctx-gcp-id"
    }
  }
}'

if [ -n "$AUTH_HEADER" ]; then
    RESPONSE=$(curl -s -X POST \
      -H "$AUTH_HEADER" \
      -H "Content-Type: application/json" \
      -d "$PAYLOAD" \
      "$ENDPOINT_URL/")
else
    RESPONSE=$(curl -s -X POST \
      -H "Content-Type: application/json" \
      -d "$PAYLOAD" \
      "$ENDPOINT_URL/")
fi

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
