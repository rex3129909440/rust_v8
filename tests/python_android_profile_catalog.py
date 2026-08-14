from __future__ import annotations

import unittest

from demo.fp.android_device_profile_catalog import (
    ANDROID_DEVICE_PROFILES,
    ANDROID_DEVICE_OS_RANGE_BY_PROFILE_ID,
)
from demo.fp.android_font_profile_catalog import build_android_font_profile
from demo.fp.ua import ANDROID_CHROME_VERSION_MAP, ANDROID_EDGE_VERSION_MAP
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
        self.assertEqual(profile.navigator.platform, "Linux armv81")
        self.assertTrue(profile.navigator.user_agent_data.mobile)
        self.assertEqual(profile.navigator.user_agent_data.platform, "Android")
        self.assertEqual(profile.navigator.user_agent_data.architecture, "")
        self.assertEqual(profile.navigator.user_agent_data.bitness, "")
        self.assertRegex(
            profile.navigator.user_agent_data.platform_version,
            r"^\d+\.\d+\.\d+$",
        )
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
        self.assertFalse(profile.plugins.plugins)
        self.assertFalse(profile.speech.voices)
        self.assertNotEqual(result.screen_profile_id, "android_pixel_4")

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
        self.assertFalse(result.profile.webgpu.available)

    def test_webgpu_availability_follows_chromium_android_vendor_support(self) -> None:
        pixel_ua = (
            "Mozilla/5.0 (Linux; Android 15; Pixel 8) "
            "AppleWebKit/537.36 (KHTML, like Gecko) "
            "Chrome/150.0.0.0 Mobile Safari/537.36 EdgA/150.0.0.0"
        )
        pixel = get_random_fp_details("US", pixel_ua, seed=11)
        self.assertTrue(pixel.profile.webgpu.available)

        samsung = next(
            item for item in ANDROID_DEVICE_PROFILES
            if item["id"] == "android_galaxy_a55"
        )
        power_vr = next(
            item for item in ANDROID_DEVICE_PROFILES
            if item["id"] == "android_moto_g_power_2022"
        )
        self.assertFalse(samsung["webgpuSupported"])
        self.assertFalse(power_vr["webgpuSupported"])
        pixel_4 = next(
            item for item in ANDROID_DEVICE_PROFILES
            if item["id"] == "android_pixel_4"
        )
        self.assertFalse(pixel_4["webgpuSupported"])

    def test_frozen_mobile_ua_uses_real_mobile_release_versions(self) -> None:
        for major in range(140, 152):
            with self.subTest(major=major):
                ua = (
                    "Mozilla/5.0 (Linux; Android 10; K) "
                    "AppleWebKit/537.36 (KHTML, like Gecko) "
                    f"Chrome/{major}.0.0.0 Mobile Safari/537.36 "
                    f"EdgA/{major}.0.0.0"
                )
                result = get_random_fp_details("US", ua, seed=major)
                data = result.profile.navigator.user_agent_data
                versions = {
                    item.brand: item.full_version for item in data.brands
                }
                self.assertEqual(
                    versions["Chromium"],
                    ANDROID_CHROME_VERSION_MAP[str(major)],
                )
                self.assertEqual(
                    versions["Microsoft Edge"],
                    ANDROID_EDGE_VERSION_MAP[str(major)],
                )
                self.assertNotEqual(data.model, "K")
                self.assertGreaterEqual(
                    int(data.platform_version.split(".", 1)[0]),
                    10,
                )
                has_restrict_own_audio = (
                    "restrictOwnAudio"
                    in result.profile.media.supported_constraints
                )
                self.assertEqual(has_restrict_own_audio, major >= 141)

    def test_every_browser_supported_device_materializes_as_one_unit(self) -> None:
        tested = 0
        for device in ANDROID_DEVICE_PROFILES:
            profile_id = str(device["id"])
            lower, upper = ANDROID_DEVICE_OS_RANGE_BY_PROFILE_ID[profile_id]
            if upper < 10:
                continue
            android_version = max(10, lower)
            model = str(device["model"])
            ua = (
                f"Mozilla/5.0 (Linux; Android {android_version}; {model}) "
                "AppleWebKit/537.36 (KHTML, like Gecko) "
                "Chrome/150.0.0.0 Mobile Safari/537.36 "
                "EdgA/150.0.0.0"
            )
            with self.subTest(profile_id=profile_id):
                result = get_random_fp_details("US", ua, seed=17)
                profile = result.profile
                self.assertEqual(result.screen_profile_id, profile_id)
                self.assertEqual(profile.navigator.user_agent_data.model, model)
                self.assertEqual(
                    int(profile.navigator.user_agent_data.platform_version.split(".", 1)[0]),
                    android_version,
                )
                self.assertEqual(result.gpu_model, device["gpu"]["model"])
                self.assertTrue(result.font_profile_id.startswith(
                    f"android-{device['oem']}-{android_version}-"
                ))
                self.assertFalse(profile.plugins.plugins)
                self.assertFalse(profile.speech.voices)
                self.assertNotIn("Direct3D", profile.webgl.unmasked_renderer)
                tested += 1
        self.assertGreaterEqual(tested, 17)

    def test_gpu_families_do_not_share_the_pixel4_capability_row(self) -> None:
        signatures = set()
        for seed in range(600):
            result = get_random_fp_details(
                "US", DEFAULT_ANDROID_EDGE_USER_AGENT, seed=seed
            )
            webgl = result.profile.webgl
            signatures.add((
                webgl.max_texture_size,
                webgl.max_renderbuffer_size,
                webgl.max_viewport_width,
                webgl.webgl2_max_samples,
                webgl.max_anisotropy,
            ))
        self.assertGreaterEqual(len(signatures), 5)

    def test_android_fonts_follow_os_generation_and_oem(self) -> None:
        pixel_4 = build_android_font_profile("en-US", 11, "google")
        self.assertNotIn("Roboto Flex", pixel_4["families"])
        samsung = build_android_font_profile("ko-KR", 14, "samsung")
        self.assertIn("Roboto Flex", samsung["families"])
        self.assertIn("SamsungOne", samsung["families"])
        self.assertIn("Samsung Sans", samsung["families"])

    def test_pixel4_evidence_row_is_not_global_default(self) -> None:
        ua = (
            "Mozilla/5.0 (Linux; Android 11; Pixel 4) "
            "AppleWebKit/537.36 (KHTML, like Gecko) "
            "Chrome/151.0.0.0 Mobile Safari/537.36"
        )
        pixel = get_random_fp_details("US", ua, seed=1)
        self.assertEqual(pixel.physical_memory_gb, 6)
        self.assertEqual(pixel.device_memory_gb, 4.0)
        self.assertEqual(
            pixel.profile.memory.performance_js_heap_size_limit,
            1_530_000_000,
        )
        self.assertEqual(pixel.profile.webgl.max_texture_size, 4096)
        self.assertEqual(len(pixel.profile.webgl.webgl2_extensions), 21)
        self.assertIn("width:16px;height:16px", pixel.profile.css.input_checkbox_radio)
        self.assertIn("width:184.364px", pixel.profile.css.input_text)
        self.assertIn("border-width:1.81818px", pixel.profile.css.input_text)
        other_signatures = {
            (
                get_random_fp_details(
                    "US", DEFAULT_ANDROID_EDGE_USER_AGENT, seed=seed
                ).profile.webgl.max_texture_size,
                get_random_fp_details(
                    "US", DEFAULT_ANDROID_EDGE_USER_AGENT, seed=seed
                ).profile.webgl.max_renderbuffer_size,
            )
            for seed in range(40)
        }
        self.assertGreater(len(other_signatures), 1)


if __name__ == "__main__":
    unittest.main()
