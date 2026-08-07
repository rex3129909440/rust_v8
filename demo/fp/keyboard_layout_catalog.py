"""Physical keyboard-layout records captured from real Chromium/Edge devices."""

from __future__ import annotations


# Microsoft Edge 150 on the local Windows US keyboard returned these 48
# unmodified physical-key mappings through navigator.keyboard.getLayoutMap().
WINDOWS_US_QWERTY_LAYOUT: tuple[tuple[str, str], ...] = (
    ("KeyK", "k"),
    ("KeyG", "g"),
    ("Digit2", "2"),
    ("Digit0", "0"),
    ("KeyV", "v"),
    ("KeyA", "a"),
    ("Backquote", "`"),
    ("KeyL", "l"),
    ("IntlBackslash", "\\"),
    ("Quote", "'"),
    ("KeyW", "w"),
    ("Digit8", "8"),
    ("KeyM", "m"),
    ("KeyH", "h"),
    ("Period", "."),
    ("Digit7", "7"),
    ("Digit1", "1"),
    ("KeyP", "p"),
    ("KeyD", "d"),
    ("KeyF", "f"),
    ("KeyO", "o"),
    ("KeyQ", "q"),
    ("KeyC", "c"),
    ("KeyN", "n"),
    ("BracketLeft", "["),
    ("KeyZ", "z"),
    ("KeyY", "y"),
    ("Digit3", "3"),
    ("Digit6", "6"),
    ("Digit5", "5"),
    ("KeyX", "x"),
    ("Slash", "/"),
    ("Backslash", "\\"),
    ("Comma", ","),
    ("Minus", "-"),
    ("Digit4", "4"),
    ("KeyB", "b"),
    ("KeyT", "t"),
    ("Digit9", "9"),
    ("KeyS", "s"),
    ("KeyI", "i"),
    ("KeyU", "u"),
    ("Equal", "="),
    ("KeyJ", "j"),
    ("Semicolon", ";"),
    ("KeyR", "r"),
    ("BracketRight", "]"),
    ("KeyE", "e"),
)


def keyboard_layout_for_profile(
    platform_name: str,
    country_code: str,
) -> tuple[tuple[str, str], ...]:
    """Return only layouts backed by a matching real-device capture."""

    if platform_name == "windows" and country_code.upper() == "US":
        return WINDOWS_US_QWERTY_LAYOUT
    return ()
