from __future__ import annotations

import unittest

from demo.get_random_fp import get_random_fp_details
from examples.run_complete_iframe_hook import build_runtime_options


class BlackBoxProfileRepairTests(unittest.TestCase):
    def test_empty_parser_body_does_not_create_a_placeholder_resource(self) -> None:
        empty = build_runtime_options()
        populated = build_runtime_options(parser_script_body="void 0")

        self.assertNotIn('<script src="xxx"></script>', empty.page.html)
        self.assertFalse(any(entry.url.endswith("/xxx") for entry in empty.network_replay))
        self.assertIn('<script src="xxx"></script>', populated.page.html)
        self.assertTrue(any(entry.url.endswith("/xxx") for entry in populated.network_replay))

    def test_resource_load_is_seeded_same_origin_and_version_linked(self) -> None:
        first = get_random_fp_details("US", seed=101)
        repeated = get_random_fp_details("US", seed=101)
        second = get_random_fp_details("US", seed=102)
        page_url = "https://page.example.test/a/b/fp?x-kpsdk-v=j-1.2.594"

        first_url = first.resource_load.script_url(page_url, "j-1.2.594")
        self.assertEqual(
            first_url,
            repeated.resource_load.script_url(page_url, "j-1.2.594"),
        )
        self.assertNotEqual(
            first_url,
            second.resource_load.script_url(page_url, "j-1.2.594"),
        )
        self.assertTrue(first_url.startswith("https://page.example.test/a/b/ips.js?"))
        self.assertIn("x-kpsdk-v=j-1.2.594", first_url)
        self.assertIn("KP_UIDz=", first_url)
        self.assertIn("x-kpsdk-im=", first_url)

    def test_us_windows_profile_contains_captured_keyboard_layout(self) -> None:
        profile = get_random_fp_details("US", seed=1).profile
        layout = {
            entry.code: entry.value
            for entry in profile.hardware_devices.keyboard_layout
        }
        self.assertEqual(len(layout), 48)
        self.assertEqual(layout["KeyA"], "a")
        self.assertEqual(layout["Digit1"], "1")
        self.assertEqual(layout["IntlBackslash"], "\\")

    def test_network_observations_vary_without_changing_platform(self) -> None:
        observations = set()
        effective_types = set()
        for seed in range(64):
            network = get_random_fp_details(
                "US", seed=seed
            ).profile.navigator.network
            observations.add((network.rtt, network.downlink))
            effective_types.add(network.effective_type)
            self.assertGreaterEqual(network.rtt, 0)
            self.assertLessEqual(network.rtt, 600)
            self.assertEqual(network.rtt % 50, 0)
            self.assertGreaterEqual(network.downlink, 0.05)
            self.assertLessEqual(network.downlink, 10.0)
            self.assertAlmostEqual(network.downlink * 20, round(network.downlink * 20))
            expected = (
                "3g"
                if network.rtt >= 270 or network.downlink <= 0.7
                else "4g"
            )
            self.assertEqual(network.effective_type, expected)
        self.assertGreaterEqual(len(observations), 40)
        self.assertEqual(effective_types, {"3g", "4g"})

    def test_windows_media_capabilities_are_codec_specific(self) -> None:
        profile = get_random_fp_details("US", seed=1).profile
        recorder = profile.media.media_recorder_types
        source = profile.media.media_source_types
        self.assertIn("video/webm;codecs=vp8*", recorder)
        self.assertNotIn("video/webm;codecs=daala*", recorder)
        self.assertIn("video/mp4;codecs=hvc1.*", source)
        self.assertNotIn("video/webm;codecs=h264*", source)


if __name__ == "__main__":
    unittest.main()
