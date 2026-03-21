from google.adk.agents.remote_a2a_agent import AGENT_CARD_WELL_KNOWN_PATH
from google.adk.agents.remote_a2a_agent import RemoteA2aAgent
from google.adk.agents.llm_agent import LlmAgent
from google.adk.tools import load_memory
from google.adk.tools.tool_context import ToolContext
from google.adk.a2a.utils.agent_to_a2a import to_a2a
import os
import uvicorn

# poly master is running on 8085
# Go Prime checker is on 8086
# Python random number agent is on 8087
# FUTURE Rust Prime Generator is on 8088
# FUTURE 8089 TBD
# FUTURE Java number factor is on 8090
# Node Prime Generator is on 8091


primecheck_agent = RemoteA2aAgent(
    name="primecheck_agent",
    description="This agent written in Go checks for primes",
    agent_card=(
        f"http://127.0.0.1:8086/{AGENT_CARD_WELL_KNOWN_PATH}"
    ),
)

gen_agent = RemoteA2aAgent(
    name="primegenerator_agent",
    description="Prime Generation Agent written in JS",
    agent_card=(
        f"http://127.0.0.1:8091/{AGENT_CARD_WELL_KNOWN_PATH}"
    ),
)

rand_agent = RemoteA2aAgent(
    name="rand_agent",
    description="Random Number Agent written in Python",
    agent_card=(
        f"http://127.0.0.1:8087/{AGENT_CARD_WELL_KNOWN_PATH}"
    ),
)

root_agent = LlmAgent(
    name="master_agent",
    model="gemini-2.5-flash",
    instruction="""
        You are the Master Agent
        you delegate to your sub agents by the a2a protocol
        If the user asks to check primes, delegate to the primecheck_agent.
        If the user asks to generate a random number and then check if the result is prime, call rand_agent first, then pass the result to primecheck_agent.
        If the user asks to generate a random number and check if the number is prime, call rand_agent first, then pass the result to primecheck_agent.
        If the users asks to generate a random number and check if it is prime, call rand_agent first then pass the result to primecheck_agent.

    """,
    sub_agents=[primecheck_agent,gen_agent,rand_agent]
)

if __name__ == "__main__":
    a2a_app = to_a2a(root_agent, port=8085)
    # Use host='0.0.0.0' to allow external access.
    uvicorn.run(a2a_app, host="0.0.0.0", port=8085)

