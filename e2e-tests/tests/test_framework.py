"""
Tests for the E2E testing framework itself
"""

import pytest
from unittest.mock import Mock, patch, AsyncMock


class TestUIStateExtractor:
    """Tests for UI state extraction."""

    @pytest.mark.asyncio
    async def test_ui_element_to_description(self):
        """Test UIElement description generation."""
        from ui_state_extractor import UIElement

        elem = UIElement(
            id="test-button",
            tag="button",
            type="button",
            text="Click Me",
            is_visible=True,
            is_enabled=True,
        )

        desc = elem.to_description()
        assert "test-button" in desc
        assert "Click Me" in desc
        assert "button" in desc

    @pytest.mark.asyncio
    async def test_ui_element_disabled(self):
        """Test UIElement with disabled state."""
        from ui_state_extractor import UIElement

        elem = UIElement(
            id="disabled-input",
            tag="input",
            type="text",
            placeholder="Enter text",
            is_visible=True,
            is_enabled=False,
        )

        desc = elem.to_description()
        assert "disabled" in desc
        assert "Enter text" in desc


class TestAgentAction:
    """Tests for agent action handling."""

    def test_action_type_enum(self):
        """Test ActionType enum values."""
        from agent_loop import ActionType

        assert ActionType.CLICK.value == "click"
        assert ActionType.TYPE.value == "type"
        assert ActionType.DONE.value == "done"
        assert ActionType.FAIL.value == "fail"

    def test_agent_action_model(self):
        """Test AgentAction pydantic model."""
        from agent_loop import AgentAction

        action = AgentAction(
            action="click",
            target="submit-button",
            description="Click the submit button",
        )

        assert action.action.value == "click"
        assert action.target == "submit-button"


class TestTestCases:
    """Tests for test case definitions."""

    def test_get_test_case(self):
        """Test retrieving test cases."""
        from test_cases import get_test_case, TEST_CASES

        tc = get_test_case("TC001")
        assert tc is not None
        assert tc.name == "Quick Note - Create Note"

        tc = get_test_case("NONEXISTENT")
        assert tc is None

    def test_test_case_fields(self):
        """Test test case field structure."""
        from test_cases import TestCase

        tc = TestCase(
            id="TEST", name="Test Case", description="A test case", task="Do something"
        )

        assert tc.id == "TEST"
        assert tc.expected_elements is None
        assert tc.expected_outcome is None
