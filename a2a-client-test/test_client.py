import asyncio
import logging
import traceback
from typing import Any
from uuid import uuid4

import httpx
from a2a.client import A2AClient, A2ACardResolver
from a2a.client.errors import A2AClientHTTPError
from a2a.types import (
    MessageSendParams,
    SendMessageRequest,
)

async def run_single_turn_test(client: A2AClient) -> None:
    """Runs a single-turn test."""
    logging.info("--- 🚀 Running single-turn test... ---")
    request = SendMessageRequest(
        id=str(uuid4()),
        params=MessageSendParams(
            message={
                "messageId": str(uuid4()),
                "role": "user",
                "parts": [{"text": "hello"}],
            }
        ),
    )
    response = await client.send_message(request)
    logging.info(f"--- 📩 Agent response: {response} ---")


async def _test_agent_at_port(port: int) -> None:
    """Tests the agent at a specific port."""
    agent_url = f"http://localhost:{port}"
    logging.info(f"--- 🔄 Connecting to agent at {agent_url}... ---")
    try:
        async with httpx.AsyncClient() as httpx_client:
            # Create a resolver to fetch the agent card
            resolver = A2ACardResolver(
                httpx_client=httpx_client,
                base_url=agent_url,
            )
            agent_card = await resolver.get_agent_card()
            # Create a client to interact with the agent
            client = A2AClient(
                httpx_client=httpx_client,
                agent_card=agent_card,
            )
            logging.info(f"--- ✅ Connection successful to {agent_url}. ---")

            await run_single_turn_test(client)

    except (A2AClientHTTPError, httpx.ConnectError) as e:
        logging.error(f"--- ❌ Connection error on {agent_url}: {e} ---")
        logging.error(f"Could not connect to the agent at {agent_url}.")
        logging.error("Please ensure the a2a server is running and accessible.")
    except Exception as e:
        logging.error(f"--- ❌ An unexpected error occurred on {agent_url}: {e} ---")
        traceback.print_exc()


async def main() -> None:
    """Main function to run the tests."""
    logging.basicConfig(level=logging.INFO)
    ports = [8081, 8082, 8083, 8084]
    for port in ports:
        await _test_agent_at_port(port)


if __name__ == "__main__":
    asyncio.run(main())
