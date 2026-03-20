"""
Run E2E Tests for DailyLogger

This script sets up the environment and runs AI-driven E2E tests.
"""

import asyncio
import os
import sys
import subprocess
from pathlib import Path

from dotenv import load_dotenv

# Load environment variables
load_dotenv()


def check_prerequisites():
    """Check that all prerequisites are met."""
    errors = []

    # Check API key
    if not os.getenv("OPENAI_API_KEY"):
        errors.append("❌ OPENAI_API_KEY not set in .env file")

    # Check Tauri app is running (by checking if CDP port is accessible)
    import socket

    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    result = sock.connect_ex(("localhost", 9222))
    sock.close()
    if result != 0:
        errors.append("❌ Tauri app not running or CDP port not enabled")
        errors.append(
            "   Start with: $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=9222'; npm run tauri dev"
        )

    return errors


def print_banner():
    """Print a nice banner."""
    print("""
╔═══════════════════════════════════════════════════════════════╗
║         DailyLogger AI-Driven E2E Testing Framework          ║
╚═══════════════════════════════════════════════════════════════╝
""")


def print_test_menu():
    """Print available test scenarios."""
    tests = [
        ("1", "Quick Note Feature", "Test the quick note capture functionality"),
        ("2", "Settings Panel", "Test opening and using the settings panel"),
        ("3", "Screenshot Gallery", "Test viewing screenshots in the gallery"),
        ("4", "Daily Summary", "Test generating a daily summary"),
        ("5", "Custom Task", "Enter a custom test task"),
        ("q", "Quit", "Exit the test runner"),
    ]

    print("\n📋 Available Test Scenarios:\n")
    for key, name, desc in tests:
        print(f"  [{key}] {name}")
        print(f"      {desc}")
        print()

    return tests


async def run_test_task(task: str):
    """Run a specific test task."""
    from agent_loop import run_test

    print(f"\n🚀 Running test: {task}\n")
    print("=" * 60)

    result = await run_test(
        task=task,
        cdp_port=int(os.getenv("TAURI_CDP_PORT", "9222")),
        max_steps=int(os.getenv("MAX_AGENT_STEPS", "50")),
    )

    print("=" * 60)
    return result


def main():
    """Main entry point."""
    print_banner()

    # Check prerequisites
    errors = check_prerequisites()
    if errors:
        print("\n⚠️  Prerequisites check failed:\n")
        for error in errors:
            print(f"   {error}")
        print("\n   Please fix these issues before running tests.")
        sys.exit(1)

    print("✅ All prerequisites met\n")

    # Interactive test selection
    while True:
        tests = print_test_menu()
        choice = input("Select a test to run: ").strip().lower()

        if choice == "q":
            print("\n👋 Goodbye!")
            break
        elif choice == "1":
            task = "Test the quick note feature: find and click the quick note button, type 'Test note from AI agent', and verify it was saved"
        elif choice == "2":
            task = "Test the settings panel: open settings, verify the API configuration section exists, then close settings"
        elif choice == "3":
            task = "Test the screenshot gallery: navigate to the screenshot gallery view and verify it displays screenshots or shows empty state"
        elif choice == "4":
            task = "Test daily summary generation: find the generate summary button and click it, verify the summary creation process starts"
        elif choice == "5":
            task = input("\nEnter your test task: ").strip()
            if not task:
                print("❌ No task entered")
                continue
        else:
            print(f"❌ Invalid choice: {choice}")
            continue

        # Run the test
        asyncio.run(run_test_task(task))

        input("\nPress Enter to continue...")


if __name__ == "__main__":
    main()
