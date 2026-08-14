from __future__ import annotations

import sys
import unittest
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(PROJECT_ROOT / "examples"))

from edge_profile import EdgeProfile, NavigatorProfile  # noqa: E402
from run_sandbox import EdgeSandbox  # noqa: E402


WINDOW_NAME_COUNTS = {
    140: 1196,
    141: 1200,
    142: 1202,
    143: 1204,
    144: 1208,
    145: 1213,
    146: 1219,
    147: 1230,
    148: 1231,
    149: 1232,
    150: 1232,
    151: 1236,
}


class BrowserVersionSurfaceTests(unittest.TestCase):
    def test_python_profile_selects_chromium_140_through_151_surfaces(self) -> None:
        javascript = r"""
(() => {
  const frame = document.createElement("iframe");
  document.body.appendChild(frame);
  const topNames = Object.getOwnPropertyNames(window)
    .filter((name) => !/^\d+$/.test(name));
  const frameNames = Object.getOwnPropertyNames(frame.contentWindow);
  const chromiumBrand = navigator.userAgentData.brands
    .find((brand) => brand.brand === "Chromium");
  return [topNames.length, frameNames.length, chromiumBrand.version].join("|");
})()
""".strip()

        for major, expected_count in WINDOW_NAME_COUNTS.items():
            with self.subTest(major=major):
                user_agent = (
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                    "AppleWebKit/537.36 (KHTML, like Gecko) "
                    f"Chrome/{major}.0.0.0 Safari/537.36 Edg/{major}.0.0.0"
                )
                profile = EdgeProfile(
                    navigator=NavigatorProfile(user_agent=user_agent)
                )
                with EdgeSandbox(profile=profile) as sandbox:
                    top_count, frame_count, brand_version = str(
                        sandbox.evaluate(javascript)
                    ).split("|")

                self.assertEqual(int(top_count), expected_count)
                self.assertEqual(int(frame_count), expected_count)
                self.assertEqual(brand_version, str(major))


if __name__ == "__main__":
    unittest.main()
