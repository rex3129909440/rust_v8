"""Apple-published Mac GPU/model candidates for Chromium profiles.

The observable WebGL renderer is derived from ANGLE's Metal backend, which
builds its renderer description from ``"ANGLE Metal Renderer: "`` plus the
active ``MTLDevice.name``.  Chromium appends the WebGL-safe version text
``"Unspecified Version"``.  Apple documents the Metal GPU-family mapping and
the CPU/GPU core counts used below.

Apple product specifications establish the model, core, memory, and display
relationships in these rows.  Only Apple-silicon rows currently have a full,
version-locked ANGLE/Dawn capability record.  Intel/AMD rows remain available
for inventory and future real-device captures, but are excluded from the
default random profile pool instead of receiving guessed runtime limits.
"""

from __future__ import annotations

import random
from typing import Iterable, Sequence

try:
    from .mac_graphics_capability_catalog import (
        ANGLE_DISPLAY_MTL_SOURCE,
        is_verified_mac_graphics_candidate,
    )
except ImportError:  # Direct import from demo/fp.
    from mac_graphics_capability_catalog import (  # type: ignore
        ANGLE_DISPLAY_MTL_SOURCE,
        is_verified_mac_graphics_candidate,
    )


ANGLE_METAL_SOURCE = ANGLE_DISPLAY_MTL_SOURCE
APPLE_METAL_FAMILY_SOURCE = "https://developer.apple.com/metal/capabilities/"


# id, MTLDevice.name, GPU family, observable logical CPU count, GPU cores,
# memory choices GiB,
# compatible screen device classes, sampling weight, Apple specification URL.
MAC_GPU_ROWS: tuple[
    tuple[str, str, str, int, int, tuple[int, ...], tuple[str, ...], int, str], ...
] = (
    (
        "mac_m1_7gpu",
        "Apple M1",
        "apple7",
        8,
        7,
        (8, 16),
        ("air13_m1", "imac24"),
        7,
        "https://support.apple.com/en-la/111883",
    ),
    (
        "mac_m1_8gpu",
        "Apple M1",
        "apple7",
        8,
        8,
        (8, 16),
        ("air13_m1", "imac24", "external"),
        10,
        "https://support.apple.com/en-la/111883",
    ),
    (
        "mac_m1_pro_8cpu_14gpu",
        "Apple M1 Pro",
        "apple7",
        8,
        14,
        (16, 32),
        ("pro14",),
        5,
        "https://support.apple.com/en-us/111902",
    ),
    (
        "mac_m1_pro_10cpu_14gpu",
        "Apple M1 Pro",
        "apple7",
        10,
        14,
        (16, 32),
        ("pro14",),
        4,
        "https://support.apple.com/en-us/111902",
    ),
    (
        "mac_m1_pro_10cpu_16gpu",
        "Apple M1 Pro",
        "apple7",
        10,
        16,
        (16, 32),
        ("pro14", "pro16"),
        7,
        "https://support.apple.com/en-us/111902",
    ),
    (
        "mac_m1_max_10cpu_24gpu",
        "Apple M1 Max",
        "apple7",
        10,
        24,
        (32, 64),
        ("pro14", "pro16", "external"),
        3,
        "https://support.apple.com/en-us/111902",
    ),
    (
        "mac_m1_max_10cpu_32gpu",
        "Apple M1 Max",
        "apple7",
        10,
        32,
        (32, 64),
        ("pro14", "pro16", "external"),
        2,
        "https://support.apple.com/en-ie/111900",
    ),
    (
        "mac_m1_ultra_20cpu_48gpu",
        "Apple M1 Ultra",
        "apple7",
        20,
        48,
        (64, 128),
        ("external",),
        1,
        "https://support.apple.com/en-ie/111900",
    ),
    (
        "mac_m1_ultra_20cpu_64gpu",
        "Apple M1 Ultra",
        "apple7",
        20,
        64,
        (64, 128),
        ("external",),
        1,
        "https://support.apple.com/en-ie/111900",
    ),
    (
        "mac_m2_8gpu",
        "Apple M2",
        "apple8",
        8,
        8,
        (8, 16, 24),
        ("air13_modern", "air15_modern", "external"),
        9,
        "https://support.apple.com/en-us/111867",
    ),
    (
        "mac_m2_10gpu",
        "Apple M2",
        "apple8",
        8,
        10,
        (8, 16, 24),
        ("air13_modern", "air15_modern", "external"),
        9,
        "https://support.apple.com/en-us/111867",
    ),
    (
        "mac_m2_pro_16gpu",
        "Apple M2 Pro",
        "apple8",
        10,
        16,
        (16, 32),
        ("pro14", "external"),
        6,
        "https://support.apple.com/en-ca/111340",
    ),
    (
        "mac_m2_pro_19gpu",
        "Apple M2 Pro",
        "apple8",
        12,
        19,
        (16, 32),
        ("pro14", "pro16", "external"),
        5,
        "https://support.apple.com/en-ca/111340",
    ),
    (
        "mac_m2_max_30gpu",
        "Apple M2 Max",
        "apple8",
        12,
        30,
        (32, 64),
        ("pro14", "pro16", "external"),
        2,
        "https://support.apple.com/en-ca/111340",
    ),
    (
        "mac_m2_max_38gpu",
        "Apple M2 Max",
        "apple8",
        12,
        38,
        (32, 64, 96),
        ("pro14", "pro16", "external"),
        2,
        "https://support.apple.com/en-ca/111340",
    ),
    (
        "mac_m2_ultra_24cpu_60gpu",
        "Apple M2 Ultra",
        "apple8",
        24,
        60,
        (64, 128, 192),
        ("external",),
        1,
        "https://support.apple.com/en-ie/111835",
    ),
    (
        "mac_m2_ultra_24cpu_76gpu",
        "Apple M2 Ultra",
        "apple8",
        24,
        76,
        (64, 128, 192),
        ("external",),
        1,
        "https://support.apple.com/en-ie/111835",
    ),
    (
        "mac_m3_8gpu",
        "Apple M3",
        "apple9",
        8,
        8,
        (8, 16, 24),
        ("air13_modern", "imac24"),
        7,
        "https://support.apple.com/en-us/118551",
    ),
    (
        "mac_m3_10gpu",
        "Apple M3",
        "apple9",
        8,
        10,
        (8, 16, 24),
        ("air13_modern", "air15_modern", "pro14", "imac24", "external"),
        10,
        "https://support.apple.com/en-us/118551",
    ),
    (
        "mac_m3_pro_11cpu_14gpu",
        "Apple M3 Pro",
        "apple9",
        11,
        14,
        (18, 36),
        ("pro14",),
        5,
        "https://support.apple.com/en-euro/117736",
    ),
    (
        "mac_m3_pro_12cpu_18gpu",
        "Apple M3 Pro",
        "apple9",
        12,
        18,
        (18, 36),
        ("pro14", "pro16"),
        7,
        "https://support.apple.com/en-euro/117736",
    ),
    (
        "mac_m3_max_14cpu_30gpu",
        "Apple M3 Max",
        "apple9",
        14,
        30,
        (36, 96),
        ("pro14", "pro16"),
        3,
        "https://support.apple.com/en-euro/117736",
    ),
    (
        "mac_m3_max_16cpu_40gpu",
        "Apple M3 Max",
        "apple9",
        16,
        40,
        (48, 64, 128),
        ("pro14", "pro16"),
        2,
        "https://support.apple.com/en-euro/117736",
    ),
    (
        "mac_m3_ultra_28cpu_60gpu",
        "Apple M3 Ultra",
        "apple9",
        28,
        60,
        (96, 256),
        ("external",),
        1,
        "https://support.apple.com/en-us/122211",
    ),
    (
        "mac_m3_ultra_32cpu_80gpu",
        "Apple M3 Ultra",
        "apple9",
        32,
        80,
        (96, 256),
        ("external",),
        1,
        "https://support.apple.com/en-us/122211",
    ),
    (
        "mac_m4_air_8gpu",
        "Apple M4",
        "apple9",
        10,
        8,
        (16, 24, 32),
        ("air13_modern",),
        8,
        "https://support.apple.com/en-us/122209",
    ),
    (
        "mac_m4_air_10gpu",
        "Apple M4",
        "apple9",
        10,
        10,
        (16, 24, 32),
        ("air13_modern", "air15_modern"),
        10,
        "https://support.apple.com/en-us/122209",
    ),
    (
        "mac_m4_imac_10gpu",
        "Apple M4",
        "apple9",
        10,
        10,
        (16, 24, 32),
        ("imac24",),
        5,
        "https://support.apple.com/en-us/121557",
    ),
    (
        "mac_m4_pro14_10gpu",
        "Apple M4",
        "apple9",
        10,
        10,
        (16, 24, 32),
        ("pro14", "external"),
        7,
        "https://support.apple.com/en-ca/121552",
    ),
    (
        "mac_m4_pro_16gpu",
        "Apple M4 Pro",
        "apple9",
        12,
        16,
        (24, 48),
        ("pro14", "pro16", "external"),
        6,
        "https://support.apple.com/en-us/121553",
    ),
    (
        "mac_m4_pro_20gpu",
        "Apple M4 Pro",
        "apple9",
        14,
        20,
        (24, 48),
        ("pro14", "pro16", "external"),
        7,
        "https://support.apple.com/en-us/121554",
    ),
    (
        "mac_m4_max_32gpu",
        "Apple M4 Max",
        "apple9",
        14,
        32,
        (36,),
        ("pro14", "pro16", "external"),
        3,
        "https://support.apple.com/en-us/121554",
    ),
    (
        "mac_m4_max_40gpu",
        "Apple M4 Max",
        "apple9",
        16,
        40,
        (48, 64, 128),
        ("pro14", "pro16", "external"),
        2,
        "https://support.apple.com/en-us/121554",
    ),
    (
        "mac_m5_air_8gpu",
        "Apple M5",
        "apple10",
        10,
        8,
        (16, 24, 32),
        ("air13_modern",),
        5,
        "https://support.apple.com/en-mide/126320",
    ),
    (
        "mac_m5_air_10gpu",
        "Apple M5",
        "apple10",
        10,
        10,
        (16, 24, 32),
        ("air13_modern", "air15_modern"),
        6,
        "https://support.apple.com/en-mide/126320",
    ),
    (
        "mac_m5_pro14_10gpu",
        "Apple M5",
        "apple10",
        10,
        10,
        (16, 24, 32),
        ("pro14",),
        8,
        "https://support.apple.com/en-ca/125405",
    ),
    (
        "mac_m5_pro_15cpu_16gpu",
        "Apple M5 Pro",
        "apple10",
        15,
        16,
        (24, 48),
        ("pro14",),
        10,
        "https://support.apple.com/en-us/126318",
    ),
    (
        "mac_m5_pro_18cpu_20gpu",
        "Apple M5 Pro",
        "apple10",
        18,
        20,
        (24, 48, 64),
        ("pro14", "pro16"),
        7,
        "https://support.apple.com/en-us/126318",
    ),
    (
        "mac_m5_max_18cpu_32gpu",
        "Apple M5 Max",
        "apple10",
        18,
        32,
        (36,),
        ("pro14", "pro16"),
        3,
        "https://support.apple.com/en-us/126318",
    ),
    (
        "mac_m5_max_18cpu_40gpu",
        "Apple M5 Max",
        "apple10",
        18,
        40,
        (48, 64, 128),
        ("pro14", "pro16"),
        2,
        "https://support.apple.com/en-us/126318",
    ),
    (
        "mac_intel_air_2020_iris_plus_dual",
        "Intel(R) Iris(TM) Plus Graphics",
        "gen11",
        4,
        48,
        (8, 16),
        ("intel13_retina",),
        2,
        "https://support.apple.com/en-euro/111991",
    ),
    (
        "mac_intel_air_2020_iris_plus_quad",
        "Intel(R) Iris(TM) Plus Graphics",
        "gen11",
        8,
        64,
        (8, 16),
        ("intel13_retina",),
        3,
        "https://support.apple.com/en-euro/111991",
    ),
    (
        "mac_intel_pro13_2020_iris_645",
        "Intel(R) Iris(TM) Plus Graphics 645",
        "gen9",
        8,
        48,
        (8, 16),
        ("intel13_retina",),
        3,
        "https://support.apple.com/en-mide/111981",
    ),
    (
        "mac_intel_pro13_2020_iris_plus",
        "Intel(R) Iris(TM) Plus Graphics",
        "gen11",
        8,
        64,
        (16, 32),
        ("intel13_retina",),
        3,
        "https://support.apple.com/en-mide/111339",
    ),
    (
        "mac_intel_pro16_2019_uhd_630",
        "Intel(R) UHD Graphics 630",
        "gen9",
        12,
        24,
        (16, 32, 64),
        ("intel16_2019",),
        2,
        "https://support.apple.com/en-euro/111932",
    ),
    (
        "mac_intel_pro16_2019_radeon_5300m",
        "AMD Radeon Pro 5300M",
        "rdna1",
        12,
        20,
        (16, 32, 64),
        ("intel16_2019",),
        2,
        "https://support.apple.com/en-euro/111932",
    ),
    (
        "mac_intel_pro16_2019_radeon_5500m",
        "AMD Radeon Pro 5500M",
        "rdna1",
        16,
        24,
        (16, 32, 64),
        ("intel16_2019",),
        2,
        "https://support.apple.com/en-euro/111932",
    ),
    (
        "mac_intel_pro16_2019_radeon_5600m",
        "AMD Radeon Pro 5600M",
        "rdna1",
        16,
        40,
        (16, 32, 64),
        ("intel16_2019",),
        1,
        "https://support.apple.com/en-euro/111932",
    ),
    (
        "mac_intel_imac27_2020_radeon_5300",
        "AMD Radeon Pro 5300",
        "rdna1",
        12,
        20,
        (8, 16, 32, 64, 128),
        ("intel_imac27_5k",),
        2,
        "https://support.apple.com/en-us/111913",
    ),
    (
        "mac_intel_imac27_2020_radeon_5500xt",
        "AMD Radeon Pro 5500 XT",
        "rdna1",
        16,
        24,
        (8, 16, 32, 64, 128),
        ("intel_imac27_5k",),
        2,
        "https://support.apple.com/en-us/111913",
    ),
    (
        "mac_intel_imac27_2020_radeon_5700xt",
        "AMD Radeon Pro 5700 XT",
        "rdna1",
        20,
        40,
        (8, 16, 32, 64, 128),
        ("intel_imac27_5k",),
        1,
        "https://support.apple.com/en-us/111913",
    ),
)


def build_mac_gpu_candidate(
    row: tuple[str, str, str, int, int, tuple[int, ...], tuple[str, ...], int, str]
) -> dict[str, object]:
    (
        profile_id,
        device_name,
        family,
        cpu_cores,
        gpu_cores,
        memory_choices,
        screen_classes,
        weight,
        source,
    ) = row
    intel_cpu = profile_id.startswith("mac_intel_")
    if device_name.startswith("Intel"):
        gpu_vendor = "intel"
        angle_vendor = "Intel"
        unmasked_vendor = "Google Inc. (Intel)"
        driver_vendor = "Intel"
    elif device_name.startswith("AMD"):
        gpu_vendor = "amd"
        angle_vendor = "AMD"
        unmasked_vendor = "Google Inc. (AMD)"
        driver_vendor = "AMD"
    else:
        gpu_vendor = "apple"
        angle_vendor = "Apple"
        unmasked_vendor = "Google Inc. (Apple)"
        driver_vendor = "Apple"
    renderer = (
        f"ANGLE ({angle_vendor}, ANGLE Metal Renderer: {device_name}, "
        "Unspecified Version)"
    )
    chip_generation = next(
        (
            generation
            for generation in ("M1", "M2", "M3", "M4", "M5")
            if device_name.startswith(f"Apple {generation}")
        ),
        "Intel" if intel_cpu else "Legacy",
    )
    chip_tier = next(
        (
            tier
            for tier in ("Ultra", "Max", "Pro")
            if device_name.endswith(f" {tier}")
        ),
        "Base",
    )
    candidate = {
        "id": profile_id,
        "vendor": gpu_vendor,
        "driverVendor": driver_vendor,
        "architecture": family,
        "cpuArchitecture": "x86" if intel_cpu else "arm",
        "cpuBitness": "64",
        "macosPlatformVersion": (
            "15.5.0" if intel_cpu else "26.5.2"
        ),
        "tier": "integrated",
        "model": device_name,
        "deviceMarker": device_name,
        "cpuCores": cpu_cores,
        "gpuCores": gpu_cores,
        "chipGeneration": chip_generation,
        "chipTier": chip_tier,
        "gpuCoreUnit": "cores" if gpu_vendor == "apple" else (
            "execution-units" if gpu_vendor == "intel" else "compute-units"
        ),
        "memoryChoicesGb": memory_choices,
        "screenClasses": screen_classes,
        "weight": weight,
        "source": source,
        "webgl": {
            "unmaskedVendor": unmasked_vendor,
            "unmaskedRenderer": renderer,
        },
        "webgpu": {
            "vendor": gpu_vendor,
            "architecture": "metal-3" if gpu_vendor == "apple" else family,
            "device": device_name,
            "description": f"{device_name} Metal adapter",
        },
    }
    candidate["graphicsVerified"] = is_verified_mac_graphics_candidate(candidate)
    candidate["graphicsCapabilityId"] = (
        f"chromium150-angle-metal-{family}"
        if candidate["graphicsVerified"]
        else None
    )
    return candidate


MAC_GPU_CANDIDATES: tuple[dict[str, object], ...] = tuple(
    build_mac_gpu_candidate(row) for row in MAC_GPU_ROWS
)


def get_mac_gpu_candidates(
    *,
    family: str | None = None,
    generation: str | None = None,
    device_class: str | None = None,
    verified_only: bool = True,
) -> tuple[dict[str, object], ...]:
    family_key = str(family or "").strip().lower()
    generation_key = str(generation or "").strip().lower()
    device_key = str(device_class or "").strip().lower()
    output = []
    for candidate in MAC_GPU_CANDIDATES:
        if verified_only and not bool(candidate.get("graphicsVerified", False)):
            continue
        if family_key and str(candidate["architecture"]).lower() != family_key:
            continue
        if generation_key and str(candidate["chipGeneration"]).lower() != generation_key:
            continue
        classes = tuple(str(item).lower() for item in candidate["screenClasses"])
        if device_key and device_key not in classes:
            continue
        output.append(candidate)
    return tuple(output)


def choose_mac_gpu_candidate(
    rng: random.Random,
    candidates: Sequence[dict[str, object]] | None = None,
) -> dict[str, object]:
    choices = (
        tuple(candidates)
        if candidates is not None
        else get_mac_gpu_candidates(verified_only=True)
    )
    if not choices:
        raise ValueError("no Mac GPU candidates available")
    weights = [int(candidate.get("weight", 1)) for candidate in choices]
    return rng.choices(choices, weights=weights, k=1)[0]


def iter_mac_gpu_names() -> Iterable[str]:
    seen: set[str] = set()
    for candidate in MAC_GPU_CANDIDATES:
        name = str(candidate["model"])
        if name not in seen:
            seen.add(name)
            yield name


__all__ = [
    "ANGLE_METAL_SOURCE",
    "APPLE_METAL_FAMILY_SOURCE",
    "MAC_GPU_CANDIDATES",
    "MAC_GPU_ROWS",
    "build_mac_gpu_candidate",
    "choose_mac_gpu_candidate",
    "get_mac_gpu_candidates",
    "iter_mac_gpu_names",
]
