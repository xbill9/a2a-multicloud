# Use an official Python runtime as a parent image.
FROM python:3.13-slim

# Set environment variables
ENV PYTHONUNBUFFERED=1
ENV PORT=8080

# Set the working directory in the container
WORKDIR /app

# Install requirements for the agent
COPY a2aeventsnyc/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt uvicorn

# Copy the agent source code into the container
COPY a2aeventsnyc/ .

# Expose the port the app runs on
EXPOSE 8080

# Run the agent
CMD ["python", "agent.py"]
