#!/bin/bash
# Test script for AWS A2A agent

# Include credentials if they exist
if [ -f .aws_creds ]; then
    source .aws_creds
fi

# Configuration
AWS_REGION=${AWS_REGION:-us-east-1}
LIGHTSAIL_SERVICE_NAME=${LIGHTSAIL_SERVICE_NAME:-a2a-lightsail-rust-aws}

# Determine Endpoint URL
if [ -n "$1" ]; then
    ENDPOINT_URL="$1"
else
    echo "Querying AWS Lightsail for public endpoint URL..."
    ENDPOINT_URL=$(aws lightsail get-container-services \
        --service-name "$LIGHTSAIL_SERVICE_NAME" \
        --region "$AWS_REGION" \
        --query 'containerServices[0].url' \
        --output text 2>/dev/null)
fi

if [ -z "$ENDPOINT_URL" ] || [ "$ENDPOINT_URL" == "None" ] || [ "$ENDPOINT_URL" == "null" ]; then
    # Fallback to the known public URL
    ENDPOINT_URL="https://a2a-lightsail-rust-aws.6wpv8vensby5c.us-east-1.cs.amazonlightsail.com"
fi

# Clean up trailing slashes
ENDPOINT_URL="${ENDPOINT_URL%/}"

echo "Testing A2A endpoint at: $ENDPOINT_URL"

# Test /health endpoint
echo "Checking health..."
HEALTH_RESP=$(curl -s -w "%{http_code}" -o /dev/null "$ENDPOINT_URL/health")
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
      "messageId": "test-msg-aws-id",
      "role": "user",
      "parts": [
        {
          "kind": "text",
          "text": "Calculate 5 Mersenne primes"
        }
      ],
      "contextId": "test-ctx-aws-id"
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
