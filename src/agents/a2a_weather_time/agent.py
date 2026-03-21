"""This module defines a simple agent that can get the weather and time."""

import datetime
from zoneinfo import ZoneInfo
from google.adk.agents import Agent
from google.adk.a2a.utils.agent_to_a2a import to_a2a
import uvicorn


def get_weather(city: str) -> dict:
    """Retrieves the current weather report
    for a specified city.

    Args:
        city (str): The name of the city for which to retrieve the weather report.

    Returns:
        dict: status and result or error msg.
    """
    if city.lower() == "new york":
        return {
            "status": "success",
            "report": (
                "The weather in New York is sunny with a temperature of 25"
                " degrees Celsius (77 degrees Fahrenheit)."
            ),
        }
    return {
        "status": "error",
        "error_message": f"Weather information for '{city}' is not available.",
    }


def get_current_time(city: str) -> dict:
    """Returns the current time in a specified city.

    Args:
        city (str): The name of the city for which to retrieve the current time.

    Returns:
        dict: status and result or error msg.
    """

    if city.lower() != "new york":
        return {
            "status": "error",
            "error_message": (f"Sorry, I don't have timezone information for {city}."),
        }
    tz_identifier = "America/New_York"
    tz = ZoneInfo(tz_identifier)
    now = datetime.datetime.now(tz)
    report = (
        f"The current time in {city} is " f'{now.strftime("%Y-%m-%d %H:%M:%S %Z%z")}'
    )
    return {"status": "success", "report": report}


def get_sunrise_sunset_time(city: str) -> dict:
    """Retrieves the sunrise and sunset times for a specified city.

    Args:
        city (str): The name of the city for which to retrieve the times.

    Returns:
        dict: status and result or error msg.
    """
    if city.lower() == "new york":
        return {
            "status": "success",
            "report": "In New York, the sun rises at 6:00 AM and sets at 8:00 PM.",
        }
    return {
        "status": "error",
        "error_message": f"Sunrise and sunset time for '{city}' is not available.",
    }


root_agent = Agent(
    name="weather_time_agent",
    model="gemini-2.5-flash",
    description=("Agent to answer questions about the time and weather in a city."),
    instruction=(
        "You are a helpful agent who can answer user questions about the time "
        "and weather and sunrise and sunset in a city."
    ),
    tools=[get_weather, get_current_time, get_sunrise_sunset_time],
)

if __name__ == "__main__":
    a2a_app = to_a2a(root_agent, port=8084)
    # Use host='0.0.0.0' to allow external access.
    uvicorn.run(a2a_app, host="0.0.0.0", port=8084)
