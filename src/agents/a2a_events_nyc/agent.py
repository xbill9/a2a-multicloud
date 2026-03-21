"""This module defines a simple agent that can get events in NYC."""

import os
import logging
from google.adk.agents import Agent
from google.adk.a2a.utils.agent_to_a2a import to_a2a

# Robust import for the search tool
try:
    from google.adk.tools import google_search
except ImportError:
    from google.adk.tools import google_web_search as google_search

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

root_agent = Agent(
    name="a2a_events_nyc",
    model="gemini-2.5-flash",
    description="Agent to find events in NYC using Google Search.",
    instruction="Find 3 upcoming events in New York City",
    # google_search is a pre-built tool which allows the agent to perform Google searches.
    tools=[google_search]
)

# Move a2a_app to module level so it can be discovered by the deployment tool.
# Pass the port to to_a2a to ensure internal configuration matches.
port = int(os.environ.get("PORT", 8080))

# Configure A2A metadata
a2a_protocol = os.environ.get("A2A_PROTOCOL", "http")
a2a_host = os.environ.get("A2A_HOST", "localhost")
a2a_port = int(os.environ.get("A2A_PORT", port))

a2a_app = to_a2a(
    root_agent, 
    port=a2a_port, 
    host=a2a_host, 
    protocol=a2a_protocol
)

if __name__ == "__main__":
    import uvicorn
    # Use host='0.0.0.0' to allow external access.
    logger.info(f"Serving agent on port {port}")
    uvicorn.run(a2a_app, host="0.0.0.0", port=port)
