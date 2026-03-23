#!/bin/bash

# Check if gcloud is authenticated
if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | grep -q "@"; then
    echo "Error: No active gcloud account found."
    echo "Please run 'gcloud auth login' and try again."
fi

# Get current project
PROJECT_ID=$(gcloud config get-value project 2>/dev/null)
if [ "$PROJECT_ID" == "(unset)" ] || [ -z "$PROJECT_ID" ]; then
    echo "Warning: No gcloud project is currently set."
    echo "Run 'gcloud config set project [PROJECT_ID]' to configure it."
fi

cat <<EOF > .env
export GOOGLE_GENAI_USE_VERTEXAI=1
export GOOGLE_CLOUD_PROJECT=$PROJECT_ID
export GOOGLE_CLOUD_LOCATION=us-central1
export IMAGEN_MODEL="imagen-3.0-fast-generate-001"
export GENAI_MODEL="gemini-2.5-flash"
EOF

source .env

echo "Current Environment"
cat .env

echo "Cloud Login"
gcloud auth list

echo "ADK Version"
adk --version
