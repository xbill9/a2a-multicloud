import argparse
import asyncio
import logging
import pprint
import traceback

import httpx
from a2a.client import A2ACardResolver
from a2a.client.errors import A2AClientHTTPError
from a2a.types import AgentCard


def _format_card(card: AgentCard) -> str:
    """Formats the agent card for pretty display."""
    lines = [
        "*" * 60,
        f"** Agent: {card.name}",
    ]
    if getattr(card, "id", None):
        lines.append(f"** ID: {card.id}")
    lines.append(f"** Version: {card.version}")
    if getattr(card, "author", None):
        lines.append(f"** Author: {card.author}")
    lines.append(f"** Description: {card.description}")
    if getattr(card, "license", None):
        lines.append(f"** License: {card.license}")
    if getattr(card, "homepage", None):
        lines.append(f"** Homepage: {card.homepage}")
    if getattr(card, "documentation", None):
        lines.append(f"** Documentation: {card.documentation}")
    if getattr(card, "source_code", None):
        lines.append(f"** Source Code: {card.source_code}")
    lines.extend(
        [
            f"** A2A Protocol Version: {card.protocol_version}",
            f"** URL: {card.url}",
        ]
    )

    lines.append("** Transports:")
    if getattr(card, "transports", None):
        for transport in card.transports:
            lines.append(f"    - Type: {transport.type}")
            lines.append(f"      URL: {transport.url}")
    else:
        lines.append("    No transports defined.")

    lines.append("** Skills:")
    if card.skills:
        for skill in card.skills:
            lines.append(f"    - Name: {skill.name}")
            lines.append(f"      ID: {skill.id}")
            lines.append(f"      Description: {skill.description}")
            lines.append(f"      Tags: {', '.join(skill.tags)}")
            if getattr(skill, "parameters", None):
                lines.append("      Parameters:")
                for param in skill.parameters:
                    lines.append(f"        - Name: {param.name}")
                    lines.append(f"          Type: {param.type}")
                    lines.append(f"          Description: {param.description}")
                    lines.append(f"          Required: {param.required}")
            if getattr(skill, "returns", None):
                lines.append("      Returns:")
                lines.append(f"        - Type: {skill.returns.type}")
                lines.append(f"          Description: {skill.returns.description}")

    else:
        lines.append("    No skills defined.")
    lines.append("*" * 60)
    return "\n".join(lines)


async def display_agent_card(url: str) -> None:
    """Fetches and displays the agent card from a given URL."""
    logging.info(f"--- 🃏 Fetching agent card from {url}... ---")
    try:
        async with httpx.AsyncClient() as httpx_client:
            resolver = A2ACardResolver(
                httpx_client=httpx_client,
                base_url=url,
            )
            agent_card = await resolver.get_agent_card()

            logging.info(f"--- ✅ Agent Card for {url}: ---")
            logging.info("\n" + _format_card(agent_card))

    except (A2AClientHTTPError, httpx.ConnectError) as e:
        logging.error(f"--- ❌ Connection error on {url}: {e} ---")
        logging.error(f"Could not connect to the agent at {url}.")
    except Exception as e:
        logging.error(f"--- ❌ An unexpected error occurred on {url}: {e} ---")
        traceback.print_exc()


async def main() -> None:
    """Main function to fetch and display agent cards."""
    logging.basicConfig(level=logging.INFO, format="%(message)s")
    parser = argparse.ArgumentParser(
        description="Fetch and display A2A Agent Cards from one or more agent URLs."
    )
    parser.add_argument(
        "urls",
        nargs="*",
        help="List of agent base URLs (e.g., http://localhost:8081). "
        "If not provided, defaults to checking common local ports.",
    )
    args = parser.parse_args()

    urls_to_check = args.urls
    if not urls_to_check:
        default_urls = [
            "http://localhost:8081",
            "http://localhost:8082",
            "http://localhost:8083",
            "http://localhost:8084",
        ]
        logging.info(
            "No URLs provided. Checking default local URLs: %s", ", ".join(default_urls)
        )
        urls_to_check = default_urls

    tasks = [display_agent_card(url) for url in urls_to_check]
    await asyncio.gather(*tasks)


if __name__ == "__main__":
    asyncio.run(main())
