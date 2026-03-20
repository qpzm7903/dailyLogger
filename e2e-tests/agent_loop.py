"""
AI Agent Loop Module

Implements the "Perceive-Think-Act" cycle for AI-driven testing.
The agent observes the UI, uses an LLM to decide the next action,
and executes that action through Playwright.
"""

import json
import os
from typing import Optional, List, Dict, Any, Callable
from dataclasses import dataclass, field
from enum import Enum
from datetime import datetime

from openai import OpenAI
from openai.types.chat import ChatCompletionMessageParam
from pydantic import BaseModel
from playwright.async_api import Page, Browser

from tauri_connection import connect_to_tauri, take_screenshot
from ui_state_extractor import extract_ui_state, UIState, get_element_by_id


class ActionType(str, Enum):
    """Types of actions the agent can perform."""

    CLICK = "click"
    TYPE = "type"
    CLEAR = "clear"
    SCROLL = "scroll"
    WAIT = "wait"
    VERIFY = "verify"
    NAVIGATE = "navigate"
    DONE = "done"
    FAIL = "fail"


class AgentAction(BaseModel):
    """Represents an action the agent wants to perform."""

    action: ActionType
    target: Optional[str] = None  # Element ID
    value: Optional[str] = None  # Text to type, etc.
    description: Optional[str] = None  # Human-readable description
    reasoning: Optional[str] = None  # Why this action


@dataclass
class AgentStep:
    """Records a single step in the agent's execution."""

    step_number: int
    ui_state: UIState
    action: AgentAction
    result: Optional[str] = None
    screenshot: Optional[str] = None
    timestamp: str = field(default_factory=lambda: datetime.now().isoformat())


@dataclass
class AgentResult:
    """Final result of agent execution."""

    success: bool
    message: str
    steps: List[AgentStep] = field(default_factory=list)
    total_steps: int = 0
    duration_seconds: float = 0


SYSTEM_PROMPT = """You are an automated software testing agent for a Tauri desktop application called DailyLogger.

Your role is to execute test scenarios by interacting with the UI elements.

## Available Actions

You can perform the following actions:

1. **click** - Click on a UI element
   - Requires: target (element ID)
   
2. **type** - Type text into an input field
   - Requires: target (element ID), value (text to type)
   
3. **clear** - Clear an input field
   - Requires: target (element ID)
   
4. **scroll** - Scroll the page
   - Optional: value ("up" or "down")
   
5. **wait** - Wait for a condition
   - Optional: value (CSS selector to wait for, or number of seconds)
   
6. **verify** - Verify a condition
   - Requires: description (what to verify)
   - Returns: success/failure based on verification
   
7. **done** - Test completed successfully
   - Optional: description (summary of what was accomplished)
   
8. **fail** - Test cannot proceed
   - Requires: description (why the test failed)

## Response Format

Respond with a JSON object:
```json
{
    "action": "click|type|clear|scroll|wait|verify|done|fail",
    "target": "element-id",
    "value": "text or additional info",
    "description": "what this action does",
    "reasoning": "why I chose this action"
}
```

## Important Guidelines

1. Always use element IDs from the provided UI state
2. If you can't find an element, try to find a similar one or wait
3. Break complex tasks into simple steps
4. Verify critical operations before moving on
5. If something unexpected happens, describe it and decide how to proceed
"""


class AITestAgent:
    """
    AI-powered test agent that can autonomously test the DailyLogger app.
    """

    def __init__(
        self,
        api_key: Optional[str] = None,
        api_base: Optional[str] = None,
        model: str = "gpt-4o",
        max_steps: int = 50,
    ):
        """
        Initialize the AI test agent.

        Args:
            api_key: OpenAI API key (or from env)
            api_base: API base URL (for compatible APIs)
            model: Model to use
            max_steps: Maximum number of steps before stopping
        """
        self.client = OpenAI(
            api_key=api_key or os.getenv("OPENAI_API_KEY"),
            base_url=api_base
            or os.getenv("OPENAI_API_BASE", "https://api.openai.com/v1"),
        )
        self.model = os.getenv("OPENAI_MODEL", model)
        self.max_steps = max_steps
        self.steps: List[AgentStep] = []
        self.screenshot_dir = os.getenv("SCREENSHOT_DIR", "./screenshots")

    async def run(
        self,
        page: Page,
        task: str,
        on_step: Optional[Callable[[AgentStep], None]] = None,
    ) -> AgentResult:
        """
        Run the agent to complete a testing task.

        Args:
            page: The Playwright page (connected to Tauri)
            task: Description of the test task to perform
            on_step: Optional callback for each step

        Returns:
            AgentResult with success/failure and execution details
        """
        import time

        start_time = time.time()

        # Prepare messages with proper typing
        messages: List[ChatCompletionMessageParam] = [
            {"role": "system", "content": SYSTEM_PROMPT},
            {
                "role": "user",
                "content": f"Test Task: {task}\n\nPlease start by examining the current UI state and proceed with the test.",
            },
        ]

        step_count = 0
        last_result = None

        while step_count < self.max_steps:
            step_count += 1

            # Perceive: Get current UI state
            ui_state = await extract_ui_state(page)
            ui_description = ui_state.to_description()

            # Add UI state to messages
            messages.append(
                {"role": "user", "content": f"Current UI State:\n\n{ui_description}"}
            )

            # Take screenshot for debugging
            screenshot_path = await take_screenshot(
                page, f"step-{step_count}", self.screenshot_dir
            )

            # Think: Ask LLM for next action
            try:
                response = self.client.chat.completions.create(
                    model=self.model,
                    messages=messages,
                    temperature=0.1,
                    response_format={"type": "json_object"},
                )

                action_json = response.choices[0].message.content
                if not action_json:
                    raise ValueError("Empty response from LLM")
                action_data = json.loads(action_json)
                action = AgentAction(**action_data)

            except Exception as e:
                return AgentResult(
                    success=False,
                    message=f"LLM error: {str(e)}",
                    steps=self.steps,
                    total_steps=step_count,
                    duration_seconds=time.time() - start_time,
                )

            # Record step
            step = AgentStep(
                step_number=step_count,
                ui_state=ui_state,
                action=action,
                screenshot=screenshot_path,
            )

            # Act: Execute the action
            try:
                result = await self._execute_action(page, action)
                step.result = result
                last_result = result

                # Add action result to messages
                messages.append({"role": "assistant", "content": action_json})
                messages.append({"role": "user", "content": f"Action result: {result}"})

            except Exception as e:
                step.result = f"Error: {str(e)}"
                messages.append(
                    {"role": "user", "content": f"Action failed with error: {str(e)}"}
                )

            self.steps.append(step)

            if on_step:
                on_step(step)

            # Check for completion
            if action.action == ActionType.DONE:
                return AgentResult(
                    success=True,
                    message=action.description or "Test completed successfully",
                    steps=self.steps,
                    total_steps=step_count,
                    duration_seconds=time.time() - start_time,
                )

            if action.action == ActionType.FAIL:
                return AgentResult(
                    success=False,
                    message=action.description or "Test failed",
                    steps=self.steps,
                    total_steps=step_count,
                    duration_seconds=time.time() - start_time,
                )

        # Max steps reached
        return AgentResult(
            success=False,
            message=f"Maximum steps ({self.max_steps}) reached without completion",
            steps=self.steps,
            total_steps=step_count,
            duration_seconds=time.time() - start_time,
        )

    async def _execute_action(self, page: Page, action: AgentAction) -> str:
        """
        Execute an action on the page.

        Args:
            page: The Playwright page
            action: The action to execute

        Returns:
            Result description
        """
        if action.action == ActionType.CLICK:
            if not action.target:
                return "Error: click action requires target"
            locator = await get_element_by_id(page, action.target)
            if locator:
                await locator.click()
                return f"Clicked on {action.target}"
            return f"Error: Element '{action.target}' not found"

        elif action.action == ActionType.TYPE:
            if not action.target or not action.value:
                return "Error: type action requires target and value"
            locator = await get_element_by_id(page, action.target)
            if locator:
                await locator.fill(action.value)
                return f"Typed '{action.value}' into {action.target}"
            return f"Error: Element '{action.target}' not found"

        elif action.action == ActionType.CLEAR:
            if not action.target:
                return "Error: clear action requires target"
            locator = await get_element_by_id(page, action.target)
            if locator:
                await locator.clear()
                return f"Cleared {action.target}"
            return f"Error: Element '{action.target}' not found"

        elif action.action == ActionType.SCROLL:
            direction = action.value or "down"
            if direction == "down":
                await page.evaluate("window.scrollBy(0, 500)")
            else:
                await page.evaluate("window.scrollBy(0, -500)")
            return f"Scrolled {direction}"

        elif action.action == ActionType.WAIT:
            if action.value and action.value.isdigit():
                import asyncio

                await asyncio.sleep(int(action.value))
                return f"Waited {action.value} seconds"
            elif action.value:
                await page.wait_for_selector(action.value, timeout=10000)
                return f"Waited for element: {action.value}"
            else:
                import asyncio

                await asyncio.sleep(1)
                return "Waited 1 second"

        elif action.action == ActionType.VERIFY:
            # Verification is handled by the LLM checking the UI state
            return f"Verification requested: {action.description}"

        elif action.action == ActionType.DONE:
            return "Test completed"

        elif action.action == ActionType.FAIL:
            return "Test failed"

        else:
            return f"Unknown action: {action.action}"


async def run_test(task: str, cdp_port: int = 9222, **agent_kwargs) -> AgentResult:
    """
    Convenience function to run a test task.

    Args:
        task: Test task description
        cdp_port: CDP port for Tauri connection
        **agent_kwargs: Additional arguments for AITestAgent

    Returns:
        AgentResult
    """
    page, browser = await connect_to_tauri(cdp_port)
    agent = AITestAgent(**agent_kwargs)

    def print_step(step: AgentStep):
        print(f"\n📍 Step {step.step_number}")
        print(f"   Action: {step.action.action.value}")
        print(f"   Target: {step.action.target}")
        print(f"   Result: {step.result}")

    result = await agent.run(page, task, on_step=print_step)

    print(
        f"\n{'✅' if result.success else '❌'} Test {'PASSED' if result.success else 'FAILED'}"
    )
    print(f"   Message: {result.message}")
    print(f"   Steps: {result.total_steps}")
    print(f"   Duration: {result.duration_seconds:.1f}s")

    return result


if __name__ == "__main__":
    import asyncio
    from dotenv import load_dotenv

    load_dotenv()

    async def main():
        result = await run_test(
            task="Test the quick note feature: click the quick note button, type 'Test note from AI agent', and save it"
        )

    asyncio.run(main())
