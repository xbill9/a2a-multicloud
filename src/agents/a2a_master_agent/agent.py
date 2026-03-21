from google.adk.agents.remote_a2a_agent import AGENT_CARD_WELL_KNOWN_PATH
from google.adk.agents.remote_a2a_agent import RemoteA2aAgent
from google.adk.agents.llm_agent import LlmAgent
from google.adk.tools import load_memory
from google.adk.tools.tool_context import ToolContext
from google.adk.a2a.utils.agent_to_a2a import to_a2a
import os
import uvicorn

# master is running on 8081

ev_agent = RemoteA2aAgent(
    name="events_agent",
    description="Events Agent",
    agent_card=(
        f"http://127.0.0.1:8082/{AGENT_CARD_WELL_KNOWN_PATH}"
    ),
)

hw_agent = RemoteA2aAgent(
    name="helloworld_agent",
    description="Hello World Agent",
    agent_card=(
        f"http://127.0.0.1:8083/{AGENT_CARD_WELL_KNOWN_PATH}"
    ),
)

wt_agent = RemoteA2aAgent(
    name="weathertime_agent",
    description="Weather and Time Agent",
    agent_card=(
        f"http://127.0.0.1:8084/{AGENT_CARD_WELL_KNOWN_PATH}"
    ),
)

root_agent = LlmAgent(
    name="master_agent",
    model="gemini-2.5-flash",
    instruction="""
        You are the Master Agent
        you delegate to your sub agents by the a2a protocol

    """,
    sub_agents=[ev_agent,hw_agent,wt_agent]
)

if __name__ == "__main__":
    a2a_app = to_a2a(root_agent, port=8081)
    # Use host='0.0.0.0' to allow external access.
    uvicorn.run(a2a_app, host="0.0.0.0", port=8081)

