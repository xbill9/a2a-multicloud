"""
Master Agent.

This agent acts as a coordinator, delegating tasks to sub-agents via the A2A protocol
"""

from google.adk.agents.remote_a2a_agent import AGENT_CARD_WELL_KNOWN_PATH, RemoteA2aAgent
from google.adk.agents.llm_agent import LlmAgent
from google.adk.a2a.utils.agent_to_a2a import to_a2a
import uvicorn
from fastmcp import FastMCP
import asyncio
from google.adk.runners import InMemoryRunner
from google.genai.types import Content, Part
import os

# master is running on 8100

ev_agent = RemoteA2aAgent(
    name="events_agent",
    description="Events Agent",
    agent_card=(
        f"http://127.0.0.1:8082/{AGENT_CARD_WELL_KNOWN_PATH.lstrip('/')}"
    ),
)

nyc_events_agent = RemoteA2aAgent(
    name="a2a_events_nyc",
    description="NYC Events Agent",
    agent_card=(
        f"http://127.0.0.1:8080/{AGENT_CARD_WELL_KNOWN_PATH.lstrip('/')}"
    ),
)

hw_agent = RemoteA2aAgent(
    name="helloworld_agent",
    description="Hello World Agent",
    agent_card=(
        f"http://127.0.0.1:8083/{AGENT_CARD_WELL_KNOWN_PATH.lstrip('/')}"
    ),
)

wt_agent = RemoteA2aAgent(
    name="weathertime_agent",
    description="Weather and Time Agent",
    agent_card=(
        f"http://127.0.0.1:8084/{AGENT_CARD_WELL_KNOWN_PATH.lstrip('/')}"
    ),
)

root_agent = LlmAgent(
    name="master_agent",
    model=os.getenv("MODEL_NAME", "gemini-2.5-flash"),
    instruction="""
        You are the Master A2A Agent
        you delegate to your sub agents by the a2a protocol
        if the user asks about NYC  call the nyc_events_agent

    """,
    sub_agents=[ev_agent,nyc_events_agent,hw_agent,wt_agent]
)

print(f"Master Agent is using model: {root_agent.model}")

runner = InMemoryRunner(agent=root_agent)

mcp_server = FastMCP("master")

@mcp_server.tool
async def ask_master_agent(query: str) -> str:
    """Ask the master agent a question."""
    try:
        import uuid
        # Use unique session ID for each call
        session_id = f"session_{uuid.uuid4().hex[:8]}"
        
        # Use run_debug to handle session creation automatically
        events = await runner.run_debug(
            user_messages=query,
            user_id="user",
            session_id=session_id,
            quiet=True
        )
        
        full_text = []
        for event in events:
            # Check for model response content
            if event.content and event.content.parts:
                for part in event.content.parts:
                    if part.text:
                        full_text.append(part.text)
                        
        if not full_text:
            return "The master agent did not return a response."
            
        return "".join(full_text)
    except Exception as e:
        return f"Error communicating with master agent: {str(e)}"


if __name__ == "__main__":
    # Create the A2A application
    app = to_a2a(root_agent, port=8100)
    
    # Get the MCP server as an ASGI application (using HTTP transport, not SSE)
    # Use path="/" because we are mounting it at "/mcp"
    mcp_app = mcp_server.http_app(path="/")
    
    # Mount the MCP server onto the same application
    app.mount("/mcp", mcp_app)

    # Ensure the MCP app's lifespan is managed by the main app
    @app.on_event("startup")
    async def startup_mcp():
        app.state.mcp_lifespan = mcp_app.lifespan(mcp_app)
        await app.state.mcp_lifespan.__aenter__()

    @app.on_event("shutdown")
    async def shutdown_mcp():
        if hasattr(app.state, "mcp_lifespan"):
            await app.state.mcp_lifespan.__aexit__(None, None, None)
    
    print("Starting Master Agent on port 8100 as both A2A and MCP server (HTTP transport)...")
    uvicorn.run(app, host="0.0.0.0", port=8100)
