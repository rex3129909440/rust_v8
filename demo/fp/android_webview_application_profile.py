"""Version-independent catalog profiles for the custom Android WebView path.

The complete Android device row is composed by the shared public device
catalog.  This isolated module adds only output/input runtime modes and the
workload's hidden Window geometry.  Historical success references are never
read by the production random path.
"""

from __future__ import annotations

import random
import re
from dataclasses import dataclass, replace
from typing import TYPE_CHECKING

try:
    from demo.fp.android_device_profile_catalog import (
        get_android_device_profile_by_id,
        get_android_device_profiles,
        materialize_android_device_profile,
    )
    from demo.fp.android_css_profile_catalog import chromium_android_css_overrides
    from demo.fp.android_media_capability_catalog import (
        build_android_media_capabilities,
    )
except ModuleNotFoundError:
    from fp.android_device_profile_catalog import (  # type: ignore
        get_android_device_profile_by_id,
        get_android_device_profiles,
        materialize_android_device_profile,
    )
    from android_css_profile_catalog import chromium_android_css_overrides  # type: ignore
    from android_media_capability_catalog import (  # type: ignore
        build_android_media_capabilities,
    )

if TYPE_CHECKING:
    from demo.get_random_fp import RandomFingerprint


WEBVIEW_APPLICATION_USER_AGENT = (
    "wizz-air/8.1.9 (com.wizzair.WizzAirApp; build:2207; android 9)"
)
# Compatibility name for older callers; it does not select a 136-only path.
WEBVIEW_136_APPLICATION_USER_AGENT = WEBVIEW_APPLICATION_USER_AGENT
_WIZZ_AIR_APPLICATION_USER_AGENT = re.compile(
    r"wizz-air/\d+(?:\.\d+){1,3} "
    r"\(com\.wizzair\.WizzAirApp; build:\d+; "
    r"android \d+(?:\.\d+)*\)",
    re.IGNORECASE,
)

_DISPLAY_STREAM_SALT = 0x5756423133364450
_AUDIO_STREAM_SALT = 0x5756423133364155
_INPUT_STREAM_SALT = 0x575642313336494E
_MEMORY_STREAM_SALT = 0x5756423133364D45
_DPR_STREAM_SALT = 0x5756424450524D4F
WEBVIEW_DPR_CHOICES = (0.5, 1.0, 1.5, 2.0, 2.5, 3.0)

# Loaded Android WebView HTTPS reference captured for this project. Chromium's
# precise MemoryInfo path exposes a runtime V8 snapshot, so this row is an
# anchor rather than a constant copied into every generated profile.
_LOADED_WEBVIEW_TOTAL_HEAP = 66_508_722
_LOADED_WEBVIEW_USED_HEAP = 23_200_000


@dataclass(frozen=True, slots=True)
class AudioRoute:
    id: str
    sample_rate: float
    frames_per_buffer: int
    max_channel_count: int
    weight: int

    @property
    def base_latency(self) -> float:
        return self.frames_per_buffer / self.sample_rate


_MODERN_AUDIO_ROUTES = (
    AudioRoute("speaker-fast-48k-128-stereo", 48_000.0, 128, 2, 65),
    AudioRoute("speaker-48k-256-stereo", 48_000.0, 256, 2, 20),
    AudioRoute("a2dp-44k1-256-stereo", 44_100.0, 256, 2, 12),
    AudioRoute("usb-48k-256-eight-channel", 48_000.0, 256, 8, 3),
)

_ENTRY_AUDIO_ROUTES = (
    AudioRoute("speaker-48k-256-stereo", 48_000.0, 256, 2, 80),
    AudioRoute("wired-44k1-256-stereo", 44_100.0, 256, 2, 20),
)


def is_wizz_air_application_user_agent(user_agent: str) -> bool:
    """Return whether a UA belongs to the supported Android app family."""

    return _WIZZ_AIR_APPLICATION_USER_AGENT.fullmatch(
        str(user_agent or "").strip()
    ) is not None


def is_android_webview_user_agent(user_agent: str) -> bool:
    """Recognize Android WebView/custom-app UA shapes without matching Chrome."""

    original = str(user_agent or "").strip()
    lower = original.lower()
    return "android" in lower and (
        not original.startswith("Mozilla/")
        or "; wv" in lower
        or " version/4.0" in lower
    )


def supports_webview_device_profile(
    user_agent: str,
    chromium_major: int,
) -> bool:
    return (
        136 <= chromium_major <= 151
        and is_wizz_air_application_user_agent(user_agent)
    )


def _device_dpr_choices(device: dict[str, object]) -> tuple[float, ...]:
    window = device.get("window", {})
    original = (
        float(window.get("devicePixelRatio", 1.0))
        if isinstance(window, dict)
        else 1.0
    )
    return tuple(dict.fromkeys((*WEBVIEW_DPR_CHOICES, original)))


def _device_modes(device: dict[str, object]) -> tuple[int, int, int]:
    tags = {str(value).lower() for value in device.get("tags", ())}
    profile_id = str(device.get("id", ""))
    wide_color = bool(tags & {"flagship", "foldable", "mainstream"}) or (
        profile_id.startswith("android_pixel_")
        or profile_id.startswith("android_galaxy_s8_plus")
    )
    entry = "entry" in tags
    return (
        2 if wide_color else 1,
        2 if wide_color else 1,
        len(_ENTRY_AUDIO_ROUTES if entry else _MODERN_AUDIO_ROUTES),
    )


def count_webview_device_mode_combinations() -> int:
    """Return discrete device/RAM/display/audio/input combinations."""

    return sum(
        len(tuple(device.get("physicalMemoryChoicesGb", ())))
        * _device_modes(device)[0]
        * _device_modes(device)[1]
        * _device_modes(device)[2]
        * len(_device_dpr_choices(device))
        * 2
        for device in get_android_device_profiles(
            minimum_android_version=8,
        )
    )


def _runtime_heap_snapshot(
    *,
    seed: int,
    physical_memory_gb: int,
    heap_size_limit: int,
) -> tuple[int, int]:
    """Create one deterministic precise-mode V8 runtime snapshot.

    Blink derives these fields from V8's live/committed heap plus external
    memory. They are workload snapshots, not hardware constants.  Keep the
    observed loaded-WebView sample as the center of a bounded distribution,
    add a small device-class adjustment, and preserve V8's strict ordering.
    An explicit seed remains reproducible while ``seed=None`` at the public
    entry point produces a fresh pair for every request.
    """

    rng = random.Random(int(seed) ^ _MEMORY_STREAM_SALT)
    physical_memory = max(1, int(physical_memory_gb))
    if physical_memory <= 2:
        device_adjustment = -8_000_000
    elif physical_memory <= 4:
        device_adjustment = 0
    elif physical_memory <= 8:
        device_adjustment = 6_000_000
    else:
        device_adjustment = 12_000_000

    total_heap = (
        _LOADED_WEBVIEW_TOTAL_HEAP
        + device_adjustment
        + rng.randint(-12_000_000, 20_000_000)
    )
    # A loaded WebView should remain well below its configured reservation
    # limit. Leave at least 2 MiB between the live and committed values.
    total_heap = max(24_000_000, min(total_heap, heap_size_limit - 2_000_000))
    live_ratio_ppm = rng.randint(300_000, 650_000)
    used_heap = (
        total_heap * live_ratio_ppm // 1_000_000
        + rng.randint(-1_500_000, 1_500_000)
    )
    used_heap = max(8_000_000, min(used_heap, total_heap - 2_000_000))
    return total_heap, used_heap


def apply_webview_input_geometry_profile(
    details: RandomFingerprint,
) -> RandomFingerprint:
    """Apply version-independent WebView DPR and Android control geometry."""

    if details.platform != "android":
        raise ValueError("WebView input geometry requires Android")
    device = get_android_device_profile_by_id(details.screen_profile_id)
    device_pixel_ratio = random.Random(
        int(details.seed) ^ _DPR_STREAM_SALT
    ).choice(_device_dpr_choices(device))
    profile = details.profile
    chromium_major = int(
        profile.navigator.user_agent_data.ua_full_version.split(".", 1)[0]
    )
    try:
        selected_android_major = int(
            str(profile.navigator.user_agent_data.platform_version).split(".", 1)[0]
        )
    except (AttributeError, TypeError, ValueError):
        selected_android_major = int(str(device.get("androidVersion", "10")).split(".", 1)[0])
    media_device = materialize_android_device_profile(
        device,
        selected_android_major,
    )
    navigator = replace(
        profile.navigator,
        user_agent_data=replace(
            profile.navigator.user_agent_data,
            form_factors=tuple(
                dict.fromkeys(
                    (*profile.navigator.user_agent_data.form_factors, "WebView")
                )
            ),
        ),
    )
    screen = replace(
        profile.screen,
        device_pixel_ratio=device_pixel_ratio,
    )
    css = replace(
        profile.css,
        **chromium_android_css_overrides(
            device_pixel_ratio,
            profile.locale.locale,
            str(device.get("oem", "aosp")),
            chromium_major,
        ),
    )
    media = replace(
        profile.media,
        **build_android_media_capabilities(
            media_device,
            chromium_major,
            webview=True,
        ),
    )
    profile = replace(
        profile,
        id=f"{profile.id}-webview-dpr{device_pixel_ratio:g}",
        navigator=navigator,
        screen=screen,
        css=css,
        media=media,
    )
    return replace(details, profile=profile)


def apply_webview_device_profile(
    details: RandomFingerprint,
) -> RandomFingerprint:
    if details.platform != "android":
        raise ValueError("WebView device profile requires Android")
    details = apply_webview_input_geometry_profile(details)
    device = get_android_device_profile_by_id(details.screen_profile_id)
    tags = {str(value).lower() for value in device.get("tags", ())}
    profile_id = str(device.get("id", ""))
    wide_color = bool(tags & {"flagship", "foldable", "mainstream"}) or (
        profile_id.startswith("android_pixel_")
        or profile_id.startswith("android_galaxy_s8_plus")
    )
    color_depth_choices = (24, 32) if wide_color else (24,)
    color_gamut_choices = ("srgb", "p3") if wide_color else ("srgb",)
    audio_routes = _ENTRY_AUDIO_ROUTES if "entry" in tags else _MODERN_AUDIO_ROUTES

    display_rng = random.Random(int(details.seed) ^ _DISPLAY_STREAM_SALT)
    color_depth = display_rng.choice(color_depth_choices)
    color_gamut = display_rng.choice(color_gamut_choices)
    device_pixel_ratio = float(details.profile.screen.device_pixel_ratio)
    audio_rng = random.Random(int(details.seed) ^ _AUDIO_STREAM_SALT)
    audio = audio_rng.choices(
        audio_routes,
        weights=tuple(value.weight for value in audio_routes),
        k=1,
    )[0]
    mouse_connected = (
        random.Random(int(details.seed) ^ _INPUT_STREAM_SALT).random() < 0.12
    )

    profile = details.profile
    chromium_major = int(
        profile.navigator.user_agent_data.ua_full_version.split(".", 1)[0]
    )
    screen = replace(
        profile.screen,
        color_depth=color_depth,
        pixel_depth=color_depth,
        device_pixel_ratio=device_pixel_ratio,
        viewport_width=0.0,
        viewport_height=0.0,
        outer_width=0.0,
        outer_height=0.0,
    )
    window = replace(
        profile.window,
        inner_width=0.0,
        inner_height=0.0,
        outer_width=0.0,
        outer_height=0.0,
        iframe_inner_width=0.0,
        iframe_inner_height=0.0,
        iframe_outer_width=0.0,
        iframe_outer_height=0.0,
    )
    audio_profile = replace(
        profile.audio,
        sample_rate=audio.sample_rate,
        base_latency=audio.base_latency,
        output_latency=0.0,
        max_channel_count=audio.max_channel_count,
    )
    total_heap_size, used_heap_size = _runtime_heap_snapshot(
        seed=int(details.seed),
        physical_memory_gb=int(details.physical_memory_gb),
        heap_size_limit=int(profile.memory.performance_js_heap_size_limit),
    )
    memory = replace(
        profile.memory,
        performance_total_js_heap_size=total_heap_size,
        performance_used_js_heap_size=used_heap_size,
        console_total_js_heap_size=total_heap_size,
        console_used_js_heap_size=used_heap_size,
    )
    pointer = "fine" if mouse_connected else "coarse"
    hover = "hover" if mouse_connected else "none"
    media_preferences = replace(
        profile.media_preferences,
        color_gamut=color_gamut,
        pointer=pointer,
        any_pointer=pointer,
        hover=hover,
        any_hover=hover,
    )
    profile = replace(
        profile,
        id=(
            f"random-android-webview-{chromium_major}-{profile_id}-"
            f"{details.physical_memory_gb}gb-{audio.id}-"
            f"{'mouse' if mouse_connected else 'touch'}-"
            f"dpr{device_pixel_ratio:g}-"
            f"{color_depth}bit-{int(details.seed):016x}"
        ),
        screen=screen,
        window=window,
        audio=audio_profile,
        memory=memory,
        media_preferences=media_preferences,
    )
    return replace(
        details,
        profile=profile,
        memory_snapshot_profile_id=(
            f"android-webview-runtime-precise-{int(details.seed):016x}"
        ),
    )


# Compatibility aliases for callers using the historical module API. Runtime
# routing uses the version-independent names above; there is no 136-only path.
supports_webview_136_device_profile = supports_webview_device_profile
count_webview_136_device_mode_combinations = count_webview_device_mode_combinations
apply_webview_136_device_profile = apply_webview_device_profile


__all__ = [
    "WEBVIEW_136_APPLICATION_USER_AGENT",
    "WEBVIEW_APPLICATION_USER_AGENT",
    "WEBVIEW_DPR_CHOICES",
    "apply_webview_device_profile",
    "apply_webview_input_geometry_profile",
    "apply_webview_136_device_profile",
    "count_webview_device_mode_combinations",
    "count_webview_136_device_mode_combinations",
    "is_wizz_air_application_user_agent",
    "is_android_webview_user_agent",
    "supports_webview_device_profile",
    "supports_webview_136_device_profile",
]
