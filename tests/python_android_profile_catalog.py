from __future__ import annotations

import unittest

from demo.fp.android_device_profile_catalog import ANDROID_DEVICE_PROFILES
from demo.get_random_fp import (
    DEFAULT_ANDROID_EDGE_USER_AGENT,
    get_random_fp_details,
)


class AndroidProfileCatalogTests(unittest.TestCase):
    def test_catalog_is_device_linked_and_not_desktop_mixed(self) -> None:
        self.assertGreaterEqual(len(ANDROID_DEVICE_PROFILES), 18)
        gpu_models = set()
        screen_shapes = set()
        for device in ANDROID_DEVICE_PROFILES:
            gpu = device["gpu"]
            self.assertEqual(device["deviceClass"], "android-phone")
            self.assertIn("OpenGL ES", gpu["webgl"]["unmaskedRenderer"])
            self.assertNotIn("Direct3D", gpu["webgl"]["unmaskedRenderer"])
            self.assertGreater(int(device["hardwareConcurrency"]), 0)
            self.assertTrue(device["physicalMemoryChoicesGb"])
            gpu_models.add(str(gpu["model"]))
            screen_shapes.add(
                (
                    int(device["screen"]["width"]),
                    int(device["screen"]["height"]),
                    float(device["window"]["devicePixelRatio"]),
                )
            )
        self.assertGreaterEqual(len(gpu_models), 14)
        self.assertGreaterEqual(len(screen_shapes), 12)

    def test_generic_android_edge_ua_selects_one_complete_device(self) -> None:
        result = get_random_fp_details(
            "US",
            DEFAULT_ANDROID_EDGE_USER_AGENT,
            seed=42,
        )
        profile = result.profile
        headers = dict(result.request_headers)
        self.assertEqual(result.platform, "android")
        self.assertEqual(profile.navigator.platform, "Linux armv8l")
        self.assertTrue(profile.navigator.user_agent_data.mobile)
        self.assertEqual(profile.navigator.user_agent_data.platform, "Android")
        self.assertNotEqual(profile.navigator.user_agent_data.model, "K")
        self.assertEqual(headers["sec-ch-ua-mobile"], "?1")
        self.assertEqual(
            headers["sec-ch-ua-model"],
            f'"{profile.navigator.user_agent_data.model}"',
        )
        self.assertIn("OpenGL ES", profile.webgl.unmasked_renderer)
        self.assertNotIn("Direct3D", profile.webgl.unmasked_renderer)
        self.assertEqual(profile.window.inner_width, 0.0)
        self.assertEqual(profile.window.inner_height, 0.0)

    def test_specific_ua_model_pins_the_device_family(self) -> None:
        user_agent = (
            "Mozilla/5.0 (Linux; Android 14; SM-A556B) "
            "AppleWebKit/537.36 (KHTML, like Gecko) "
            "Chrome/150.0.0.0 Mobile Safari/537.36 EdgA/150.0.0.0"
        )
        result = get_random_fp_details("DE", user_agent, seed=7)
        self.assertEqual(result.gpu_model, "Xclipse 530")
        self.assertEqual(result.profile.screen.width, 360)
        self.assertEqual(result.profile.screen.height, 800)
        self.assertEqual(result.profile.screen.device_pixel_ratio, 2.25)
        self.assertEqual(result.profile.navigator.user_agent_data.model, "SM-A556B")


if __name__ == "__main__":
    unittest.main()
