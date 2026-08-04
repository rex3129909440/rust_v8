from __future__ import annotations

import sys
import unittest
from dataclasses import fields, is_dataclass
from datetime import datetime
from pathlib import Path
from zoneinfo import ZoneInfo, ZoneInfoNotFoundError


PROJECT_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(PROJECT_ROOT / "examples"))

from mac_edge_profile import (  # noqa: E402
    APPLE_GPU_FAMILY,
    APPLE_GPU_NAME,
    APPLE_M2_COMPRESSED_TEXTURE_FORMATS,
    MAC_DEVICE_MEMORY_GB,
    MAC_EDGE_150_USER_AGENT,
    MAC_HARDWARE_CONCURRENCY,
    MAC_PHYSICAL_MEMORY_GB,
    local_time_zone,
    mac_edge_150_profile,
)
from edge_profile import EdgeProfile, LocalFontProfile, PermissionsProfile  # noqa: E402
from run_sandbox import EdgeSandbox  # noqa: E402


LIBRARY = PROJECT_ROOT / "dist" / "windows-x64" / "edge_sandbox.dll"


REALM_PROBE = r'''(async () => {
  const snapshot = root => [
    root.navigator.userAgent,
    root.navigator.platform,
    root.navigator.userAgentData.platform,
    root.navigator.userAgentData.brands.map(value => value.brand).join(",")
  ].join("~");

  const frame = document.createElement("iframe");
  document.body.appendChild(frame);
  const workerSource = `postMessage([
    navigator.userAgent,
    navigator.platform,
    navigator.userAgentData.platform,
    navigator.userAgentData.brands.map(value => value.brand).join(",")
  ].join("~"));`;
  const workerUrl = URL.createObjectURL(new Blob(
    [workerSource],
    {type: "text/javascript"}
  ));
  const worker = new Worker(workerUrl);
  const workerValue = await new Promise((resolve, reject) => {
    worker.onmessage = event => resolve(event.data);
    worker.onerror = event => reject(new Error(event.message));
  });
  worker.terminate();
  URL.revokeObjectURL(workerUrl);
  return [snapshot(window), snapshot(frame.contentWindow), workerValue].join("|");
})()'''


HOST_PROFILE_REALM_PROBE = r'''(async () => {
  const snapshot = root => [
    root.Intl.DateTimeFormat().resolvedOptions().timeZone,
    new root.Date().getTimezoneOffset(),
    root.navigator.hardwareConcurrency,
    root.navigator.deviceMemory
  ].join("~");

  const frame = document.createElement("iframe");
  document.body.appendChild(frame);
  const workerSource = `postMessage((${snapshot.toString()})(self));`;
  const workerUrl = URL.createObjectURL(new Blob(
    [workerSource],
    {type: "text/javascript"}
  ));
  const worker = new Worker(workerUrl);
  const workerValue = await new Promise((resolve, reject) => {
    worker.onmessage = event => resolve(event.data);
    worker.onerror = event => reject(new Error(event.message));
  });
  worker.terminate();
  URL.revokeObjectURL(workerUrl);
  return [snapshot(window), snapshot(frame.contentWindow), workerValue].join("|");
})()'''


SURFACE_PROBE = r'''(async () => {
  const hints = await navigator.userAgentData.getHighEntropyValues([
    "architecture", "bitness", "platformVersion", "uaFullVersion",
    "wow64", "formFactors"
  ]);
  const gl = document.createElement("canvas").getContext("webgl");
  const debug = gl.getExtension("WEBGL_debug_renderer_info");
  return [
    navigator.userAgent,
    navigator.appVersion,
    navigator.platform,
    navigator.language,
    navigator.hardwareConcurrency,
    navigator.deviceMemory,
    navigator.maxTouchPoints,
    navigator.userAgentData.platform,
    hints.architecture,
    hints.bitness,
    hints.platformVersion,
    hints.uaFullVersion,
    hints.wow64,
    hints.formFactors.join(","),
    screen.width,
    screen.height,
    screen.availHeight,
    innerWidth,
    innerHeight,
    devicePixelRatio,
    gl.getParameter(debug.UNMASKED_VENDOR_WEBGL),
    gl.getParameter(debug.UNMASKED_RENDERER_WEBGL),
    new AudioContext().sampleRate,
    document.fonts.check('12px "SF Pro Text"')
  ].join("|");
})()'''


MAC_DEVICE_PROBE = r'''(async () => {
  const midi = await navigator.requestMIDIAccess();
  return [
    speechSynthesis.getVoices().map(voice => voice.name).join(","),
    document.fonts.check('12px "SF Pro Text"'),
    document.fonts.check('12px "Segoe UI"'),
    (await navigator.usb.getDevices()).length,
    (await navigator.hid.getDevices()).length,
    (await navigator.serial.getPorts()).length,
    midi.inputs.size,
    midi.outputs.size
  ].join("|");
})()'''


GPU_REALM_PROBE = r'''(async () => {
  const snapshot = async root => {
    const gl = new root.OffscreenCanvas(8, 8).getContext("webgl2");
    const debug = gl.getExtension("WEBGL_debug_renderer_info");
    const formats = Array.from(gl.getParameter(gl.COMPRESSED_TEXTURE_FORMATS));
    const pointRange = Array.from(gl.getParameter(gl.ALIASED_POINT_SIZE_RANGE));
    const precision = gl.getShaderPrecisionFormat(gl.FRAGMENT_SHADER, gl.HIGH_FLOAT);
    const adapter = await root.navigator.gpu.requestAdapter();
    return [
      gl.getParameter(debug.UNMASKED_VENDOR_WEBGL),
      gl.getParameter(debug.UNMASKED_RENDERER_WEBGL),
      gl.getParameter(gl.MAX_SAMPLES),
      pointRange.join(","),
      formats.length,
      formats.includes(0x9270),
      formats.includes(0x93b0),
      formats.includes(0x93d0),
      precision.rangeMin,
      precision.rangeMax,
      precision.precision,
      adapter.info.vendor,
      adapter.info.architecture,
      adapter.info.device,
      adapter.info.description,
      adapter.features.has("texture-compression-astc"),
      adapter.features.has("texture-compression-etc2"),
      adapter.limits.maxTextureDimension2D,
      adapter.limits.maxTextureArrayLayers,
      adapter.limits.maxComputeWorkgroupStorageSize,
      adapter.limits.maxColorAttachments
    ].join("~");
  };

  const frame = document.createElement("iframe");
  document.body.appendChild(frame);
  const workerSource = `(${snapshot.toString()})(self).then(value => postMessage(value));`;
  const workerUrl = URL.createObjectURL(new Blob(
    [workerSource],
    {type: "text/javascript"}
  ));
  const worker = new Worker(workerUrl);
  const workerValue = await new Promise((resolve, reject) => {
    worker.onmessage = event => resolve(event.data);
    worker.onerror = event => reject(new Error(event.message));
  });
  const values = await Promise.all([snapshot(window), snapshot(frame.contentWindow)]);
  worker.terminate();
  URL.revokeObjectURL(workerUrl);
  return [values[0], values[1], workerValue].join("|");
})()'''


WEB_CODECS_PROBE = r'''Promise.all([
  AudioDecoder.isConfigSupported({
    codec: "mp4a.40.2", numberOfChannels: 2, sampleRate: 48000
  }),
  AudioEncoder.isConfigSupported({
    codec: "mp4a.40.2", numberOfChannels: 2, sampleRate: 48000
  }),
  VideoDecoder.isConfigSupported({
    codec: "avc1.42001e", codedWidth: 1920, codedHeight: 1080
  }),
  VideoEncoder.isConfigSupported({
    codec: "avc1.42001e", width: 1920, height: 1080
  }),
  AudioDecoder.isConfigSupported({
    codec: "windows-only-codec", numberOfChannels: 2, sampleRate: 48000
  })
]).then(values => values.map(value => value.supported).join("|"))'''


CUSTOM_FONT_REALM_PROBE = r'''(async () => {
  const snapshot = root => {
    const fonts = root.document ? root.document.fonts : root.fonts;
    return [
      fonts.check('12px "User Mac Sans"'),
      fonts.check('12px "SF Pro Text"'),
      fonts.check('12px "Segoe UI"')
    ].join("~");
  };
  const frame = document.createElement("iframe");
  document.body.appendChild(frame);
  const source = `postMessage((${snapshot.toString()})(self));`;
  const url = URL.createObjectURL(new Blob([source], {type: "text/javascript"}));
  const worker = new Worker(url);
  const workerValue = await new Promise((resolve, reject) => {
    worker.onmessage = event => resolve(event.data);
    worker.onerror = event => reject(new Error(event.message));
  });
  const local = await queryLocalFonts();
  worker.terminate();
  URL.revokeObjectURL(url);
  return [
    snapshot(window),
    snapshot(frame.contentWindow),
    workerValue,
    local.map(font => font.postscriptName).join(",")
  ].join("|");
})()'''


@unittest.skipUnless(LIBRARY.is_file(), "production edge_sandbox.dll is unavailable")
class MacEdgeProfileTests(unittest.TestCase):
    def test_timezone_uses_icu_dst_and_keeps_native_date_functions(self) -> None:
        profile = mac_edge_150_profile()
        sandbox = EdgeSandbox(library=LIBRARY, profile=profile)
        try:
            value = sandbox.evaluate(
                r'''[
                  Intl.DateTimeFormat().resolvedOptions().timeZone,
                  new Date("2025-01-15T12:00:00Z").getTimezoneOffset(),
                  new Date("2025-07-15T12:00:00Z").getTimezoneOffset(),
                  Function.prototype.toString.call(Date),
                  Function.prototype.toString.call(Date.prototype.getTimezoneOffset)
                ].join("|")'''
            )
        finally:
            sandbox.close()

        time_zone = profile.locale.time_zone
        self.assertIsNotNone(time_zone)
        try:
            zone = ZoneInfo(time_zone)
            winter = datetime.fromisoformat("2025-01-15T12:00:00+00:00")
            summer = datetime.fromisoformat("2025-07-15T12:00:00+00:00")
            winter_offset = -int(
                winter.astimezone(zone).utcoffset().total_seconds() / 60
            )
            summer_offset = -int(
                summer.astimezone(zone).utcoffset().total_seconds() / 60
            )
        except ZoneInfoNotFoundError:
            # local_time_zone() uses a fixed GMT ID only when no IANA host ID
            # is discoverable, so both historical probes use the fixed offset.
            winter_offset = profile.locale.time_zone_offset_minutes
            summer_offset = profile.locale.time_zone_offset_minutes
        self.assertEqual(
            value,
            f"{time_zone}|{winter_offset}|{summer_offset}|"
            "function Date() { [native code] }|"
            "function getTimezoneOffset() { [native code] }",
        )

    def test_default_profile_uses_host_timezone_and_real_m2_pro_values(self) -> None:
        expected_time_zone, expected_offset = local_time_zone()
        profile = mac_edge_150_profile()

        self.assertEqual(profile.locale.time_zone, expected_time_zone)
        self.assertEqual(profile.locale.time_zone_offset_minutes, expected_offset)
        self.assertEqual(MAC_PHYSICAL_MEMORY_GB, 32)
        self.assertEqual(
            profile.navigator.hardware_concurrency,
            MAC_HARDWARE_CONCURRENCY,
        )
        self.assertEqual(profile.navigator.device_memory_gb, MAC_DEVICE_MEMORY_GB)

        sandbox = EdgeSandbox(library=LIBRARY, profile=profile)
        try:
            value = sandbox.evaluate(HOST_PROFILE_REALM_PROBE)
        finally:
            sandbox.close()

        expected = "~".join(
            (
                expected_time_zone,
                str(expected_offset),
                str(MAC_HARDWARE_CONCURRENCY),
                str(int(MAC_DEVICE_MEMORY_GB)),
            )
        )
        self.assertEqual(value.split("|"), [expected, expected, expected])

    def test_timezone_and_hardware_remain_explicitly_configurable(self) -> None:
        profile = mac_edge_150_profile(
            time_zone="America/Los_Angeles",
            hardware_concurrency=37,
            device_memory_gb=31.5,
        )
        sandbox = EdgeSandbox(library=LIBRARY, profile=profile)
        try:
            value = sandbox.evaluate(HOST_PROFILE_REALM_PROBE)
        finally:
            sandbox.close()
        expected = (
            f"America/Los_Angeles~{profile.locale.time_zone_offset_minutes}~37~31.5"
        )
        self.assertEqual(value.split("|"), [expected, expected, expected])

    def test_prompt_permissions_do_not_disclose_profile_values(self) -> None:
        sandbox = EdgeSandbox(library=LIBRARY, profile=mac_edge_150_profile())
        try:
            value = sandbox.evaluate(
                r'''(async () => {
                  const location = await new Promise(resolve => {
                    navigator.geolocation.getCurrentPosition(
                      () => resolve("success"),
                      error => resolve(`error:${error.code}`)
                    );
                  });
                  const media = await navigator.mediaDevices
                    .getUserMedia({audio: true})
                    .then(() => "success", error => `error:${error.name}`);
                  const fonts = await queryLocalFonts()
                    .then(() => "success", error => `error:${error.name}`);
                  const devices = await navigator.mediaDevices.enumerateDevices();
                  return [
                    location,
                    media,
                    fonts,
                    devices.every(device =>
                      device.deviceId === "" &&
                      device.label === "" &&
                      device.groupId === ""
                    )
                  ].join("|");
                })()'''
            )
        finally:
            sandbox.close()
        self.assertEqual(
            value,
            "error:1|error:NotAllowedError|error:NotAllowedError|true",
        )

    def test_css_font_metrics_and_webgpu_are_loaded_from_the_mac_profile(self) -> None:
        sandbox = EdgeSandbox(library=LIBRARY, profile=mac_edge_150_profile())
        try:
            value = sandbox.evaluate(
                r'''(async () => {
                  const input = document.createElement("input");
                  document.body.appendChild(input);
                  const rect = input.getBoundingClientRect();
                  const style = getComputedStyle(input);
                  const context = document.createElement("canvas").getContext("2d");
                  context.font = '10px "SF Pro Text"';
                  const sf = context.measureText("abcd").width;
                  context.font = '10px "Missing UI"';
                  const fallback = context.measureText("abcd").width;
                  const adapter = await navigator.gpu.requestAdapter();
                  return [
                    rect.width,
                    rect.height,
                    style.fontFamily,
                    (sf / fallback).toFixed(3),
                    adapter.info.device,
                    adapter.info.description,
                    adapter.info.subgroupMinSize,
                    adapter.info.subgroupMaxSize,
                    adapter.info.isFallbackAdapter
                  ].join("|");
                })()'''
            )
        finally:
            sandbox.close()
        self.assertEqual(value, "177|21|Arial|0.965|||32|32|false")

    def test_default_windows_profile_is_not_replaced_by_the_mac_preset(self) -> None:
        sandbox = EdgeSandbox(library=LIBRARY)
        try:
            value = sandbox.evaluate(
                r'''(async () => {
                  const input = document.createElement("input");
                  document.body.appendChild(input);
                  const rect = input.getBoundingClientRect();
                  const adapter = await navigator.gpu.requestAdapter();
                  return [
                    navigator.platform,
                    navigator.userAgentData.platform,
                    rect.width,
                    rect.height,
                    adapter.info.device,
                    adapter.info.description,
                    adapter.info.subgroupMinSize,
                    adapter.info.subgroupMaxSize,
                    adapter.info.isFallbackAdapter
                  ].join("|");
                })()'''
            )
        finally:
            sandbox.close()
        self.assertEqual(
            value,
            "Win32|Windows|177|21|Edge WebGPU Adapter|"
            "Microsoft Edge WebGPU software adapter|4|128|true",
        )

    def test_profile_crosses_window_iframe_worker_and_trace(self) -> None:
        sandbox = EdgeSandbox(library=LIBRARY, profile=mac_edge_150_profile())
        try:
            before_trace = sandbox.evaluate(REALM_PROBE)
            sandbox.enable_native_trace()
            after_trace = sandbox.evaluate(REALM_PROBE)
        finally:
            sandbox.close()

        expected = "~".join(
            (
                MAC_EDGE_150_USER_AGENT,
                "MacIntel",
                "macOS",
                "Not;A=Brand,Chromium,Google Chrome",
            )
        )
        self.assertEqual(before_trace.split("|"), [expected, expected, expected])
        self.assertEqual(after_trace, before_trace)

    def test_surface_uses_coherent_apple_silicon_values(self) -> None:
        sandbox = EdgeSandbox(library=LIBRARY, profile=mac_edge_150_profile())
        try:
            values = sandbox.evaluate(SURFACE_PROBE).split("|")
        finally:
            sandbox.close()

        self.assertEqual(values[0], MAC_EDGE_150_USER_AGENT)
        self.assertEqual(values[1], MAC_EDGE_150_USER_AGENT.removeprefix("Mozilla/"))
        self.assertEqual(
            values[2:14],
            [
                "MacIntel",
                "en-US",
                "10",
                str(int(MAC_DEVICE_MEMORY_GB)),
                "0",
                "macOS",
                "arm",
                "64",
                "15.5.0",
                "150.0.0.0",
                "false",
                "Desktop",
            ],
        )
        self.assertEqual(
            values[14:],
            [
                "1512",
                "982",
                "944",
                "1440",
                "820",
                "2",
                "Google Inc. (Apple)",
                "ANGLE (Apple, ANGLE Metal Renderer: Apple M2 Pro, Unspecified Version)",
                "48000",
                "true",
            ],
        )

    def test_all_apple_gpu_fields_are_explicit(self) -> None:
        profile = mac_edge_150_profile()
        self.assertEqual(
            [field.name for field in fields(profile.webgl) if getattr(profile.webgl, field.name) is None],
            [],
        )
        self.assertEqual(
            [field.name for field in fields(profile.webgpu) if getattr(profile.webgpu, field.name) is None],
            [],
        )
        self.assertEqual(profile.webgpu.architecture, APPLE_GPU_FAMILY)
        self.assertEqual(profile.webgpu.device, APPLE_GPU_NAME)
        self.assertEqual(
            profile.webgl.compressed_texture_formats,
            APPLE_M2_COMPRESSED_TEXTURE_FORMATS,
        )

    def test_apple_gpu_crosses_window_iframe_worker_and_trace(self) -> None:
        sandbox = EdgeSandbox(library=LIBRARY, profile=mac_edge_150_profile())
        try:
            before_trace = sandbox.evaluate(GPU_REALM_PROBE)
            sandbox.enable_native_trace()
            after_trace = sandbox.evaluate(GPU_REALM_PROBE)
        finally:
            sandbox.close()

        expected = "~".join(
            (
                "Google Inc. (Apple)",
                "ANGLE (Apple, ANGLE Metal Renderer: Apple M2 Pro, Unspecified Version)",
                "4",
                "1,511",
                str(len(APPLE_M2_COMPRESSED_TEXTURE_FORMATS)),
                "true",
                "true",
                "true",
                "127",
                "127",
                "23",
                "apple",
                "apple8",
                "",
                "",
                "true",
                "true",
                "16384",
                "2048",
                "32768",
                "8",
            )
        )
        self.assertEqual(before_trace.split("|"), [expected, expected, expected])
        self.assertEqual(after_trace, before_trace)

    def test_mac_fonts_voices_and_empty_devices_replace_windows_defaults(self) -> None:
        sandbox = EdgeSandbox(library=LIBRARY, profile=mac_edge_150_profile())
        try:
            value = sandbox.evaluate(MAC_DEVICE_PROBE)
        finally:
            sandbox.close()
        self.assertEqual(value, "Samantha,Alex|true|true|0|0|0|0|0")

    def test_non_gpu_os_sensitive_groups_do_not_fall_back(self) -> None:
        profile = mac_edge_150_profile()
        for group in (profile.media, profile.permissions, profile.sensors):
            self.assertEqual(
                [field.name for field in fields(group) if getattr(group, field.name) is None],
                [],
            )
        self.assertIsNotNone(profile.plugins)
        self.assertIsNotNone(profile.plugins.plugins)
        self.assertEqual(len(profile.plugins.plugins), 5)

    def test_only_user_state_fields_are_intentionally_unbound(self) -> None:
        missing: list[str] = []

        def collect(value: object, path: str) -> None:
            if not is_dataclass(value):
                return
            for field in fields(value):
                child = getattr(value, field.name)
                child_path = f"{path}.{field.name}"
                if child is None:
                    missing.append(child_path)
                elif is_dataclass(child):
                    collect(child, child_path)

        collect(mac_edge_150_profile(), "profile")
        self.assertEqual(
            missing,
            [
                "profile.navigator.do_not_track",
                "profile.geolocation",
                "profile.timing",
            ],
        )

    def test_mac_webcodecs_lists_reach_the_native_profile(self) -> None:
        sandbox = EdgeSandbox(library=LIBRARY, profile=mac_edge_150_profile())
        try:
            value = sandbox.evaluate(WEB_CODECS_PROBE)
        finally:
            sandbox.close()
        self.assertEqual(value, "true|true|true|true|false")

    def test_user_font_configuration_replaces_mac_defaults_in_all_realms(self) -> None:
        profile = mac_edge_150_profile(
            font_families=("User Mac Sans", "Helvetica Neue"),
            local_fonts=(
                LocalFontProfile(
                    "UserMacSans-Regular",
                    "User Mac Sans Regular",
                    "User Mac Sans",
                    "Regular",
                ),
            ),
            allow_unknown_font_families=False,
            local_fonts_permission="granted",
        )
        sandbox = EdgeSandbox(library=LIBRARY, profile=profile)
        try:
            value = sandbox.evaluate(CUSTOM_FONT_REALM_PROBE)
        finally:
            sandbox.close()
        self.assertEqual(
            value,
            "true~true~true|true~true~true|true~true~true|UserMacSans-Regular",
        )

    def test_user_can_opt_into_unknown_font_support(self) -> None:
        profile = mac_edge_150_profile(
            font_families=(),
            local_fonts=(),
            allow_unknown_font_families=True,
        )
        sandbox = EdgeSandbox(library=LIBRARY, profile=profile)
        try:
            value = sandbox.evaluate(
                'document.fonts.check(\'12px "Unlisted User Font"\')'
            )
        finally:
            sandbox.close()
        self.assertEqual(value, "true")

    def test_mac_fonts_are_instance_scoped_and_do_not_change_dll_defaults(self) -> None:
        probe = "queryLocalFonts().then(fonts => fonts.map(font => font.postscriptName).join(','))"

        mac_sandbox = EdgeSandbox(
            library=LIBRARY,
            profile=mac_edge_150_profile(local_fonts_permission="granted"),
        )
        try:
            mac_fonts = mac_sandbox.evaluate(probe)
        finally:
            mac_sandbox.close()

        default_sandbox = EdgeSandbox(
            library=LIBRARY,
            profile=EdgeProfile(
                permissions=PermissionsProfile(local_fonts="granted")
            ),
        )
        try:
            default_fonts = default_sandbox.evaluate(probe)
        finally:
            default_sandbox.close()

        self.assertEqual(
            mac_fonts,
            "SFProText-Regular,SFProDisplay-Regular,HelveticaNeue,Menlo-Regular",
        )
        self.assertEqual(default_fonts, "EdgeSandboxSans-Regular")


if __name__ == "__main__":
    unittest.main()
