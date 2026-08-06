"""Evidence-backed Apple-silicon GPU candidates for macOS Chromium profiles.

The observable WebGL renderer is derived from ANGLE's Metal backend, which
builds its renderer description from ``"ANGLE Metal Renderer: "`` plus the
active ``MTLDevice.name``.  Chromium appends the WebGL-safe version text
``"Unspecified Version"``.  Apple documents the Metal GPU-family mapping and
the CPU/GPU core counts used below.

This catalog intentionally contains Apple silicon only.  An Intel Mac needs a
captured Intel/AMD adapter and must not be synthesized from these rows.
"""

from __future__ import annotations

import random
from typing import Iterable, Sequence


ANGLE_METAL_SOURCE = (
    "https://chromium.googlesource.com/angle/angle/+/"
    "662226a3243caa9963ae8778c81b84ce71b4d2f6/"
    "src/libANGLE/renderer/metal/DisplayMtl.mm"
)
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
        ("air13_m1",),
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
        ("air13_m1",),
        10,
        "https://support.apple.com/en-la/111883",
    ),
    (
        "mac_m2_8gpu",
        "Apple M2",
        "apple8",
        8,
        8,
        (8, 16, 24),
        ("air13_modern",),
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
        ("air13_modern",),
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
        ("pro14",),
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
        ("pro14", "pro16"),
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
        ("pro14", "pro16"),
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
        ("pro14", "pro16"),
        2,
        "https://support.apple.com/en-ca/111340",
    ),
    (
        "mac_m3_10gpu",
        "Apple M3",
        "apple9",
        8,
        10,
        (8, 16, 24),
        ("pro14",),
        7,
        "https://support.apple.com/en-us/117735",
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
        "mac_m4_pro_20gpu",
        "Apple M4 Pro",
        "apple9",
        14,
        20,
        (24, 48),
        ("pro14", "pro16"),
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
        ("pro14", "pro16"),
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
        ("pro14", "pro16"),
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
    return {
        "id": profile_id,
        "vendor": gpu_vendor,
        "driverVendor": driver_vendor,
        "architecture": family,
        "cpuArchitecture": "x86" if intel_cpu else "arm",
        "cpuBitness": "64",
        "macosPlatformVersion": (
            "26.0.0" if profile_id.startswith("mac_m5_") else "15.5.0"
        ),
        "tier": "integrated",
        "model": device_name,
        "deviceMarker": device_name,
        "cpuCores": cpu_cores,
        "gpuCores": gpu_cores,
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
            "architecture": family,
            "device": device_name,
            "description": f"{device_name} Metal adapter",
        },
    }


MAC_GPU_CANDIDATES: tuple[dict[str, object], ...] = tuple(
    build_mac_gpu_candidate(row) for row in MAC_GPU_ROWS
)


def get_mac_gpu_candidates(
    *,
    family: str | None = None,
    device_class: str | None = None,
) -> tuple[dict[str, object], ...]:
    family_key = str(family or "").strip().lower()
    device_key = str(device_class or "").strip().lower()
    output = []
    for candidate in MAC_GPU_CANDIDATES:
        if family_key and str(candidate["architecture"]).lower() != family_key:
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
    choices = tuple(candidates) if candidates is not None else MAC_GPU_CANDIDATES
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
