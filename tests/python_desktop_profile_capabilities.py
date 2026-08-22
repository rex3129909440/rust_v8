from __future__ import annotations

import unittest

from demo.fp.desktop_media_capability_catalog import (
    chromium_desktop_supported_constraints,
)
from demo.get_random_fp import (
    DEFAULT_MAC_USER_AGENT,
    DEFAULT_WINDOWS_USER_AGENT,
    audit_random_fp,
    get_random_fp_details,
)


WINDOWS_CHROME_140_USER_AGENT = (
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
    "AppleWebKit/537.36 (KHTML, like Gecko) "
    "Chrome/140.0.0.0 Safari/537.36"
)


class DesktopProfileCapabilityTests(unittest.TestCase):
    def test_windows_150_d3d12_limits_follow_dawn_tiers(self) -> None:
        profiles = [
            get_random_fp_details(
                "US",
                DEFAULT_WINDOWS_USER_AGENT,
                seed=seed,
            )
            for seed in range(500)
        ]
        available = [item for item in profiles if item.profile.webgpu.available]
        self.assertTrue(available)
        for fingerprint in available:
            webgpu = fingerprint.profile.webgpu
            self.assertEqual(webgpu.max_texture_dimension_1d, 16_384)
            self.assertEqual(webgpu.max_texture_dimension_2d, 16_384)
            self.assertEqual(webgpu.max_texture_array_layers, 2_048)
            self.assertEqual(webgpu.max_dynamic_uniform_buffers_per_pipeline_layout, 10)
            self.assertEqual(webgpu.max_dynamic_storage_buffers_per_pipeline_layout, 8)
            self.assertEqual(webgpu.max_sampled_textures_per_shader_stage, 48)
            self.assertEqual(webgpu.max_storage_buffers_per_shader_stage, 16)
            self.assertEqual(webgpu.max_storage_textures_per_shader_stage, 8)
            self.assertEqual(webgpu.max_buffer_size, 2_147_483_648)
            self.assertEqual(webgpu.max_vertex_attributes, 30)
            self.assertEqual(webgpu.max_inter_stage_shader_variables, 28)
            self.assertEqual(webgpu.max_color_attachment_bytes_per_sample, 128)
            self.assertEqual(webgpu.max_compute_workgroup_storage_size, 32_768)
            self.assertEqual(webgpu.max_compute_invocations_per_workgroup, 1_024)
            self.assertEqual(webgpu.max_immediate_size, 64)
            self.assertIn("depth32float-stencil8", webgpu.features)
            self.assertIn("texture-compression-bc", webgpu.features)
            self.assertIn("core-features-and-limits", webgpu.features)

    def test_windows_webgl_caps_are_linked_to_adapter_vendor(self) -> None:
        seen_vendors = set()
        for seed in range(2_000):
            fingerprint = get_random_fp_details(
                "US",
                DEFAULT_WINDOWS_USER_AGENT,
                seed=seed,
            )
            webgl = fingerprint.profile.webgl
            vendor = fingerprint.profile.webgpu.vendor
            seen_vendors.add(vendor)
            self.assertEqual(webgl.webgl2_max_samples, 8)
            self.assertIn("EXT_texture_compression_rgtc", webgl.webgl1_extensions)
            self.assertIn("EXT_render_snorm", webgl.webgl2_extensions)
            if vendor == "nvidia":
                self.assertEqual(webgl.max_vertex_uniform_vectors, 4_095)
                self.assertEqual(webgl.webgl2_max_vertex_uniform_components, 16_380)
            else:
                self.assertEqual(webgl.max_vertex_uniform_vectors, 4_096)
                self.assertEqual(webgl.webgl2_max_vertex_uniform_components, 16_384)
        self.assertTrue({"nvidia", "amd", "intel"}.issubset(seen_vendors))

    def test_desktop_supported_constraints_are_complete_and_versioned(self) -> None:
        modern = get_random_fp_details(
            "US", DEFAULT_WINDOWS_USER_AGENT, seed=9
        ).profile.media.supported_constraints
        old = get_random_fp_details(
            "US", WINDOWS_CHROME_140_USER_AGENT, seed=9
        ).profile.media.supported_constraints
        mac = get_random_fp_details(
            "US", DEFAULT_MAC_USER_AGENT, seed=9
        ).profile.media.supported_constraints

        self.assertEqual(modern, chromium_desktop_supported_constraints(150))
        self.assertEqual(mac, modern)
        self.assertEqual(old, chromium_desktop_supported_constraints(140))
        self.assertIn("voiceIsolation", modern)
        self.assertIn("suppressLocalAudioPlayback", modern)
        self.assertIn("restrictOwnAudio", modern)
        self.assertNotIn("restrictOwnAudio", old)

    def test_windows_audio_rows_keep_rate_and_period_linked(self) -> None:
        rows = {
            (
                item.profile.audio.sample_rate,
                item.profile.audio.base_latency,
                item.profile.audio.output_latency,
            )
            for seed in range(500)
            for item in (
                get_random_fp_details(
                    "US", DEFAULT_WINDOWS_USER_AGENT, seed=seed
                ),
            )
        }
        self.assertEqual(
            rows,
            {
                (48_000.0, 480 / 48_000.0, 0.0),
                (44_100.0, 448 / 44_100.0, 0.0),
            },
        )

    def test_desktop_accessibility_preferences_form_valid_os_states(self) -> None:
        windows = [
            get_random_fp_details(
                "US", DEFAULT_WINDOWS_USER_AGENT, seed=seed
            )
            for seed in range(2_000)
        ]
        self.assertEqual(
            {item.profile.media_preferences.forced_colors for item in windows},
            {False, True},
        )
        self.assertTrue(
            any(item.profile.media_preferences.pointer == "coarse" for item in windows)
        )
        self.assertTrue(
            any(
                item.profile.media_preferences.contrast == "custom"
                for item in windows
            )
        )
        for fingerprint in windows:
            preferences = fingerprint.profile.media_preferences
            if preferences.forced_colors:
                self.assertIn(preferences.contrast, {"more", "less", "custom"})
                self.assertFalse(preferences.inverted_colors)
            else:
                self.assertEqual(preferences.contrast, "no-preference")
            if preferences.pointer == "coarse":
                self.assertGreater(fingerprint.profile.navigator.max_touch_points, 0)
                self.assertEqual(preferences.any_pointer, "coarse")
                self.assertEqual(preferences.hover, "none")
                self.assertEqual(preferences.any_hover, "none")
            self.assertEqual(audit_random_fp(fingerprint), ())

        mac = [
            get_random_fp_details("US", DEFAULT_MAC_USER_AGENT, seed=seed)
            for seed in range(500)
        ]
        self.assertEqual(
            {item.profile.media_preferences.contrast for item in mac},
            {"no-preference", "more"},
        )
        self.assertTrue(
            all(not item.profile.media_preferences.forced_colors for item in mac)
        )
        self.assertTrue(
            all(item.profile.media_preferences.pointer == "fine" for item in mac)
        )


if __name__ == "__main__":
    unittest.main()
