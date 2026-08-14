from __future__ import annotations

import unittest

from demo.fp.v8_memory_profile_catalog import (
    BLINK_MEMORY_BUCKETS,
    is_known_memory_snapshot,
    quantize_blink_memory_size,
    v8_150_heap_size_limit_catalog,
    v8_150_precise_heap_size_limit,
)
from demo.fp.android_device_profile_catalog import ANDROID_DEVICE_PROFILES
from demo.fp.pc_navigator_hardware_catalog import PC_NAVIGATOR_HARDWARE_PROFILES
from demo.get_random_fp import (
    DEFAULT_ANDROID_EDGE_USER_AGENT,
    DEFAULT_MAC_USER_AGENT,
    DEFAULT_WINDOWS_USER_AGENT,
    get_random_fp_details,
)


class V8MemoryProfileCatalogTests(unittest.TestCase):
    def test_blink_quantization_reproduces_the_complete_official_table(self) -> None:
        self.assertEqual(len(BLINK_MEMORY_BUCKETS), 100)
        self.assertEqual(BLINK_MEMORY_BUCKETS[0], 10_000_000)
        self.assertEqual(BLINK_MEMORY_BUCKETS[-1], 3_760_000_000)
        self.assertEqual(quantize_blink_memory_size(389_472_983), 410_000_000)
        self.assertEqual(quantize_blink_memory_size(38_947_298), 39_600_000)
        self.assertEqual(quantize_blink_memory_size(28_947_298), 29_400_000)
        self.assertEqual(quantize_blink_memory_size(18_947_298), 19_300_000)
        self.assertEqual(quantize_blink_memory_size(13_947_298), 14_300_000)
        self.assertEqual(quantize_blink_memory_size(0), 10_000_000)

    def test_desktop_v8_150_heap_limit_catalog_covers_every_ram_threshold(self) -> None:
        expected = {
            1: 562_036_736,
            2: 1_124_073_472,
            3: 1_711_276_032,
            4: 2_248_146_944,
            6: 3_321_888_768,
            8: 4_395_630_592,
            16: 4_395_630_592,
            1024: 4_395_630_592,
        }
        for platform in ("windows", "macos"):
            self.assertEqual(
                {
                    memory: v8_150_precise_heap_size_limit(memory, platform)
                    for memory in expected
                },
                expected,
            )
        self.assertEqual(
            v8_150_heap_size_limit_catalog(
                "windows",
                expected,
                include_bucketized=False,
            ),
            tuple(sorted(set(expected.values()))),
        )
        windows_memory_pool = {
            int(profile["physicalRamHintGb"])
            for profile in PC_NAVIGATOR_HARDWARE_PROFILES
        }
        self.assertEqual(
            set(
                v8_150_heap_size_limit_catalog(
                    "windows",
                    windows_memory_pool,
                    include_bucketized=False,
                )
            ),
            set(expected.values()),
        )

    def test_android_v8_150_uses_its_distinct_memory_ratios(self) -> None:
        expected = {
            2: 549_453_824,
            3: 830_472_192,
            4: 1_098_907_648,
            6: 1_635_778_560,
            8: 2_248_146_944,
            12: 3_321_888_768,
            16: 4_395_630_592,
        }
        self.assertEqual(
            {
                memory: v8_150_precise_heap_size_limit(memory, "android")
                for memory in expected
            },
            expected,
        )
        android_memory_pool = {
            int(memory)
            for profile in ANDROID_DEVICE_PROFILES
            for memory in profile["physicalMemoryChoicesGb"]
        }
        self.assertEqual(
            set(
                v8_150_heap_size_limit_catalog(
                    "android",
                    android_memory_pool,
                    include_bucketized=False,
                )
            ),
            set(expected.values()),
        )

    def test_random_profiles_use_complete_observed_heap_snapshots(self) -> None:
        requests = (
            ("windows", DEFAULT_WINDOWS_USER_AGENT),
            ("macos", DEFAULT_MAC_USER_AGENT),
            ("android", DEFAULT_ANDROID_EDGE_USER_AGENT),
        )
        seen_snapshot_ids: dict[str, set[str]] = {
            platform: set() for platform, _ in requests
        }
        for platform, user_agent in requests:
            for seed in range(300):
                result = get_random_fp_details("US", user_agent, seed=seed)
                memory = result.profile.memory
                self.assertEqual(result.platform, platform)
                self.assertTrue(
                    is_known_memory_snapshot(
                        result.memory_snapshot_profile_id,
                        platform,
                        memory.performance_total_js_heap_size,
                        memory.performance_used_js_heap_size,
                    )
                )
                expected_limit = v8_150_precise_heap_size_limit(
                    result.physical_memory_gb,
                    platform,
                )
                if result.screen_profile_id == "android_pixel_4":
                    expected_limit = 1_530_000_000
                self.assertEqual(
                    memory.performance_js_heap_size_limit,
                    expected_limit,
                )
                self.assertEqual(
                    (
                        memory.console_js_heap_size_limit,
                        memory.console_total_js_heap_size,
                        memory.console_used_js_heap_size,
                    ),
                    (
                        memory.performance_js_heap_size_limit,
                        memory.performance_total_js_heap_size,
                        memory.performance_used_js_heap_size,
                    ),
                )
                self.assertLessEqual(
                    memory.performance_used_js_heap_size,
                    memory.performance_total_js_heap_size,
                )
                seen_snapshot_ids[platform].add(result.memory_snapshot_profile_id)

        self.assertEqual(len(seen_snapshot_ids["windows"]), 10)
        self.assertEqual(len(seen_snapshot_ids["macos"]), 10)
        self.assertEqual(len(seen_snapshot_ids["android"]), 9)


if __name__ == "__main__":
    unittest.main()
