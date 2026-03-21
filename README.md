# A2A Hello World

## Project Overview
This project serves as a foundational "Hello World" example for developing agents using the Google Agent Development Kit (ADK) and the Agent-to-Agent (A2A) protocol. It demonstrates how to build a simple Python-based agent capable of interacting with external tools (like fetching weather and time information) and provides a comprehensive set of scripts for local development, testing, and deployment to Google Cloud Run. The primary goal is to offer a clear, runnable template for developers to quickly get started with ADK and A2A agent development, showcasing the power of inter-agent communication.

## Setup Scripts

*   `init.sh`: Initializes the environment by prompting the user for their Google Cloud project ID and Gemini API Key. It also runs `gcloud auth application-default login` to get user credentials.
*   `set_env.sh`: Sets various environment variables required for the other scripts to run. It reads the project ID and Gemini key from the files created by `init.sh`. This script also sets the `PUBLIC_URL` environment variable, which is used to configure the public URL for the agent.

## ADK Execution Scripts

These scripts facilitate running the agent in various modes and environments:

*   `cli.sh` / `run.sh`: Runs the agent in command-line mode, allowing you to interact with it from your terminal. `run.sh` is an alias for `cli.sh`.
*   `local.sh`: Runs the agent in a local web server, accessible typically at `http://localhost:8080`.
*   `web.sh`: Runs the agent in a local web server and automatically opens the UI in your default web browser.
*   `api_server.sh`: Runs the agent in API server mode, exposing its functionalities via a RESTful API.
*   `cloudrun.sh`: Deploys the agent as a scalable service to Google Cloud Run, making it accessible publicly.

## Agent-Specific Execution Scripts

These scripts are tailored for specific agents within the project, demonstrating various A2A interactions and functionalities:

*   `a2acard.sh`: Runs the `a2a-agentcard` agent, which handles agent card-related interactions.
*   `a2aevents.sh`: Executes the `a2a-events` agent, designed for finding events with Google Search Tool.
*   `a2ahello.sh`: Runs the `a2a-hello-world` agent, a basic example of A2A communication.
*   `a2amaster.sh`: Starts the `a2a-master-agent`, which orchestrates interactions between other agents.
*   `a2atest.sh`: A utility script for testing A2A agent functionalities using the client test script.
*   `a2aweather.sh`: Runs the `a2a-weather-time` agent, demonstrating tool usage for weather and time information.

### Poly Agents Scripts
These scripts manage agents in a multi-language "Poly" environment:
*   `poly-python.sh`: Runs the Python-based random number agent (`poly_rand`).
*   `poly-master.sh`: Runs the Poly Master Agent which orchestrates agents across different languages (Go, JS, Python).
*   `poly-go.sh`: Runs the Go-based prime checker agent.
*   `adk-python.sh` / `adk-master-python.sh` / `adk-random-python.sh`: These scripts launch the ADK Web UI for the Poly Python agents.

### Utility Scripts
*   `inspect.sh`: Launches the A2A Inspector tool to visualize agent interactions.
*   `set_adc.sh`: Helper script to set up Google Application Default Credentials.

## Agent Details

The project includes several agents, each demonstrating different capabilities:

*   **A2A Hello World** (`src/agents/a2a_hello_world`): Basic tool usage (weather, time).
*   **A2A Events** (`src/agents/a2a_events`): Uses Google Search to find local events.
*   **A2A Master Agent** (`src/agents/a2a_master_agent`): Demonstrates orchestration by delegating to other agents.
*   **Poly Rand Agent** (`poly-python/agents/poly_rand`): Generates random numbers, even numbers, or odd numbers.
*   **Poly Master Agent** (`poly-python/agents/poly_master`): Orchestrates a multi-language setup, delegating to Go, JS, and Python agents.

## A2A Protocol Integration

This project highlights the integration of the Agent-to-Agent (A2A) protocol, enabling seamless communication between different agents. The agent-specific scripts (e.g., `a2ahello.sh`, `a2aweather.sh`) are designed to run agents in A2A mode, allowing them to send and receive messages from other agents.

### Cross-Language Orchestration
The Poly Master Agent (`poly-master.sh`) demonstrates how an ADK agent can orchestrate sub-agents written in different languages (Go, Node.js, Python) using the A2A protocol. This is a powerful pattern for building heterogeneous multi-agent systems.

## Development

To extend or modify this agent:

1.  **Locate Agent Code:** The main agent implementations are in `src/agents/` and `poly-python/agents/`.
2.  **Add Tools:** Define new tools within the agent's `tools` list.
3.  **Modify Agent Logic:** Adjust the agent's prompt or add new functionalities within the `agent.py` file.
4.  **Dependencies:** If new Python packages are required, add them to the respective `requirements.txt` in the agent's directory.
5.  **Testing:** Utilize the `a2atest.sh` script to verify inter-agent communication.
6.  **Inspection:** Use `inspect.sh` to debug and visualize A2A message flows.
