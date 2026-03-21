"""This module defines a simple agent that can get events in NYC."""

import os
from google.adk.agents import Agent
from google.adk.a2a.utils.agent_to_a2a import to_a2a
from google.adk.tools import google_search

root_agent = Agent(
    name="a2aeventsnyc",
    model="gemini-2.5-flash",
    description="Agent to find events in NYC using Google Search.",
    instruction="Find 3 upcoming events in New York City",
    # google_search is a pre-built tool which allows the agent to perform Google searches.
    tools=[google_search]
)

# Move a2a_app to module level so it can be discovered by the deployment tool.
a2a_app = to_a2a(root_agent)
