"""Model-linked macOS screen/window profiles.

``nativeWidth``/``nativeHeight`` are Apple-published physical panel values.
``screen.width``/``screen.height`` are CSS-pixel values.  A row whose CSS size
is exactly half the native panel is the direct Retina 2x mode; additional rows
are documented macOS scaled modes.  Browser chrome and available-work-area
deductions are kept explicit because they are window-manager state, not panel
specifications.
"""

from __future__ import annotations

import random
from typing import Sequence


# id, device class, native W/H, CSS W/H, DPR, portable, weight, source URL.
MAC_SCREEN_ROWS: tuple[
    tuple[str, str, int, int, int, int, float, bool, int, str], ...
] = (
    (
        "mac_air13_m1_1280x800_2x",
        "air13_m1",
        2560,
        1600,
        1280,
        800,
        2.0,
        True,
        8,
        "https://support.apple.com/en-la/111883",
    ),
    (
        "mac_air13_m1_1440x900_scaled",
        "air13_m1",
        2560,
        1600,
        1440,
        900,
        2.0,
        True,
        10,
        "https://support.apple.com/en-la/111883",
    ),
    (
        "mac_air13_m1_1680x1050_scaled",
        "air13_m1",
        2560,
        1600,
        1680,
        1050,
        2.0,
        True,
        4,
        "https://support.apple.com/en-la/111883",
    ),
    (
        "mac_air13_modern_1280x832_2x",
        "air13_modern",
        2560,
        1664,
        1280,
        832,
        2.0,
        True,
        8,
        "https://support.apple.com/en-us/122209",
    ),
    (
        "mac_air13_modern_1470x956_scaled",
        "air13_modern",
        2560,
        1664,
        1470,
        956,
        2.0,
        True,
        14,
        "https://support.apple.com/en-us/122209",
    ),
    (
        "mac_air15_modern_1440x932_2x",
        "air15_modern",
        2880,
        1864,
        1440,
        932,
        2.0,
        True,
        8,
        "https://support.apple.com/en-us/122210",
    ),
    (
        "mac_air15_modern_1710x1107_scaled",
        "air15_modern",
        2880,
        1864,
        1710,
        1107,
        2.0,
        True,
        10,
        "https://support.apple.com/en-us/122210",
    ),
    (
        "mac_pro14_1512x982_2x",
        "pro14",
        3024,
        1964,
        1512,
        982,
        2.0,
        True,
        18,
        "https://support.apple.com/en-us/121552",
    ),
    (
        "mac_pro14_1800x1169_scaled",
        "pro14",
        3024,
        1964,
        1800,
        1169,
        2.0,
        True,
        7,
        "https://support.apple.com/en-us/121552",
    ),
    (
        "mac_pro16_1728x1117_2x",
        "pro16",
        3456,
        2234,
        1728,
        1117,
        2.0,
        True,
        14,
        "https://support.apple.com/en-us/121554",
    ),
    (
        "mac_pro16_2056x1329_scaled",
        "pro16",
        3456,
        2234,
        2056,
        1329,
        2.0,
        True,
        5,
        "https://support.apple.com/en-us/121554",
    ),
    (
        "mac_imac24_2240x1260_2x",
        "imac24",
        4480,
        2520,
        2240,
        1260,
        2.0,
        False,
        10,
        "https://support.apple.com/en-us/121557",
    ),
    (
        "mac_studio_display_2560x1440_2x",
        "external",
        5120,
        2880,
        2560,
        1440,
        2.0,
        False,
        4,
        "https://www.apple.com/studio-display/specs/",
    ),
    (
        "mac_external_1920x1080_1x_captured",
        "external",
        1920,
        1080,
        1920,
        1080,
        1.0,
        False,
        8,
        "local-real-device-capture:mac-edge-font-profile-2026-08-07T03-14-02.915Z(1).json",
    ),
    (
        "mac_intel13_1280x800_2x",
        "intel13_retina",
        2560,
        1600,
        1280,
        800,
        2.0,
        True,
        6,
        "https://support.apple.com/en-mide/111339",
    ),
    (
        "mac_intel13_1440x900_scaled",
        "intel13_retina",
        2560,
        1600,
        1440,
        900,
        2.0,
        True,
        10,
        "https://support.apple.com/en-mide/111339",
    ),
    (
        "mac_intel13_1680x1050_scaled",
        "intel13_retina",
        2560,
        1600,
        1680,
        1050,
        2.0,
        True,
        4,
        "https://support.apple.com/en-mide/111339",
    ),
    (
        "mac_intel16_1536x960_2x",
        "intel16_2019",
        3072,
        1920,
        1536,
        960,
        2.0,
        True,
        5,
        "https://support.apple.com/en-euro/111932",
    ),
    (
        "mac_intel16_1792x1120_scaled",
        "intel16_2019",
        3072,
        1920,
        1792,
        1120,
        2.0,
        True,
        10,
        "https://support.apple.com/en-euro/111932",
    ),
    (
        "mac_intel16_2048x1280_scaled",
        "intel16_2019",
        3072,
        1920,
        2048,
        1280,
        2.0,
        True,
        4,
        "https://support.apple.com/en-euro/111932",
    ),
    (
        "mac_intel_imac27_2560x1440_2x",
        "intel_imac27_5k",
        5120,
        2880,
        2560,
        1440,
        2.0,
        False,
        10,
        "https://support.apple.com/en-us/111913",
    ),
)


# CSS work-area observations shared with the existing macOS sandbox profiles.
# Values not present in that captured set use the same menu-bar/Dock geometry
# as the closest panel class rather than a single global subtraction.
# (width, height) -> (availHeight, availTop)
_MAC_WORK_AREAS: dict[tuple[int, int], tuple[int, int]] = {
    (1280, 800): (707, 25),
    (1440, 900): (807, 25),
    (1680, 1050): (957, 25),
    (1280, 832): (739, 33),
    (1470, 956): (863, 33),
    (1440, 932): (839, 33),
    (1710, 1107): (1014, 33),
    # Chromium 150 / macOS 26.5.2 / M5 14-inch built-in display capture.
    (1512, 982): (879, 33),
    (1800, 1169): (1067, 39),
    (1728, 1117): (1024, 33),
    (2056, 1329): (1227, 39),
    (2240, 1260): (1167, 25),
    (2560, 1440): (1407, 25),
    # User-provided Chromium 150 capture from a second M5 Mac display.
    (1920, 1080): (969, 25),
    (1536, 960): (867, 25),
    (1792, 1120): (1027, 25),
    (2048, 1280): (1187, 25),
}


# outerHeight is window-manager state and is not always identical to the
# available work-area height.  Keep captured exceptions explicit.
_MAC_OUTER_HEIGHTS: dict[tuple[int, int], int] = {
    (1512, 982): 876,
}


def build_mac_screen_profile(
    row: tuple[str, str, int, int, int, int, float, bool, int, str]
) -> dict[str, object]:
    (
        profile_id,
        device_class,
        native_width,
        native_height,
        width,
        height,
        dpr,
        portable,
        weight,
        source,
    ) = row
    try:
        avail_height, avail_top = _MAC_WORK_AREAS[(width, height)]
    except KeyError as error:
        raise ValueError(
            f"no macOS work-area evidence for {width}x{height}"
        ) from error
    # Chromium 150 captures measured 118 CSS px on the built-in DPR-2 display
    # and 121 CSS px on the DPR-1 secondary display.
    browser_chrome_height = 118 if dpr >= 1.5 else 121
    outer_width = width
    outer_height = _MAC_OUTER_HEIGHTS.get((width, height), avail_height)
    inner_width = outer_width
    inner_height = max(360, outer_height - browser_chrome_height)
    color_depth = 24 if device_class == "external" else 30
    dynamic_range = "high" if device_class in {"pro14", "pro16"} else "standard"
    return {
        "id": profile_id,
        "deviceClass": device_class,
        "nativeWidth": native_width,
        "nativeHeight": native_height,
        "portable": portable,
        "weight": weight,
        "source": source,
        "tags": ("mac", "hidpi", "laptop" if portable else "desktop"),
        "colorGamut": "p3",
        "dynamicRange": dynamic_range,
        "window": {
            "innerWidth": inner_width,
            "innerHeight": inner_height,
            "outerWidth": outer_width,
            "outerHeight": outer_height,
            "devicePixelRatio": dpr,
            "screenX": 0,
            "screenY": avail_top,
        },
        "screen": {
            "width": width,
            "height": height,
            "availWidth": width,
            "availHeight": avail_height,
            "availLeft": 0,
            "availTop": avail_top,
            "colorDepth": color_depth,
            "pixelDepth": color_depth,
            "orientation": {"type": "landscape-primary", "angle": 0},
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
    }


MAC_SCREEN_PROFILES: tuple[dict[str, object], ...] = tuple(
    build_mac_screen_profile(row) for row in MAC_SCREEN_ROWS
)


def get_mac_screen_profiles(
    device_classes: Sequence[str] | None = None,
    *,
    include_external: bool = True,
) -> tuple[dict[str, object], ...]:
    class_keys = {str(item).strip().lower() for item in (device_classes or ())}
    output = []
    for profile in MAC_SCREEN_PROFILES:
        device_class = str(profile["deviceClass"]).lower()
        if device_class == "external" and not include_external:
            continue
        if class_keys and device_class not in class_keys:
            continue
        output.append(profile)
    return tuple(output)


def choose_mac_screen_profile_for_gpu(
    rng: random.Random,
    gpu_profile: dict[str, object],
    *,
    include_external: bool = True,
) -> dict[str, object]:
    classes = tuple(
        dict.fromkeys(
            str(item).strip().lower()
            for item in gpu_profile.get("screenClasses", ())
            if str(item).strip()
        )
    )
    if not classes:
        raise ValueError("selected Mac GPU has no device-class evidence")

    # ``external`` is both the required display class for a Mac mini/Studio
    # and a possible current display for a portable Mac.  Keep the host class
    # separate so choosing an external monitor cannot silently turn a
    # MacBook into a battery-less desktop.
    host_classes = classes
    if not include_external and any(item != "external" for item in classes):
        host_classes = tuple(item for item in classes if item != "external")
    class_weights = []
    for host_class in host_classes:
        class_weights.append(
            max(
                1,
                sum(
                    int(profile.get("weight", 1))
                    for profile in MAC_SCREEN_PROFILES
                    if str(profile["deviceClass"]).lower() == host_class
                ),
            )
        )
    host_class = rng.choices(host_classes, weights=class_weights, k=1)[0]
    host_profiles = get_mac_screen_profiles(
        (host_class,),
        include_external=True,
    )
    if not host_profiles:
        raise ValueError("no host device profile matches the selected Mac GPU")
    host_portable = bool(host_profiles[0].get("portable", False))

    display_class = host_class
    if include_external and host_class != "external" and rng.random() < 0.2:
        display_class = "external"
    candidates = get_mac_screen_profiles(
        (display_class,),
        include_external=True,
    )
    if not candidates:
        raise ValueError("no screen profile matches the selected Mac GPU")
    weights = [int(profile.get("weight", 1)) for profile in candidates]
    selected = dict(rng.choices(candidates, weights=weights, k=1)[0])
    selected["hostDeviceClass"] = host_class
    selected["hostPortable"] = host_portable
    selected["hostHasCamera"] = (
        host_portable
        or "imac" in host_class
        or selected.get("id") == "mac_studio_display_2560x1440_2x"
    )
    return selected


__all__ = [
    "MAC_SCREEN_PROFILES",
    "MAC_SCREEN_ROWS",
    "build_mac_screen_profile",
    "choose_mac_screen_profile_for_gpu",
    "get_mac_screen_profiles",
]
