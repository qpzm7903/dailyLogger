"""
E2E Test for Vue Dev Server

Tests the AI agent loop against the running Vue frontend.
"""

import asyncio
import os
import json
import sys

from dotenv import load_dotenv


def safe_print(text: str):
    """Print text safely, handling encoding issues."""
    try:
        print(text)
    except UnicodeEncodeError:
        # Replace non-encodable characters
        print(
            text.encode(sys.stdout.encoding or "utf-8", errors="replace").decode(
                sys.stdout.encoding or "utf-8"
            )
        )


from playwright.async_api import async_playwright
from openai import OpenAI
from openai.types.chat import ChatCompletionMessageParam

# Load environment variables
load_dotenv()


async def extract_page_elements(page) -> str:
    """Extract interactive elements from the page."""
    result = await page.evaluate("""() => {
        const result = {
            buttons: [],
            inputs: [],
            headings: [],
            visibleText: ''
        };
        
        // Helper to clean text (remove problematic unicode)
        const cleanText = (t) => t ? t.replace(/[^\\x00-\\x7F\\u4e00-\\u9fff]/g, '?') : t;
        
        // Get buttons
        document.querySelectorAll('button').forEach(b => {
            const text = b.innerText?.trim().slice(0, 50);
            if (text && b.offsetParent !== null) {
                result.buttons.push({
                    text: cleanText(text),
                    id: b.id || null,
                    className: b.className?.slice(0, 50) || null
                });
            }
        });
        
        // Get inputs
        document.querySelectorAll('input, textarea').forEach(i => {
            if (i.offsetParent !== null) {
                result.inputs.push({
                    type: i.type || 'text',
                    placeholder: i.placeholder || null,
                    name: i.name || null,
                    id: i.id || null
                });
            }
        });
        
        // Get headings
        document.querySelectorAll('h1, h2, h3').forEach(h => {
            const text = h.innerText?.trim();
            if (text) {
                result.headings.push(cleanText(text));
            }
        });
        
        // Get visible text snippets
        result.visibleText = document.body.innerText.slice(0, 800).replace(/[^\\x00-\\x7F\\u4e00-\\u9fff]/g, '?');
        
        return JSON.stringify(result, null, 2);
    }""")
    return result


async def run_e2e_test():
    """Run a real E2E test against the Vue dev server."""

    print("=" * 60)
    print("  DailyLogger E2E Test - AI Agent Mode")
    print("=" * 60)
    print()

    # Initialize OpenAI client
    client = OpenAI(
        api_key=os.getenv("OPENAI_API_KEY"),
        base_url=os.getenv("OPENAI_API_BASE", "https://api.openai.com/v1"),
    )
    model = os.getenv("OPENAI_MODEL", "glm-5")

    print(f"Model: {model}")
    print(f"API Base: {os.getenv('OPENAI_API_BASE')}")
    print()

    # Connect to Vue dev server
    print("Connecting to Vue dev server at http://localhost:1420...")

    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=False)
        context = await browser.new_context()
        page = await context.new_page()

        try:
            await page.goto("http://localhost:1420", timeout=30000)
            await page.wait_for_load_state("networkidle")
        except Exception as e:
            print(f"Failed to connect: {e}")
            await browser.close()
            return

        print(f"Connected! Page title: {await page.title()}")
        print()

        # Define test task
        task = """Explore the DailyLogger application:
1. Describe what you see on the main page
2. Find and click the settings button (gear icon)
3. Describe the settings panel if it opens
4. Close the settings panel if opened
5. Report what features are visible in the app
"""

        print(f"Test Task:\n{task}")
        print()

        # System prompt
        system_prompt = """You are an automated test agent for a Vue.js application called DailyLogger.

Your job is to explore the application and report findings.

## Response Format

Always respond with a JSON object:
```json
{
    "action": "click|type|scroll|wait|describe|done|fail",
    "target": "CSS selector or element description",
    "value": "additional value if needed",
    "description": "explanation of what you're doing"
}
```

## Actions:
- click: Click an element (use CSS selector like 'button', '.settings-btn', '#my-id')
- type: Type text into input (target = selector, value = text to type)
- scroll: Scroll page (value = "up" or "down")
- wait: Wait some time (value = number of seconds)
- describe: Just describe observations, no action taken
- done: Test completed successfully
- fail: Cannot proceed with test

## Guidelines:
- Use CSS selectors for targets
- If an action fails, try alternatives
- Report what you see clearly
- Complete all test objectives before marking done
"""

        messages: list[ChatCompletionMessageParam] = [
            {"role": "system", "content": system_prompt},
            {
                "role": "user",
                "content": f"Task: {task}\n\nBegin by describing what you see on the page.",
            },
        ]

        step = 0
        max_steps = 20

        while step < max_steps:
            step += 1
            print(f"\n{'=' * 50}")
            print(f"Step {step}/{max_steps}")
            print("=" * 50)

            # Take screenshot
            screenshot_dir = os.getenv("SCREENSHOT_DIR", "./screenshots")
            os.makedirs(screenshot_dir, exist_ok=True)
            screenshot_path = os.path.join(screenshot_dir, f"e2e-step-{step}.png")
            await page.screenshot(path=screenshot_path)
            print(f"Screenshot saved: {screenshot_path}")

            # Get page elements
            page_elements = await extract_page_elements(page)
            print(f"\nPage Elements:\n{page_elements[:800]}...")

            # Ask AI for next action
            messages.append(
                {
                    "role": "user",
                    "content": f"Current page state:\n{page_elements}\n\nWhat is your next action?",
                }
            )

            try:
                response = client.chat.completions.create(
                    model=model, messages=messages, temperature=0.1
                )

                ai_response = response.choices[0].message.content or ""
                print(f"\nAI Response:\n{ai_response[:500]}...")

                messages.append({"role": "assistant", "content": ai_response})

                # Parse JSON action
                if "{" in ai_response and "}" in ai_response:
                    json_start = ai_response.index("{")
                    json_end = ai_response.rindex("}") + 1
                    action_data = json.loads(ai_response[json_start:json_end])

                    action = action_data.get("action", "describe")
                    target = action_data.get("target", "")
                    value = action_data.get("value", "")
                    desc = action_data.get("description", "")

                    print(f"\nAction: {action}")
                    if target:
                        print(f"  Target: {target}")
                    if value:
                        print(f"  Value: {value}")
                    if desc:
                        print(f"  Description: {desc}")

                    # Execute action
                    if action == "click":
                        try:
                            await page.click(target, timeout=5000)
                            print("  Click executed")
                            await asyncio.sleep(0.5)
                        except Exception as e:
                            print(f"  Click failed: {e}")

                    elif action == "type":
                        try:
                            await page.fill(target, value, timeout=5000)
                            print(f"  Typed: {value}")
                        except Exception as e:
                            print(f"  Type failed: {e}")

                    elif action == "scroll":
                        direction = -300 if value == "up" else 300
                        await page.evaluate(f"window.scrollBy(0, {direction})")
                        print("  Scrolled")

                    elif action == "wait":
                        seconds = int(value) if value.isdigit() else 2
                        await asyncio.sleep(seconds)
                        print(f"  Waited {seconds}s")

                    elif action == "done":
                        print("\n" + "=" * 60)
                        print("TEST COMPLETED SUCCESSFULLY")
                        print(f"Total steps: {step}")
                        print("=" * 60)
                        break

                    elif action == "fail":
                        print("\n" + "=" * 60)
                        print("TEST FAILED")
                        print(f"Reason: {desc}")
                        print("=" * 60)
                        break

                else:
                    print("  No valid JSON action found in response")

            except json.JSONDecodeError as e:
                print(f"  JSON parse error: {e}")
            except Exception as e:
                print(f"  Error: {e}")
                import traceback

                traceback.print_exc()

        # Final screenshot
        final_path = os.path.join(screenshot_dir, "e2e-final.png")
        await page.screenshot(path=final_path)
        print(f"\nFinal screenshot: {final_path}")

        await browser.close()

    print("\nE2E test session ended!")


if __name__ == "__main__":
    asyncio.run(run_e2e_test())
