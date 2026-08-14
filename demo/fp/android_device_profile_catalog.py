"""Android Edge device profiles kept separate from desktop catalogs.

Screen metrics and UA models are based on Chromium DevTools' maintained
emulated-device list. SoC/GPU/RAM pairings come from the corresponding vendor
device specifications. GPU strings use the Android ANGLE renderer shape; they
never carry Windows PCI IDs or the D3D11 backend.

Primary references:
- https://github.com/ChromeDevTools/devtools-frontend/blob/main/front_end/models/emulation/EmulatedDevices.ts
- https://store.google.com/product/pixel_9_pro_specs
- https://www.samsung.com/ca/business/smartphones/galaxy-s/galaxy-s24-ultra-titanium-black-256gb-sm-s928wzkexac/
- https://www.qualcomm.com/smartphones/products/8-series/snapdragon-8-elite-mobile-platform
- https://www.mediatek.com/products/smartphones/mediatek-dimensity-9400
- https://dawn.googlesource.com/dawn/+/refs/heads/main/src/dawn/gpu_info.json
"""

from __future__ import annotations

import random
from typing import Sequence


# id, UA-CH model, Android version, hardwareConcurrency, physical RAM choices,
# CSS screen width/height, DPR, GPU vendor, GPU model, Dawn architecture,
# GL ES version, weight, tags.
ANDROID_DEVICE_ROWS: tuple[
    tuple[
        str, str, str, int, tuple[int, ...], int, int, float,
        str, str, str, str, int, tuple[str, ...]
    ], ...
] = (
    # Google Tensor devices. Chromium supplies the CSS screen/DPR records.
    ("android_pixel_9_pro_fold", "Pixel 9 Pro Fold", "14", 8, (16,), 412, 922, 2.625, "ARM", "Mali-G715", "valhall", "3.2", 2, ("phone", "foldable", "flagship")),
    ("android_pixel_8", "Pixel 8", "14", 9, (8,), 412, 915, 2.625, "ARM", "Mali-G715", "valhall", "3.2", 7, ("phone", "mainstream")),
    ("android_pixel_7", "Pixel 7", "13", 8, (8,), 412, 915, 2.625, "ARM", "Mali-G710", "valhall", "3.2", 6, ("phone", "mainstream")),
    ("android_pixel_6", "Pixel 6", "12", 8, (8,), 412, 892, 3.5, "ARM", "Mali-G78", "valhall", "3.2", 5, ("phone", "mainstream")),
    ("android_pixel_5", "Pixel 5", "11", 8, (8,), 393, 851, 2.75, "Qualcomm", "Adreno (TM) 620", "adreno-6xx", "3.2", 3, ("phone", "mainstream")),
    # Pixel 4 values below are the project's connected-device HTTPS evidence:
    # 393x830 CSS screen, DPR 2.75, 8 logical processors, 6 GiB physical RAM
    # (exposed as Chromium's 4 GiB deviceMemory bucket), and ANGLE Adreno 640.
    ("android_pixel_4", "Pixel 4", "11", 8, (6,), 393, 830, 2.75, "Qualcomm", "Adreno (TM) 640", "adreno-6xx", "3.2", 2, ("phone", "legacy", "evidence")),
    ("android_pixel_3", "Pixel 3", "11", 8, (4,), 393, 786, 2.75, "Qualcomm", "Adreno (TM) 630", "adreno-6xx", "3.2", 1, ("phone", "legacy")),

    # Samsung flagship/foldable devices and their regional SoC variants.
    ("android_galaxy_z_fold_6", "SM-F956U", "14", 8, (12,), 412, 968, 2.625, "Qualcomm", "Adreno (TM) 750", "adreno-7xx", "3.2", 2, ("phone", "foldable", "flagship")),
    ("android_galaxy_z_fold_5", "SM-F946U", "13", 8, (12,), 344, 882, 2.625, "Qualcomm", "Adreno (TM) 740", "adreno-7xx", "3.2", 2, ("phone", "foldable", "flagship")),
    ("android_galaxy_s24_ultra", "SM-S928B", "14", 8, (12,), 480, 1040, 3.0, "Qualcomm", "Adreno (TM) 750", "adreno-7xx", "3.2", 12, ("phone", "flagship")),
    ("android_galaxy_a55", "SM-A556B", "14", 8, (8, 12), 360, 800, 2.25, "Samsung", "Xclipse 530", "rdna-2", "3.2", 25, ("phone", "mainstream")),
    ("android_galaxy_s20_qualcomm", "SM-G981U", "13", 8, (12,), 412, 915, 3.5, "Qualcomm", "Adreno (TM) 650", "adreno-6xx", "3.2", 8, ("phone", "legacy", "flagship")),
    ("android_galaxy_s20_exynos", "SM-G981B", "13", 8, (8, 12), 412, 915, 3.5, "ARM", "Mali-G77", "valhall", "3.2", 7, ("phone", "legacy", "flagship")),
    ("android_galaxy_a71", "SM-A715F", "13", 8, (6, 8), 412, 914, 2.625, "Qualcomm", "Adreno (TM) 618", "adreno-6xx", "3.2", 12, ("phone", "mainstream")),
    ("android_galaxy_a51", "SM-A515F", "13", 8, (4, 6, 8), 412, 914, 2.625, "ARM", "Mali-G72", "bifrost", "3.2", 12, ("phone", "mainstream")),
    ("android_galaxy_s8_plus_qualcomm", "SM-G955U", "9", 8, (4, 6), 360, 740, 4.0, "Qualcomm", "Adreno (TM) 540", "adreno-5xx", "3.2", 4, ("phone", "legacy")),
    ("android_galaxy_s8_plus_exynos", "SM-G955F", "9", 8, (4, 6), 360, 740, 4.0, "ARM", "Mali-G71", "bifrost", "3.2", 4, ("phone", "legacy")),

    # Other Chromium-maintained Android device records.
    ("android_surface_duo", "Surface Duo", "11.0", 8, (6,), 540, 720, 2.5, "Qualcomm", "Adreno (TM) 640", "adreno-6xx", "3.2", 1, ("phone", "foldable")),
    ("android_moto_g_power_2022", "moto g power (2022)", "11", 8, (4,), 412, 823, 1.75, "Imagination Technologies", "PowerVR Rogue GE8320", "rogue", "3.2", 8, ("phone", "entry")),
    ("android_moto_g4", "Moto G (4)", "6.0.1", 8, (2, 3), 360, 640, 3.0, "Qualcomm", "Adreno (TM) 405", "adreno-4xx", "3.1", 3, ("phone", "legacy", "entry")),
)


# Hardware/OS compatibility is independent from WebGPU availability.  These
# ranges describe versions that the concrete device can actually run; Chrome
# 140+ additionally requires Android 10 or newer.
ANDROID_DEVICE_OS_RANGE_BY_PROFILE_ID: dict[str, tuple[int, int]] = {
    "android_pixel_9_pro_fold": (14, 16),
    "android_pixel_8": (14, 16),
    "android_pixel_7": (13, 16),
    "android_pixel_6": (12, 16),
    "android_pixel_5": (11, 14),
    "android_pixel_4": (10, 13),
    "android_pixel_3": (9, 12),
    "android_galaxy_z_fold_6": (14, 16),
    "android_galaxy_z_fold_5": (13, 16),
    "android_galaxy_s24_ultra": (14, 16),
    "android_galaxy_a55": (14, 16),
    "android_galaxy_s20_qualcomm": (10, 13),
    "android_galaxy_s20_exynos": (10, 13),
    "android_galaxy_a71": (10, 13),
    "android_galaxy_a51": (10, 13),
    "android_galaxy_s8_plus_qualcomm": (7, 9),
    "android_galaxy_s8_plus_exynos": (7, 9),
    "android_surface_duo": (10, 12),
    "android_moto_g_power_2022": (11, 12),
    "android_moto_g4": (6, 8),
}

ANDROID_DEVICE_OEM_BY_PROFILE_ID: dict[str, str] = {
    **{profile_id: "google" for profile_id in (
        "android_pixel_9_pro_fold", "android_pixel_8", "android_pixel_7",
        "android_pixel_6", "android_pixel_5", "android_pixel_4",
        "android_pixel_3",
    )},
    **{profile_id: "samsung" for profile_id in (
        "android_galaxy_z_fold_6", "android_galaxy_z_fold_5",
        "android_galaxy_s24_ultra", "android_galaxy_a55",
        "android_galaxy_s20_qualcomm", "android_galaxy_s20_exynos",
        "android_galaxy_a71", "android_galaxy_a51",
        "android_galaxy_s8_plus_qualcomm", "android_galaxy_s8_plus_exynos",
    )},
    "android_surface_duo": "microsoft",
    "android_moto_g_power_2022": "motorola",
    "android_moto_g4": "motorola",
}

ANDROID_GRAPHICS_PROFILE_BY_PROFILE_ID: dict[str, str] = {
    "android_pixel_9_pro_fold": "mali-valhall-modern",
    "android_pixel_8": "mali-valhall-modern",
    "android_pixel_7": "mali-valhall-modern",
    "android_pixel_6": "mali-valhall-modern",
    "android_pixel_5": "adreno-6xx-mainstream",
    "android_pixel_4": "pixel4-adreno-640-evidence",
    "android_pixel_3": "adreno-6xx-mainstream",
    "android_galaxy_z_fold_6": "adreno-7xx-flagship",
    "android_galaxy_z_fold_5": "adreno-7xx-flagship",
    "android_galaxy_s24_ultra": "adreno-7xx-flagship",
    "android_galaxy_a55": "xclipse-rdna2",
    "android_galaxy_s20_qualcomm": "adreno-6xx-mainstream",
    "android_galaxy_s20_exynos": "mali-valhall-modern",
    "android_galaxy_a71": "adreno-6xx-mainstream",
    "android_galaxy_a51": "mali-bifrost",
    "android_galaxy_s8_plus_qualcomm": "adreno-5xx",
    "android_galaxy_s8_plus_exynos": "mali-bifrost",
    "android_surface_duo": "adreno-6xx-mainstream",
    "android_moto_g_power_2022": "powervr-rogue",
    "android_moto_g4": "adreno-4xx",
}

ANDROID_MEDIA_TIER_BY_PROFILE_ID: dict[str, str] = {
    profile_id: (
        "av1-hardware"
        if profile_id in {
            "android_pixel_9_pro_fold", "android_pixel_8",
            "android_galaxy_z_fold_6", "android_galaxy_z_fold_5",
            "android_galaxy_s24_ultra", "android_galaxy_a55",
        }
        else "chromium-software-av1"
    )
    for profile_id in ANDROID_DEVICE_OS_RANGE_BY_PROFILE_ID
}

# The connected Pixel 4 capture is the only device-specific heap-limit row.
# Other devices continue through V8's RAM/platform calculation.
ANDROID_HEAP_LIMIT_BY_PROFILE_ID: dict[str, int] = {
    "android_pixel_4": 1_530_000_000,
}


def android_webgpu_supported(driver_vendor: str, android_version: int) -> bool:
    """Apply Chromium's Android WebGPU vendor/OS gate independently."""

    vendor = str(driver_vendor)
    version = int(android_version)
    if vendor in {"ARM", "Qualcomm", "Intel"}:
        return version >= 12
    if vendor == "Imagination Technologies":
        return version >= 16
    return False


def _unmasked_vendor(driver_vendor: str) -> str:
    return f"Google Inc. ({driver_vendor})"


def build_android_device_profile(row: tuple[object, ...]) -> dict[str, object]:
    (
        profile_id, model, android_version, concurrency, memory_choices,
        width, height, dpr, driver_vendor, gpu_model, webgpu_architecture,
        gl_es_version, weight, tags,
    ) = row
    renderer = f"ANGLE ({driver_vendor}, {gpu_model}, OpenGL ES {gl_es_version})"
    profile_key = str(profile_id)
    os_range = ANDROID_DEVICE_OS_RANGE_BY_PROFILE_ID[profile_key]
    return {
        "id": profile_id,
        "deviceClass": "android-phone",
        "model": model,
        "androidVersion": android_version,
        "supportedAndroidVersions": os_range,
        "oem": ANDROID_DEVICE_OEM_BY_PROFILE_ID[profile_key],
        "graphicsProfileId": ANDROID_GRAPHICS_PROFILE_BY_PROFILE_ID[profile_key],
        "mediaTier": ANDROID_MEDIA_TIER_BY_PROFILE_ID[profile_key],
        "jsHeapSizeLimit": ANDROID_HEAP_LIMIT_BY_PROFILE_ID.get(profile_key),
        "hardwareConcurrency": concurrency,
        "physicalMemoryChoicesGb": memory_choices,
        "maxTouchPoints": 5,
        "weight": weight,
        "tags": tags,
        # Chromium's Android WebGPU blocklist explicitly enables ARM,
        # Qualcomm and Intel adapters. Other vendors remain in the inventory
        # but are not used by the generated profile until adapter-unavailable
        # semantics are representable by the sandbox.
        "webgpuSupported": android_webgpu_supported(
            str(driver_vendor), int(str(android_version).split(".", 1)[0])
        ),
        "gpu": {
            "driverVendor": driver_vendor,
            "vendor": driver_vendor.lower().replace(" technologies", ""),
            "model": gpu_model,
            "webgpuArchitecture": webgpu_architecture,
            "webgl": {
                "unmaskedVendor": _unmasked_vendor(str(driver_vendor)),
                "unmaskedRenderer": renderer,
            },
        },
        "screen": {
            "width": width,
            "height": height,
            "availWidth": width,
            "availHeight": height,
            "colorDepth": 24,
            "pixelDepth": 24,
            "orientation": {"type": "portrait-primary", "angle": 0},
        },
        "window": {
            "innerWidth": width,
            "innerHeight": height,
            "outerWidth": width,
            "outerHeight": height,
            "devicePixelRatio": dpr,
            "screenX": 0,
            "screenY": 0,
        },
        "visualViewport": {
            "width": width,
            "height": height,
            "offsetLeft": 0,
            "offsetTop": 0,
            "pageLeft": 0,
            "pageTop": 0,
            "scale": 1,
        },
    }


ANDROID_DEVICE_PROFILES: tuple[dict[str, object], ...] = tuple(
    build_android_device_profile(row) for row in ANDROID_DEVICE_ROWS
)


def get_android_device_profiles(
    model: str | None = None,
    android_version: int | None = None,
    minimum_android_version: int | None = None,
) -> tuple[dict[str, object], ...]:
    model_key = str(model or "").strip().lower()
    output = ANDROID_DEVICE_PROFILES
    if model_key and model_key not in {"k", "android"}:
        output = tuple(
            profile for profile in output
            if str(profile.get("model", "")).lower() == model_key
        )
    if android_version is not None:
        requested = int(android_version)
        output = tuple(
            profile
            for profile in output
            if (
                ANDROID_DEVICE_OS_RANGE_BY_PROFILE_ID[str(profile.get("id", ""))][0]
                <= requested
                <= ANDROID_DEVICE_OS_RANGE_BY_PROFILE_ID[
                    str(profile.get("id", ""))
                ][1]
            )
        )
    if minimum_android_version is not None:
        minimum = int(minimum_android_version)
        output = tuple(
            profile
            for profile in output
            if ANDROID_DEVICE_OS_RANGE_BY_PROFILE_ID[
                str(profile.get("id", ""))
            ][1] >= minimum
        )
    return output


def choose_android_version_for_device(
    rng: random.Random,
    device: dict[str, object],
    requested_android_version: int | None = None,
    *,
    minimum_android_version: int = 10,
) -> int:
    """Choose an OS the selected device can run without reading the frozen UA."""

    profile_id = str(device.get("id", ""))
    lower, upper = ANDROID_DEVICE_OS_RANGE_BY_PROFILE_ID[profile_id]
    lower = max(lower, int(minimum_android_version))
    if requested_android_version is not None:
        requested = int(requested_android_version)
        if not lower <= requested <= upper:
            raise ValueError("Android UA model has no compatible platform version")
        return requested
    versions = tuple(range(lower, upper + 1))
    if not versions:
        raise ValueError("Android device has no browser-supported platform version")
    # Installed devices skew toward newer supported releases, while retained
    # older versions remain reachable for compatibility testing.
    weights = tuple(range(1, len(versions) + 1))
    return rng.choices(versions, weights=weights, k=1)[0]


def materialize_android_device_profile(
    device: dict[str, object],
    android_version: int,
) -> dict[str, object]:
    """Return a concrete device/OS row with OS-dependent capabilities."""

    output = dict(device)
    output["androidVersion"] = str(int(android_version))
    gpu = dict(output.get("gpu", {}))
    output["gpu"] = gpu
    output["webgpuSupported"] = android_webgpu_supported(
        str(gpu.get("driverVendor", "")), int(android_version)
    )
    return output


def choose_android_device_profile(
    rng: random.Random,
    candidates: Sequence[dict[str, object]],
) -> dict[str, object]:
    if not candidates:
        raise ValueError("Android UA model has no compatible device profile")
    weights = [max(1, int(item.get("weight", 1))) for item in candidates]
    return rng.choices(tuple(candidates), weights=weights, k=1)[0]


def count_android_device_profiles() -> int:
    return len(ANDROID_DEVICE_PROFILES)


def get_android_device_profile_by_id(profile_id: str) -> dict[str, object]:
    key = str(profile_id)
    for profile in ANDROID_DEVICE_PROFILES:
        if profile.get("id") == key:
            return profile
    raise KeyError(f"unknown Android device profile {key!r}")


__all__ = [
    "ANDROID_DEVICE_PROFILES",
    "ANDROID_DEVICE_OS_RANGE_BY_PROFILE_ID",
    "ANDROID_DEVICE_OEM_BY_PROFILE_ID",
    "ANDROID_GRAPHICS_PROFILE_BY_PROFILE_ID",
    "android_webgpu_supported",
    "choose_android_device_profile",
    "choose_android_version_for_device",
    "count_android_device_profiles",
    "get_android_device_profile_by_id",
    "get_android_device_profiles",
    "materialize_android_device_profile",
]
