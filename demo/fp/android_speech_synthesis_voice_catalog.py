"""Country-aware Android Chromium speech-synthesis voice profiles.

Android does not define one universal installed-voice list.  Chromium asks the
active Android TextToSpeech engine which locales it supports and exposes one
native voice per returned locale.  The legacy Chromium implementation derives
the visible name from ``Locale.getDisplayLanguage()`` plus
``Locale.getDisplayCountry()``.  This catalog follows that observable shape
without importing the desktop Microsoft/Apple voice inventories.

Stock Android WebView does not expose ``window.speechSynthesis``.  These rows
therefore describe Android Chromium, or an application host that explicitly
enables the platform TTS bridge; WebView surface gating remains a native-runtime
responsibility.
"""

from __future__ import annotations

import random
from collections.abc import Sequence

try:
    from demo.fp.speech_synthesis_voice_catalog import (
        LANGUAGE_DISPLAY_BY_BASE,
        REGION_DISPLAY_BY_CODE,
        normalize_locale,
    )
except ModuleNotFoundError:
    from speech_synthesis_voice_catalog import (  # type: ignore
        LANGUAGE_DISPLAY_BY_BASE,
        REGION_DISPLAY_BY_CODE,
        normalize_locale,
    )


# Language families exposed by common Android TTS engines.  The precise set is
# installation-dependent; generated profiles select only from the profile's
# country-derived navigator.languages list, never from an unrelated desktop
# voice catalog.
ANDROID_TTS_LANGUAGE_BASES = frozenset(
    {
        "af", "am", "ar", "as", "az", "bg", "bn", "bs", "ca", "cs",
        "cy", "da", "de", "el", "en", "es", "et", "eu", "fa", "fi",
        "fil", "fr", "ga", "gl", "gu", "he", "hi", "hr", "hu", "hy",
        "id", "is", "it", "ja", "jv", "ka", "kk", "km", "kn", "ko",
        "lo", "lt", "lv", "mk", "ml", "mn", "mr", "ms", "my", "nb",
        "ne", "nl", "pa", "pl", "pt", "ro", "ru", "si", "sk", "sl",
        "sq", "sr", "sv", "sw", "ta", "te", "th", "tr", "uk", "ur",
        "uz", "vi", "yue", "zh", "zu",
    }
)


def _locale_parts(locale: str) -> tuple[str, str]:
    normalized = normalize_locale(locale)
    if not normalized:
        return "", ""
    parts = normalized.split("-")
    language = parts[0]
    region = next(
        (
            part.upper()
            for part in parts[1:]
            if len(part) == 2 and part.isalpha()
        ),
        "",
    )
    return language, region


def _android_voice(locale: str, *, is_default: bool) -> dict[str, object] | None:
    normalized = normalize_locale(locale)
    language, region = _locale_parts(normalized)
    if not normalized or language not in ANDROID_TTS_LANGUAGE_BASES:
        return None
    display_language = LANGUAGE_DISPLAY_BY_BASE.get(language, language)
    display_region = REGION_DISPLAY_BY_CODE.get(region, region) if region else ""
    # Chromium's Android implementation joins these values with one space,
    # not the parenthesised desktop voice-name convention.
    name = f"{display_language} {display_region}".strip()
    return {
        "voiceURI": name,
        "name": name,
        "lang": normalized,
        "localService": True,
        "default": is_default,
    }


def choose_android_speech_synthesis_voice_profile(
    rng: random.Random,
    country_code: str,
    languages: Sequence[str],
) -> dict[str, object]:
    """Select a small installed-locale subset tied to one country profile.

    The primary navigator language is always preferred.  Secondary languages
    are independently installed with a high probability, while an English-US
    fallback may be present on non-English devices.  All choices use the
    caller's deterministic RNG, so a fixed profile seed remains reproducible.
    """

    candidates = list(
        dict.fromkeys(
            normalized
            for value in languages
            if (normalized := normalize_locale(str(value)))
        )
    )
    if not candidates:
        candidates = ["en-US"]

    selected = [candidates[0]]
    for locale in candidates[1:]:
        if len(selected) >= 5:
            break
        if rng.random() < 0.72:
            selected.append(locale)

    if (
        not any(locale.lower().startswith("en-") or locale.lower() == "en" for locale in selected)
        and len(selected) < 5
        and rng.random() < 0.58
    ):
        selected.append("en-US")

    voices: list[dict[str, object]] = []
    for locale in selected:
        voice = _android_voice(locale, is_default=not voices)
        if voice is not None:
            voices.append(voice)
    if not voices:
        fallback = _android_voice("en-US", is_default=True)
        if fallback is not None:
            voices.append(fallback)

    country = str(country_code or "US").upper()
    return {
        "id": (
            f"android-tts-{country.lower()}-"
            f"{normalize_locale(candidates[0]).lower()}-{len(voices)}v"
        ),
        "country": country,
        "primaryLocale": normalize_locale(candidates[0]),
        "locales": tuple(selected),
        "voices": tuple(voices),
        "speechSynthesis": {"voices": tuple(voices)},
    }


__all__ = [
    "ANDROID_TTS_LANGUAGE_BASES",
    "choose_android_speech_synthesis_voice_profile",
]
