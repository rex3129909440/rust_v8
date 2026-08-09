from __future__ import annotations

from collections import Counter
import random
import unittest

from demo.fp.mac_screen_profile_catalog import choose_mac_screen_profile_for_gpu
from demo.fp.mac_webgl_gpu_catalog import get_mac_gpu_candidates
from demo.get_random_fp import (
    DEFAULT_ANDROID_EDGE_USER_AGENT,
    DEFAULT_MAC_USER_AGENT,
    DEFAULT_WINDOWS_USER_AGENT,
    audit_random_fp,
    get_random_fp_details,
)


class RandomProfileConsistencyTests(unittest.TestCase):
    def test_windows_touch_points_and_screen_depth_are_seeded_device_profiles(self) -> None:
        samples = [
            get_random_fp_details("US", DEFAULT_WINDOWS_USER_AGENT, seed=seed)
            for seed in range(2_000)
        ]
        combinations = {
            (
                sample.profile.navigator.max_touch_points,
                sample.profile.screen.color_depth,
                sample.profile.screen.pixel_depth,
            )
            for sample in samples
        }

        self.assertEqual(
            {sample.profile.navigator.max_touch_points for sample in samples},
            {0, 5, 10},
        )
        self.assertEqual(
            {
                (sample.profile.screen.color_depth, sample.profile.screen.pixel_depth)
                for sample in samples
            },
            {(24, 24), (32, 32)},
        )
        self.assertIn((10, 32, 32), combinations)
        self.assertTrue(
            all(color_depth == pixel_depth for _, color_depth, pixel_depth in combinations)
        )

        first = get_random_fp_details("US", DEFAULT_WINDOWS_USER_AGENT, seed=913)
        repeated = get_random_fp_details("US", DEFAULT_WINDOWS_USER_AGENT, seed=913)
        self.assertEqual(
            (
                first.profile.navigator.max_touch_points,
                first.profile.screen.color_depth,
                first.profile.screen.pixel_depth,
                first.screen_profile_id,
            ),
            (
                repeated.profile.navigator.max_touch_points,
                repeated.profile.screen.color_depth,
                repeated.profile.screen.pixel_depth,
                repeated.screen_profile_id,
            ),
        )

        mac = get_random_fp_details("US", DEFAULT_MAC_USER_AGENT, seed=913).profile
        android = get_random_fp_details(
            "US", DEFAULT_ANDROID_EDGE_USER_AGENT, seed=913
        ).profile
        self.assertEqual(mac.navigator.max_touch_points, 0)
        self.assertIn((mac.screen.color_depth, mac.screen.pixel_depth), {(24, 24), (30, 30)})
        self.assertEqual(android.navigator.max_touch_points, 5)
        self.assertEqual((android.screen.color_depth, android.screen.pixel_depth), (24, 24))

    def test_network_activation_and_media_preferences_are_seeded_profiles(self) -> None:
        samples = [
            get_random_fp_details("US", DEFAULT_WINDOWS_USER_AGENT, seed=seed)
            for seed in range(500)
        ]
        networks = [sample.profile.navigator.network for sample in samples]
        self.assertEqual(min(network.rtt for network in networks), 0)
        self.assertEqual(max(network.rtt for network in networks), 600)
        self.assertTrue(all(network.rtt % 50 == 0 for network in networks))
        self.assertTrue(
            all(abs(network.downlink * 20 - round(network.downlink * 20)) < 1e-9 for network in networks)
        )
        self.assertEqual({network.save_data for network in networks}, {False, True})
        self.assertEqual(
            {sample.profile.navigator.user_activation_is_active for sample in samples},
            {False, True},
        )
        for sample in samples:
            self.assertEqual(
                sample.profile.media_preferences.reduced_data,
                sample.profile.navigator.network.save_data,
            )
            self.assertFalse(
                sample.profile.navigator.user_activation_is_active
                and not sample.profile.navigator.user_activation_has_been_active
            )

        android_postures = {
            get_random_fp_details(
                "US", DEFAULT_ANDROID_EDGE_USER_AGENT, seed=seed
            ).profile.hardware_devices.device_posture
            for seed in range(500)
        }
        self.assertEqual(android_postures, {"continuous", "folded"})

    def test_document_state_can_be_passed_to_standalone_evaluate_profile(self) -> None:
        profile = get_random_fp_details(
            "US",
            seed=7,
        ).profile
        self.assertEqual(profile.document.body_child_element_count, 2)
        self.assertEqual(profile.document.body_client_height, 0.0)
        self.assertIsInstance(profile.document.has_focus, bool)
        self.assertEqual(profile.document.visibility_state, "visible")
        self.assertFalse(profile.document.is_popup)

        overridden = get_random_fp_details(
            "US",
            seed=7,
            body_child_element_count=3,
            body_client_height=19,
            document_has_focus=False,
            document_visibility_state="hidden",
            is_popup=True,
        ).profile
        self.assertEqual(overridden.document.body_child_element_count, 3)
        self.assertEqual(overridden.document.body_client_height, 19.0)
        self.assertFalse(overridden.document.has_focus)
        self.assertEqual(overridden.document.visibility_state, "hidden")
        self.assertTrue(overridden.document.is_popup)

        with self.assertRaisesRegex(ValueError, "document_visibility_state"):
            get_random_fp_details(
                "US",
                seed=7,
                document_visibility_state="prerender",
            )

        focus_values = {
            get_random_fp_details("US", seed=seed).profile.document.has_focus
            for seed in range(100)
        }
        self.assertEqual(focus_values, {False, True})

        for explicit in (False, True):
            configured = get_random_fp_details(
                "US",
                seed=7,
                document_has_focus=explicit,
            ).profile
            self.assertIs(configured.document.has_focus, explicit)

    def test_seeded_cross_platform_samples_are_internally_consistent(self) -> None:
        requests = (
            DEFAULT_WINDOWS_USER_AGENT,
            DEFAULT_MAC_USER_AGENT,
            DEFAULT_ANDROID_EDGE_USER_AGENT,
        )
        platforms = Counter()
        for user_agent in requests:
            for seed in range(300):
                fingerprint = get_random_fp_details(
                    "US",
                    user_agent,
                    seed=seed,
                    include_external_mac_screen=True,
                )
                platforms[fingerprint.platform] += 1
                self.assertEqual(audit_random_fp(fingerprint), ())
        self.assertEqual(platforms, {"windows": 300, "macos": 300, "android": 300})

    def test_external_mac_display_does_not_change_host_form_factor(self) -> None:
        air = next(
            item for item in get_mac_gpu_candidates(verified_only=True)
            if item["id"] == "mac_m4_air_8gpu"
        )
        external = None
        for seed in range(100):
            selected = choose_mac_screen_profile_for_gpu(
                random.Random(seed),
                air,
                include_external=True,
            )
            if selected["deviceClass"] == "external":
                external = selected
                break
        self.assertIsNotNone(external)
        self.assertEqual(external["hostDeviceClass"], "air13_modern")
        self.assertTrue(external["hostPortable"])
        self.assertTrue(external["hostHasCamera"])

    def test_headless_mac_host_camera_depends_on_the_selected_display(self) -> None:
        ultra = next(
            item for item in get_mac_gpu_candidates(verified_only=True)
            if item["id"] == "mac_m1_ultra_20cpu_48gpu"
        )
        seen = {}
        for seed in range(100):
            selected = choose_mac_screen_profile_for_gpu(
                random.Random(seed),
                ultra,
                include_external=True,
            )
            seen[selected["id"]] = bool(selected["hostHasCamera"])
        self.assertTrue(seen["mac_studio_display_2560x1440_2x"])
        self.assertFalse(seen["mac_external_1920x1080_1x_captured"])


if __name__ == "__main__":
    unittest.main()
