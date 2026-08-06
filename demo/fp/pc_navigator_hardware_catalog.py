"""
PC navigator hardware catalog.

This module models browser-exposed Navigator hardware fields for PC
desktop/laptop fingerprints:

- navigator.hardwareConcurrency
- navigator.deviceMemory
- navigator.maxTouchPoints

Important browser constraints:

- hardwareConcurrency is the number of logical processors available to the
  user agent. Browsers may report a lower value than the real CPU thread count.
- deviceMemory is privacy-coarsened. Chromium-style values are powers of two
  and high-memory PCs commonly still report 8 instead of 16/32/64.
- maxTouchPoints is 0 for ordinary non-touch desktop/laptop PCs. Windows touch
  laptops and Surface-like devices commonly report 5 or 10.
"""

from __future__ import annotations

import random
from typing import Iterable


# id, hardwareConcurrency, deviceMemory, maxTouchPoints, physicalRamHintGb,
# weight, tags
PC_NAVIGATOR_HARDWARE_ROWS: tuple[tuple[str, int, int | float, int, int, int, tuple[str, ...]], ...] = (
    # Legacy / low-end Windows laptops and office desktops.
    ("pc_1c_1g_notouch_legacy", 1, 1, 0, 1, 3, ("legacy", "lowend", "notouch", "obsolete")),
    ("pc_2c_1g_notouch_legacy", 2, 1, 0, 1, 5, ("legacy", "lowend", "notouch", "obsolete")),
    ("pc_2c_2g_notouch_legacy", 2, 2, 0, 2, 14, ("legacy", "lowend", "notouch")),
    ("pc_2c_4g_notouch_legacy", 2, 4, 0, 4, 18, ("legacy", "lowend", "notouch")),
    ("pc_4c_2g_notouch_legacy", 4, 2, 0, 2, 12, ("legacy", "lowend", "notouch")),
    ("pc_4c_4g_notouch_lowend", 4, 4, 0, 4, 42, ("lowend", "office", "notouch")),
    ("pc_4c_8g_notouch_office", 4, 8, 0, 8, 78, ("office", "mainstream", "notouch")),
    ("pc_6c_4g_notouch_office", 6, 4, 0, 8, 20, ("office", "lowend", "notouch")),
    ("pc_6c_8g_notouch_office", 6, 8, 0, 8, 52, ("office", "mainstream", "notouch")),

    # Common mainstream consumer laptops/desktops.
    ("pc_8c_4g_notouch_lowmem", 8, 4, 0, 8, 18, ("mainstream", "lowmem", "notouch")),
    ("pc_8c_8g_notouch_mainstream", 8, 8, 0, 16, 92, ("mainstream", "notouch")),
    ("pc_10c_8g_notouch_mainstream", 10, 8, 0, 16, 52, ("mainstream", "notouch")),
    ("pc_12c_8g_notouch_mainstream", 12, 8, 0, 16, 64, ("mainstream", "notouch")),
    ("pc_14c_8g_notouch_mainstream", 14, 8, 0, 16, 42, ("mainstream", "notouch")),
    ("pc_16c_8g_notouch_performance", 16, 8, 0, 32, 48, ("performance", "notouch")),
    ("pc_18c_8g_notouch_mainstream", 18, 8, 0, 24, 22, ("mainstream", "performance", "notouch")),

    # Gaming and workstation PCs. deviceMemory remains 8 because browser output
    # is coarsened/capped even when physical RAM is much larger.
    ("pc_20c_8g_notouch_gaming", 20, 8, 0, 32, 34, ("gaming", "performance", "notouch")),
    ("pc_22c_8g_notouch_performance_laptop", 22, 8, 0, 32, 16, ("laptop", "performance", "notouch")),
    ("pc_24c_8g_notouch_gaming", 24, 8, 0, 32, 26, ("gaming", "performance", "notouch")),
    ("pc_28c_8g_notouch_performance", 28, 8, 0, 32, 14, ("performance", "notouch")),
    ("pc_32c_8g_notouch_workstation", 32, 8, 0, 64, 16, ("workstation", "notouch")),
    ("pc_36c_8g_notouch_workstation", 36, 8, 0, 64, 8, ("workstation", "notouch")),
    ("pc_38c_8g_notouch_workstation", 38, 8, 0, 128, 1, ("workstation", "notouch")),
    ("pc_40c_8g_notouch_workstation", 40, 8, 0, 64, 8, ("workstation", "notouch")),
    ("pc_44c_8g_notouch_workstation", 44, 8, 0, 128, 3, ("workstation", "notouch")),
    ("pc_48c_8g_notouch_workstation", 48, 8, 0, 128, 6, ("workstation", "notouch")),
    ("pc_52c_8g_notouch_workstation", 52, 8, 0, 128, 2, ("workstation", "notouch")),
    ("pc_56c_8g_notouch_workstation", 56, 8, 0, 128, 4, ("workstation", "notouch")),
    ("pc_60c_8g_notouch_workstation", 60, 8, 0, 256, 1, ("workstation", "notouch")),
    ("pc_64c_8g_notouch_workstation", 64, 8, 0, 128, 4, ("workstation", "notouch")),
    ("pc_72c_8g_notouch_workstation", 72, 8, 0, 256, 2, ("workstation", "notouch")),
    ("pc_76c_8g_notouch_workstation", 76, 8, 0, 256, 1, ("workstation", "notouch")),
    ("pc_88c_8g_notouch_workstation", 88, 8, 0, 256, 1, ("workstation", "notouch")),
    ("pc_96c_8g_notouch_workstation", 96, 8, 0, 192, 2, ("workstation", "notouch")),
    ("pc_112c_8g_notouch_workstation", 112, 8, 0, 512, 1, ("workstation", "notouch")),
    ("pc_120c_8g_notouch_workstation", 120, 8, 0, 512, 1, ("workstation", "notouch")),
    ("pc_128c_8g_notouch_workstation", 128, 8, 0, 256, 1, ("workstation", "notouch")),
    ("pc_172c_8g_notouch_workstation", 172, 8, 0, 1024, 1, ("workstation", "notouch")),
    ("pc_192c_8g_notouch_workstation", 192, 8, 0, 1024, 1, ("workstation", "notouch")),

    # Windows touch laptops / 2-in-1 / Surface-like PCs.
    ("pc_4c_8g_touch5_convertible", 4, 8, 5, 8, 16, ("touch", "convertible", "surface")),
    ("pc_6c_8g_touch5_convertible", 6, 8, 5, 8, 12, ("touch", "convertible")),
    ("pc_8c_4g_touch5_convertible", 8, 4, 5, 8, 8, ("touch", "convertible", "lowmem")),
    ("pc_8c_8g_touch5_convertible", 8, 8, 5, 16, 20, ("touch", "convertible")),
    ("pc_8c_8g_touch10_surface", 8, 8, 10, 16, 22, ("touch", "surface")),
    ("pc_10c_8g_touch10_surface", 10, 8, 10, 16, 18, ("touch", "surface")),
    ("pc_12c_8g_touch10_surface", 12, 8, 10, 32, 12, ("touch", "surface", "performance")),
    ("pc_14c_8g_touch10_surface", 14, 8, 10, 32, 8, ("touch", "surface", "performance")),
    ("pc_16c_8g_touch10_surface", 16, 8, 10, 32, 6, ("touch", "surface", "performance")),
    ("pc_22c_8g_touch10_performance", 22, 8, 10, 32, 4, ("touch", "convertible", "performance")),

    # Windows on ARM / Copilot+ PCs. These rows are selected only through the
    # explicit arm64 selector so x64 Windows UA profiles do not get paired with
    # Snapdragon CPU/GPU hardware.
    ("pc_arm64_6c_8g_touch10_snapdragon_x2_plus", 6, 8, 10, 16, 2, ("arm64", "touch", "copilot_pc")),
    ("pc_arm64_8c_8g_touch10_snapdragon_x", 8, 8, 10, 16, 4, ("arm64", "touch", "copilot_pc")),
    ("pc_arm64_10c_8g_touch10_snapdragon_x_plus", 10, 8, 10, 16, 5, ("arm64", "touch", "copilot_pc")),
    ("pc_arm64_12c_8g_touch10_snapdragon_x_elite", 12, 8, 10, 32, 6, ("arm64", "touch", "copilot_pc", "performance")),
    ("pc_arm64_18c_8g_touch10_snapdragon_x2_elite", 18, 8, 10, 48, 2, ("arm64", "touch", "copilot_pc", "performance")),

    # macOS desktop/laptop Chrome-like navigator values for PC-class devices.
    ("pc_mac_8c_8g_notouch", 8, 8, 0, 8, 24, ("mac", "notouch")),
    ("pc_mac_10c_8g_notouch", 10, 8, 0, 16, 20, ("mac", "notouch")),
    ("pc_mac_12c_8g_notouch", 12, 8, 0, 16, 12, ("mac", "notouch")),
    ("pc_mac_14c_8g_notouch", 14, 8, 0, 24, 8, ("mac", "notouch")),
    ("pc_mac_16c_8g_notouch", 16, 8, 0, 32, 8, ("mac", "performance", "notouch")),
    ("pc_mac_20c_8g_notouch", 20, 8, 0, 48, 4, ("mac", "performance", "notouch")),
    ("pc_mac_24c_8g_notouch", 24, 8, 0, 64, 2, ("mac", "performance", "notouch")),

    # Virtualized or restricted browser contexts. Keep separate so callers can
    # exclude them unless explicitly modeling VM/automation environments.
    ("pc_vm_1c_2g_notouch", 1, 2, 0, 2, 5, ("virtual", "notouch")),
    ("pc_vm_2c_4g_notouch", 2, 4, 0, 4, 10, ("virtual", "notouch")),
    ("pc_vm_4c_8g_notouch", 4, 8, 0, 8, 12, ("virtual", "notouch")),
    ("pc_vm_8c_8g_notouch", 8, 8, 0, 16, 6, ("virtual", "notouch")),
    ("pc_vdi_2c_8g_notouch", 2, 8, 0, 8, 4, ("virtual", "vdi", "notouch")),
)


def build_pc_navigator_hardware_profile(
    row: tuple[str, int, int | float, int, int, int, tuple[str, ...]]
) -> dict[str, object]:
    profile_id, concurrency, device_memory, max_touch_points, physical_ram, weight, tags = row
    return {
        "id": profile_id,
        "deviceClass": "pc",
        "hardwareConcurrency": concurrency,
        "deviceMemory": device_memory,
        "maxTouchPoints": max_touch_points,
        "physicalRamHintGb": physical_ram,
        "weight": weight,
        "tags": tags,
        "navigator": {
            "hardwareConcurrency": concurrency,
            "deviceMemory": device_memory,
            "maxTouchPoints": max_touch_points,
        },
    }


PC_NAVIGATOR_HARDWARE_PROFILES: tuple[dict[str, object], ...] = tuple(
    build_pc_navigator_hardware_profile(row) for row in PC_NAVIGATOR_HARDWARE_ROWS
)


def get_pc_navigator_hardware_profiles(
    tag: str | None = None,
    include_virtual: bool = False,
) -> tuple[dict[str, object], ...]:
    tag_key = str(tag or "").strip().lower()
    output = []
    for profile in PC_NAVIGATOR_HARDWARE_PROFILES:
        tags = tuple(str(item).lower() for item in profile.get("tags", ()))
        if not include_virtual and "virtual" in tags:
            continue
        # "windows" is a selector for the default Windows PC generator. Most
        # real Windows rows are tagged by form factor instead of repeating a
        # windows tag on every row, so include every non-mac/non-arm64 PC row
        # here. ARM64 rows are selected only when the UA profile is ARM64.
        if tag_key == "windows":
            if "mac" in tags or "arm64" in tags or "obsolete" in tags:
                continue
            output.append(profile)
            continue
        if tag_key and tag_key not in tags:
            continue
        output.append(profile)
    return tuple(output)


def choose_pc_navigator_hardware_profile(
    rng: random.Random,
    tag: str | None = None,
    include_virtual: bool = False,
) -> dict[str, object]:
    profiles = get_pc_navigator_hardware_profiles(tag=tag, include_virtual=include_virtual)
    if not profiles:
        profiles = get_pc_navigator_hardware_profiles(include_virtual=include_virtual)
    if not profiles:
        raise ValueError("no PC navigator hardware profiles available")
    weights = [int(profile.get("weight", 1)) for profile in profiles]
    return rng.choices(profiles, weights=weights, k=1)[0]


def choose_pc_navigator_hardware_profile_for_gpu_tier(
    rng: random.Random,
    gpu_tier: str | None,
    tag: str | None = "windows",
) -> dict[str, object]:
    profiles = get_pc_navigator_hardware_profiles(tag=tag, include_virtual=False)
    tier = str(gpu_tier or "").strip().lower()
    if not profiles:
        return choose_pc_navigator_hardware_profile(rng, tag=tag, include_virtual=False)

    def profile_tags(profile: dict[str, object]) -> tuple[str, ...]:
        return tuple(str(item).lower() for item in profile.get("tags", ()))

    def concurrency(profile: dict[str, object]) -> int:
        return int(profile.get("hardwareConcurrency", 0) or 0)

    if tier == "workstation":
        candidates = tuple(
            profile
            for profile in profiles
            if "workstation" in profile_tags(profile)
            or ("performance" in profile_tags(profile) and concurrency(profile) >= 16)
        )
    elif tier in {"enthusiast", "high"}:
        candidates = tuple(
            profile
            for profile in profiles
            if (
                "gaming" in profile_tags(profile)
                or "performance" in profile_tags(profile)
                or "workstation" in profile_tags(profile)
            )
            and concurrency(profile) >= 12
        )
    elif tier in {"laptop", "mainstream"}:
        candidates = tuple(
            profile
            for profile in profiles
            if "workstation" not in profile_tags(profile)
            and "legacy" not in profile_tags(profile)
            and concurrency(profile) >= 6
        )
    elif tier in {"integrated", "entry", "legacy"}:
        candidates = tuple(
            profile
            for profile in profiles
            if "workstation" not in profile_tags(profile)
            and concurrency(profile) <= 24
        )
    else:
        candidates = profiles

    if not candidates:
        candidates = profiles
    weights = [int(profile.get("weight", 1)) for profile in candidates]
    return rng.choices(candidates, weights=weights, k=1)[0]


def choose_pc_navigator_hardware_profile_for_gpu(
    rng: random.Random,
    gpu_profile: dict[str, object] | None,
    tag: str | None = "windows",
) -> dict[str, object]:
    profiles = get_pc_navigator_hardware_profiles(tag=tag, include_virtual=False)
    if not profiles:
        return choose_pc_navigator_hardware_profile(rng, tag=tag, include_virtual=False)

    tier = str((gpu_profile or {}).get("tier", "") or "").strip().lower()
    architecture = str((gpu_profile or {}).get("architecture", "") or "").strip().lower()
    model = str((gpu_profile or {}).get("model", "") or "").strip().lower()

    def profile_tags(profile: dict[str, object]) -> tuple[str, ...]:
        return tuple(str(item).lower() for item in profile.get("tags", ()))

    def concurrency(profile: dict[str, object]) -> int:
        return int(profile.get("hardwareConcurrency", 0) or 0)

    modern_integrated_arches = {
        "xe2",
        "xe-lpg",
        "xe-lp",
        "gen-12",
        "gen-12lp",
        "gen-11",
        "rdna3.5",
        "rdna3",
        "rdna2",
        "adreno-x2",
        "adreno-x1",
        "adreno",
    }
    modern_integrated_model_needles = (
        "arc(tm)",
        "iris(r) xe",
        "iris xe",
        "890m",
        "880m",
        "860m",
        "840m",
        "780m",
        "760m",
        "740m",
        "680m",
        "660m",
        "610m",
    )

    if tier == "integrated" and (
        architecture in modern_integrated_arches
        or any(needle in model for needle in modern_integrated_model_needles)
    ):
        candidates = tuple(
            profile
            for profile in profiles
            if "legacy" not in profile_tags(profile)
            and "lowend" not in profile_tags(profile)
            and "workstation" not in profile_tags(profile)
            and concurrency(profile) >= 6
            and concurrency(profile) <= 24
        )
        if candidates:
            weights = [int(profile.get("weight", 1)) for profile in candidates]
            return rng.choices(candidates, weights=weights, k=1)[0]

    return choose_pc_navigator_hardware_profile_for_gpu_tier(
        rng,
        tier,
        tag=tag,
    )


def build_pc_navigator_hardware_patch(profile: dict[str, object]) -> dict[str, object]:
    return {
        "navigator": profile.get("navigator", {}),
        "navigatorHardwareProfileId": profile.get("id", ""),
        "physicalRamHintGb": profile.get("physicalRamHintGb", None),
    }


def iter_pc_hardware_concurrency_values(tag: str | None = None) -> Iterable[int]:
    seen: set[int] = set()
    for profile in get_pc_navigator_hardware_profiles(tag=tag, include_virtual=True):
        value = int(profile["hardwareConcurrency"])
        if value not in seen:
            seen.add(value)
            yield value


def iter_pc_device_memory_values(tag: str | None = None) -> Iterable[int | float]:
    seen: set[int | float] = set()
    for profile in get_pc_navigator_hardware_profiles(tag=tag, include_virtual=True):
        value = profile["deviceMemory"]
        if value not in seen:
            seen.add(value)  # type: ignore[arg-type]
            yield value  # type: ignore[misc]


def iter_pc_max_touch_points_values(tag: str | None = None) -> Iterable[int]:
    seen: set[int] = set()
    for profile in get_pc_navigator_hardware_profiles(tag=tag, include_virtual=True):
        value = int(profile["maxTouchPoints"])
        if value not in seen:
            seen.add(value)
            yield value
