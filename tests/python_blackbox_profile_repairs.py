from __future__ import annotations

import unittest

from demo.get_random_fp import get_random_fp_details


class BlackBoxProfileRepairTests(unittest.TestCase):
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
        observations = {
            (
                get_random_fp_details("US", seed=seed).profile.navigator.network.rtt,
                get_random_fp_details("US", seed=seed).profile.navigator.network.downlink,
            )
            for seed in range(24)
        }
        self.assertGreaterEqual(len(observations), 2)
        self.assertTrue(observations <= {(100, 1.7), (100, 10.0), (50, 10.0)})

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
