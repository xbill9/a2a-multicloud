import os
import subprocess
import json
import logging
import time
from dotenv import load_dotenv
from pythonjsonlogger import jsonlogger

# Configure logging
logger = logging.getLogger(__name__)
logHandler = logging.StreamHandler()
formatter = jsonlogger.JsonFormatter(
    "%(asctime)s %(levelname)s %(name)s %(message)s"
)
logHandler.setFormatter(formatter)
logger.addHandler(logHandler)
logger.setLevel(logging.INFO)

def run_command(command, error_msg, capture=True, check=True, retries=0, retry_delay=5, error_substring_to_retry=None):
    """Utility to run shell commands with optional retries for specific errors."""
    for attempt in range(retries + 1):
        try:
            result = subprocess.run(command, check=check, text=True, capture_output=capture)
            return result.stdout.strip() if capture else True
        except subprocess.CalledProcessError as e:
            if error_substring_to_retry and error_substring_to_retry in e.stderr and attempt < retries:
                logger.warning(
                    f"Command failed with '{error_substring_to_retry}'. Retrying in {retry_delay}s... (Attempt {attempt+1}/{retries})"
                )
                time.sleep(retry_delay)
                retry_delay *= 2  # Exponential backoff
            else:
                if check:
                    logger.error(f"❌ {error_msg}", extra={"error_details": e.stderr if capture else "N/A", "command": command})
                return None
    return None # Should not be reached if check=True and all retries fail

def setup_service_account(project_id):
    """Ensures the service account exists and has the necessary roles efficiently."""
    sa_name = "a2a-agent"
    sa_email = f"{sa_name}@{project_id}.iam.gserviceaccount.com"
    member = f"serviceAccount:{sa_email}"
    
    # 1. Check if Service Account exists
    logger.info(f"🔍 Checking if service account {sa_email} exists...", extra={"sa_email": sa_email})
    check_cmd = ["gcloud", "iam", "service-accounts", "describe", sa_email, f"--project={project_id}", "--format=json"]
    
    # We use check=False to handle the "Not Found" case manually
    sa_info = run_command(check_cmd, "Checking SA existence", capture=True, check=False)
    
    if sa_info is None:
        logger.info(f"🛠️ Creating service account: {sa_name}...", extra={"sa_name": sa_name})
        create_cmd = [
            "gcloud", "iam", "service-accounts", "create", sa_name,
            f"--display-name=ADK Visual Builder Service Account",
            f"--project={project_id}"
        ]
        run_command(create_cmd, "Failed to create service account.")
        # Wait for service account propagation
        logger.info("⏳ Waiting for service account propagation (10s)...")
        time.sleep(10)
    else:
        logger.info(f"✅ Service account {sa_name} already exists.")

    # 2. Define roles to assign
    required_roles = [
        "roles/cloudbuild.builds.builder",
        "roles/iam.serviceAccountUser",
        "roles/storage.admin",
        "roles/aiplatform.user",
        "roles/run.admin",
        "roles/logging.logWriter",
        "roles/artifactregistry.writer",
        "roles/storage.objectViewer",
    ]
    
    # 3. Fetch current IAM policy to find missing roles (Fast check)
    logger.info("🔐 Checking current IAM policy for missing roles...")
    get_policy_cmd = ["gcloud", "projects", "get-iam-policy", project_id, "--format=json"]
    policy_json = run_command(get_policy_cmd, "Failed to fetch IAM policy")
    
    missing_roles = []
    if policy_json:
        policy = json.loads(policy_json)
        # Create a set of roles the member already has
        existing_roles = set()
        for binding in policy.get("bindings", []):
            if member in binding.get("members", []):
                existing_roles.add(binding.get("role"))
        
        missing_roles = [r for r in required_roles if r not in existing_roles]
    else:
        # Fallback to applying all if policy fetch failed but didn't crash
        missing_roles = required_roles

    if not missing_roles:
        logger.info("✅ All required IAM roles are already assigned.")
    else:
        logger.info(f"⚡ Assigning {len(missing_roles)} missing IAM roles...", extra={"missing_roles": missing_roles})
        for role in missing_roles:
            bind_cmd = [
                "gcloud", "projects", "add-iam-policy-binding", project_id,
                f"--member={member}",
                f"--role={role}",
                "--condition=None",
                "--quiet"
            ]
            run_command(bind_cmd, f"Failed to assign role {role}", retries=5, retry_delay=10, error_substring_to_retry="Service account .* does not exist.")
        
        # Wait for IAM propagation only if we actually changed something
        logger.info("⏳ Waiting for IAM propagation (30s)...")
        time.sleep(30)

    return sa_email

def deploy_agent():
    # 1. Load configuration
    load_dotenv()
    project_id = os.getenv("GOOGLE_CLOUD_PROJECT")
    
    if not project_id:
        logger.error("❌ Error: GOOGLE_CLOUD_PROJECT not found in .env file.")
        return

    location = os.getenv("GOOGLE_CLOUD_LOCATION", "us-central1")
    agent_path = os.getenv("AGENT_PATH", "./src/agents/a2a_events_nyc")
    service_name = os.getenv("SERVICE_NAME", "a2a-events-nyc")
    app_name = os.getenv("APP_NAME", "a2aeventsnyc")

    # 2. Setup Service Account and Permissions
    sa_email = setup_service_account(project_id)
    if not sa_email:
        logger.error("❌ Service account setup failed. Aborting deployment.")
        return

    # 3. Execute Deployment Command
    command = [
        "adk", "deploy", "cloud_run",
        f"--project={project_id}",
        f"--region={location}",
        f"--service_name={service_name}",
        f"--app_name={app_name}",
        f"--artifact_service_uri=memory://",
        f"--with_ui",
        agent_path,
        f"--",
        f"--service-account={sa_email}",
        f"--build-service-account=projects/{project_id}/serviceAccounts/{sa_email}",
        f"--allow-unauthenticated",
        f"--set-env-vars=GOOGLE_CLOUD_PROJECT={project_id},GOOGLE_CLOUD_LOCATION={location},GOOGLE_GENAI_USE_VERTEXAI=1",
    ]

    logger.info(f"🚀 Deploying agent '{app_name}' to {project_id} using {sa_email}...", extra={
        "app_name": app_name,
        "project_id": project_id,
        "sa_email": sa_email
    })
    
    run_command(command, "Deployment failed", capture=False, retries=5, error_substring_to_retry="Service account .* does not exist.")

if __name__ == "__main__":
    deploy_agent()
