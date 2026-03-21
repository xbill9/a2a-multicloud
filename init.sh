#!/bin/bash

# Check if gcloud is authenticated
if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | grep -q "@"; then
    echo "Error: No active gcloud account found."
    echo "Please run 'gcloud auth login' and try again."
    exit 1
fi

if [ -f "$HOME/project_id.txt" ]; then
    PROJECT_ID=$(cat "$HOME/project_id.txt")
else
    read -p "Enter Project ID: " PROJECT_ID
    echo "$PROJECT_ID" > "$HOME/project_id.txt"
fi

gcloud config set project "$PROJECT_ID"

gcloud services enable aiplatform.googleapis.com
gcloud services enable cloudresourcemanager.googleapis.com
gcloud services enable artifactregistry.googleapis.com
gcloud services enable cloudbuild.googleapis.com
gcloud services enable run.googleapis.com

gcloud services enable cloudaicompanion.googleapis.com


#curl -s https://raw.githubusercontent.com/haren-bh/gcpbillingactivate/main/activate.py | python3

cat <<EOF > .env
GOOGLE_GENAI_USE_VERTEXAI=1
GOOGLE_CLOUD_PROJECT=$PROJECT_ID
GOOGLE_CLOUD_LOCATION=us-central1
IMAGEN_MODEL="imagen-3.0-fast-generate-001"
GENAI_MODEL="gemini-2.5-flash"
EOF

source .env

if [ -z "$CLOUD_SHELL" ]; then
    if ! gcloud auth application-default print-access-token > /dev/null 2>&1; then
        echo "ADC expired or not found. Initializing login..."
        gcloud auth application-default login
    else
        echo "ADC is valid."
    fi
fi

if [ ! -f ".requirements_installed" ]; then
    pip install -r requirements.txt
    touch .requirements_installed
fi

echo "Environment setup"
cat .env

echo "Cloud Login"
gcloud auth list

echo "ADK update"
pip install google-adk --upgrade
adk --version
