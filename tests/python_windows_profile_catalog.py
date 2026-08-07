from __future__ import annotations

import unittest

from demo.get_random_fp import get_random_fp_details
from demo.fp.pc_navigator_hardware_catalog import (
    PC_NAVIGATOR_HARDWARE_PROFILES,
    get_compatible_pc_navigator_hardware_profiles_for_gpu,
)
from demo.fp.screen_profile_catalog import (
    PC_SCREEN_PROFILES,
    get_compatible_pc_screen_profiles_for_device,
)
from demo.fp.windows_webgl_gpu_catalog import WINDOWS_WEBGL_GPU_CANDIDATES
from examples.windows_edge_profile import windows_edge_150_profile


def _tags(profile: dict[str, object]) -> set[str]:
    return {str(item).lower() for item in profile.get("tags", ())}


class WindowsProfileCatalogTests(unittest.TestCase):
    def test_windows_baseline_does_not_inherit_mac_surface_values(self) -> None:
        profile = windows_edge_150_profile()

        self.assertEqual(profile.navigator.platform, "Win32")
        self.assertEqual(profile.canvas.font_bounding_box_ascent, 12.0)
        self.assertEqual(profile.canvas.actual_bounding_box_ascent, 8.0)
        self.assertIn("width:169px", profile.css.input_text)
        self.assertEqual(profile.webgl.max_viewport_width, 32_767)
        self.assertEqual(profile.webgl.max_vertex_uniform_vectors, 4_096)
        self.assertEqual(profile.webgpu.max_buffer_size, 268_435_456)
        self.assertEqual(profile.webgpu.max_vertex_attributes, 16)
        self.assertEqual(profile.webgpu.max_color_attachment_bytes_per_sample, 32)
        self.assertEqual(profile.webgpu.max_compute_invocations_per_workgroup, 256)
        self.assertEqual(len(profile.media.media_recorder_types), 5)

    def test_every_physical_windows_gpu_has_a_large_compatible_space(self) -> None:
        possible_combinations = 0
        minimum_per_gpu = None
        physical_gpu_count = 0

        for gpu in WINDOWS_WEBGL_GPU_CANDIDATES:
            if str(gpu.get("tier", "")) == "virtual":
                continue
            physical_gpu_count += 1
            hardware_rows = get_compatible_pc_navigator_hardware_profiles_for_gpu(
                gpu,
                tag="windows",
            )
            self.assertTrue(hardware_rows, gpu.get("id"))
            per_gpu = sum(
                len(
                    get_compatible_pc_screen_profiles_for_device(
                        hardware,
                        tag="windows",
                        gpu_profile=gpu,
                    )
                )
                for hardware in hardware_rows
            )
            self.assertGreater(per_gpu, 0, gpu.get("id"))
            possible_combinations += per_gpu
            minimum_per_gpu = per_gpu if minimum_per_gpu is None else min(minimum_per_gpu, per_gpu)

        self.assertGreaterEqual(physical_gpu_count, 280)
        self.assertGreaterEqual(len(PC_NAVIGATOR_HARDWARE_PROFILES), 125)
        self.assertGreaterEqual(len(PC_SCREEN_PROFILES), 100)
        self.assertGreaterEqual(minimum_per_gpu or 0, 1_000)
        self.assertGreaterEqual(possible_combinations, 500_000)

    def test_portable_desktop_workstation_and_arm64_boundaries(self) -> None:
        gpu_by_id = {str(item["id"]): item for item in WINDOWS_WEBGL_GPU_CANDIDATES}

        laptop = gpu_by_id["win_nvidia_rtx_4090_laptop"]
        laptop_hardware = get_compatible_pc_navigator_hardware_profiles_for_gpu(laptop)
        self.assertTrue(laptop_hardware)
        self.assertTrue(
            all(_tags(item) & {"laptop", "touch", "convertible", "surface"} for item in laptop_hardware)
        )
        self.assertTrue(all(int(item["hardwareConcurrency"]) >= 12 for item in laptop_hardware))
        self.assertTrue(all(int(item["physicalRamHintGb"]) >= 16 for item in laptop_hardware))

        desktop = gpu_by_id["win_nvidia_rtx_5090"]
        desktop_hardware = get_compatible_pc_navigator_hardware_profiles_for_gpu(desktop)
        self.assertTrue(desktop_hardware)
        self.assertTrue(
            all(
                not (_tags(item) & {"laptop", "touch", "convertible", "surface", "arm64"})
                for item in desktop_hardware
            )
        )
        self.assertGreaterEqual(len({int(item["physicalRamHintGb"]) for item in desktop_hardware}), 4)

        workstation = gpu_by_id["win_nvidia_rtx_pro_6000_blackwell"]
        workstation_hardware = get_compatible_pc_navigator_hardware_profiles_for_gpu(workstation)
        self.assertTrue(workstation_hardware)
        self.assertTrue(all("workstation" in _tags(item) for item in workstation_hardware))
        self.assertTrue(all(int(item["physicalRamHintGb"]) >= 32 for item in workstation_hardware))

        arm = gpu_by_id["win_qualcomm_adreno_x1_85"]
        arm_hardware = get_compatible_pc_navigator_hardware_profiles_for_gpu(arm, tag="arm64")
        self.assertTrue(arm_hardware)
        self.assertTrue(all("arm64" in _tags(item) for item in arm_hardware))

    def test_seeded_sample_is_broad_and_has_no_mac_baseline_leaks(self) -> None:
        gpu_ids: set[str] = set()
        hardware_ids: set[str] = set()
        screen_ids: set[str] = set()
        combinations: set[tuple[str, str, str]] = set()
        dpr_values: set[float] = set()

        for seed in range(2_000):
            fingerprint = get_random_fp_details("US", seed=seed)
            profile = fingerprint.profile
            gpu_ids.add(fingerprint.webgl_gpu_profile_id)
            hardware_ids.add(fingerprint.navigator_hardware_profile_id)
            screen_ids.add(fingerprint.screen_profile_id)
            combinations.add(
                (
                    fingerprint.webgl_gpu_profile_id,
                    fingerprint.navigator_hardware_profile_id,
                    fingerprint.screen_profile_id,
                )
            )
            dpr_values.add(float(profile.screen.device_pixel_ratio))

            self.assertEqual(profile.window.inner_width, 0.0)
            self.assertEqual(profile.window.inner_height, 0.0)
            self.assertEqual(profile.canvas.font_bounding_box_ascent, 12.0)
            self.assertIn("width:169px", profile.css.input_text)
            self.assertEqual(profile.webgl.max_viewport_width, 32_767)
            self.assertEqual(profile.webgl.max_vertex_uniform_vectors, 4_096)
            self.assertEqual(profile.webgpu.max_buffer_size, 268_435_456)
            self.assertEqual(profile.webgpu.max_vertex_attributes, 16)
            self.assertEqual(profile.webgpu.max_color_attachment_bytes_per_sample, 32)
            self.assertEqual(profile.webgpu.max_compute_invocations_per_workgroup, 256)
            self.assertEqual(len(profile.media.media_recorder_types), 5)

        self.assertGreaterEqual(len(gpu_ids), 200)
        self.assertGreaterEqual(len(hardware_ids), 70)
        self.assertGreaterEqual(len(screen_ids), 65)
        self.assertGreaterEqual(len(combinations), 1_700)
        self.assertGreaterEqual(len(dpr_values), 6)


if __name__ == "__main__":
    unittest.main()
