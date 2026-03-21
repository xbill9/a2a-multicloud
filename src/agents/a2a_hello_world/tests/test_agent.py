import unittest
import sys
import os

# Add the parent directory to the Python path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from agent import get_hello_world  # noqa: E402


class TestAgent(unittest.TestCase):

    def test_get_hello_world_success(self):
        response = get_hello_world()
        self.assertEqual(response["status"], "success")
        self.assertEqual(response["message"], "hello world")


if __name__ == "__main__":
    unittest.main()
