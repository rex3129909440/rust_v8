"""AOSP system-font inventory for Android Edge profiles."""

from __future__ import annotations


AOSP_FONT_SOURCE = (
    "https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/data/fonts/fonts.xml"
)

ANDROID_CORE_FONT_FAMILIES: tuple[str, ...] = (
    "Roboto",
    "Roboto Condensed",
    "Roboto Flex",
    "Roboto Mono",
    "Noto Sans",
    "Noto Serif",
    "Noto Color Emoji",
    "Droid Sans",
    "Droid Sans Mono",
)

ANDROID_CORE_LOCAL_FONTS: tuple[tuple[str, str, str, str], ...] = (
    ("Roboto-Regular", "Roboto", "Roboto", "Regular"),
    ("Roboto-Medium", "Roboto Medium", "Roboto", "Medium"),
    ("Roboto-Bold", "Roboto Bold", "Roboto", "Bold"),
    ("RobotoCondensed-Regular", "Roboto Condensed", "Roboto Condensed", "Regular"),
    ("RobotoFlex-Regular", "Roboto Flex", "Roboto Flex", "Regular"),
    ("RobotoMono-Regular", "Roboto Mono", "Roboto Mono", "Regular"),
    ("NotoSans-Regular", "Noto Sans", "Noto Sans", "Regular"),
    ("NotoSerif-Regular", "Noto Serif", "Noto Serif", "Regular"),
    ("NotoColorEmoji", "Noto Color Emoji", "Noto Color Emoji", "Regular"),
    ("DroidSansMono", "Droid Sans Mono", "Droid Sans Mono", "Regular"),
)

_LANGUAGE_FAMILIES: dict[str, tuple[str, ...]] = {
    "ar": ("Noto Naskh Arabic", "Noto Sans Arabic"),
    "bn": ("Noto Sans Bengali",),
    "gu": ("Noto Sans Gujarati",),
    "he": ("Noto Sans Hebrew",),
    "hi": ("Noto Sans Devanagari",),
    "ja": ("Noto Sans CJK JP", "Noto Serif CJK JP"),
    "kn": ("Noto Sans Kannada",),
    "ko": ("Noto Sans CJK KR", "Noto Serif CJK KR"),
    "ml": ("Noto Sans Malayalam",),
    "mr": ("Noto Sans Devanagari",),
    "pa": ("Noto Sans Gurmukhi",),
    "ta": ("Noto Sans Tamil",),
    "te": ("Noto Sans Telugu",),
    "th": ("Noto Sans Thai",),
    "zh": ("Noto Sans CJK SC", "Noto Serif CJK SC"),
    "zh-cn": ("Noto Sans CJK SC", "Noto Serif CJK SC"),
    "zh-sg": ("Noto Sans CJK SC", "Noto Serif CJK SC"),
    "zh-hk": ("Noto Sans CJK HK", "Noto Serif CJK TC"),
    "zh-tw": ("Noto Sans CJK TC", "Noto Serif CJK TC"),
}


def _locale_key(locale: str) -> str:
    normalized = str(locale).strip().replace("_", "-").lower()
    return normalized if normalized in _LANGUAGE_FAMILIES else normalized.split("-", 1)[0]


def build_android_font_profile(locale: str) -> dict[str, object]:
    key = _locale_key(locale)
    additions = _LANGUAGE_FAMILIES.get(key, ())
    families = tuple(dict.fromkeys((*ANDROID_CORE_FONT_FAMILIES, *additions)))
    local_fonts = list(ANDROID_CORE_LOCAL_FONTS)
    for family in additions:
        postscript = family.replace(" ", "") + "-Regular"
        local_fonts.append((postscript, family, family, "Regular"))
    return {
        "id": f"android-aosp-{key or 'default'}",
        "source": AOSP_FONT_SOURCE,
        "families": families,
        "allowUnknownFamilies": False,
        "localFonts": tuple(dict.fromkeys(local_fonts)),
        # Roboto is the baseline. Unknown AOSP faces are intentionally not
        # assigned guessed width ratios.
        "metrics": (("Roboto", 1.0, False),),
    }


__all__ = ["AOSP_FONT_SOURCE", "build_android_font_profile"]
