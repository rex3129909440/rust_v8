from __future__ import annotations

from collections import Counter
from collections import defaultdict
import unittest

from demo.get_random_fp import get_random_fp_details
from demo.fp.pc_navigator_hardware_catalog import (
    PC_NAVIGATOR_HARDWARE_PROFILES,
    get_compatible_pc_navigator_hardware_profiles_for_gpu,
)
from demo.fp.screen_profile_catalog import (
    BASE_PC_SCREEN_SIZE_ROWS,
    PC_SCREEN_PROFILES,
    get_compatible_pc_screen_profiles_for_device,
    materialize_pc_screen_profile_for_windows,
)
from demo.fp.windows_webgl_gpu_catalog import (
    NVIDIA_OPEN_GPU_EXTENSION_CANDIDATES,
    WINDOWS_WEBGL_GPU_CANDIDATES,
)
from demo.fp.windows_css_profile_catalog import chromium150_windows_css_overrides
from demo.fp.nvidia_open_gpu_extension_catalog import (
    DAWN_SOURCE_SHA256,
    NVIDIA_SOURCE_SHA256,
    count_device_pairs,
    count_products,
)
from demo.fp.windows_pci_device_catalog import WINDOWS_GPU_DEVICE_VARIANTS
from examples.windows_edge_profile import windows_edge_150_profile


def _tags(profile: dict[str, object]) -> set[str]:
    return {str(item).lower() for item in profile.get("tags", ())}


class WindowsProfileCatalogTests(unittest.TestCase):
    def test_windows_gpu_ids_and_renderer_shapes_are_evidence_linked(self) -> None:
        self.assertGreaterEqual(len(WINDOWS_GPU_DEVICE_VARIANTS), 260)
        self.assertGreaterEqual(
            sum(len(items) for items in WINDOWS_GPU_DEVICE_VARIANTS.values()),
            600,
        )
        self.assertGreaterEqual(len(WINDOWS_WEBGL_GPU_CANDIDATES), 1_200)
        self.assertNotIn(
            "0x00002A01",
            {
                str(item.get("deviceId", ""))
                for item in WINDOWS_WEBGL_GPU_CANDIDATES
            },
        )

        renderer_shapes_by_device: dict[
            tuple[str, str],
            list[dict[str, object]],
        ] = defaultdict(list)
        for gpu in WINDOWS_WEBGL_GPU_CANDIDATES:
            device_id = str(gpu.get("deviceId", ""))
            renderer = str(gpu["webgl"]["unmaskedRenderer"])
            renderer_shapes_by_device[
                (str(gpu.get("baseProfileId", "")), device_id)
            ].append(gpu)
            self.assertRegex(device_id, r"^0x[0-9A-F]{8}$")
            self.assertTrue(str(gpu.get("evidenceName", "")))
            self.assertTrue(str(gpu.get("webgpuArchitecture", "")))
            if bool(gpu.get("rendererDeviceIdExposed")):
                self.assertIn(f"({device_id})", renderer)
            else:
                self.assertNotIn(f"({device_id})", renderer)

        # Every evidence-backed device ID—not only RTX 5060—must expose both
        # ANGLE renderer shapes while retaining the same internal device ID.
        self.assertEqual(
            len(renderer_shapes_by_device),
            sum(len(items) for items in WINDOWS_GPU_DEVICE_VARIANTS.values())
            + len(NVIDIA_OPEN_GPU_EXTENSION_CANDIDATES) // 2,
        )
        for device_key, shapes in renderer_shapes_by_device.items():
            self.assertEqual(len(shapes), 2, device_key)
            self.assertEqual(
                {bool(item["rendererDeviceIdExposed"]) for item in shapes},
                {False, True},
                device_key,
            )

        rtx_5060 = [
            item
            for item in WINDOWS_WEBGL_GPU_CANDIDATES
            if item.get("baseProfileId") == "win_nvidia_rtx_5060"
        ]
        self.assertEqual(len(rtx_5060), 4)
        self.assertEqual(
            {str(item["deviceId"]) for item in rtx_5060},
            {"0x00002D05", "0x00002F06"},
        )
        self.assertEqual(
            {bool(item["rendererDeviceIdExposed"]) for item in rtx_5060},
            {False, True},
        )

        webgpu_by_base = {
            str(item["baseProfileId"]): bool(item["webgpuSupported"])
            for item in WINDOWS_WEBGL_GPU_CANDIDATES
        }
        self.assertFalse(webgpu_by_base["win_intel_hd_4000"])
        self.assertFalse(webgpu_by_base["win_intel_hd_5300"])
        self.assertTrue(webgpu_by_base["win_intel_hd_5500"])
        self.assertTrue(webgpu_by_base["win_nvidia_gtx_760"])

        # The current NVIDIA source is an additive inventory. Only complete
        # product/Device-ID rows whose architecture exists in pinned Edge 150
        # Dawn enter the browser pool; the full inventory remains inspectable.
        self.assertGreaterEqual(count_products(), 220)
        self.assertGreaterEqual(count_device_pairs(), 330)
        self.assertGreaterEqual(count_products(browser_eligible_only=True), 150)
        self.assertGreaterEqual(count_device_pairs(browser_eligible_only=True), 250)
        self.assertRegex(NVIDIA_SOURCE_SHA256, r"^[0-9a-f]{64}$")
        self.assertRegex(DAWN_SOURCE_SHA256, r"^[0-9a-f]{64}$")

    def test_windows_baseline_does_not_inherit_mac_surface_values(self) -> None:
        profile = windows_edge_150_profile()

        self.assertEqual(profile.navigator.platform, "Win32")
        self.assertEqual(profile.canvas.font_bounding_box_ascent, 9.0)
        self.assertEqual(profile.canvas.actual_bounding_box_ascent, 7.0)
        self.assertIn("width:169px", profile.css.input_text)
        self.assertEqual(profile.webgl.max_viewport_width, 32_767)
        self.assertEqual(profile.webgl.max_vertex_uniform_vectors, 4_096)
        self.assertEqual(profile.webgpu.max_buffer_size, 268_435_456)
        self.assertEqual(profile.webgpu.max_vertex_attributes, 16)
        self.assertEqual(profile.webgpu.max_color_attachment_bytes_per_sample, 32)
        self.assertEqual(profile.webgpu.max_compute_invocations_per_workgroup, 256)
        self.assertEqual(len(profile.media.media_recorder_types), 15)
        self.assertIn("video/webm;codecs=vp8*", profile.media.media_recorder_types)
        self.assertNotIn("video/webm;codecs=daala*", profile.media.media_recorder_types)

    def test_windows_11_screen_and_memory_catalog_invariants(self) -> None:
        screen_by_id = {str(item["id"]): item for item in PC_SCREEN_PROFILES}
        self.assertNotIn("pc_1920x1080_1p25_windows", screen_by_id)
        full_hd = screen_by_id["pc_1920x1080_1x_desktop"]
        self.assertEqual(full_hd["screen"]["availHeight"], 1_032)
        self.assertEqual(full_hd["taskbarCssHeight"], 48)
        self.assertEqual(full_hd["physicalWidth"], 1_920)
        self.assertEqual(full_hd["physicalHeight"], 1_080)

        physical_memory_values = {
            int(item["physicalRamHintGb"])
            for item in PC_NAVIGATOR_HARDWARE_PROFILES
        }
        self.assertIn(3, physical_memory_values)
        self.assertIn(6, physical_memory_values)
        self.assertIn(12, physical_memory_values)
        self.assertIn(96, physical_memory_values)

        logical_processor_values = {
            int(item["hardwareConcurrency"])
            for item in PC_NAVIGATOR_HARDWARE_PROFILES
        }
        self.assertIn(32, logical_processor_values)
        self.assertGreaterEqual(max(logical_processor_values), 192)

        # hardwareConcurrency is not deviceMemory and has no eight-thread cap.
        explicit = windows_edge_150_profile(
            hardware_concurrency=32,
            device_memory_gb=32.0,
        )
        self.assertEqual(explicit.navigator.hardware_concurrency, 32)
        self.assertEqual(explicit.navigator.device_memory_gb, 32.0)

    def test_screen_extension_is_additive_and_windows_work_area_is_versioned(self) -> None:
        self.assertEqual(len(BASE_PC_SCREEN_SIZE_ROWS), 104)
        self.assertEqual(
            tuple(
                str(item["id"])
                for item in PC_SCREEN_PROFILES[: len(BASE_PC_SCREEN_SIZE_ROWS)]
            ),
            tuple(str(row[0]) for row in BASE_PC_SCREEN_SIZE_ROWS),
        )
        self.assertGreaterEqual(len(PC_SCREEN_PROFILES), 124)
        extension = PC_SCREEN_PROFILES[len(BASE_PC_SCREEN_SIZE_ROWS) :]
        self.assertTrue(extension)
        self.assertTrue(all(item["evidenceSources"] for item in extension))
        self.assertEqual(
            len({str(item["id"]) for item in PC_SCREEN_PROFILES}),
            len(PC_SCREEN_PROFILES),
        )

        base = next(
            item for item in PC_SCREEN_PROFILES
            if item["id"] == "pc_1920x1080_1x_desktop"
        )
        windows_10 = materialize_pc_screen_profile_for_windows(base, "10.0.0")
        windows_11 = materialize_pc_screen_profile_for_windows(base, "15.0.0")
        windows_11_depth_32 = materialize_pc_screen_profile_for_windows(
            base,
            "15.0.0",
            color_depth=32,
        )
        self.assertEqual(windows_10["screen"]["availHeight"], 1_040)
        self.assertEqual(windows_10["taskbarCssHeight"], 40)
        self.assertEqual(windows_11["screen"]["availHeight"], 1_032)
        self.assertEqual(windows_11["taskbarCssHeight"], 48)
        self.assertEqual(
            (
                windows_11_depth_32["screen"]["colorDepth"],
                windows_11_depth_32["screen"]["pixelDepth"],
            ),
            (32, 32),
        )
        self.assertTrue(str(windows_11_depth_32["id"]).endswith("_depth32"))
        self.assertEqual(base["screen"]["availHeight"], 1_032)

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
        # Form-factor constraints intentionally remove laptop/desktop cross
        # products. Keep a broad space without reintroducing invalid pairs.
        self.assertGreaterEqual(minimum_per_gpu or 0, 800)
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
        logical_processors: Counter[int] = Counter()
        physical_memory: Counter[int] = Counter()
        screen_sizes: Counter[tuple[int, int]] = Counter()

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
            logical_processors[int(profile.navigator.hardware_concurrency)] += 1
            physical_memory[int(fingerprint.physical_memory_gb)] += 1
            screen_sizes[
                (int(profile.screen.width), int(profile.screen.height))
            ] += 1

            self.assertEqual(profile.window.inner_width, 0.0)
            self.assertEqual(profile.window.inner_height, 0.0)
            self.assertEqual(profile.canvas.font_bounding_box_ascent, 9.0)
            expected_css = chromium150_windows_css_overrides(
                float(profile.screen.device_pixel_ratio),
                profile.navigator.language,
            )
            self.assertEqual(profile.css.input_text, expected_css["input_text"])
            self.assertEqual(profile.css.input_file, expected_css["input_file"])
            self.assertEqual(profile.webgl.max_viewport_width, 32_767)
            self.assertEqual(profile.webgl.max_vertex_uniform_vectors, 4_096)
            self.assertEqual(profile.webgpu.max_buffer_size, 268_435_456)
            self.assertEqual(profile.webgpu.max_vertex_attributes, 16)
            self.assertEqual(profile.webgpu.max_color_attachment_bytes_per_sample, 32)
            self.assertEqual(profile.webgpu.max_compute_invocations_per_workgroup, 256)
            self.assertEqual(len(profile.media.media_recorder_types), 15)
            self.assertIn(
                "video/mp4;codecs=av01.*",
                profile.media.media_recorder_types,
            )

        self.assertGreaterEqual(len(gpu_ids), 200)
        self.assertGreaterEqual(len(hardware_ids), 70)
        self.assertGreaterEqual(len(screen_ids), 65)
        self.assertGreaterEqual(len(combinations), 1_700)
        self.assertGreaterEqual(len(dpr_values), 6)

        # GPU catalog growth must not flatten the independently weighted CPU,
        # RAM, and display distributions. Mainstream states remain dominant;
        # workstation-size values stay available only as a low-probability tail.
        self.assertGreaterEqual(
            sum(count for cores, count in logical_processors.items() if cores <= 16),
            1_700,
        )
        self.assertGreater(
            sum(count for cores, count in logical_processors.items() if cores >= 32),
            0,
        )
        self.assertGreaterEqual(physical_memory[16] + physical_memory[32], 1_350)
        self.assertLess(sum(count for ram, count in physical_memory.items() if ram >= 96), 30)
        self.assertGreaterEqual(
            sum(
                screen_sizes[size]
                for size in (
                    (1920, 1080),
                    (1536, 864),
                    (2560, 1440),
                    (1366, 768),
                    (1280, 720),
                )
            ),
            1_350,
        )


if __name__ == "__main__":
    unittest.main()
