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
    ("android_pixel_9_pro_fold", "Pixel 9 Pro Fold", "14", 8, (16,), 412, 922, 2.625, "ARM", "Mali-G715", "valhall", "3.2", 16, ("phone", "foldable", "flagship")),
    ("android_pixel_8", "Pixel 8", "14", 9, (8,), 412, 915, 2.625, "ARM", "Mali-G715", "valhall", "3.2", 30, ("phone", "mainstream")),
    ("android_pixel_7", "Pixel 7", "13", 8, (8,), 412, 915, 2.625, "ARM", "Mali-G710", "valhall", "3.2", 28, ("phone", "mainstream")),
    ("android_pixel_6", "Pixel 6", "12", 8, (8,), 412, 892, 3.5, "ARM", "Mali-G78", "valhall", "3.2", 18, ("phone", "mainstream")),
    ("android_pixel_5", "Pixel 5", "11", 8, (8,), 393, 851, 2.75, "Qualcomm", "Adreno (TM) 620", "adreno-6xx", "3.2", 12, ("phone", "mainstream")),
    ("android_pixel_3", "Pixel 3", "11", 8, (4,), 393, 786, 2.75, "Qualcomm", "Adreno (TM) 630", "adreno-6xx", "3.2", 6, ("phone", "legacy")),

    # Samsung flagship/foldable devices and their regional SoC variants.
    ("android_galaxy_z_fold_6", "SM-F956U", "14", 8, (12,), 412, 968, 2.625, "Qualcomm", "Adreno (TM) 750", "adreno-7xx", "3.2", 18, ("phone", "foldable", "flagship")),
    ("android_galaxy_z_fold_5", "SM-F946U", "13", 8, (12,), 344, 882, 2.625, "Qualcomm", "Adreno (TM) 740", "adreno-7xx", "3.2", 14, ("phone", "foldable", "flagship")),
    ("android_galaxy_s24_ultra", "SM-S928B", "14", 8, (12,), 480, 1040, 3.0, "Qualcomm", "Adreno (TM) 750", "adreno-7xx", "3.2", 24, ("phone", "flagship")),
    ("android_galaxy_a55", "SM-A556B", "14", 8, (8, 12), 360, 800, 2.25, "Samsung", "Xclipse 530", "rdna-2", "3.2", 22, ("phone", "mainstream")),
    ("android_galaxy_s20_qualcomm", "SM-G981U", "13", 8, (12,), 412, 915, 3.5, "Qualcomm", "Adreno (TM) 650", "adreno-6xx", "3.2", 10, ("phone", "legacy", "flagship")),
    ("android_galaxy_s20_exynos", "SM-G981B", "13", 8, (8, 12), 412, 915, 3.5, "ARM", "Mali-G77", "valhall", "3.2", 9, ("phone", "legacy", "flagship")),
    ("android_galaxy_a71", "SM-A715F", "13", 8, (6, 8), 412, 914, 2.625, "Qualcomm", "Adreno (TM) 618", "adreno-6xx", "3.2", 10, ("phone", "mainstream")),
    ("android_galaxy_a51", "SM-A515F", "13", 8, (4, 6, 8), 412, 914, 2.625, "ARM", "Mali-G72", "bifrost", "3.2", 10, ("phone", "mainstream")),
    ("android_galaxy_s8_plus_qualcomm", "SM-G955U", "9", 8, (4, 6), 360, 740, 4.0, "Qualcomm", "Adreno (TM) 540", "adreno-5xx", "3.2", 4, ("phone", "legacy")),
    ("android_galaxy_s8_plus_exynos", "SM-G955F", "9", 8, (4, 6), 360, 740, 4.0, "ARM", "Mali-G71", "bifrost", "3.2", 4, ("phone", "legacy")),

    # Other Chromium-maintained Android device records.
    ("android_surface_duo", "Surface Duo", "11.0", 8, (6,), 540, 720, 2.5, "Qualcomm", "Adreno (TM) 640", "adreno-6xx", "3.2", 5, ("phone", "foldable")),
    ("android_moto_g_power_2022", "moto g power (2022)", "11", 8, (4,), 412, 823, 1.75, "Imagination Technologies", "PowerVR Rogue GE8320", "rogue", "3.2", 7, ("phone", "entry")),
    ("android_moto_g4", "Moto G (4)", "6.0.1", 8, (2, 3), 360, 640, 3.0, "Qualcomm", "Adreno (TM) 405", "adreno-4xx", "3.1", 3, ("phone", "legacy", "entry")),
)


def _unmasked_vendor(driver_vendor: str) -> str:
    return f"Google Inc. ({driver_vendor})"


def build_android_device_profile(row: tuple[object, ...]) -> dict[str, object]:
    (
        profile_id, model, android_version, concurrency, memory_choices,
        width, height, dpr, driver_vendor, gpu_model, webgpu_architecture,
        gl_es_version, weight, tags,
    ) = row
    renderer = f"ANGLE ({driver_vendor}, {gpu_model}, OpenGL ES {gl_es_version})"
    return {
        "id": profile_id,
        "deviceClass": "android-phone",
        "model": model,
        "androidVersion": android_version,
        "hardwareConcurrency": concurrency,
        "physicalMemoryChoicesGb": memory_choices,
        "maxTouchPoints": 5,
        "weight": weight,
        "tags": tags,
        # Chromium's Android WebGPU blocklist explicitly enables ARM,
        # Qualcomm and Intel adapters. Other vendors remain in the inventory
        # but are not used by the generated profile until adapter-unavailable
        # semantics are representable by the sandbox.
        "webgpuSupported": str(driver_vendor) in {"ARM", "Qualcomm", "Intel"},
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


# Conservative official-update ranges used by the generated WebGPU-capable
# Android branch. The lower bound is at least Android 12 because Chromium's
# default Android WebGPU rollout requires Android 12+; the upper bound avoids
# pairing a current UA with hardware whose vendor support ended years earlier.
ANDROID_WEBGPU_OS_RANGE_BY_PROFILE_ID: dict[str, tuple[int, int]] = {
    "android_pixel_9_pro_fold": (14, 17),
    "android_pixel_8": (14, 17),
    "android_pixel_7": (13, 17),
    "android_pixel_6": (12, 17),
    "android_pixel_5": (12, 14),
    "android_pixel_3": (12, 12),
    "android_galaxy_z_fold_6": (14, 17),
    "android_galaxy_z_fold_5": (13, 17),
    "android_galaxy_s24_ultra": (14, 17),
    "android_galaxy_a55": (14, 17),
    "android_galaxy_s20_qualcomm": (12, 13),
    "android_galaxy_s20_exynos": (12, 13),
    "android_galaxy_a71": (12, 13),
    "android_galaxy_a51": (12, 13),
    "android_surface_duo": (12, 12),
    "android_moto_g_power_2022": (12, 12),
}


def get_android_device_profiles(
    model: str | None = None,
    android_version: int | None = None,
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
                str(profile.get("id", "")) in ANDROID_WEBGPU_OS_RANGE_BY_PROFILE_ID
                and ANDROID_WEBGPU_OS_RANGE_BY_PROFILE_ID[
                    str(profile.get("id", ""))
                ][0]
                <= requested
                <= ANDROID_WEBGPU_OS_RANGE_BY_PROFILE_ID[
                    str(profile.get("id", ""))
                ][1]
            )
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
