"""
PC-only screen/window/viewport catalog.

This module is for desktop/laptop browser fingerprints only. Do not mix mobile
or tablet CSS viewport sizes into this file. Values such as 384x832, 390x844,
414x896, and 412x915 are mobile viewport sizes and must stay out of this PC
catalog.

Modeling notes:

- screen.width/height are browser-exposed CSS-pixel screen dimensions.
- devicePixelRatio is separate; the same physical panel can expose different
  CSS dimensions under Windows/macOS scaling.
- window values model a maximized desktop Chrome-like browser unless a later
  caller chooses to add non-maximized window modes.
- The output patch matches the runtime env shape: window, screen,
  visualViewport, and mediaViewport.
"""

from __future__ import annotations

import random
from copy import deepcopy
from functools import lru_cache
from typing import Iterable

try:
    from .pc_screen_extension_catalog import PC_SCREEN_EXTENSION_ROWS
except ImportError:  # Support importing this catalog directly.
    from pc_screen_extension_catalog import PC_SCREEN_EXTENSION_ROWS  # type: ignore


# id, css_screen_width, css_screen_height, dpr, weight, tags
#
# The list focuses on real PC desktop/laptop CSS screen sizes seen globally:
# low-end laptops, common 1080p desktops, high-DPI Windows scaling, macOS
# laptop CSS sizes, ultrawide monitors, QHD, and 4K scaled desktops.
PC_SCREEN_SIZE_ROWS: tuple[tuple[str, int, int, float, int, tuple[str, ...]], ...] = (
    ("pc_800x600_1x_legacy", 800, 600, 1.0, 5, ("legacy", "desktop")),
    ("pc_1024x576_1x_netbook", 1024, 576, 1.0, 5, ("legacy", "netbook", "laptop")),
    ("pc_1024x600_1x_netbook", 1024, 600, 1.0, 8, ("legacy", "netbook", "laptop")),
    ("pc_1024x768_1x_legacy", 1024, 768, 1.0, 18, ("legacy", "desktop")),
    ("pc_1152x864_1x_legacy", 1152, 864, 1.0, 10, ("legacy", "desktop")),
    ("pc_1280x600_1x_netbook", 1280, 600, 1.0, 6, ("legacy", "netbook", "laptop")),
    ("pc_1280x720_1x_lowend", 1280, 720, 1.0, 45, ("lowend", "laptop")),
    ("pc_1280x720_1p5_fhd_scaled", 1280, 720, 1.5, 24, ("windows", "scaled", "fhd")),
    ("pc_1280x720_2x_qhd_scaled", 1280, 720, 2.0, 8, ("windows", "scaled", "qhd")),
    ("pc_1280x768_1x_lowend", 1280, 768, 1.0, 22, ("lowend", "laptop")),
    ("pc_1280x800_1x_laptop", 1280, 800, 1.0, 38, ("laptop",)),
    ("pc_1280x960_1x_legacy", 1280, 960, 1.0, 12, ("legacy", "desktop")),
    ("pc_1280x1024_1x_legacy", 1280, 1024, 1.0, 28, ("legacy", "desktop")),
    ("pc_1280x1200_1x_desktop", 1280, 1200, 1.0, 16, ("desktop", "productivity")),
    ("pc_1360x768_1x_laptop", 1360, 768, 1.0, 30, ("laptop",)),
    ("pc_1365x768_1x_laptop", 1365, 768, 1.0, 10, ("laptop", "windows")),
    ("pc_1366x768_1x_laptop", 1366, 768, 1.0, 80, ("laptop", "windows")),
    ("pc_1400x1050_1x_legacy", 1400, 1050, 1.0, 8, ("legacy", "desktop")),
    ("pc_1440x900_1x_laptop", 1440, 900, 1.0, 46, ("laptop", "desktop")),
    ("pc_1470x956_2x_mac_air", 1470, 956, 2.0, 12, ("mac", "hidpi")),
    ("pc_1440x960_2x_mac", 1440, 960, 2.0, 16, ("mac", "hidpi")),
    ("pc_1496x967_2x_mac", 1496, 967, 2.0, 8, ("mac", "hidpi")),
    ("pc_1512x982_2x_mac", 1512, 982, 2.0, 28, ("mac", "hidpi")),
    ("pc_1536x864_1p25_windows", 1536, 864, 1.25, 76, ("windows", "scaled")),
    ("pc_1600x900_1x_desktop", 1600, 900, 1.0, 52, ("desktop", "windows")),
    ("pc_1600x1000_2x_windows", 1600, 1000, 2.0, 8, ("windows", "scaled", "hidpi")),
    ("pc_1600x1067_1p5_surface", 1600, 1067, 1.5, 10, ("windows", "surface", "scaled")),
    ("pc_1600x1200_1x_legacy", 1600, 1200, 1.0, 8, ("legacy", "desktop")),
    ("pc_1664x1110_2x_mac_air", 1664, 1110, 2.0, 8, ("mac", "hidpi")),
    ("pc_1680x1050_1x_desktop", 1680, 1050, 1.0, 30, ("desktop",)),
    ("pc_1707x960_1p5_windows_qhd", 1707, 960, 1.5, 26, ("windows", "scaled", "qhd")),
    ("pc_1707x960_2p25_4k_scaled", 1707, 960, 2.25, 5, ("windows", "scaled", "4k")),
    ("pc_1728x1117_2x_mac", 1728, 1117, 2.0, 22, ("mac", "hidpi")),
    ("pc_1800x1169_2x_mac", 1800, 1169, 2.0, 6, ("mac", "hidpi")),
    ("pc_1920x1080_1x_desktop", 1920, 1080, 1.0, 95, ("desktop", "windows")),
    ("pc_1920x1080_1p5_qhd_scaled", 1920, 1080, 1.5, 10, ("windows", "scaled", "qhd")),
    ("pc_1920x1080_2x_4k_scaled", 1920, 1080, 2.0, 24, ("windows", "scaled", "4k")),
    ("pc_1920x1200_1x_desktop", 1920, 1200, 1.0, 28, ("desktop", "productivity")),
    ("pc_1920x1200_1p25_windows", 1536, 960, 1.25, 12, ("windows", "scaled", "productivity")),
    ("pc_1920x1200_1p5_windows", 1280, 800, 1.5, 8, ("windows", "scaled", "productivity")),
    ("pc_1920x1280_1x_surface_laptop13", 1920, 1280, 1.0, 5, ("windows", "surface", "laptop", "productivity")),
    ("pc_1920x1280_1p5_surface_laptop13", 1280, 853, 1.5, 8, ("windows", "surface", "laptop", "scaled")),
    ("pc_2048x1152_1p25_windows", 2048, 1152, 1.25, 14, ("windows", "scaled")),
    ("pc_2048x1280_2x_mac", 2048, 1280, 2.0, 12, ("mac", "hidpi")),
    ("pc_2056x1329_2x_mac", 2056, 1329, 2.0, 4, ("mac", "hidpi")),
    ("pc_2194x1234_1p75_4k_scaled", 2194, 1234, 1.75, 8, ("windows", "scaled", "4k")),
    ("pc_2256x1504_2x_surface", 1128, 752, 2.0, 8, ("windows", "surface", "hidpi")),
    ("pc_2256x1504_1p5_surface", 1504, 1003, 1.5, 14, ("windows", "surface", "scaled")),
    ("pc_2304x1536_2x_surface_laptop", 1152, 768, 2.0, 8, ("windows", "surface", "hidpi")),
    ("pc_2304x1536_1p5_surface_laptop", 1536, 1024, 1.5, 10, ("windows", "surface", "scaled")),
    ("pc_2304x1440_2x_mac", 2304, 1440, 2.0, 10, ("mac", "hidpi")),
    ("pc_2400x1600_2x_surface", 1200, 800, 2.0, 8, ("windows", "surface", "hidpi")),
    ("pc_2560x1080_1x_ultrawide", 2560, 1080, 1.0, 16, ("desktop", "ultrawide")),
    ("pc_2560x1080_1p25_ultrawide", 2048, 864, 1.25, 6, ("desktop", "ultrawide", "scaled")),
    ("pc_2560x1440_1x_qhd", 2560, 1440, 1.0, 42, ("desktop", "qhd", "gaming")),
    ("pc_2560x1440_1p25_windows", 2048, 1152, 1.25, 22, ("windows", "scaled", "qhd")),
    ("pc_2560x1440_1p5_windows", 1707, 960, 1.5, 28, ("windows", "scaled", "qhd")),
    ("pc_2560x1440_2x_windows", 1280, 720, 2.0, 8, ("windows", "scaled", "qhd")),
    ("pc_2560x1600_1x_laptop", 2560, 1600, 1.0, 16, ("laptop", "productivity")),
    ("pc_2560x1600_1p25_windows", 2048, 1280, 1.25, 14, ("windows", "scaled", "productivity")),
    ("pc_2560x1600_1p5_windows", 1707, 1067, 1.5, 20, ("windows", "scaled")),
    ("pc_2560x1600_2x_windows", 1280, 800, 2.0, 8, ("windows", "hidpi", "productivity")),
    ("pc_2560x1664_2x_mac_air", 1280, 832, 2.0, 10, ("mac", "hidpi")),
    ("pc_2736x1824_2x_surface", 1368, 912, 2.0, 14, ("windows", "surface", "hidpi")),
    ("pc_2736x1824_1p5_surface", 1824, 1216, 1.5, 12, ("windows", "surface", "scaled")),
    ("pc_2752x1152_1p25_ultrawide", 2752, 1152, 1.25, 6, ("desktop", "ultrawide", "scaled")),
    ("pc_2880x1800_1x_laptop", 2880, 1800, 1.0, 4, ("laptop", "hidpi", "productivity")),
    ("pc_2880x1800_1p5_windows", 1920, 1200, 1.5, 12, ("windows", "scaled", "hidpi", "productivity")),
    ("pc_2880x1800_2x_windows", 1440, 900, 2.0, 10, ("windows", "hidpi")),
    ("pc_2880x1800_2x_mac", 1440, 900, 2.0, 20, ("mac", "hidpi")),
    ("pc_2880x1864_2x_mac_air", 1440, 932, 2.0, 8, ("mac", "hidpi")),
    ("pc_2880x1920_2x_surface_pro", 1440, 960, 2.0, 10, ("windows", "surface", "hidpi")),
    ("pc_2880x1920_1p5_surface_pro", 1920, 1280, 1.5, 8, ("windows", "surface", "scaled")),
    ("pc_2944x1840_2x_windows_arm_laptop", 1472, 920, 2.0, 8, ("windows", "hidpi", "arm64")),
    ("pc_3000x2000_2x_surface", 1500, 1000, 2.0, 14, ("windows", "surface", "hidpi")),
    ("pc_3024x1964_2x_mac", 1512, 982, 2.0, 20, ("mac", "hidpi")),
    ("pc_3072x1728_1p25_4k_scaled", 3072, 1728, 1.25, 10, ("windows", "scaled", "4k")),
    ("pc_3072x1920_2x_mac", 1536, 960, 2.0, 16, ("mac", "hidpi")),
    ("pc_3200x1800_2x_windows", 1600, 900, 2.0, 12, ("windows", "hidpi")),
    ("pc_3200x2000_2x_windows", 1600, 1000, 2.0, 8, ("windows", "hidpi")),
    ("pc_3200x2000_2p5_windows", 1280, 800, 2.5, 6, ("windows", "scaled", "hidpi")),
    ("pc_3270x2180_2x_surface_laptop15", 1635, 1090, 2.0, 8, ("windows", "surface", "hidpi")),
    ("pc_3270x2180_1p5_surface_laptop15", 2180, 1453, 1.5, 4, ("windows", "surface", "scaled", "productivity")),
    ("pc_3440x1440_1x_ultrawide", 3440, 1440, 1.0, 18, ("desktop", "ultrawide")),
    ("pc_3440x1440_1p25_ultrawide", 2752, 1152, 1.25, 6, ("desktop", "ultrawide", "scaled")),
    ("pc_3440x1440_1p5_ultrawide", 2293, 960, 1.5, 4, ("desktop", "ultrawide", "scaled")),
    ("pc_3456x2160_2x_mac", 1728, 1080, 2.0, 5, ("mac", "hidpi")),
    ("pc_3456x2234_2x_mac", 1728, 1117, 2.0, 8, ("mac", "hidpi")),
    ("pc_3840x1080_1x_super_ultrawide", 3840, 1080, 1.0, 7, ("desktop", "super_ultrawide")),
    ("pc_3840x1200_1x_super_ultrawide", 3840, 1200, 1.0, 4, ("desktop", "super_ultrawide")),
    ("pc_3840x1600_1x_ultrawide", 3840, 1600, 1.0, 8, ("desktop", "ultrawide")),
    ("pc_3840x2160_1x_4k", 3840, 2160, 1.0, 10, ("desktop", "4k")),
    ("pc_3840x2160_1p5_windows", 2560, 1440, 1.5, 24, ("windows", "scaled", "4k")),
    ("pc_3840x2160_2x_windows", 1920, 1080, 2.0, 24, ("windows", "scaled", "4k")),
    ("pc_3840x2160_2p5_windows", 1536, 864, 2.5, 8, ("windows", "scaled", "4k")),
    ("pc_3840x2400_2x_windows", 1920, 1200, 2.0, 10, ("windows", "hidpi", "productivity")),
    ("pc_3840x2400_2p5_windows", 1536, 960, 2.5, 8, ("windows", "scaled", "hidpi", "productivity")),
    ("pc_3840x2400_3x_windows", 1280, 800, 3.0, 5, ("windows", "scaled", "hidpi")),
    ("pc_4096x1152_1p25_super_ultrawide", 4096, 1152, 1.25, 4, ("desktop", "super_ultrawide", "scaled")),
    ("pc_5120x1440_1x_super_ultrawide", 5120, 1440, 1.0, 7, ("desktop", "super_ultrawide")),
    ("pc_5120x1440_1p25_super_ultrawide", 4096, 1152, 1.25, 4, ("desktop", "super_ultrawide", "scaled")),
    ("pc_5120x2160_1p5_ultrawide", 3413, 1440, 1.5, 5, ("desktop", "ultrawide", "hidpi")),
    ("pc_5120x2160_2x_ultrawide", 2560, 1080, 2.0, 4, ("desktop", "ultrawide", "hidpi")),
    ("pc_5120x2880_2x_5k", 2560, 1440, 2.0, 6, ("desktop", "5k", "hidpi")),
)

# Preserve the original rows and append the evidence-backed extension. No
# existing ID, value or weight is replaced.
BASE_PC_SCREEN_SIZE_ROWS = PC_SCREEN_SIZE_ROWS
_BASE_PC_SCREEN_IDS = frozenset(row[0] for row in BASE_PC_SCREEN_SIZE_ROWS)
_EXTENSION_SCREEN_SOURCES = {
    row[0]: tuple(row[6]) for row in PC_SCREEN_EXTENSION_ROWS
}
if _BASE_PC_SCREEN_IDS & _EXTENSION_SCREEN_SOURCES.keys():
    raise ValueError("PC screen extension attempts to replace an existing profile")
PC_SCREEN_SIZE_ROWS = BASE_PC_SCREEN_SIZE_ROWS + tuple(
    row[:6] for row in PC_SCREEN_EXTENSION_ROWS
)


def _maximized_pc_window_rect(
    screen_width: int,
    screen_height: int,
    tags: tuple[str, ...],
) -> tuple[int, int, int, int]:
    # Windows-style maximized browser model:
    # - outerHeight loses taskbar area
    # - innerHeight additionally loses browser tabs/address/bookmark bars
    # - width stays screen-wide for maximized windows
    outer_width = screen_width
    # Windows 11's default taskbar is 48 CSS pixels. Keep the older 40-pixel
    # geometry only for rows explicitly representing legacy systems.
    taskbar_height = 40 if "legacy" in tags else 48
    outer_height = max(480, screen_height - taskbar_height)
    inner_width = outer_width
    inner_height = max(360, outer_height - 88)
    return inner_width, inner_height, outer_width, outer_height


def build_pc_screen_profile(row: tuple[str, int, int, float, int, tuple[str, ...]]) -> dict[str, object]:
    profile_id, width, height, dpr, weight, tags = row
    inner_width, inner_height, outer_width, outer_height = _maximized_pc_window_rect(
        width,
        height,
        tags,
    )
    taskbar_height = height - outer_height
    return {
        "id": profile_id,
        "catalog": (
            "existing-pc-screen"
            if profile_id in _BASE_PC_SCREEN_IDS
            else "public-screen-extension"
        ),
        "evidenceSources": _EXTENSION_SCREEN_SOURCES.get(profile_id, ()),
        "deviceClass": "pc",
        "weight": weight,
        "tags": tags,
        "physicalWidth": int(round(width * dpr)),
        "physicalHeight": int(round(height * dpr)),
        "taskbarCssHeight": taskbar_height,
        "window": {
            "innerWidth": inner_width,
            "innerHeight": inner_height,
            "outerWidth": outer_width,
            "outerHeight": outer_height,
            "devicePixelRatio": dpr,
            "screenX": 0,
            "screenY": 0,
            "pageXOffset": 0,
            "pageYOffset": 0,
        },
        "screen": {
            "width": width,
            "height": height,
            "availWidth": width,
            "availHeight": outer_height,
            "colorDepth": 24,
            "pixelDepth": 24,
            "orientation": {
                "type": "landscape-primary",
                "angle": 0,
            },
        },
        "visualViewport": {
            "width": inner_width,
            "height": inner_height,
            "offsetLeft": 0,
            "offsetTop": 0,
            "pageLeft": 0,
            "pageTop": 0,
            "scale": 1,
        },
        "mediaViewport": {
            "width": inner_width,
            "height": inner_height,
        },
    }


PC_SCREEN_PROFILES: tuple[dict[str, object], ...] = tuple(
    build_pc_screen_profile(row) for row in PC_SCREEN_SIZE_ROWS
)


# Common browser-exposed CSS screen sizes receive a distribution boost while
# the complete legacy, HiDPI, and ultrawide tail remains selectable.
_COMMON_SCREEN_SIZE_BOOSTS: dict[tuple[int, int], int] = {
    (1920, 1080): 10,
    (2560, 1440): 5,
    (1536, 864): 4,
    (1366, 768): 3,
    (1280, 720): 2,
    (1920, 1200): 2,
    (2560, 1600): 2,
    (3840, 2160): 2,
    (1600, 900): 2,
    (1440, 900): 2,
    (3440, 1440): 2,
}


def get_pc_screen_profile_weight(profile: dict[str, object]) -> int:
    screen = profile.get("screen", {})
    if not isinstance(screen, dict):
        return max(1, int(profile.get("weight", 1) or 1))
    size = (int(screen.get("width", 0) or 0), int(screen.get("height", 0) or 0))
    return max(
        1,
        int(profile.get("weight", 1) or 1) * _COMMON_SCREEN_SIZE_BOOSTS.get(size, 1),
    )


@lru_cache(maxsize=8)
def get_pc_screen_profiles(tag: str | None = None) -> tuple[dict[str, object], ...]:
    tag_key = str(tag or "").strip().lower()
    if not tag_key:
        return PC_SCREEN_PROFILES
    if tag_key == "windows":
        # Treat "windows" as the default Windows PC selector, not just rows
        # that explicitly carry the windows tag. Legacy, desktop, laptop, and
        # ultrawide rows are valid Windows screen states and should be sampled.
        return tuple(
            profile
            for profile in PC_SCREEN_PROFILES
            if "mac" not in tuple(str(item).lower() for item in profile.get("tags", ()))
        )
    return tuple(
        profile
        for profile in PC_SCREEN_PROFILES
        if tag_key in tuple(str(item).lower() for item in profile.get("tags", ()))
    )


def choose_pc_screen_profile(
    rng: random.Random,
    tag: str | None = None,
) -> dict[str, object]:
    profiles = get_pc_screen_profiles(tag=tag)
    if not profiles:
        profiles = PC_SCREEN_PROFILES
    weights = [get_pc_screen_profile_weight(profile) for profile in profiles]
    return rng.choices(profiles, weights=weights, k=1)[0]


def choose_pc_screen_profile_for_hardware(
    rng: random.Random,
    hardware_profile: dict[str, object] | None,
    tag: str | None = "windows",
    gpu_profile: dict[str, object] | None = None,
) -> dict[str, object]:
    candidates = get_compatible_pc_screen_profiles_for_device(
        hardware_profile,
        tag=tag,
        gpu_profile=gpu_profile,
    )
    if not candidates:
        return choose_pc_screen_profile(rng, tag=tag)
    weights = [get_pc_screen_profile_weight(profile) for profile in candidates]
    return rng.choices(candidates, weights=weights, k=1)[0]


def materialize_pc_screen_profile_for_windows(
    profile: dict[str, object],
    platform_version: str,
) -> dict[str, object]:
    """Return an OS-specific copy of a Windows screen profile.

    The immutable catalog remains additive and platform-neutral. Chromium's
    frozen ``Windows NT 10.0`` UA does not distinguish Windows 10 from 11, but
    UA-CH platform versions 10 and 15 do. Their default taskbar work areas are
    modeled separately here so a Windows 10 profile is not paired with the
    Windows 11 48-CSS-pixel work area.
    """

    try:
        platform_major = int(str(platform_version).split(".", 1)[0])
    except (TypeError, ValueError) as error:
        raise ValueError("invalid Windows UA-CH platform version") from error
    taskbar_height = 48 if platform_major >= 13 else 40

    materialized = deepcopy(profile)
    screen = materialized.get("screen", {})
    window = materialized.get("window", {})
    viewport = materialized.get("visualViewport", {})
    media_viewport = materialized.get("mediaViewport", {})
    if not isinstance(screen, dict) or not isinstance(window, dict):
        raise ValueError("selected screen profile is malformed")

    screen_height = int(screen.get("height", 0) or 0)
    outer_height = max(0, screen_height - taskbar_height)
    inner_height = max(0, outer_height - 88)
    screen["availHeight"] = outer_height
    window["outerHeight"] = outer_height
    window["innerHeight"] = inner_height
    if isinstance(viewport, dict):
        viewport["height"] = inner_height
    if isinstance(media_viewport, dict):
        media_viewport["height"] = inner_height

    materialized["taskbarCssHeight"] = taskbar_height
    materialized["windowsPlatformVersion"] = str(platform_version)
    materialized["id"] = (
        f"{profile.get('id', 'pc_screen')}__win{platform_major}_taskbar{taskbar_height}"
    )
    return materialized


def _compute_compatible_pc_screen_profiles_for_device(
    hardware_profile: dict[str, object] | None,
    tag: str | None = "windows",
    gpu_profile: dict[str, object] | None = None,
) -> tuple[dict[str, object], ...]:
    """Filter screen rows by form factor without collapsing the long tail."""

    profiles = get_pc_screen_profiles(tag=tag)
    if not profiles:
        return ()

    hardware_tags = tuple(
        str(item).lower()
        for item in (hardware_profile or {}).get("tags", ())
    )
    gpu_tier = str((gpu_profile or {}).get("tier", "") or "").lower()
    gpu_model = str((gpu_profile or {}).get("model", "") or "").lower()
    gpu_form_factor = str(
        (gpu_profile or {}).get("formFactor", "mixed") or "mixed"
    ).lower()
    portable_gpu = gpu_form_factor == "portable" or gpu_tier == "laptop" or any(
        needle in gpu_model
        for needle in ("laptop gpu", "max-q", "geforce mx")
    )

    def screen_tags(profile: dict[str, object]) -> tuple[str, ...]:
        return tuple(str(item).lower() for item in profile.get("tags", ()))

    def screen_size(profile: dict[str, object]) -> tuple[int, int]:
        screen = profile.get("screen", {})
        if not isinstance(screen, dict):
            return 0, 0
        return int(screen.get("width", 0) or 0), int(screen.get("height", 0) or 0)

    def dpr(profile: dict[str, object]) -> float:
        window = profile.get("window", {})
        if not isinstance(window, dict):
            return 1.0
        return float(window.get("devicePixelRatio", 1.0) or 1.0)

    def has_any(profile: dict[str, object], values: set[str]) -> bool:
        tags = screen_tags(profile)
        return any(value in tags for value in values)

    if "arm64" in hardware_tags or "copilot_pc" in hardware_tags:
        candidates = tuple(
            profile
            for profile in profiles
            if has_any(profile, {"arm64", "surface", "hidpi", "laptop", "scaled"})
            and not has_any(profile, {"ultrawide", "super_ultrawide", "legacy", "netbook", "5k", "6k", "8k"})
        )
    elif portable_gpu or "laptop" in hardware_tags or "surface" in hardware_tags or "touch" in hardware_tags or "convertible" in hardware_tags:
        candidates = tuple(
            profile
            for profile in profiles
            if has_any(profile, {"surface", "laptop", "hidpi", "scaled", "lowend"})
            and not has_any(profile, {"arm64", "ultrawide", "super_ultrawide", "5k", "6k", "8k"})
            and (
                "legacy" in hardware_tags
                or not has_any(profile, {"legacy", "netbook"})
            )
        )
    elif "workstation" in hardware_tags:
        candidates = tuple(
            profile
            for profile in profiles
            if has_any(profile, {"desktop", "productivity", "qhd", "4k", "ultrawide", "super_ultrawide", "hidpi"})
            and not has_any(profile, {"arm64", "legacy", "netbook", "surface", "lowend", "laptop"})
        )
    elif "gaming" in hardware_tags or "performance" in hardware_tags:
        candidates = tuple(
            profile
            for profile in profiles
            if has_any(profile, {"desktop", "qhd", "4k", "ultrawide", "windows", "productivity"})
            and not has_any(profile, {"arm64", "legacy", "netbook", "surface", "lowend", "laptop", "5k"})
        )
    elif "legacy" in hardware_tags or "lowend" in hardware_tags:
        candidates = tuple(
            profile
            for profile in profiles
            if not has_any(profile, {"arm64", "surface", "ultrawide", "super_ultrawide", "qhd", "4k", "5k", "6k", "8k", "hidpi"})
            and screen_size(profile)[0] <= 1920
            and screen_size(profile)[1] <= 1200
            and dpr(profile) <= 1.5
        )
    else:
        candidates = tuple(
            profile
            for profile in profiles
            if not has_any(profile, {"arm64", "surface", "super_ultrawide", "5k", "6k", "8k", "legacy", "netbook", "laptop"})
        )

    return candidates


_DEVICE_SCREEN_COMPATIBILITY_CACHE: dict[
    tuple[str, str, str, str, str],
    tuple[dict[str, object], ...],
] = {}


def get_compatible_pc_screen_profiles_for_device(
    hardware_profile: dict[str, object] | None,
    tag: str | None = "windows",
    gpu_profile: dict[str, object] | None = None,
) -> tuple[dict[str, object], ...]:
    """Return a cached screen pool for one hardware/GPU family."""

    hardware = hardware_profile or {}
    gpu = gpu_profile or {}
    cache_key = (
        str(hardware.get("id", "")),
        str(gpu.get("baseProfileId", gpu.get("id", ""))),
        str(gpu.get("tier", "")),
        str(gpu.get("model", "")),
        str(tag or "").lower(),
    )
    cached = _DEVICE_SCREEN_COMPATIBILITY_CACHE.get(cache_key)
    if cached is None:
        cached = _compute_compatible_pc_screen_profiles_for_device(
            hardware_profile,
            tag=tag,
            gpu_profile=gpu_profile,
        )
        _DEVICE_SCREEN_COMPATIBILITY_CACHE[cache_key] = cached
    return cached


def build_pc_screen_patch(profile: dict[str, object]) -> dict[str, object]:
    return {
        "window": profile.get("window", {}),
        "screen": profile.get("screen", {}),
        "visualViewport": profile.get("visualViewport", {}),
        "mediaViewport": profile.get("mediaViewport", {}),
        "screenProfileId": profile.get("id", ""),
        "screenDeviceClass": "pc",
    }


def iter_pc_screen_sizes(tag: str | None = None) -> Iterable[tuple[int, int]]:
    seen: set[tuple[int, int]] = set()
    for profile in get_pc_screen_profiles(tag=tag):
        screen = profile.get("screen", {})
        if not isinstance(screen, dict):
            continue
        size = (int(screen["width"]), int(screen["height"]))
        if size not in seen:
            seen.add(size)
            yield size
