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
- deviceMemory is privacy-coarsened and must remain separate from physical
  RAM. The version-aware composer calculates the exposed bucket after this
  catalog selects a physical-memory row.
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

    # Desktop memory variants.  A GPU family is compatible with many CPU/RAM
    # combinations; keeping these as independent evidence-backed rows creates
    # a large valid space without inventing arbitrary cross-products.
    ("pc_desktop_6c_16g_notouch_mainstream", 6, 8, 0, 16, 34, ("desktop", "mainstream", "office", "notouch")),
    ("pc_desktop_6c_12g_notouch_office", 6, 8, 0, 12, 8, ("desktop", "office", "mainstream", "notouch")),
    ("pc_desktop_8c_16g_notouch_mainstream", 8, 8, 0, 16, 82, ("desktop", "mainstream", "gaming", "notouch")),
    ("pc_desktop_8c_12g_notouch_mainstream", 8, 8, 0, 12, 12, ("desktop", "mainstream", "notouch")),
    ("pc_desktop_8c_32g_notouch_mainstream", 8, 8, 0, 32, 70, ("desktop", "mainstream", "gaming", "notouch")),
    ("pc_desktop_10c_16g_notouch_mainstream", 10, 8, 0, 16, 40, ("desktop", "mainstream", "gaming", "notouch")),
    ("pc_desktop_10c_32g_notouch_performance", 10, 8, 0, 32, 34, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_12c_16g_notouch_mainstream", 12, 8, 0, 16, 50, ("desktop", "mainstream", "gaming", "notouch")),
    ("pc_desktop_12c_24g_notouch_mainstream", 12, 8, 0, 24, 8, ("desktop", "mainstream", "gaming", "notouch")),
    ("pc_desktop_12c_32g_notouch_performance", 12, 8, 0, 32, 52, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_12c_64g_notouch_performance", 12, 8, 0, 64, 8, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_14c_16g_notouch_mainstream", 14, 8, 0, 16, 35, ("desktop", "mainstream", "gaming", "notouch")),
    ("pc_desktop_14c_32g_notouch_performance", 14, 8, 0, 32, 38, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_14c_64g_notouch_performance", 14, 8, 0, 64, 6, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_16c_16g_notouch_mainstream", 16, 8, 0, 16, 30, ("desktop", "mainstream", "gaming", "notouch")),
    ("pc_desktop_16c_24g_notouch_mainstream", 16, 8, 0, 24, 6, ("desktop", "mainstream", "gaming", "notouch")),
    ("pc_desktop_16c_32g_notouch_performance", 16, 8, 0, 32, 45, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_16c_64g_notouch_performance", 16, 8, 0, 64, 10, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_20c_16g_notouch_performance", 20, 8, 0, 16, 16, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_20c_32g_notouch_performance", 20, 8, 0, 32, 30, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_20c_48g_notouch_performance", 20, 8, 0, 48, 4, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_20c_64g_notouch_performance", 20, 8, 0, 64, 8, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_24c_16g_notouch_performance", 24, 8, 0, 16, 12, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_24c_32g_notouch_performance", 24, 8, 0, 32, 28, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_24c_48g_notouch_performance", 24, 8, 0, 48, 4, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_24c_64g_notouch_performance", 24, 8, 0, 64, 8, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_24c_96g_notouch_performance", 24, 8, 0, 96, 2, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_28c_32g_notouch_performance", 28, 8, 0, 32, 8, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_28c_64g_notouch_performance", 28, 8, 0, 64, 4, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_32c_32g_notouch_performance", 32, 8, 0, 32, 8, ("desktop", "performance", "gaming", "notouch")),
    ("pc_desktop_32c_64g_notouch_performance", 32, 8, 0, 64, 5, ("desktop", "performance", "gaming", "notouch")),

    # Portable Windows systems.  These rows deliberately keep physical RAM
    # separate from navigator.deviceMemory; the composer applies Chromium's
    # observable bucket after selecting a complete device combination.
    ("pc_laptop_4c_4g_notouch_legacy", 4, 4, 0, 4, 4, ("laptop", "legacy", "lowend", "notouch")),
    ("pc_laptop_4c_8g_notouch_lowend", 4, 8, 0, 8, 14, ("laptop", "lowend", "notouch")),
    ("pc_laptop_6c_8g_notouch_office", 6, 8, 0, 8, 18, ("laptop", "office", "notouch")),
    ("pc_laptop_8c_8g_notouch_mainstream", 8, 8, 0, 8, 28, ("laptop", "mainstream", "notouch")),
    ("pc_laptop_8c_12g_notouch_mainstream", 8, 8, 0, 12, 10, ("laptop", "mainstream", "notouch")),
    ("pc_laptop_8c_16g_notouch_mainstream", 8, 8, 0, 16, 42, ("laptop", "mainstream", "notouch")),
    ("pc_laptop_10c_8g_notouch_mainstream", 10, 8, 0, 8, 18, ("laptop", "mainstream", "notouch")),
    ("pc_laptop_10c_16g_notouch_mainstream", 10, 8, 0, 16, 36, ("laptop", "mainstream", "notouch")),
    ("pc_laptop_12c_16g_notouch_mainstream", 12, 8, 0, 16, 42, ("laptop", "mainstream", "notouch")),
    ("pc_laptop_12c_24g_notouch_mainstream", 12, 8, 0, 24, 10, ("laptop", "mainstream", "notouch")),
    ("pc_laptop_12c_32g_notouch_performance", 12, 8, 0, 32, 20, ("laptop", "performance", "notouch")),
    ("pc_laptop_14c_16g_notouch_mainstream", 14, 8, 0, 16, 38, ("laptop", "mainstream", "notouch")),
    ("pc_laptop_14c_32g_notouch_performance", 14, 8, 0, 32, 22, ("laptop", "performance", "notouch")),
    ("pc_laptop_16c_16g_notouch_mainstream", 16, 8, 0, 16, 30, ("laptop", "mainstream", "notouch")),
    ("pc_laptop_16c_24g_notouch_mainstream", 16, 8, 0, 24, 10, ("laptop", "mainstream", "notouch")),
    ("pc_laptop_16c_32g_notouch_performance", 16, 8, 0, 32, 30, ("laptop", "performance", "notouch")),
    ("pc_laptop_16c_64g_notouch_performance", 16, 8, 0, 64, 5, ("laptop", "performance", "notouch")),
    ("pc_laptop_16c_96g_notouch_performance", 16, 8, 0, 96, 1, ("laptop", "performance", "notouch")),
    ("pc_laptop_20c_16g_notouch_performance", 20, 8, 0, 16, 15, ("laptop", "performance", "notouch")),
    ("pc_laptop_20c_32g_notouch_performance", 20, 8, 0, 32, 26, ("laptop", "performance", "notouch")),
    ("pc_laptop_20c_64g_notouch_performance", 20, 8, 0, 64, 5, ("laptop", "performance", "notouch")),
    ("pc_laptop_22c_16g_notouch_performance", 22, 8, 0, 16, 12, ("laptop", "performance", "notouch")),
    ("pc_laptop_22c_32g_notouch_performance", 22, 8, 0, 32, 30, ("laptop", "performance", "notouch")),
    ("pc_laptop_22c_64g_notouch_performance", 22, 8, 0, 64, 8, ("laptop", "performance", "notouch")),
    ("pc_laptop_24c_16g_notouch_performance", 24, 8, 0, 16, 10, ("laptop", "performance", "notouch")),
    ("pc_laptop_24c_32g_notouch_performance", 24, 8, 0, 32, 24, ("laptop", "performance", "notouch")),
    ("pc_laptop_24c_48g_notouch_performance", 24, 8, 0, 48, 6, ("laptop", "performance", "notouch")),
    ("pc_laptop_24c_64g_notouch_performance", 24, 8, 0, 64, 8, ("laptop", "performance", "notouch")),
    ("pc_laptop_28c_32g_notouch_performance", 28, 8, 0, 32, 10, ("laptop", "performance", "notouch")),
    ("pc_laptop_28c_64g_notouch_performance", 28, 8, 0, 64, 4, ("laptop", "performance", "notouch")),
    ("pc_laptop_32c_32g_notouch_performance", 32, 8, 0, 32, 10, ("laptop", "performance", "notouch")),
    ("pc_laptop_32c_64g_notouch_performance", 32, 8, 0, 64, 5, ("laptop", "performance", "notouch")),

    # Gaming and workstation PCs. The placeholder deviceMemory value is
    # replaced by the version-aware bucket in get_random_fp.py.
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
    ("pc_workstation_4c_16g_notouch", 4, 8, 0, 16, 3, ("workstation", "entry_workstation", "notouch")),
    ("pc_workstation_8c_16g_notouch", 8, 8, 0, 16, 8, ("workstation", "entry_workstation", "notouch")),
    ("pc_workstation_8c_32g_notouch", 8, 8, 0, 32, 8, ("workstation", "entry_workstation", "notouch")),
    ("pc_workstation_12c_16g_notouch", 12, 8, 0, 16, 6, ("workstation", "entry_workstation", "notouch")),
    ("pc_workstation_12c_32g_notouch", 12, 8, 0, 32, 10, ("workstation", "notouch")),
    ("pc_workstation_16c_32g_notouch", 16, 8, 0, 32, 12, ("workstation", "notouch")),
    ("pc_workstation_16c_64g_notouch", 16, 8, 0, 64, 8, ("workstation", "notouch")),
    ("pc_workstation_24c_32g_notouch", 24, 8, 0, 32, 7, ("workstation", "notouch")),
    ("pc_workstation_24c_64g_notouch", 24, 8, 0, 64, 8, ("workstation", "notouch")),
    ("pc_workstation_24c_96g_notouch", 24, 8, 0, 96, 4, ("workstation", "notouch")),
    ("pc_workstation_32c_96g_notouch", 32, 8, 0, 96, 3, ("workstation", "notouch")),

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


# Relative weights follow the broad shape of Steam's Windows survey. Rare
# workstation and mixed-memory states stay in the catalog with low weights.
_LOGICAL_PROCESSOR_WEIGHTS = {
    1: 1, 2: 3, 4: 47, 6: 99, 8: 100, 10: 27, 12: 18, 14: 17,
    16: 20, 18: 2, 20: 8, 22: 1, 24: 10, 28: 1, 32: 3,
}
_PHYSICAL_MEMORY_WEIGHTS = {
    1: 1, 2: 1, 4: 4, 8: 19, 12: 4, 16: 100, 24: 5,
    32: 90, 48: 3, 64: 10, 96: 2, 128: 2, 192: 1, 256: 1,
    512: 1, 1024: 1,
}


def get_pc_navigator_hardware_profile_weight(profile: dict[str, object]) -> int:
    concurrency = int(profile.get("hardwareConcurrency", 0) or 0)
    physical_ram = int(profile.get("physicalRamHintGb", 0) or 0)
    base = int(profile.get("weight", 1) or 1)
    cpu = _LOGICAL_PROCESSOR_WEIGHTS.get(concurrency, 1)
    memory = _PHYSICAL_MEMORY_WEIGHTS.get(physical_ram, 1)
    return max(1, base * cpu * memory // 100)


_HARDWARE_POOL_WEIGHT_CACHE: dict[
    int,
    tuple[tuple[dict[str, object], ...], tuple[float, ...]],
] = {}


def get_pc_navigator_hardware_pool_weights(
    profiles: tuple[dict[str, object], ...],
) -> tuple[float, ...]:
    """Normalize catalog rows to the target CPU/RAM bucket distribution.

    Several form-factor rows can represent the same CPU/RAM bucket. Without
    normalization, merely adding another real row would make that bucket more
    likely. The bucket receives one survey-derived weight, then its rows split
    that weight according to their local catalog weights.
    """

    cache_key = id(profiles)
    cached = _HARDWARE_POOL_WEIGHT_CACHE.get(cache_key)
    if cached is not None and cached[0] is profiles:
        return cached[1]

    pair_base_totals: dict[tuple[int, int], float] = {}
    cpu_memory_buckets: dict[int, set[int]] = {}
    for profile in profiles:
        cpu = int(profile.get("hardwareConcurrency", 0) or 0)
        memory = int(profile.get("physicalRamHintGb", 0) or 0)
        pair = (cpu, memory)
        pair_base_totals[pair] = pair_base_totals.get(pair, 0.0) + max(
            1.0,
            float(profile.get("weight", 1) or 1),
        )
        cpu_memory_buckets.setdefault(cpu, set()).add(memory)

    cpu_memory_totals = {
        cpu: sum(_PHYSICAL_MEMORY_WEIGHTS.get(memory, 1) for memory in memories)
        for cpu, memories in cpu_memory_buckets.items()
    }
    weights = []
    for profile in profiles:
        cpu = int(profile.get("hardwareConcurrency", 0) or 0)
        memory = int(profile.get("physicalRamHintGb", 0) or 0)
        base = max(1.0, float(profile.get("weight", 1) or 1))
        cpu_weight = float(_LOGICAL_PROCESSOR_WEIGHTS.get(cpu, 1))
        memory_weight = float(_PHYSICAL_MEMORY_WEIGHTS.get(memory, 1))
        pair_share = base / pair_base_totals[(cpu, memory)]
        memory_share = memory_weight / max(1.0, cpu_memory_totals[cpu])
        weights.append(cpu_weight * memory_share * pair_share)
    result = tuple(weights)
    _HARDWARE_POOL_WEIGHT_CACHE[cache_key] = (profiles, result)
    return result


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
    weights = get_pc_navigator_hardware_pool_weights(profiles)
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
    weights = get_pc_navigator_hardware_pool_weights(candidates)
    return rng.choices(candidates, weights=weights, k=1)[0]


def choose_pc_navigator_hardware_profile_for_gpu(
    rng: random.Random,
    gpu_profile: dict[str, object] | None,
    tag: str | None = "windows",
) -> dict[str, object]:
    profiles = get_pc_navigator_hardware_profiles(tag=tag, include_virtual=False)
    if not profiles:
        return choose_pc_navigator_hardware_profile(rng, tag=tag, include_virtual=False)

    candidates = get_compatible_pc_navigator_hardware_profiles_for_gpu(
        gpu_profile,
        tag=tag,
    )
    if not candidates:
        return choose_pc_navigator_hardware_profile_for_gpu_tier(
            rng,
            str((gpu_profile or {}).get("tier", "") or ""),
            tag=tag,
        )
    weights = get_pc_navigator_hardware_pool_weights(candidates)
    return rng.choices(candidates, weights=weights, k=1)[0]


def _compute_compatible_pc_navigator_hardware_profiles_for_gpu(
    gpu_profile: dict[str, object] | None,
    tag: str | None = "windows",
) -> tuple[dict[str, object], ...]:
    """Return coherent hardware rows while preserving broad catalog coverage."""

    profiles = get_pc_navigator_hardware_profiles(tag=tag, include_virtual=False)
    if not profiles:
        return ()

    tier = str((gpu_profile or {}).get("tier", "") or "").strip().lower()
    architecture = str((gpu_profile or {}).get("architecture", "") or "").strip().lower()
    model = str((gpu_profile or {}).get("model", "") or "").strip().lower()

    def profile_tags(profile: dict[str, object]) -> tuple[str, ...]:
        return tuple(str(item).lower() for item in profile.get("tags", ()))

    def concurrency(profile: dict[str, object]) -> int:
        return int(profile.get("hardwareConcurrency", 0) or 0)

    def physical_ram(profile: dict[str, object]) -> int:
        return int(profile.get("physicalRamHintGb", 0) or 0)

    def has_any(profile: dict[str, object], values: set[str]) -> bool:
        tags = profile_tags(profile)
        return any(value in tags for value in values)

    portable_gpu = tier == "laptop" or any(
        needle in model
        for needle in (
            "laptop gpu",
            "max-q",
            "geforce mx",
        )
    )
    recent_arches = {
        "blackwell", "ada", "ampere", "turing", "rdna4", "rdna3.5",
        "rdna3", "rdna2", "xe2-battlemage", "xe2", "xe-lpg", "xe-hpg",
        "xe-lp", "gen-12", "gen-12lp", "gen-11", "adreno-x2", "adreno-x1",
    }
    legacy_arches = {
        "gen-6", "gen-7", "gen-7.5", "gen-8", "kepler", "gcn1",
    }

    if str(tag or "").lower() == "arm64":
        return tuple(
            profile
            for profile in profiles
            if has_any(profile, {"arm64", "copilot_pc"})
        )

    if tier == "virtual":
        return ()

    if portable_gpu:
        demanding_mobile = any(
            needle in model
            for needle in (
                "5090", "5080", "5070", "4090", "4080", "4070",
                "3080", "3070", "2080", "quadro rtx", "rtx a",
            )
        )
        minimum_cpu = 12 if demanding_mobile and architecture in recent_arches else 4
        minimum_ram = 16 if demanding_mobile and architecture in recent_arches else 4
        return tuple(
            profile
            for profile in profiles
            if has_any(profile, {"laptop", "touch", "convertible", "surface"})
            and not has_any(profile, {"arm64", "workstation", "obsolete"})
            and minimum_cpu <= concurrency(profile) <= 32
            and minimum_ram <= physical_ram(profile) <= 96
        )

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
        return tuple(
            profile
            for profile in profiles
            if not has_any(profile, {"legacy", "lowend", "workstation", "obsolete", "arm64"})
            and 6 <= concurrency(profile) <= 32
            and 8 <= physical_ram(profile) <= 64
        )

    if tier in {"legacy", "integrated"} or architecture in legacy_arches:
        return tuple(
            profile
            for profile in profiles
            if not has_any(profile, {"workstation", "arm64"})
            and has_any(profile, {"legacy", "lowend", "office", "laptop"})
            and concurrency(profile) <= 16
            and physical_ram(profile) <= 32
        )

    if tier == "workstation":
        recent_workstation = architecture in recent_arches
        return tuple(
            profile
            for profile in profiles
            if "workstation" in profile_tags(profile)
            and concurrency(profile) >= (16 if recent_workstation else 4)
            and physical_ram(profile) >= (32 if recent_workstation else 16)
        )

    if tier in {"enthusiast", "high"}:
        return tuple(
            profile
            for profile in profiles
            if has_any(profile, {"gaming", "performance"})
            and not has_any(profile, {"laptop", "touch", "convertible", "surface", "arm64", "workstation"})
            and 12 <= concurrency(profile) <= 64
            and physical_ram(profile) >= 16
        )

    if tier in {"mainstream", "entry"}:
        return tuple(
            profile
            for profile in profiles
            if not has_any(profile, {"laptop", "touch", "convertible", "surface", "arm64", "workstation", "obsolete"})
            and 4 <= concurrency(profile) <= 32
            and 4 <= physical_ram(profile) <= 64
        )

    return tuple(
        profile for profile in profiles if "arm64" not in profile_tags(profile)
    )


_GPU_HARDWARE_COMPATIBILITY_CACHE: dict[
    tuple[str, str, str, str, str],
    tuple[dict[str, object], ...],
] = {}


def get_compatible_pc_navigator_hardware_profiles_for_gpu(
    gpu_profile: dict[str, object] | None,
    tag: str | None = "windows",
) -> tuple[dict[str, object], ...]:
    """Return a cached coherent hardware pool for one GPU family."""

    gpu = gpu_profile or {}
    cache_key = (
        str(gpu.get("baseProfileId", gpu.get("id", ""))),
        str(gpu.get("tier", "")),
        str(gpu.get("architecture", "")),
        str(gpu.get("model", "")),
        str(tag or "").lower(),
    )
    cached = _GPU_HARDWARE_COMPATIBILITY_CACHE.get(cache_key)
    if cached is None:
        cached = _compute_compatible_pc_navigator_hardware_profiles_for_gpu(
            gpu_profile,
            tag=tag,
        )
        _GPU_HARDWARE_COMPATIBILITY_CACHE[cache_key] = cached
    return cached


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
