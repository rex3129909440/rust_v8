"""macOS font inventories used by generated Mac sandbox profiles.

Tahoe profiles use a Chromium Local Font Access capture.  The older Sequoia
fallback remains a conservative common subset.  Neither inventory depends on
the requested locale because Accept-Language does not install or remove fonts.
"""

from __future__ import annotations

try:
    from .mac_tahoe_m5_font_catalog import (
        MACOS_TAHOE_FONT_FAMILIES,
        MACOS_TAHOE_FONT_METRICS,
        MACOS_TAHOE_LOCAL_FONTS,
    )
except ImportError:  # Direct import from demo/fp.
    from mac_tahoe_m5_font_catalog import (  # type: ignore
        MACOS_TAHOE_FONT_FAMILIES,
        MACOS_TAHOE_FONT_METRICS,
        MACOS_TAHOE_LOCAL_FONTS,
    )


APPLE_SEQUOIA_FONT_SOURCE = "https://support.apple.com/en-ie/120414"
APPLE_TAHOE_FONT_SOURCE = "https://support.apple.com/en-euro/122869"


MAC_CORE_FONT_FAMILIES: tuple[str, ...] = (
    "-apple-system",
    "BlinkMacSystemFont",
    "American Typewriter",
    "Andale Mono",
    "Apple Braille",
    "Apple Chancery",
    "Apple Color Emoji",
    "Apple Symbols",
    "AppleGothic",
    "AppleMyungjo",
    "Arial",
    "Arial Black",
    "Arial Hebrew",
    "Arial Narrow",
    "Arial Rounded MT Bold",
    "Avenir",
    "Avenir Next",
    "Ayuthaya",
    "Baskerville",
    "Big Caslon",
    "Brush Script MT",
    "Chalkboard",
    "Chalkboard SE",
    "Chalkduster",
    "Charter",
    "Cochin",
    "Comic Sans MS",
    "Copperplate",
    "Courier",
    "Courier New",
    "Damascus",
    "Devanagari MT",
    "Devanagari Sangam MN",
    "Didot",
    "DIN Alternate",
    "DIN Condensed",
    "Futura",
    "Galvji",
    "Geneva",
    "Georgia",
    "Gill Sans",
    "Helvetica",
    "Helvetica Neue",
    "Hiragino Maru Gothic ProN",
    "Hiragino Mincho ProN",
    "Hiragino Sans",
    "Hiragino Sans GB",
    "Hoefler Text",
    "Impact",
    "InaiMathi",
    "Lucida Grande",
    "Luminari",
    "Marker Felt",
    "Menlo",
    "Microsoft Sans Serif",
    "Monaco",
    "Noteworthy",
    "Optima",
    "Palatino",
    "Papyrus",
    "Phosphate",
    "PingFang HK",
    "PingFang SC",
    "PingFang TC",
    "Rockwell",
    "Savoye LET",
    "SignPainter",
    "Skia",
    "Snell Roundhand",
    "Tahoma",
    "Thonburi",
    "Times",
    "Times New Roman",
    "Trebuchet MS",
    "Verdana",
    "Zapfino",
)


# postscriptName, fullName, family, style
MAC_CORE_LOCAL_FONTS: tuple[tuple[str, str, str, str], ...] = (
    ("Helvetica", "Helvetica", "Helvetica", "Regular"),
    ("HelveticaNeue", "Helvetica Neue", "Helvetica Neue", "Regular"),
    ("ArialMT", "Arial", "Arial", "Regular"),
    ("TimesNewRomanPSMT", "Times New Roman", "Times New Roman", "Regular"),
    ("CourierNewPSMT", "Courier New", "Courier New", "Regular"),
    ("Menlo-Regular", "Menlo Regular", "Menlo", "Regular"),
    ("Monaco", "Monaco", "Monaco", "Regular"),
    ("Avenir-Book", "Avenir Book", "Avenir", "Book"),
    ("AvenirNext-Regular", "Avenir Next Regular", "Avenir Next", "Regular"),
    ("Baskerville", "Baskerville", "Baskerville", "Regular"),
    ("AppleColorEmoji", "Apple Color Emoji", "Apple Color Emoji", "Regular"),
)


# family, canvas width scale, monospace
MAC_CORE_FONT_METRICS: tuple[tuple[str, float, bool], ...] = (
    ("-apple-system", 0.965, False),
    ("BlinkMacSystemFont", 0.965, False),
    ("Helvetica", 0.980, False),
    ("Helvetica Neue", 0.972, False),
    ("Arial", 1.000, False),
    ("Times", 0.990, False),
    ("Times New Roman", 1.000, False),
    ("Courier New", 1.000, True),
    ("Menlo", 1.000, True),
    ("Monaco", 1.000, True),
    ("Avenir", 0.975, False),
    ("Avenir Next", 0.978, False),
)


_LANGUAGE_FONT_ADDITIONS: dict[
    str,
    tuple[
        tuple[str, ...],
        tuple[tuple[str, str, str, str], ...],
        tuple[tuple[str, float, bool], ...],
    ],
] = {
    "zh-cn": (
        ("PingFang SC", "Heiti SC", "Kaiti SC", "Hiragino Sans GB"),
        (
            ("PingFangSC-Regular", "PingFang SC Regular", "PingFang SC", "Regular"),
            ("STHeitiSC-Light", "Heiti SC Light", "Heiti SC", "Light"),
            ("STKaitiSC-Regular", "Kaiti SC Regular", "Kaiti SC", "Regular"),
            ("HiraginoSansGB-W3", "Hiragino Sans GB W3", "Hiragino Sans GB", "W3"),
        ),
        (("PingFang SC", 1.000, False), ("Hiragino Sans GB", 1.000, False)),
    ),
    "zh-sg": (
        ("PingFang SC", "Heiti SC", "Hiragino Sans GB"),
        (("PingFangSC-Regular", "PingFang SC Regular", "PingFang SC", "Regular"),),
        (("PingFang SC", 1.000, False),),
    ),
    "zh-tw": (
        ("PingFang TC", "Heiti TC", "Kaiti TC", "Hiragino Sans TC"),
        (
            ("PingFangTC-Regular", "PingFang TC Regular", "PingFang TC", "Regular"),
            ("STHeitiTC-Light", "Heiti TC Light", "Heiti TC", "Light"),
        ),
        (("PingFang TC", 1.000, False),),
    ),
    "zh-hk": (
        ("PingFang HK", "PingFang TC", "Hiragino Sans CNS"),
        (("PingFangHK-Regular", "PingFang HK Regular", "PingFang HK", "Regular"),),
        (("PingFang HK", 1.000, False),),
    ),
    "ja": (
        ("Hiragino Sans", "Hiragino Mincho ProN", "Hiragino Maru Gothic ProN"),
        (
            ("HiraginoSans-W3", "Hiragino Sans W3", "Hiragino Sans", "W3"),
            ("HiraMinProN-W3", "Hiragino Mincho ProN W3", "Hiragino Mincho ProN", "W3"),
        ),
        (("Hiragino Sans", 1.000, False),),
    ),
    "ko": (
        ("Apple SD Gothic Neo", "AppleGothic", "AppleMyungjo"),
        (
            (
                "AppleSDGothicNeo-Regular",
                "Apple SD Gothic Neo Regular",
                "Apple SD Gothic Neo",
                "Regular",
            ),
            ("AppleGothic", "AppleGothic Regular", "AppleGothic", "Regular"),
        ),
        (("Apple SD Gothic Neo", 1.000, False),),
    ),
    "ar": (
        ("Al Bayan", "Al Nile", "Damascus", "Kigelia Arabic"),
        (("AlBayan", "Al Bayan Plain", "Al Bayan", "Plain"),),
        (("Al Bayan", 1.000, False),),
    ),
    "he": (
        ("Arial Hebrew", "Corsiva Hebrew"),
        (("ArialHebrew", "Arial Hebrew", "Arial Hebrew", "Regular"),),
        (("Arial Hebrew", 1.000, False),),
    ),
    "hi": (
        ("Devanagari MT", "Devanagari Sangam MN"),
        (("DevanagariMT", "Devanagari MT", "Devanagari MT", "Regular"),),
        (("Devanagari MT", 1.000, False),),
    ),
    "th": (
        ("Ayuthaya", "Thonburi"),
        (("Ayuthaya", "Ayuthaya", "Ayuthaya", "Regular"),),
        (("Ayuthaya", 1.000, False),),
    ),
}


def _language_key(locale: str) -> str:
    normalized = str(locale).strip().replace("_", "-").lower()
    if normalized in _LANGUAGE_FONT_ADDITIONS:
        return normalized
    return normalized.split("-", 1)[0]


def build_mac_font_profile(
    locale: str,
    macos_platform_version: str = "15.5.0",
) -> dict[str, object]:
    tahoe = str(macos_platform_version).strip().startswith("26.")
    if tahoe:
        # Captured with granted Local Font Access in Chromium 150 on an M5 Mac
        # running macOS 26.5.2.  The same 631 faces were returned on both the
        # built-in DPR-2 display and a DPR-1 secondary display.
        families = MACOS_TAHOE_FONT_FAMILIES
        local_fonts = MACOS_TAHOE_LOCAL_FONTS
        metrics = MACOS_TAHOE_FONT_METRICS
    else:
        # Locale affects font fallback order, not the fonts installed by the
        # operating system.  Do not synthesize a different installed inventory
        # by merging locale-specific/downloadable font groups.
        families = MAC_CORE_FONT_FAMILIES
        local_fonts = MAC_CORE_LOCAL_FONTS
        metrics = MAC_CORE_FONT_METRICS
    return {
        "id": "macos-tahoe-stock" if tahoe else "macos-sequoia-stock",
        "source": APPLE_TAHOE_FONT_SOURCE if tahoe else APPLE_SEQUOIA_FONT_SOURCE,
        "sourceKind": (
            "real-device-capture" if tahoe else "apple-published-inventory"
        ),
        "families": families,
        "allowUnknownFamilies": False,
        "localFonts": local_fonts,
        "metrics": metrics,
    }


__all__ = [
    "APPLE_SEQUOIA_FONT_SOURCE",
    "APPLE_TAHOE_FONT_SOURCE",
    "MAC_CORE_FONT_FAMILIES",
    "MAC_CORE_FONT_METRICS",
    "MAC_CORE_LOCAL_FONTS",
    "build_mac_font_profile",
]
