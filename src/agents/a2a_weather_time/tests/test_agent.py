import unittest
import sys
import os

# Add the parent directory to the Python path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from agent import get_weather, get_current_time  # noqa: E402


class TestAgent(unittest.TestCase):

    def test_get_weather_success(self):
        response = get_weather("new york")
        self.assertEqual(response["status"], "success")
        self.assertIn("sunny", response["report"])

    def test_get_weather_fail(self):
        response = get_weather("london")
        self.assertEqual(response["status"], "error")
        self.assertIn("not available", response["error_message"])

    def test_get_current_time_success(self):
        response = get_current_time("new york")
        self.assertEqual(response["status"], "success")
        self.assertIn("current time in new york is", response["report"].lower())

    def test_get_current_time_fail(self):
        response = get_current_time("london")
        self.assertEqual(response["status"], "error")
        self.assertIn("don't have timezone information", response["error_message"])


if __name__ == "__main__":
    unittest.main()
