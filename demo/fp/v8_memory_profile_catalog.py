"""V8 15.0 / Blink evidence for ``performance.memory`` profiles.

``jsHeapSizeLimit`` is a V8 heap-configuration result.  In contrast,
``totalJSHeapSize`` and ``usedJSHeapSize`` are runtime snapshots: V8 exposes
committed physical heap memory and the live object size at the instant Blink
samples the isolate.  They are therefore selected as complete observed
snapshot pairs and are never generated as independent random numbers.

Primary implementation references:
- https://chromium.googlesource.com/v8/v8/+/refs/heads/main/src/heap/heap.cc
- https://chromium.googlesource.com/v8/v8/+/refs/heads/main/src/api/api.cc
- https://chromium.googlesource.com/chromium/src/+/main/third_party/blink/renderer/core/timing/memory_info.cc
"""

from __future__ import annotations

from dataclasses import dataclass
import math
import random
import struct
from typing import Iterable, Sequence


MIB = 1024**2
GIB = 1024**3
V8_NORMAL_PAGE_SIZE = 1 << 18
V8_EVIDENCE_VERSION = "15.0.245.2"


@dataclass(frozen=True, slots=True)
class V8MemorySnapshot:
    """One indivisible, observed V8 heap snapshot."""

    id: str
    total_js_heap_size: int
    used_js_heap_size: int
    platforms: tuple[str, ...]
    weight: int
    evidence: str


# These pairs are preserved exactly.  The first is the user-provided Edge 148
# browser sample, the second is the complete Edge 150 M5 capture, and the final
# nine are direct HeapStatistics observations from this project's embedded V8
# 15.0.245.2 with independently created retained-object workloads.
V8_MEMORY_SNAPSHOTS: tuple[V8MemorySnapshot, ...] = (
    V8MemorySnapshot(
        id="edge148_windows_loaded_page_user_sample",
        total_js_heap_size=98_833_423,
        used_js_heap_size=62_981_207,
        platforms=("windows",),
        weight=24,
        evidence="user-test/success.json Edge/Chromium 148 browser sample",
    ),
    V8MemorySnapshot(
        id="edge150_macos_m5_loaded_page_capture",
        total_js_heap_size=189_287_527,
        used_js_heap_size=180_511_835,
        platforms=("macos",),
        weight=24,
        evidence="demo/full-edge-profile-2026-08-07T04-06-07.238Z.json",
    ),
    V8MemorySnapshot(
        id="v8_15_retained_0_objects",
        total_js_heap_size=8_388_608,
        used_js_heap_size=7_002_608,
        platforms=("windows", "macos", "android"),
        weight=12,
        evidence="embedded V8 15.0.245.2 retained-object workload probe",
    ),
    V8MemorySnapshot(
        id="v8_15_retained_1000_objects",
        total_js_heap_size=9_310_208,
        used_js_heap_size=6_582_720,
        platforms=("windows", "macos", "android"),
        weight=14,
        evidence="embedded V8 15.0.245.2 retained-object workload probe",
    ),
    V8MemorySnapshot(
        id="v8_15_retained_5000_objects",
        total_js_heap_size=9_834_496,
        used_js_heap_size=7_149_000,
        platforms=("windows", "macos", "android"),
        weight=16,
        evidence="embedded V8 15.0.245.2 retained-object workload probe",
    ),
    V8MemorySnapshot(
        id="v8_15_retained_10000_objects",
        total_js_heap_size=11_833_344,
        used_js_heap_size=7_721_512,
        platforms=("windows", "macos", "android"),
        weight=18,
        evidence="embedded V8 15.0.245.2 retained-object workload probe",
    ),
    V8MemorySnapshot(
        id="v8_15_retained_25000_objects",
        total_js_heap_size=12_533_760,
        used_js_heap_size=9_576_920,
        platforms=("windows", "macos", "android"),
        weight=18,
        evidence="embedded V8 15.0.245.2 retained-object workload probe",
    ),
    V8MemorySnapshot(
        id="v8_15_retained_50000_objects",
        total_js_heap_size=17_592_320,
        used_js_heap_size=11_866_480,
        platforms=("windows", "macos", "android"),
        weight=16,
        evidence="embedded V8 15.0.245.2 retained-object workload probe",
    ),
    V8MemorySnapshot(
        id="v8_15_retained_100000_objects",
        total_js_heap_size=23_248_896,
        used_js_heap_size=17_799_720,
        platforms=("windows", "macos", "android"),
        weight=14,
        evidence="embedded V8 15.0.245.2 retained-object workload probe",
    ),
    V8MemorySnapshot(
        id="v8_15_retained_250000_objects",
        total_js_heap_size=43_364_352,
        used_js_heap_size=32_828_400,
        platforms=("windows", "macos", "android"),
        weight=10,
        evidence="embedded V8 15.0.245.2 retained-object workload probe",
    ),
    V8MemorySnapshot(
        id="v8_15_retained_500000_objects",
        total_js_heap_size=74_006_528,
        used_js_heap_size=56_718_360,
        platforms=("windows", "macos", "android"),
        weight=6,
        evidence="embedded V8 15.0.245.2 retained-object workload probe",
    ),
)


def _round_up_to_power_of_two(value: int) -> int:
    if value <= 1:
        return 1
    return 1 << (value - 1).bit_length()


def _round_up(value: int, alignment: int) -> int:
    return (value + alignment - 1) // alignment * alignment


def v8_150_precise_heap_size_limit(
    physical_memory_gb: int | float,
    platform_name: str,
) -> int:
    """Return V8 15.0's default 64-bit heap reservation limit.

    This mirrors V8's ``OldGenerationSizeFromPhysicalMemory``,
    ``YoungGenerationSizeFromPhysicalMemory`` and ``MaxReserved`` path with
    the default Scavenger collector.  Desktop reaches the 4-GiB old-generation
    cap at 8 GiB of physical memory.  Android uses the documented 1:4 ratio,
    the smaller non-high-end young generation below 8 GiB, and reaches the
    same cap at 16 GiB.
    """

    physical_bytes = int(float(physical_memory_gb) * GIB)
    if physical_bytes <= 0:
        raise ValueError("physical_memory_gb must be positive")
    platform = str(platform_name).strip().lower()
    if platform not in {"windows", "macos", "android"}:
        raise ValueError(f"unsupported V8 memory platform {platform_name!r}")

    old_generation_ratio = 4 if platform == "android" else 2
    old_generation = physical_bytes // old_generation_ratio
    old_generation = min(max(old_generation, 256 * MIB), 4 * GIB)
    old_generation = _round_up(old_generation, V8_NORMAL_PAGE_SIZE)

    high_end_android = platform == "android" and physical_bytes // GIB >= 8
    if platform == "android" and not high_end_android:
        semi_space_ratio = 128
        maximum_semi_space = 8 * MIB
    else:
        semi_space_ratio = 32
        maximum_semi_space = 32 * MIB

    target_heap_size = physical_bytes // 4
    semi_space = target_heap_size // semi_space_ratio
    semi_space = min(max(semi_space, 2 * MIB), maximum_semi_space)
    semi_space = _round_up(semi_space, V8_NORMAL_PAGE_SIZE)
    # V8 still rounds the Scavenger maximum semi-space to a power of two.
    semi_space = _round_up_to_power_of_two(semi_space)
    young_generation = 3 * semi_space
    return old_generation + young_generation


def _float32(value: float) -> float:
    return struct.unpack("<f", struct.pack("<f", float(value)))[0]


def _build_blink_memory_buckets() -> tuple[int, ...]:
    """Build Blink's complete 100-value legacy memory quantization table."""

    number_of_buckets = 100
    next_bucket = _float32(10_000_000.0)
    largest_bucket = _float32(4_000_000_000.0)
    ratio = _float32(largest_bucket / next_bucket)
    scaling_factor = _float32(
        math.exp(_float32(math.log(ratio) / number_of_buckets))
    )
    next_power_of_ten = int(
        math.pow(10.0, math.floor(math.log10(next_bucket)) + 1) + 0.5
    )
    granularity = next_power_of_ten // 1000
    buckets: list[int] = []
    for _ in range(number_of_buckets):
        current = int(next_bucket)
        buckets.append(current - current % granularity)
        next_bucket = _float32(next_bucket * scaling_factor)
        if next_bucket >= next_power_of_ten:
            next_power_of_ten *= 10
            granularity *= 10
    return tuple(buckets)


BLINK_MEMORY_BUCKETS: tuple[int, ...] = _build_blink_memory_buckets()


def quantize_blink_memory_size(value: int) -> int:
    """Return the exact Blink bucket used when precise memory is unavailable."""

    size = max(0, int(value))
    for bucket in BLINK_MEMORY_BUCKETS:
        if size <= bucket:
            return bucket
    return BLINK_MEMORY_BUCKETS[-1]


def v8_150_heap_size_limit_catalog(
    platform_name: str,
    physical_memory_values_gb: Iterable[int | float],
    *,
    include_bucketized: bool = True,
) -> tuple[int, ...]:
    """Return every heap limit reachable by the supplied physical-RAM pool."""

    precise = {
        v8_150_precise_heap_size_limit(value, platform_name)
        for value in physical_memory_values_gb
    }
    if not include_bucketized:
        return tuple(sorted(precise))
    return tuple(sorted(precise | {quantize_blink_memory_size(v) for v in precise}))


def memory_snapshots_for_platform(platform_name: str) -> tuple[V8MemorySnapshot, ...]:
    platform = str(platform_name).strip().lower()
    return tuple(
        snapshot
        for snapshot in V8_MEMORY_SNAPSHOTS
        if platform in snapshot.platforms
    )


def choose_v8_memory_snapshot(
    rng: random.Random,
    platform_name: str,
    *,
    candidates: Sequence[V8MemorySnapshot] | None = None,
) -> V8MemorySnapshot:
    """Choose one complete evidence row without breaking its value relation."""

    choices = tuple(candidates or memory_snapshots_for_platform(platform_name))
    if not choices:
        raise ValueError(f"no V8 memory snapshots for {platform_name!r}")
    return rng.choices(
        choices,
        weights=tuple(max(1, snapshot.weight) for snapshot in choices),
        k=1,
    )[0]


def is_known_memory_snapshot(
    snapshot_id: str,
    platform_name: str,
    total_js_heap_size: int,
    used_js_heap_size: int,
) -> bool:
    return any(
        snapshot.id == snapshot_id
        and str(platform_name).lower() in snapshot.platforms
        and snapshot.total_js_heap_size == int(total_js_heap_size)
        and snapshot.used_js_heap_size == int(used_js_heap_size)
        for snapshot in V8_MEMORY_SNAPSHOTS
    )


__all__ = [
    "BLINK_MEMORY_BUCKETS",
    "V8_EVIDENCE_VERSION",
    "V8_MEMORY_SNAPSHOTS",
    "V8MemorySnapshot",
    "choose_v8_memory_snapshot",
    "is_known_memory_snapshot",
    "memory_snapshots_for_platform",
    "quantize_blink_memory_size",
    "v8_150_heap_size_limit_catalog",
    "v8_150_precise_heap_size_limit",
]
