from __future__ import annotations

import random
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
for value in (ROOT, ROOT / "demo", ROOT / "demo" / "fp"):
    text = str(value)
    if text not in sys.path:
        sys.path.insert(0, text)

from android_device_profile_catalog import (  # noqa: E402
    get_android_device_profile_by_id,
    materialize_android_device_profile,
)
from android_media_capability_catalog import (  # noqa: E402
    AAC,
    AV1_8,
    FLAC,
    H264_HIGH,
    HEVC_MAIN,
    OPUS,
    VP9_8,
    build_android_media_capabilities,
)
from android_speech_synthesis_voice_catalog import (  # noqa: E402
    choose_android_speech_synthesis_voice_profile,
)


class AndroidSpeechSynthesisProfileTests(unittest.TestCase):
    def test_country_languages_drive_android_native_voice_rows(self) -> None:
        cases = (
            ("US", ("en-US", "es-US"), "en-US"),
            ("JP", ("ja-JP", "en-US"), "ja-JP"),
            ("DE", ("de-DE", "en-US"), "de-DE"),
            ("HK", ("zh-HK", "zh", "en-HK", "en"), "zh-HK"),
        )
        for country, languages, expected_primary in cases:
            with self.subTest(country=country):
                profile = choose_android_speech_synthesis_voice_profile(
                    random.Random(803431),
                    country,
                    languages,
                )
                voices = tuple(profile["voices"])
                self.assertGreaterEqual(len(voices), 1)
                self.assertEqual(voices[0]["lang"], expected_primary)
                self.assertTrue(voices[0]["default"])
                self.assertTrue(all(voice["localService"] for voice in voices))
                self.assertFalse(
                    any(
                        "Microsoft" in str(voice["name"])
                        or "Apple" in str(voice["name"])
                        for voice in voices
                    )
                )

    def test_fixed_seed_is_reproducible(self) -> None:
        first = choose_android_speech_synthesis_voice_profile(
            random.Random(42), "IN", ("hi-IN", "en-IN", "en-US")
        )
        second = choose_android_speech_synthesis_voice_profile(
            random.Random(42), "IN", ("hi-IN", "en-IN", "en-US")
        )
        self.assertEqual(first, second)


class AndroidMediaCapabilityTests(unittest.TestCase):
    def test_pixel4_webview_matches_captured_media_layers(self) -> None:
        pixel4 = materialize_android_device_profile(
            get_android_device_profile_by_id("android_pixel_4"),
            11,
        )
        media = build_android_media_capabilities(
            pixel4,
            150,
            webview=True,
        )
        for media_type in (AAC, OPUS, FLAC, H264_HIGH, HEVC_MAIN, VP9_8, AV1_8):
            self.assertIn(media_type, media["can_play_probably_types"])
            self.assertIn(media_type, media["decoding_supported_types"])
            self.assertIn(media_type, media["decoding_smooth_types"])
        self.assertIn(AAC, media["media_source_types"])
        self.assertIn(H264_HIGH, media["media_source_types"])
        self.assertIn(HEVC_MAIN, media["media_source_types"])
        self.assertIn(AAC, media["media_recorder_types"])
        self.assertNotIn(VP9_8, media["media_recorder_types"])
        self.assertNotIn(AV1_8, media["decoding_power_efficient_types"])
        self.assertEqual(media["encoding_supported_types"], ())
        self.assertEqual(media["encoding_smooth_types"], ())
        self.assertEqual(media["encoding_power_efficient_types"], ())

    def test_hardware_av1_tier_changes_only_efficiency(self) -> None:
        software = materialize_android_device_profile(
            get_android_device_profile_by_id("android_pixel_4"),
            11,
        )
        hardware = materialize_android_device_profile(
            get_android_device_profile_by_id("android_pixel_8"),
            14,
        )
        software_media = build_android_media_capabilities(software, 150)
        hardware_media = build_android_media_capabilities(hardware, 150)
        self.assertIn(AV1_8, software_media["decoding_supported_types"])
        self.assertNotIn(AV1_8, software_media["decoding_power_efficient_types"])
        self.assertIn(AV1_8, hardware_media["decoding_supported_types"])
        self.assertIn(AV1_8, hardware_media["decoding_power_efficient_types"])

    def test_restrict_own_audio_version_gate(self) -> None:
        device = materialize_android_device_profile(
            get_android_device_profile_by_id("android_pixel_4"),
            11,
        )
        self.assertNotIn(
            "restrictOwnAudio",
            build_android_media_capabilities(device, 140)["supported_constraints"],
        )
        self.assertIn(
            "restrictOwnAudio",
            build_android_media_capabilities(device, 141)["supported_constraints"],
        )


if __name__ == "__main__":
    unittest.main()
