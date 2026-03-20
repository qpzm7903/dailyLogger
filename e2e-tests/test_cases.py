"""
Test Cases for DailyLogger E2E Tests

Each test case is defined as a function that can be run by the AI agent.
"""

from dataclasses import dataclass
from typing import Optional


@dataclass
class TestCase:
    """Represents a test case definition."""

    id: str
    name: str
    description: str
    task: str  # Natural language task for the AI agent
    expected_elements: Optional[list] = None  # Elements that should be visible
    expected_outcome: Optional[str] = None  # Expected final state


# Define test cases
TEST_CASES = [
    TestCase(
        id="TC001",
        name="Quick Note - Create Note",
        description="Test creating a new quick note via the quick note modal",
        task="""Test the quick note feature:
1. Find and click the button to open quick note modal
2. Type 'E2E test note from AI agent' in the text area
3. Add a tag 'test-tag' if possible
4. Click the save/submit button
5. Verify the note was saved successfully
""",
        expected_elements=["quick note button", "text input", "save button"],
        expected_outcome="Note is saved and visible in the history",
    ),
    TestCase(
        id="TC002",
        name="Settings - API Configuration",
        description="Test viewing and modifying API settings",
        task="""Test the settings panel:
1. Find and click the settings button (gear icon)
2. Verify the settings panel opens
3. Check that API Base URL, API Key, and Model fields exist
4. Close the settings panel
5. Verify the app returns to main view
""",
        expected_elements=["settings button", "API Base URL input", "Model selector"],
        expected_outcome="Settings can be opened and closed",
    ),
    TestCase(
        id="TC003",
        name="Screenshot Gallery - View Screenshots",
        description="Test viewing screenshots in the gallery",
        task="""Test the screenshot gallery:
1. Find and click the button to view screenshots/gallery
2. Verify the gallery view is displayed
3. If screenshots exist, verify they are shown
4. If no screenshots, verify empty state is shown
5. Navigate back to main view
""",
        expected_elements=["gallery button", "screenshot container or empty state"],
        expected_outcome="Gallery view can be accessed",
    ),
    TestCase(
        id="TC004",
        name="History View - Browse Records",
        description="Test browsing historical records",
        task="""Test the history view:
1. Find and click the history button
2. Verify the history list is displayed
3. Scroll through the list if multiple records exist
4. Click on a record to view details if available
5. Return to main view
""",
        expected_elements=["history button", "record list"],
        expected_outcome="History can be browsed",
    ),
    TestCase(
        id="TC005",
        name="Report Generation - Daily Summary",
        description="Test generating a daily summary report",
        task="""Test the daily summary generation:
1. Find the report/summary generation button
2. Click to start generation
3. Verify the generation process starts (loading indicator)
4. Wait for completion or handle any error states
5. Verify result is shown or saved
""",
        expected_elements=["generate summary button"],
        expected_outcome="Summary generation can be initiated",
    ),
    TestCase(
        id="TC006",
        name="Tag Cloud - Filter by Tag",
        description="Test filtering records by tags",
        task="""Test the tag filtering feature:
1. Find the tag cloud or tag filter section
2. Click on a tag to filter records
3. Verify records are filtered by the selected tag
4. Clear the filter
5. Verify all records are shown again
""",
        expected_elements=["tag cloud", "tag filter"],
        expected_outcome="Records can be filtered by tags",
    ),
]


def get_test_case(test_id: str) -> Optional[TestCase]:
    """Get a test case by its ID."""
    for tc in TEST_CASES:
        if tc.id == test_id:
            return tc
    return None


def list_test_cases():
    """Print all available test cases."""
    print("\n📋 Available Test Cases:\n")
    for tc in TEST_CASES:
        print(f"  [{tc.id}] {tc.name}")
        print(f"      {tc.description}")
        print()


if __name__ == "__main__":
    list_test_cases()
