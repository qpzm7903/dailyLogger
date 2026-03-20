"""
UI State Extractor Module

Extracts simplified, LLM-friendly representation of the UI state
from a Playwright page for AI agent decision making.
"""

from typing import List, Dict, Any, Optional
from dataclasses import dataclass, field
from playwright.async_api import Page


@dataclass
class UIElement:
    """Represents a UI element in a simplified format."""

    id: str
    tag: str
    type: Optional[str] = None  # button, input, textarea, etc.
    text: Optional[str] = None
    placeholder: Optional[str] = None
    label: Optional[str] = None
    aria_label: Optional[str] = None
    is_visible: bool = True
    is_enabled: bool = True
    is_focused: bool = False
    value: Optional[str] = None

    def to_description(self) -> str:
        """Convert to a human-readable description for LLM."""
        parts = [f"<{self.tag}"]

        if self.id:
            parts.append(f"id='{self.id}'")
        if self.type:
            parts.append(f"type='{self.type}'")
        if self.aria_label:
            parts.append(f"aria-label='{self.aria_label}'")
        if self.placeholder:
            parts.append(f"placeholder='{self.placeholder}'")
        if self.label:
            parts.append(f"label='{self.label}'")
        if not self.is_enabled:
            parts.append("disabled")
        if not self.is_visible:
            parts.append("hidden")

        parts.append(">")

        if self.text:
            parts.append(self.text)
        elif self.value:
            parts.append(f"[value: {self.value}]")

        parts.append(f"</{self.tag}>")

        return " ".join(parts)


@dataclass
class UIState:
    """Represents the current state of the UI."""

    elements: List[UIElement] = field(default_factory=list)
    page_title: str = ""
    url: str = ""
    focused_element_id: Optional[str] = None

    def to_description(self) -> str:
        """Convert to a description for LLM consumption."""
        lines = [
            f"Page: {self.page_title}",
            f"URL: {self.url}",
            "",
            "Visible UI Elements:",
            "-" * 40,
        ]

        for elem in self.elements:
            if elem.is_visible:
                lines.append(elem.to_description())

        if self.focused_element_id:
            lines.append("")
            lines.append(f"Currently focused: {self.focused_element_id}")

        return "\n".join(lines)


async def extract_ui_state(page: Page) -> UIState:
    """
    Extract the current UI state from a Playwright page.

    This function queries the page for interactive elements and
    returns a simplified representation suitable for LLM processing.

    Args:
        page: The Playwright page to extract UI state from

    Returns:
        UIState object containing simplified UI information
    """
    state = UIState()

    # Get page metadata
    state.page_title = await page.title()
    state.url = page.url

    # Get focused element
    focused_id = await page.evaluate("""() => {
        const focused = document.activeElement;
        return focused?.id || focused?.getAttribute('data-testid') || null;
    }""")
    state.focused_element_id = focused_id

    # Extract interactive elements using JavaScript
    elements_data = await page.evaluate("""() => {
        const elements = [];
        const interactiveSelectors = [
            'button',
            'input:not([type="hidden"])',
            'textarea',
            'select',
            '[role="button"]',
            '[role="link"]',
            '[role="menuitem"]',
            '[role="tab"]',
            '[role="checkbox"]',
            '[role="radio"]',
            '[role="switch"]',
            'a[href]',
            '[clickable]',
            '[onclick]',
            '[data-testid]'
        ];
        
        const selector = interactiveSelectors.join(', ');
        const nodes = document.querySelectorAll(selector);
        
        nodes.forEach((el, index) => {
            const rect = el.getBoundingClientRect();
            const computed = window.getComputedStyle(el);
            
            // Skip invisible elements
            if (rect.width === 0 || rect.height === 0) return;
            if (computed.visibility === 'hidden') return;
            if (computed.display === 'none') return;
            
            const id = el.id || 
                      el.getAttribute('data-testid') || 
                      el.getAttribute('name') ||
                      `elem-${index}`;
            
            const elem = {
                id: id,
                tag: el.tagName.toLowerCase(),
                type: el.type || el.getAttribute('role') || null,
                text: el.innerText?.trim().slice(0, 100) || null,
                placeholder: el.placeholder || el.getAttribute('placeholder') || null,
                label: el.getAttribute('aria-label') || 
                       el.getAttribute('data-label') || 
                       el.title || null,
                aria_label: el.getAttribute('aria-label'),
                is_visible: rect.width > 0 && rect.height > 0,
                is_enabled: !el.disabled,
                value: el.value || null
            };
            
            elements.push(elem);
        });
        
        return elements;
    }""")

    # Convert to UIElement objects
    for data in elements_data:
        elem = UIElement(
            id=data["id"],
            tag=data["tag"],
            type=data["type"],
            text=data["text"],
            placeholder=data["placeholder"],
            label=data["label"],
            aria_label=data["aria_label"],
            is_visible=data["is_visible"],
            is_enabled=data["is_enabled"],
            value=data["value"],
        )
        state.elements.append(elem)

    return state


async def get_element_by_id(page: Page, element_id: str) -> Optional[Any]:
    """
    Get a Playwright locator for an element by its ID.

    Args:
        page: The Playwright page
        element_id: The element's ID or data-testid

    Returns:
        Playwright locator or None if not found
    """
    # Try by ID first
    locator = page.locator(f"#{element_id}")
    if await locator.count() > 0:
        return locator

    # Try by data-testid
    locator = page.locator(f"[data-testid='{element_id}']")
    if await locator.count() > 0:
        return locator

    # Try by name
    locator = page.locator(f"[name='{element_id}']")
    if await locator.count() > 0:
        return locator

    return None


async def get_accessibility_tree(page: Page) -> Dict[str, Any]:
    """
    Get the accessibility tree of the page.

    This is a more structured representation of the UI that's
    especially useful for screen readers and accessibility testing.

    Args:
        page: The Playwright page

    Returns:
        Accessibility tree as a dictionary
    """
    return await page.accessibility.snapshot()


if __name__ == "__main__":
    import asyncio
    from tauri_connection import connect_to_tauri

    async def test():
        page, browser = await connect_to_tauri()
        state = await extract_ui_state(page)
        print(state.to_description())

    asyncio.run(test())
