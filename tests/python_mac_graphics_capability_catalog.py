from __future__ import annotations

import sys
import random
import unittest
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(PROJECT_ROOT))
sys.path.insert(0, str(PROJECT_ROOT / "demo" / "fp"))
sys.path.insert(0, str(PROJECT_ROOT / "examples"))

from mac_edge_profile import mac_edge_150_profile  # noqa: E402
from mac_graphics_capability_catalog import (  # noqa: E402
    APPLE_SILICON_COMPRESSED_TEXTURE_FORMATS,
    APPLE_SILICON_WEBGPU_FEATURES,
    build_mac_graphics_capabilities,
)
from mac_font_profile_catalog import build_mac_font_profile  # noqa: E402
from mac_screen_profile_catalog import (  # noqa: E402
    MAC_SCREEN_PROFILES,
    choose_mac_screen_profile_for_gpu,
)
from mac_webgl_gpu_catalog import (  # noqa: E402
    MAC_GPU_CANDIDATES,
    get_mac_gpu_candidates,
)


class MacGraphicsCapabilityCatalogTests(unittest.TestCase):
    def test_default_pool_contains_only_fully_sourced_apple_silicon(self) -> None:
        candidates = get_mac_gpu_candidates()
        self.assertTrue(candidates)
        self.assertTrue(
            all(candidate["graphicsVerified"] for candidate in candidates)
        )
        self.assertEqual(
            {candidate["vendor"] for candidate in candidates},
            {"apple"},
        )
        self.assertEqual(
            {candidate["architecture"] for candidate in candidates},
            {"apple7", "apple8", "apple9", "apple10"},
        )

    def test_random_pool_covers_every_published_m1_through_m5_tier(self) -> None:
        candidates = get_mac_gpu_candidates()
        tiers_by_generation = {
            generation: {
                str(candidate["chipTier"])
                for candidate in candidates
                if candidate["chipGeneration"] == generation
            }
            for generation in ("M1", "M2", "M3", "M4", "M5")
        }
        self.assertEqual(tiers_by_generation["M1"], {"Base", "Pro", "Max", "Ultra"})
        self.assertEqual(tiers_by_generation["M2"], {"Base", "Pro", "Max", "Ultra"})
        self.assertEqual(tiers_by_generation["M3"], {"Base", "Pro", "Max", "Ultra"})
        self.assertEqual(tiers_by_generation["M4"], {"Base", "Pro", "Max"})
        self.assertEqual(tiers_by_generation["M5"], {"Base", "Pro", "Max"})

    def test_published_cpu_gpu_and_memory_configurations_are_linked(self) -> None:
        candidates = {candidate["id"]: candidate for candidate in get_mac_gpu_candidates()}
        expected = {
            "mac_m1_ultra_20cpu_64gpu": (20, 64, (64, 128)),
            "mac_m2_ultra_24cpu_76gpu": (24, 76, (64, 128, 192)),
            "mac_m3_pro_12cpu_18gpu": (12, 18, (18, 36)),
            "mac_m3_ultra_32cpu_80gpu": (32, 80, (96, 256)),
            "mac_m4_pro_16gpu": (12, 16, (24, 48)),
            "mac_m4_max_40gpu": (16, 40, (48, 64, 128)),
            "mac_m5_pro_15cpu_16gpu": (15, 16, (24, 48)),
            "mac_m5_max_18cpu_40gpu": (18, 40, (48, 64, 128)),
        }
        for profile_id, (cpu, gpu, memory) in expected.items():
            candidate = candidates[profile_id]
            self.assertEqual(candidate["cpuCores"], cpu, profile_id)
            self.assertEqual(candidate["gpuCores"], gpu, profile_id)
            self.assertEqual(candidate["memoryChoicesGb"], memory, profile_id)

    def test_every_hardware_candidate_has_a_compatible_screen(self) -> None:
        available_classes = {profile["deviceClass"] for profile in MAC_SCREEN_PROFILES}
        for candidate in get_mac_gpu_candidates():
            classes = set(candidate["screenClasses"])
            self.assertTrue(classes <= available_classes, candidate["id"])
            selected = choose_mac_screen_profile_for_gpu(
                random.Random(150),
                candidate,
                include_external=False,
            )
            self.assertIn(selected["deviceClass"], classes, candidate["id"])

    def test_m5_captured_tahoe_font_inventory_is_shared_across_generations(self) -> None:
        expected = build_mac_font_profile("en-US", "26.5.2")
        self.assertEqual(expected["sourceKind"], "real-device-capture")
        for generation in ("M1", "M2", "M3", "M4", "M5"):
            candidate = get_mac_gpu_candidates(generation=generation)[0]
            profile = build_mac_font_profile(
                "en-US", str(candidate["macosPlatformVersion"])
            )
            self.assertEqual(profile["families"], expected["families"], generation)
            self.assertEqual(profile["localFonts"], expected["localFonts"], generation)
            self.assertEqual(profile["metrics"], expected["metrics"], generation)

    def test_legacy_intel_amd_inventory_is_not_randomly_selected(self) -> None:
        default_ids = {candidate["id"] for candidate in get_mac_gpu_candidates()}
        inventory = get_mac_gpu_candidates(verified_only=False)
        legacy = [candidate for candidate in inventory if candidate["vendor"] != "apple"]
        self.assertTrue(legacy)
        self.assertTrue(all(not candidate["graphicsVerified"] for candidate in legacy))
        self.assertTrue(all(candidate["id"] not in default_ids for candidate in legacy))

        with self.assertRaisesRegex(ValueError, "no complete public"):
            build_mac_graphics_capabilities(legacy[0])

    def test_webgl_msaa_tracks_the_published_apple_family(self) -> None:
        for candidate in get_mac_gpu_candidates():
            webgl, _ = build_mac_graphics_capabilities(candidate)
            expected_samples = (
                8 if candidate["architecture"] == "apple10" else 4
            )
            self.assertEqual(
                webgl["webgl2_max_samples"],
                expected_samples,
                candidate["id"],
            )
            self.assertEqual(webgl["max_viewport_width"], 16_384)
            self.assertEqual(webgl["max_vertex_uniform_vectors"], 1_024)
            self.assertEqual(webgl["webgl2_max_uniform_block_size"], 16_384)
            self.assertEqual(
                webgl["compressed_texture_formats"],
                APPLE_SILICON_COMPRESSED_TEXTURE_FORMATS,
            )

    def test_webgpu_uses_chromium_tiered_mac2_limits(self) -> None:
        for candidate in get_mac_gpu_candidates():
            _, webgpu = build_mac_graphics_capabilities(candidate)
            self.assertEqual(webgpu["architecture"], "metal-3")
            self.assertEqual(webgpu["features"], APPLE_SILICON_WEBGPU_FEATURES)
            self.assertEqual(webgpu["max_sampled_textures_per_shader_stage"], 48)
            self.assertEqual(webgpu["max_storage_buffers_per_shader_stage"], 10)
            self.assertEqual(webgpu["max_compute_invocations_per_workgroup"], 1_024)
            self.assertEqual(webgpu["max_buffer_size"], 4_294_967_292)

    def test_fixed_m5_pro_preset_matches_shared_capability_record(self) -> None:
        candidate = next(
            item
            for item in MAC_GPU_CANDIDATES
            if item["id"] == "mac_m5_pro_15cpu_16gpu"
        )
        webgl, webgpu = build_mac_graphics_capabilities(candidate)
        profile = mac_edge_150_profile()

        for field_name, expected in webgl.items():
            self.assertEqual(
                getattr(profile.webgl, field_name),
                expected,
                field_name,
            )
        for field_name, expected in webgpu.items():
            self.assertEqual(
                getattr(profile.webgpu, field_name),
                expected,
                field_name,
            )


if __name__ == "__main__":
    unittest.main()
