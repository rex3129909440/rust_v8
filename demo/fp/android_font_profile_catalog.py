"""Version- and OEM-linked system-font inventories for Android profiles.

The common rows come from AOSP.  OEM faces are added only to matching device
families; they are never mixed into every Android profile.
"""

from __future__ import annotations


AOSP_FONT_SOURCE = (
    "https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/data/fonts/fonts.xml"
)
SAMSUNG_ONE_SOURCE = "https://design.samsung.com/global/contents/samsung-one/"

ANDROID_CORE_FONT_FAMILIES: tuple[str, ...] = (
    "Roboto",
    "Roboto Condensed",
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


def build_android_font_profile(
    locale: str,
    android_version: int = 14,
    oem: str = "google",
) -> dict[str, object]:
    key = _locale_key(locale)
    additions = _LANGUAGE_FAMILIES.get(key, ())
    version = int(android_version)
    oem_key = str(oem).strip().lower() or "aosp"
    core_families = list(ANDROID_CORE_FONT_FAMILIES)
    local_fonts = [
        row for row in ANDROID_CORE_LOCAL_FONTS
        if row[2] != "Roboto Flex"
    ]
    # Roboto Flex became an AOSP system family on recent Android releases. It
    # must not leak into retained Android 10/11 device profiles.
    if version >= 14:
        core_families.append("Roboto Flex")
        local_fonts.append(
            ("RobotoFlex-Regular", "Roboto Flex", "Roboto Flex", "Regular")
        )
    sources = [AOSP_FONT_SOURCE]
    if oem_key == "samsung":
        core_families.extend(("SamsungOne", "Samsung Sans"))
        local_fonts.extend((
            ("SamsungOne-400", "SamsungOne", "SamsungOne", "Regular"),
            ("SamsungSans-Regular", "Samsung Sans", "Samsung Sans", "Regular"),
        ))
        sources.append(SAMSUNG_ONE_SOURCE)
    families = tuple(dict.fromkeys((*core_families, *additions)))
    for family in additions:
        postscript = family.replace(" ", "") + "-Regular"
        local_fonts.append((postscript, family, family, "Regular"))
    return {
        "id": f"android-{oem_key}-{version}-{key or 'default'}",
        "source": " | ".join(sources),
        "families": families,
        "allowUnknownFamilies": False,
        "localFonts": tuple(dict.fromkeys(local_fonts)),
        # Roboto is the baseline. Unknown AOSP faces are intentionally not
        # assigned guessed width ratios.
        "metrics": (("Roboto", 1.0, False),),
    }


__all__ = [
    "AOSP_FONT_SOURCE",
    "SAMSUNG_ONE_SOURCE",
    "build_android_font_profile",
]
