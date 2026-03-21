"""This module defines a simple agent that can get events in NYC."""

from google.adk.agents import Agent
from google.adk.a2a.utils.agent_to_a2a import to_a2a
from google.adk.tools import google_search
import uvicorn

root_agent = Agent(
    name="events_agent",
    model="gemini-2.5-flash",
    description="Agent to find events in NYC using Google Search.",
    instruction="Find 3 upcoming events in New York City",
    # google_search is a pre-built tool which allows the agent to perform Google searches.
    tools=[google_search]
)

if __name__ == "__main__":
    a2a_app = to_a2a(root_agent, port=8082)
    # Use host='0.0.0.0' to allow external access.
    uvicorn.run(a2a_app, host="0.0.0.0", port=8082)
