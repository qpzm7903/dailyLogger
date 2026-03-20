"""
Tauri CDP Connection Module

Provides functionality to connect Playwright to a Tauri application
via Chrome DevTools Protocol (CDP).
"""

import asyncio
import os
from typing import Optional, Tuple

from playwright.async_api import async_playwright, Browser, Page, BrowserContext


class TauriConnectionError(Exception):
    """Raised when connection to Tauri app fails."""

    pass


async def connect_to_tauri(
    cdp_port: int = 9222, timeout: int = 30000
) -> Tuple[Page, Browser]:
    """
    Connect to a running Tauri application via CDP.

    Prerequisites:
        1. Tauri app must be running with CDP port enabled:
           Windows: $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222"
           Then run: npm run tauri dev

    Args:
        cdp_port: The CDP port the Tauri app is listening on
        timeout: Connection timeout in milliseconds

    Returns:
        Tuple of (Page, Browser) - the main page and browser instance

    Raises:
        TauriConnectionError: If connection fails
    """
    playwright = await async_playwright().start()

    cdp_url = f"http://localhost:{cdp_port}"

    try:
        browser = await playwright.chromium.connect_over_cdp(cdp_url, timeout=timeout)
    except Exception as e:
        raise TauriConnectionError(
            f"Failed to connect to Tauri app at {cdp_url}. "
            f"Make sure the app is running with CDP enabled. Error: {e}"
        )

    # Get the default context (Tauri window)
    contexts = browser.contexts
    if not contexts:
        raise TauriConnectionError("No browser context found")

    context = contexts[0]
    pages = context.pages

    if not pages:
        raise TauriConnectionError("No pages found in Tauri window")

    page = pages[0]

    print(f"✅ Connected to Tauri app")
    print(f"   Page title: {await page.title()}")
    print(f"   URL: {page.url}")

    return page, browser


def get_tauri_launch_env(cdp_port: int = 9222) -> dict:
    """
    Get environment variables needed to launch Tauri with CDP enabled.

    Usage:
        env = get_tauri_launch_env()
        # Then set these env vars before running 'npm run tauri dev'

    Returns:
        Dict of environment variables
    """
    return {
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS": f"--remote-debugging-port={cdp_port}"
    }


async def wait_for_app_ready(page: Page, timeout: int = 10000) -> bool:
    """
    Wait for the Tauri app to be fully loaded.

    Args:
        page: The Playwright page
        timeout: Max wait time in milliseconds

    Returns:
        True if app is ready, False if timeout
    """
    try:
        # Wait for the main app container to be visible
        await page.wait_for_selector("#app", timeout=timeout)
        return True
    except Exception:
        return False


async def take_screenshot(
    page: Page, name: str, output_dir: str = "./screenshots"
) -> str:
    """
    Take a screenshot and save it.

    Args:
        page: The Playwright page
        name: Screenshot name (without extension)
        output_dir: Directory to save screenshots

    Returns:
        Path to the saved screenshot
    """
    os.makedirs(output_dir, exist_ok=True)
    path = os.path.join(output_dir, f"{name}.png")
    await page.screenshot(path=path)
    return path


# Synchronous wrapper for convenience
def connect_to_tauri_sync(cdp_port: int = 9222) -> Tuple[Page, Browser]:
    """Synchronous wrapper for connect_to_tauri."""
    return asyncio.run(connect_to_tauri(cdp_port))


if __name__ == "__main__":
    # Test connection
    async def test():
        page, browser = await connect_to_tauri()
        print(f"Connected! Page title: {await page.title()}")

    asyncio.run(test())
