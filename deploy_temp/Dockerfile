
FROM python:3.11-slim
WORKDIR /app

# Create a non-root user
RUN adduser --disabled-password --gecos "" myuser

# Switch to the non-root user
USER myuser

# Set up environment variables - Start
ENV PATH="/home/myuser/.local/bin:$PATH"

ENV GOOGLE_GENAI_USE_VERTEXAI=1
ENV GOOGLE_CLOUD_PROJECT=comglitn
ENV GOOGLE_CLOUD_LOCATION=us-central1

# Set up environment variables - End

# Install ADK - Start
RUN pip install google-adk==1.27.2
# Install ADK - End

# Copy agent - Start

# Set permission
COPY --chown=myuser:myuser "agents/a2aeventsnyc/" "/app/agents/a2aeventsnyc/"

# Copy agent - End

# Install Agent Deps - Start
RUN pip install -r "/app/agents/a2aeventsnyc/requirements.txt"
# Install Agent Deps - End

EXPOSE 8000

CMD adk api_server --port=8000 --host=0.0.0.0 --session_service_uri=memory:// --artifact_service_uri=memory://    --a2a "/app/agents"
