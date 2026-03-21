"""This module defines a simple agent that returns a "hello world" message."""

from google.adk.agents import Agent
from google.adk.a2a.utils.agent_to_a2a import to_a2a


def get_hello_world() -> dict:
    """Returns a simple hello world message."""
    return {"status": "success", "message": "hello world"}


root_agent = Agent(
    name="hello_world_agent",
    model="gemini-2.5-flash",
    description="Agent that returns a simple 'hello world' message.",
    instruction="You are a helpful agent who can return a 'hello world' message.",
    tools=[get_hello_world],
)

if __name__ == "__main__":
    import uvicorn

    a2a_app = to_a2a(root_agent, port=8083)
    # Use host='0.0.0.0' to allow external access.
    uvicorn.run(a2a_app, host="0.0.0.0", port=8083)
