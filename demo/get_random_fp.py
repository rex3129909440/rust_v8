"""Compose and verify country-aware random sandbox fingerprints.

This module is intentionally import-only.  Call :func:`get_random_fp` with an
ISO 3166-1 alpha-2 country code and pass the returned ``EdgeProfile`` directly
to ``EdgeSandbox``.  The catalogs in ``demo/fp`` remain the source of truth for
country locale/time-zone data, UA/UA-CH values, platform-linked hardware,
screens, ANGLE GPUs, fonts, and speech synthesis voices.

No JavaScript Proxy or trace mode is involved.  ``verify_random_fp`` starts a
normal isolated sandbox, reads the configured surfaces, and compares every
observation with the selected typed profile.
"""

from __future__ import annotations

import base64
import math
import random
import re
import secrets
import sys
from dataclasses import dataclass, replace
from functools import lru_cache
from pathlib import Path
from typing import Iterable, Sequence
from urllib.parse import urlencode, urlsplit, urlunsplit
from zoneinfo import ZoneInfo, ZoneInfoNotFoundError


DEMO_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = DEMO_DIR.parent
FP_DIR = DEMO_DIR / "fp"
SOURCE_CHECKOUT = (PROJECT_ROOT / "examples" / "edge_profile.py").is_file()

for import_root in (PROJECT_ROOT, FP_DIR):
    import_text = str(import_root)
    if import_text not in sys.path:
        sys.path.insert(0, import_text)

from fingerprint_runtime_composer import (  # noqa: E402
    choose_language_list,
    choose_timezone_for_country,
    is_supported_country_code,
    normalize_country_code,
)
from android_device_profile_catalog import (  # noqa: E402
    choose_android_device_profile,
    choose_android_version_for_device,
    get_android_device_profile_by_id,
    get_android_device_profiles,
    materialize_android_device_profile,
)
from android_graphics_capability_catalog import (  # noqa: E402
    build_android_graphics_capabilities,
)
from android_font_profile_catalog import build_android_font_profile  # noqa: E402
from android_css_profile_catalog import chromium_android_css_overrides  # noqa: E402
from mac_chromium150_capture_catalog import (  # noqa: E402
    CHROME_MAC_REMOTE_SPEECH_VOICES,
    MAC_CHROMIUM150_AUDIO,
    MAC_CHROMIUM150_CANVAS,
    MAC_CHROMIUM150_KEYBOARD_LAYOUT,
    MAC_CHROMIUM150_MEDIA_LISTS,
    MAC_CHROMIUM150_RTC_AUDIO_CODECS,
    MAC_CHROMIUM150_RTC_HEADER_EXTENSIONS,
    MAC_CHROMIUM150_RTC_VIDEO_CODECS,
    MACOS_LOCAL_SPEECH_VOICES,
)
from keyboard_layout_catalog import keyboard_layout_for_profile  # noqa: E402
from mac_font_profile_catalog import build_mac_font_profile  # noqa: E402
from mac_graphics_capability_catalog import (  # noqa: E402
    build_mac_graphics_capabilities,
)
from mac_screen_profile_catalog import (  # noqa: E402
    choose_mac_screen_profile_for_gpu,
)
from mac_webgl_gpu_catalog import (  # noqa: E402
    choose_mac_gpu_candidate,
    get_mac_gpu_candidates,
)
from pc_navigator_hardware_catalog import (  # noqa: E402
    choose_pc_navigator_hardware_profile_for_gpu,
)
from ua import (  # noqa: E402
    generate_headers_from_ua,
    generate_ua_data_from_ua,
    get_base_platform,
)
from screen_profile_catalog import (  # noqa: E402
    choose_pc_screen_profile_for_hardware,
    choose_windows_screen_depth,
    materialize_pc_screen_profile_for_windows,
)
from speech_synthesis_voice_catalog import (  # noqa: E402
    choose_speech_synthesis_voice_profile,
)
from timezone_geolocation_catalog import (  # noqa: E402
    get_time_zone_reference_location,
)
from windows_webgl_gpu_catalog import (  # noqa: E402
    choose_weighted_windows_webgl_gpu_candidate,
    get_windows_webgl_gpu_candidates,
)
from windows_font_profile_catalog import build_windows_font_profile  # noqa: E402
from windows_css_profile_catalog import (  # noqa: E402
    chromium148_zh_cn_dpr1_css_overrides,
    chromium150_windows_css_overrides,
)
from v8_memory_profile_catalog import (  # noqa: E402
    choose_v8_memory_snapshot,
    is_known_memory_snapshot,
    v8_150_precise_heap_size_limit,
)

if SOURCE_CHECKOUT:
    from examples.edge_profile import (  # type: ignore
        BatteryProfile,
        DocumentProfile,
        EdgeProfile,
        FontMetricProfile,
        FontProfile,
        GeolocationProfile,
        LocaleProfile,
        LocalFontProfile,
        KeyboardLayoutEntryProfile,
        MediaDeviceProfile,
        NavigatorProfile,
        NetworkProfile,
        RtcCodecProfile,
        RtcHeaderExtensionProfile,
        ScreenProfile,
        SensorsProfile,
        SpeechProfile,
        SpeechVoiceProfile,
        TimingProfile,
        UserAgentBrandProfile,
        UserAgentDataProfile,
        WindowProfile,
    )
else:  # Installed wheel.
    from edge_sandbox.edge_profile import (
        BatteryProfile,
        DocumentProfile,
        EdgeProfile,
        FontMetricProfile,
        FontProfile,
        GeolocationProfile,
        LocaleProfile,
        LocalFontProfile,
        KeyboardLayoutEntryProfile,
        MediaDeviceProfile,
        NavigatorProfile,
        NetworkProfile,
        RtcCodecProfile,
        RtcHeaderExtensionProfile,
        ScreenProfile,
        SensorsProfile,
        SpeechProfile,
        SpeechVoiceProfile,
        TimingProfile,
        UserAgentBrandProfile,
        UserAgentDataProfile,
        WindowProfile,
    )

if SOURCE_CHECKOUT:
    from examples.mac_edge_profile import mac_edge_150_profile  # type: ignore
    from examples.windows_edge_profile import windows_edge_150_profile  # type: ignore
else:  # Installed wheel.
    from edge_sandbox.mac_edge_profile import mac_edge_150_profile
    from edge_sandbox.windows_edge_profile import windows_edge_150_profile


DEFAULT_TEST_COUNTRIES: tuple[str, ...] = (
    "US",
    "CN",
    "DE",
    "BR",
    "IN",
    "JP",
    "GB",
    "AU",
)

DEFAULT_WINDOWS_USER_AGENT = (
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
    "AppleWebKit/537.36 (KHTML, like Gecko) "
    "Chrome/150.0.0.0 Safari/537.36"
)
DEFAULT_MAC_USER_AGENT = (
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
    "AppleWebKit/537.36 (KHTML, like Gecko) "
    "Chrome/150.0.0.0 Safari/537.36"
)
DEFAULT_ANDROID_EDGE_USER_AGENT = (
    "Mozilla/5.0 (Linux; Android 10; K) "
    "AppleWebKit/537.36 (KHTML, like Gecko) "
    "Chrome/150.0.0.0 Mobile Safari/537.36 EdgA/150.0.0.0"
)

_PROBE_SEPARATOR = "\x1f"

# ICU may return a legacy canonical spelling even when it accepted an IANA
# link name.  Both identifiers describe the same zone and therefore the same
# configured Date/Intl behavior.
_ICU_TIME_ZONE_ALIASES: dict[str, str] = {
    "America/Indiana/Indianapolis": "America/Indianapolis",
    "America/Kentucky/Louisville": "America/Louisville",
    "Asia/Kathmandu": "Asia/Katmandu",
    "Asia/Kolkata": "Asia/Calcutta",
    "Europe/Kyiv": "Europe/Kiev",
}

_WINDOWS_WEBGL1_EXTENSIONS = (
    "ANGLE_instanced_arrays",
    "EXT_blend_minmax",
    "EXT_color_buffer_half_float",
    "EXT_float_blend",
    "EXT_frag_depth",
    "EXT_shader_texture_lod",
    "EXT_texture_compression_bptc",
    "EXT_texture_filter_anisotropic",
    "OES_element_index_uint",
    "OES_fbo_render_mipmap",
    "OES_standard_derivatives",
    "OES_texture_float",
    "OES_texture_float_linear",
    "OES_texture_half_float",
    "OES_texture_half_float_linear",
    "OES_vertex_array_object",
    "WEBGL_color_buffer_float",
    "WEBGL_compressed_texture_s3tc",
    "WEBGL_compressed_texture_s3tc_srgb",
    "WEBGL_debug_renderer_info",
    "WEBGL_debug_shaders",
    "WEBGL_depth_texture",
    "WEBGL_draw_buffers",
    "WEBGL_lose_context",
    "WEBGL_multi_draw",
)
_WINDOWS_WEBGL2_EXTENSIONS = (
    "EXT_color_buffer_float",
    "EXT_color_buffer_half_float",
    "EXT_float_blend",
    "EXT_texture_compression_bptc",
    "EXT_texture_filter_anisotropic",
    "OES_draw_buffers_indexed",
    "OES_texture_float_linear",
    "WEBGL_compressed_texture_s3tc",
    "WEBGL_compressed_texture_s3tc_srgb",
    "WEBGL_debug_renderer_info",
    "WEBGL_debug_shaders",
    "WEBGL_lose_context",
    "WEBGL_multi_draw",
)
_WINDOWS_COMPRESSED_TEXTURE_FORMATS = (
    *range(0x83F0, 0x83F4),
    *range(0x8C4C, 0x8C50),
    *range(0x8E8C, 0x8E90),
)

_ANDROID_WEBGL1_EXTENSIONS = (
    "ANGLE_instanced_arrays",
    "EXT_blend_minmax",
    "EXT_color_buffer_half_float",
    "EXT_float_blend",
    "EXT_frag_depth",
    "EXT_shader_texture_lod",
    "EXT_texture_filter_anisotropic",
    "OES_element_index_uint",
    "OES_fbo_render_mipmap",
    "OES_standard_derivatives",
    "OES_texture_float",
    "OES_texture_float_linear",
    "OES_texture_half_float",
    "OES_texture_half_float_linear",
    "OES_vertex_array_object",
    "WEBGL_compressed_texture_astc",
    "WEBGL_compressed_texture_etc",
    "WEBGL_compressed_texture_etc1",
    "WEBGL_debug_renderer_info",
    "WEBGL_debug_shaders",
    "WEBGL_depth_texture",
    "WEBGL_draw_buffers",
    "WEBGL_lose_context",
)
_ANDROID_WEBGL2_EXTENSIONS = (
    "EXT_color_buffer_float",
    "EXT_color_buffer_half_float",
    "EXT_float_blend",
    "EXT_texture_filter_anisotropic",
    "OES_draw_buffers_indexed",
    "OES_texture_float_linear",
    "WEBGL_compressed_texture_astc",
    "WEBGL_compressed_texture_etc",
    "WEBGL_debug_renderer_info",
    "WEBGL_debug_shaders",
    "WEBGL_lose_context",
)
_ANDROID_COMPRESSED_TEXTURE_FORMATS = (
    0x8D64,
    *range(0x9274, 0x9279),
    *range(0x93B0, 0x93BE),
)


@dataclass(frozen=True, slots=True)
class ResourceLoadProfile:
    """Per-evaluation script resource context linked to a random profile.

    Only opaque URL-safe request tokens are randomized.  The caller supplies
    the actual HTTPS page URL and version so the generated script URL stays
    same-origin, inherits the page directory, and uses the same version value.
    """

    script_name: str
    uid_token: str
    integrity_token: str

    def script_url(self, page_url: str, x_kpsdk_v: str) -> str:
        parsed = urlsplit(page_url)
        if parsed.scheme.lower() != "https" or not parsed.netloc:
            raise ValueError("page_url must be an absolute HTTPS URL")
        version = str(x_kpsdk_v).strip()
        if not version:
            raise ValueError("x_kpsdk_v must not be blank")
        directory = parsed.path.rsplit("/", 1)[0]
        path = f"{directory}/{self.script_name}"
        query = urlencode(
            (
                ("KP_UIDz", self.uid_token),
                ("x-kpsdk-v", version),
                ("x-kpsdk-im", self.integrity_token),
            )
        )
        return urlunsplit((parsed.scheme, parsed.netloc, path, query, ""))


@dataclass(frozen=True, slots=True)
class RandomFingerprint:
    """One typed profile plus the catalog choices used to construct it."""

    profile: EdgeProfile
    seed: int
    country_code: str
    time_zone: str
    browser: str
    platform: str
    request_headers: tuple[tuple[str, str], ...]
    user_agent_profile_id: str
    navigator_hardware_profile_id: str
    screen_profile_id: str
    webgl_gpu_profile_id: str
    speech_voice_profile_id: str
    font_profile_id: str
    cpu_logical_processors: int
    device_memory_gb: float
    physical_memory_gb: int
    memory_snapshot_profile_id: str
    gpu_model: str
    gpu_core_count: int | None
    gpu_core_unit: str
    geolocation_reference: tuple[float, float]
    resource_load: ResourceLoadProfile


@dataclass(frozen=True, slots=True)
class FingerprintVerification:
    """Result of reading one generated fingerprint from a real sandbox."""

    fingerprint: RandomFingerprint
    observations: tuple[tuple[str, str], ...]


def audit_random_fp(fingerprint: RandomFingerprint) -> tuple[str, ...]:
    """Return cross-surface consistency errors for one generated profile.

    This audit is deliberately independent from any site or workload.  It
    checks only relationships that must agree inside the browser profile.
    ``get_random_fp_details`` runs it before returning, so callers cannot
    accidentally receive a profile that violates these invariants.
    """

    issues: list[str] = []
    profile = fingerprint.profile
    navigator = profile.navigator
    ua_data = navigator.user_agent_data
    screen = profile.screen
    window = profile.window
    storage = profile.storage
    webgpu = profile.webgpu

    ua_match = re.search(r"\b(?:Chrome|Chromium)/(\d+)", navigator.user_agent)
    chromium_major = int(ua_match.group(1)) if ua_match else 0
    if chromium_major < 140 or chromium_major > 151:
        issues.append("navigator.userAgent has no supported Chromium major")

    edge_match = re.search(r"\b(?:Edg|EdgA)/(\d+)", navigator.user_agent)
    if fingerprint.browser == "edge":
        if edge_match is None:
            issues.append("Edge profile has no Edg/EdgA token")
        elif int(edge_match.group(1)) != chromium_major:
            issues.append("Chrome and Edge UA majors differ")
    elif edge_match is not None:
        issues.append("Chrome profile unexpectedly contains an Edge token")

    expected_platforms = {
        "windows": ("Win32", "Windows", False),
        "macos": ("MacIntel", "macOS", False),
        "android": ("Linux armv81", "Android", True),
    }
    expected = expected_platforms.get(fingerprint.platform)
    if expected is None:
        issues.append(f"unsupported profile platform {fingerprint.platform!r}")
    else:
        js_platform, ch_platform, mobile = expected
        if navigator.platform != js_platform:
            issues.append("navigator.platform conflicts with selected platform")
        if ua_data.platform != ch_platform:
            issues.append("UA-CH platform conflicts with selected platform")
        if bool(ua_data.mobile) != mobile:
            issues.append("UA-CH mobile flag conflicts with selected platform")

    if not navigator.languages or navigator.languages[0] != navigator.language:
        issues.append("navigator.language is not the first languages entry")
    if navigator.hardware_concurrency != fingerprint.cpu_logical_processors:
        issues.append("hardwareConcurrency conflicts with selection metadata")
    if float(navigator.device_memory_gb) != float(fingerprint.device_memory_gb):
        issues.append("deviceMemory conflicts with selection metadata")
    if fingerprint.physical_memory_gb <= 0:
        issues.append("physical memory must be positive")
    allowed_touch_points = {
        "windows": {0, 5, 10},
        "macos": {0},
        "android": {5},
    }.get(fingerprint.platform, set())
    if navigator.max_touch_points not in allowed_touch_points:
        issues.append("maxTouchPoints conflicts with selected device platform")

    network = navigator.network
    network_rtt = int(network.rtt) if network.rtt is not None else -1
    if not 0 <= network_rtt <= 600 or network_rtt % 50:
        issues.append("NetworkInformation.rtt is outside Chromium's generated buckets")
    downlink = float(network.downlink or 0)
    if not 0.05 <= downlink <= 10.0 or abs(downlink * 20 - round(downlink * 20)) > 1e-9:
        issues.append("NetworkInformation.downlink is outside Chromium's generated buckets")
    if network_rtt >= 2_000 or downlink <= 0.05:
        expected_effective_type = "slow-2g"
    elif network_rtt >= 1_400 or downlink <= 0.07:
        expected_effective_type = "2g"
    elif network_rtt >= 270 or downlink <= 0.7:
        expected_effective_type = "3g"
    else:
        expected_effective_type = "4g"
    if network.effective_type != expected_effective_type:
        issues.append("effectiveType conflicts with RTT/downlink")
    if profile.media_preferences.reduced_data != bool(network.save_data):
        issues.append("prefers-reduced-data conflicts with NetworkInformation.saveData")
    if navigator.user_activation_is_active and not navigator.user_activation_has_been_active:
        issues.append("userActivation.isActive requires hasBeenActive")
    posture = str(profile.hardware_devices.device_posture or "")
    if posture not in {"continuous", "folded"}:
        issues.append("device posture is invalid")
    if posture == "folded" and fingerprint.platform != "android":
        issues.append("non-foldable desktop profile uses folded posture")
    if profile.media_preferences.forced_colors and profile.media_preferences.contrast == "no-preference":
        issues.append("forced colors conflict with prefers-contrast")

    allowed_device_memory = (
        {1.0, 2.0, 4.0, 8.0}
        if fingerprint.platform == "android"
        else ({2.0, 4.0, 8.0, 16.0, 32.0} if chromium_major >= 147 else {2.0, 4.0, 8.0})
    )
    if float(navigator.device_memory_gb) not in allowed_device_memory:
        issues.append("deviceMemory is not a Chromium bucket for this platform/version")

    if not screen or not window:
        issues.append("screen/window profile is missing")
    else:
        width = float(screen.width or 0)
        height = float(screen.height or 0)
        avail_width = float(screen.avail_width or 0)
        avail_height = float(screen.avail_height or 0)
        avail_left = float(screen.avail_left or 0)
        avail_top = float(screen.avail_top or 0)
        if width <= 0 or height <= 0:
            issues.append("physical screen dimensions must be positive")
        if avail_width <= 0 or avail_height <= 0:
            issues.append("available screen dimensions must be positive")
        if avail_left + avail_width > width or avail_top + avail_height > height:
            issues.append("available screen rectangle exceeds physical screen")
        if screen.color_depth != screen.pixel_depth:
            issues.append("screen colorDepth and pixelDepth differ")
        allowed_screen_depths = {
            "windows": {24, 32},
            "macos": {24, 30},
            "android": {24},
        }.get(fingerprint.platform, set())
        if screen.color_depth not in allowed_screen_depths:
            issues.append("screen depth conflicts with selected device platform")
        if float(screen.device_pixel_ratio or 0) <= 0:
            issues.append("devicePixelRatio must be positive")
        if float(screen.viewport_width or 0) != 0.0 or float(screen.viewport_height or 0) != 0.0:
            issues.append("configured hidden viewport must keep screen viewport at zero")
        if float(window.inner_width or 0) != 0.0 or float(window.inner_height or 0) != 0.0:
            issues.append("configured hidden viewport must keep window.inner* at zero")
        if float(window.outer_width or 0) != float(screen.outer_width or 0):
            issues.append("window.outerWidth conflicts with screen outer width")
        if fingerprint.platform == "windows":
            try:
                platform_major = int(
                    str(ua_data.platform_version).split(".", 1)[0]
                )
            except (TypeError, ValueError):
                issues.append("Windows UA-CH platform version is invalid")
            else:
                expected_taskbar_height = 48 if platform_major >= 13 else 40
                observed_taskbar_height = int(height) - int(avail_height)
                if observed_taskbar_height != expected_taskbar_height:
                    issues.append(
                        "Windows work area conflicts with UA-CH platform version"
                    )
        if float(window.outer_height or 0) != float(screen.outer_height or 0):
            issues.append("window.outerHeight conflicts with screen outer height")

    if storage is None or storage.quota_bytes is None or storage.usage_bytes is None:
        issues.append("storage estimate is incomplete")
    else:
        available = int(storage.quota_bytes) - int(storage.usage_bytes)
        if int(storage.usage_bytes) < 0 or available < 0:
            issues.append("storage usage/quota relationship is invalid")
        if available != 10 * 1024**3:
            issues.append("storage estimate does not follow Chromium static quota")

    memory = profile.memory
    expected_heap_limit = _v8_heap_limit_for_profile(
        fingerprint.physical_memory_gb,
        fingerprint.platform,
    )
    if fingerprint.platform == "android":
        try:
            selected_android_device = get_android_device_profile_by_id(
                fingerprint.screen_profile_id
            )
        except KeyError:
            selected_android_device = None
        if (
            selected_android_device is not None
            and selected_android_device.get("jsHeapSizeLimit") is not None
        ):
            expected_heap_limit = int(selected_android_device["jsHeapSizeLimit"])
    if memory.performance_js_heap_size_limit != expected_heap_limit:
        issues.append("performance.memory heap limit conflicts with physical memory")
    if memory.console_js_heap_size_limit != memory.performance_js_heap_size_limit:
        issues.append("console.memory and performance.memory heap limits differ")
    if not (
        0 <= int(memory.performance_used_js_heap_size or -1)
        <= int(memory.performance_total_js_heap_size or -1)
        <= int(memory.performance_js_heap_size_limit or -1)
    ):
        issues.append("performance.memory used/total/limit relationship is invalid")
    if (
        memory.console_used_js_heap_size != memory.performance_used_js_heap_size
        or memory.console_total_js_heap_size != memory.performance_total_js_heap_size
    ):
        issues.append("console.memory conflicts with performance.memory")
    if not is_known_memory_snapshot(
        fingerprint.memory_snapshot_profile_id,
        fingerprint.platform,
        int(memory.performance_total_js_heap_size or 0),
        int(memory.performance_used_js_heap_size or 0),
    ):
        issues.append("performance.memory snapshot is not an evidence catalog row")

    if fingerprint.platform in {"windows", "macos", "android"}:
        if not str(webgpu.device or ""):
            issues.append("internal WebGPU adapter identity is missing")
        if webgpu.developer_features is not False:
            issues.append("WebGPU developer features would expose private adapter fields")

    if fingerprint.platform == "macos":
        if profile.sensors.available is not False:
            issues.append("Mac profile exposes unavailable sensor readings")
        if profile.permissions.speaker_selection != "unsupported":
            issues.append("Mac speaker-selection permission should be unsupported")
        if profile.permissions.top_level_storage_access != "invalid-origin":
            issues.append("Mac top-level-storage-access permission should reject invalid origin")
        # Selection metadata is encoded in the generated hardware id because
        # Screen itself intentionally describes the active display, which may
        # be external.
        if "_host_imac" in fingerprint.navigator_hardware_profile_id.lower():
            battery = profile.battery
            if not (
                battery.charging is True
                and battery.charging_time == 0.0
                and battery.level == 1.0
            ):
                issues.append("iMac host is paired with a portable battery state")
        hardware_id = fingerprint.navigator_hardware_profile_id.lower()
        expected_camera = (
            any(marker in hardware_id for marker in ("_host_air", "_host_pro"))
            or "_host_imac" in hardware_id
            or fingerprint.screen_profile_id
            == "mac_studio_display_2560x1440_2x"
        )
        has_camera = any(
            device.kind == "videoinput" for device in profile.media.devices
        )
        if has_camera != expected_camera:
            issues.append("Mac camera inventory conflicts with host/display class")
    elif fingerprint.platform == "windows":
        try:
            windows_platform_major = int(str(ua_data.platform_version).split(".", 1)[0])
        except ValueError:
            windows_platform_major = -1
        if windows_platform_major >= 13 and fingerprint.physical_memory_gb < 4:
            issues.append("Windows 11 profile is below its 4 GiB memory minimum")
        if windows_platform_major not in {10, 15}:
            issues.append("generated Windows platformVersion is unsupported")
        expected_font_prefix = (
            "windows-11-" if windows_platform_major >= 13 else "windows-10-"
        )
        if not fingerprint.font_profile_id.startswith(expected_font_prefix):
            issues.append("Windows font inventory conflicts with platformVersion")
        if profile.sensors.available is not bool(navigator.max_touch_points):
            issues.append("Windows sensor availability conflicts with device form factor")
    elif fingerprint.platform == "android":
        permissions = profile.permissions
        expected_android_permissions = {
            "accelerometer": "granted",
            "background_sync": "granted",
            "camera": "prompt",
            "clipboard_read": "prompt",
            "clipboard_write": "granted",
            "geolocation": "prompt",
            "gyroscope": "granted",
            "magnetometer": "granted",
            "microphone": "prompt",
            "midi": "prompt",
            "notifications": "prompt",
            "payment_handler": "granted",
            "persistent_storage": "prompt",
            "speaker_selection": "unsupported",
            "storage_access": "granted",
            "top_level_storage_access": "invalid-origin",
            "window_management": "denied",
        }
        if any(
            getattr(permissions, name) != value
            for name, value in expected_android_permissions.items()
        ):
            issues.append("Android untouched permission states conflict with HTTPS evidence")
        if profile.sensors.available is not True:
            issues.append("Android profile disables available motion/orientation sensors")
        if profile.plugins.plugins:
            issues.append("Android clean profile unexpectedly exposes desktop PDF plugins")
        if profile.speech.voices:
            issues.append("Android clean profile unexpectedly injects desktop speech voices")
        try:
            selected_android_device = get_android_device_profile_by_id(
                fingerprint.screen_profile_id
            )
            android_major = int(str(ua_data.platform_version).split(".", 1)[0])
            expected_webgl, expected_webgpu = build_android_graphics_capabilities(
                materialize_android_device_profile(
                    selected_android_device,
                    android_major,
                ),
                chromium_major,
            )
        except (KeyError, TypeError, ValueError):
            issues.append("Android device/OS selection metadata is invalid")
        else:
            for field in (
                "max_texture_size",
                "max_renderbuffer_size",
                "max_viewport_width",
                "max_viewport_height",
                "webgl2_max_samples",
            ):
                if getattr(profile.webgl, field) != expected_webgl[field]:
                    issues.append("Android WebGL capabilities conflict with GPU family")
                    break
            if profile.webgpu.available is not expected_webgpu["available"]:
                issues.append("Android WebGPU availability conflicts with device/OS")

    headers = dict(fingerprint.request_headers)
    if headers.get("user-agent") != navigator.user_agent:
        issues.append("request User-Agent conflicts with navigator.userAgent")
    if headers.get("sec-ch-ua-platform") != f'"{ua_data.platform}"':
        issues.append("request sec-ch-ua-platform conflicts with UA-CH")
    if headers.get("sec-ch-ua-arch") != f'"{ua_data.architecture}"':
        issues.append("request sec-ch-ua-arch conflicts with UA-CH")
    if headers.get("sec-ch-ua-bitness") != f'"{ua_data.bitness}"':
        issues.append("request sec-ch-ua-bitness conflicts with UA-CH")
    if headers.get("sec-ch-ua-platform-version") != f'"{ua_data.platform_version}"':
        issues.append("request platform-version conflicts with UA-CH")
    expected_mobile_header = "?1" if ua_data.mobile else "?0"
    if headers.get("sec-ch-ua-mobile") != expected_mobile_header:
        issues.append("request sec-ch-ua-mobile conflicts with UA-CH")
    if not headers.get("accept-language", "").startswith(navigator.language):
        issues.append("Accept-Language conflicts with navigator.language")

    if not profile.fonts.families:
        issues.append("font family profile is empty")
    latitude, longitude = fingerprint.geolocation_reference
    if not (-90 <= latitude <= 90 and -180 <= longitude <= 180):
        issues.append("geolocation reference is outside valid bounds")
    return tuple(issues)


def validate_random_fp(fingerprint: RandomFingerprint) -> None:
    """Raise when a generated profile violates a cross-surface invariant."""

    issues = audit_random_fp(fingerprint)
    if issues:
        raise RuntimeError("inconsistent generated fingerprint: " + "; ".join(issues))


def _require_country_code(country_code: str) -> str:
    normalized = normalize_country_code(country_code)
    if not normalized or not is_supported_country_code(normalized):
        raise ValueError(
            "country_code must be a supported ISO 3166-1 alpha-2 code"
        )
    return normalized


def _resolve_seed(seed: int | None) -> int:
    if seed is None:
        return secrets.randbits(63)
    if isinstance(seed, bool) or not isinstance(seed, int):
        raise TypeError("seed must be an integer or None")
    return seed & ((1 << 63) - 1)


def _random_urlsafe_token(rng: random.Random, byte_length: int) -> str:
    payload = bytes(rng.getrandbits(8) for _ in range(byte_length))
    return base64.urlsafe_b64encode(payload).rstrip(b"=").decode("ascii")


def _build_resource_load_profile(rng: random.Random) -> ResourceLoadProfile:
    return ResourceLoadProfile(
        script_name="ips.js",
        # 64 and 32 random bytes produce stable 86/43-character URL-safe
        # opaque identifiers without placeholder text or invalid URL bytes.
        uid_token=_random_urlsafe_token(rng, 64),
        integrity_token=_random_urlsafe_token(rng, 32),
    )


def _resolve_user_agent(
    requested_user_agent: str | None,
) -> tuple[dict[str, object], str]:
    """Validate a Chromium UA and derive its platform/profile data."""

    ua_string = (
        DEFAULT_WINDOWS_USER_AGENT
        if requested_user_agent is None
        else str(requested_user_agent).strip()
    )
    if not ua_string or "\0" in ua_string:
        raise ValueError("user_agent must be a non-empty Chromium UA")
    chrome_match = re.search(r"\b(?:Chrome|Chromium)/(\d+)(?:\.\d+){0,3}", ua_string)
    if chrome_match is None:
        raise ValueError("user_agent must contain a Chrome or Chromium version")
    chromium_major = int(chrome_match.group(1))
    if not 140 <= chromium_major <= 151:
        raise ValueError("user_agent Chromium major must be between 140 and 151")

    is_windows = "Windows NT" in ua_string
    is_macos = "Macintosh" in ua_string or "Mac OS X" in ua_string
    is_android = "Android" in ua_string and "Mobile" in ua_string
    if sum((is_windows, is_macos, is_android)) != 1:
        raise ValueError(
            "user_agent must identify exactly one of Windows, macOS, or Android Mobile"
        )
    platform_name = (
        "windows" if is_windows else "macos" if is_macos else "android"
    )
    browser_name = (
        "edge" if re.search(r"\b(?:Edg|EdgA)/\d", ua_string) else "chrome"
    )
    if browser_name == "edge":
        edge_match = re.search(r"\b(?:Edg|EdgA)/(\d+)(?:\.\d+){0,3}", ua_string)
        if edge_match is None or not 140 <= int(edge_match.group(1)) <= 151:
            raise ValueError("user_agent Edge major must be between 140 and 151")
        if int(edge_match.group(1)) != chromium_major:
            raise ValueError("user_agent Chrome and Edge majors must match")

    ua_options: dict[str, object] = {}
    android_ua_is_frozen = False
    if platform_name == "macos":
        # Chromium's frozen UA still says Intel/10_15_7 on Apple silicon.  The
        # high-entropy UA-CH fields carry the selected Apple-silicon profile.
        ua_options.update(
            architecture="arm",
            bitness="64",
            platform="macOS",
            platformVersion="15.5.0",
            formFactors=("Desktop",),
        )
    elif platform_name == "android":
        android_match = re.search(r"\bAndroid\s+([\d.]+)", ua_string)
        model_match = re.search(
            r"\bAndroid\s+[\d.]+;\s*(.+?)(?:\s+Build/[^)]*)?\)\s+AppleWebKit/",
            ua_string,
        )
        android_version_parts = (
            android_match.group(1).split(".")[:3] if android_match else []
        )
        android_platform_version = ".".join(
            android_version_parts + ["0"] * (3 - len(android_version_parts))
        )
        parsed_model = model_match.group(1).strip() if model_match else "K"
        # Chromium's reduced Android UA deliberately freezes the low-entropy
        # OS/model tokens to "Android 10; K".  Keep that fact as parser
        # metadata; it is not the concrete device selected for UA-CH.
        android_ua_is_frozen = (
            parsed_model.upper() == "K"
            and android_platform_version == "10.0.0"
        )
        ua_options.update(
            architecture="",
            bitness="",
            platform="Android",
            platformVersion=android_platform_version,
            model=parsed_model,
            mobile=True,
            formFactors=("Mobile",),
        )
    ua_data = generate_ua_data_from_ua(ua_string, ua_options)
    headers = generate_headers_from_ua(ua_string, ua_options)
    profile_id = f"requested-{platform_name}-{browser_name}-{chromium_major}"
    return (
        {
            "id": profile_id,
            "browser": browser_name,
            "channel": "requested" if requested_user_agent is not None else "default",
            "weight": 1,
            "tags": (
                platform_name,
                "arm64"
                if (
                    platform_name == "android"
                    or str(ua_data.get("architecture", "")).lower() == "arm"
                )
                else "x64",
            ),
            "userAgent": ua_string,
            "appVersion": ua_string.removeprefix("Mozilla/"),
            "platform": get_base_platform(ua_string),
            "userAgentData": ua_data,
            "headers": headers,
            "chromiumMajor": chromium_major,
            "androidUaIsFrozen": android_ua_is_frozen,
        },
        platform_name,
    )


def _time_zone_offset_minutes(time_zone: str) -> int | None:
    """Return the current JS offset when host tzdata is available.

    Windows Python does not bundle the IANA database.  Returning ``None`` in
    that case leaves the auxiliary typed field untouched; the native sandbox
    still configures ICU with ``time_zone``, which controls Date and Intl.
    """

    try:
        offset = __import__("datetime").datetime.now(ZoneInfo(time_zone)).utcoffset()
    except ZoneInfoNotFoundError:
        return None
    if offset is None:
        return 0
    return -int(offset.total_seconds() / 60)


def _user_agent_data_profile(data: dict[str, object]) -> UserAgentDataProfile:
    basic_brands = tuple(
        item for item in data.get("brands", ()) if isinstance(item, dict)
    )
    full_versions = {
        str(item.get("brand", "")): str(item.get("version", ""))
        for item in data.get("fullVersionList", ())
        if isinstance(item, dict)
    }
    brands = tuple(
        UserAgentBrandProfile(
            brand=str(item.get("brand", "")),
            version=str(item.get("version", "")),
            full_version=full_versions.get(
                str(item.get("brand", "")),
                f"{item.get('version', '')}.0.0.0",
            ),
        )
        for item in basic_brands
    )
    return UserAgentDataProfile(
        brands=brands,
        mobile=bool(data.get("mobile", False)),
        platform=str(data.get("platform", "Windows")),
        architecture=str(data.get("architecture", "x86")),
        bitness=str(data.get("bitness", "64")),
        model=str(data.get("model", "")),
        platform_version=str(data.get("platformVersion", "")),
        ua_full_version=str(data.get("uaFullVersion", "")),
        wow64=bool(data.get("wow64", False)),
        form_factors=tuple(str(item) for item in data.get("formFactors", ())),
    )


@lru_cache(maxsize=4)
def _windows_gpu_pool(
    arm64: bool,
    include_virtual_gpu: bool,
) -> tuple[dict[str, object], ...]:
    candidates = get_windows_webgl_gpu_candidates(
        vendor="qualcomm" if arm64 else None,
        include_virtual=include_virtual_gpu,
    )
    if arm64:
        return candidates
    return tuple(
        candidate
        for candidate in candidates
        if str(candidate.get("driverVendor", "")).lower() != "qualcomm"
    )


def _choose_gpu(
    rng: random.Random,
    *,
    arm64: bool,
    include_virtual_gpu: bool,
) -> dict[str, object]:
    candidates = _windows_gpu_pool(arm64, include_virtual_gpu)
    if not candidates:
        raise ValueError("no platform-consistent Windows GPU candidates available")
    return choose_weighted_windows_webgl_gpu_candidate(rng, candidates)


def _screen_profiles(
    selected: dict[str, object],
) -> tuple[ScreenProfile, WindowProfile]:
    screen = selected.get("screen", {})
    window = selected.get("window", {})
    viewport = selected.get("visualViewport", {})
    if not isinstance(screen, dict) or not isinstance(window, dict):
        raise ValueError("selected screen profile is malformed")
    if not isinstance(viewport, dict):
        viewport = {}
    orientation = screen.get("orientation", {})
    if not isinstance(orientation, dict):
        orientation = {}

    inner_width = float(window.get("innerWidth", 0))
    inner_height = float(window.get("innerHeight", 0))
    outer_width = float(window.get("outerWidth", 0))
    outer_height = float(window.get("outerHeight", 0))
    return (
        ScreenProfile(
            width=int(screen.get("width", 0)),
            height=int(screen.get("height", 0)),
            avail_width=int(screen.get("availWidth", 0)),
            avail_height=int(screen.get("availHeight", 0)),
            avail_left=int(screen.get("availLeft", 0)),
            avail_top=int(screen.get("availTop", 0)),
            color_depth=int(screen.get("colorDepth", 24)),
            pixel_depth=int(screen.get("pixelDepth", 24)),
            viewport_width=float(viewport.get("width", inner_width)),
            viewport_height=float(viewport.get("height", inner_height)),
            outer_width=outer_width,
            outer_height=outer_height,
            screen_x=float(window.get("screenX", 0)),
            screen_y=float(window.get("screenY", 0)),
            device_pixel_ratio=float(window.get("devicePixelRatio", 1)),
            orientation_type=str(orientation.get("type", "landscape-primary")),
            orientation_angle=int(orientation.get("angle", 0)),
            visual_viewport_offset_left=float(viewport.get("offsetLeft", 0)),
            visual_viewport_offset_top=float(viewport.get("offsetTop", 0)),
            visual_viewport_page_left=float(viewport.get("pageLeft", 0)),
            visual_viewport_page_top=float(viewport.get("pageTop", 0)),
            visual_viewport_scale=float(viewport.get("scale", 1)),
        ),
        WindowProfile(
            inner_width=inner_width,
            inner_height=inner_height,
            outer_width=outer_width,
            outer_height=outer_height,
        ),
    )


def _mac_css_for_screen(css: object, selected: dict[str, object]) -> object:
    """Apply Chromium 150 macOS input geometry captured at the selected DPR."""

    window = selected.get("window", {})
    if not isinstance(window, dict):
        window = {}
    dpr = float(window.get("devicePixelRatio", 2.0))
    if dpr >= 1.5:
        width, height = 139.0, 15.5
    else:
        width, height = 145.0, 15.0
    return replace(
        css,
        input_text=(
            "display:inline-block;box-sizing:content-box;"
            f"width:{width:g}px;height:{height:g}px;"
            "padding:1px 2px;border-width:2px;border-style:inset;"
            "border-color:rgb(118, 118, 118);"
            "background-color:rgb(255, 255, 255)"
        ),
    )


def _windows_css_for_screen_and_locale(
    css: object,
    selected: dict[str, object],
    locale: str,
    chromium_major: int,
) -> object:
    """Apply observed Windows Chromium control geometry coherently."""

    window = selected.get("window", {})
    if not isinstance(window, dict):
        window = {}
    dpr = float(window.get("devicePixelRatio", 1.0))
    if chromium_major == 148 and locale.lower() == "zh-cn" and abs(dpr - 1.0) < 1e-9:
        overrides = chromium148_zh_cn_dpr1_css_overrides()
    else:
        overrides = chromium150_windows_css_overrides(dpr, locale)
    return replace(css, **overrides)


def _android_css_for_device(
    css: object,
    selected: dict[str, object],
    locale: str,
    chromium_major: int,
) -> object:
    """Apply Chromium's Android control theme for the selected device."""

    window = selected.get("window", {})
    if not isinstance(window, dict):
        window = {}
    overrides = chromium_android_css_overrides(
        float(window.get("devicePixelRatio", 1.0)),
        locale,
        str(selected.get("oem", "aosp")),
        chromium_major,
    )
    return replace(css, **overrides)


def _speech_profile(selected: dict[str, object]) -> SpeechProfile:
    voices = []
    for item in selected.get("voices", ()):
        if not isinstance(item, dict):
            continue
        voices.append(
            SpeechVoiceProfile(
                voice_uri=str(item.get("voiceURI", item.get("name", ""))),
                name=str(item.get("name", "")),
                lang=str(item.get("lang", "")),
                local_service=bool(item.get("localService", True)),
                is_default=bool(item.get("default", False)),
            )
        )
    return SpeechProfile(voices=tuple(voices))


_MAC_DEFAULT_VOICE_NAMES: dict[str, str] = {
    "ar-001": "Majed",
    "bg-bg": "Daria",
    "ca-es": "Montse",
    "cs-cz": "Zuzana",
    "da-dk": "Sara",
    "de-de": "Anna",
    "el-gr": "Melina",
    "en-au": "Karen",
    "en-gb": "Daniel",
    "en-ie": "Moira",
    "en-in": "Rishi",
    "en-us": "Samantha",
    "en-za": "Tessa",
    "es-es": "Mónica",
    "es-mx": "Paulina",
    "fi-fi": "Satu",
    "fr-ca": "Amélie",
    "fr-fr": "Thomas",
    "he-il": "Carmit",
    "hi-in": "Lekha",
    "hr-hr": "Lana",
    "hu-hu": "Tünde",
    "id-id": "Damayanti",
    "it-it": "Alice",
    "ja-jp": "Kyoko",
    "ko-kr": "Yuna",
    "ms-my": "Amira",
    "nb-no": "Nora",
    "nl-be": "Ellen",
    "nl-nl": "Xander",
    "pl-pl": "Zosia",
    "pt-br": "Luciana",
    "pt-pt": "Joana",
    "ro-ro": "Ioana",
    "ru-ru": "Milena",
    "sk-sk": "Laura",
    "sl-si": "Tina",
    "sv-se": "Alva",
    "th-th": "Kanya",
    "tr-tr": "Yelda",
    "uk-ua": "Lesya",
    "vi-vn": "Linh",
    "yue-hk": "善怡",
    "zh-cn": "婷婷",
    "zh-tw": "美嘉",
}


def _mac_speech_profile(
    languages: Sequence[str],
    browser: str,
) -> SpeechProfile:
    preferred = tuple(str(item).replace("_", "-").lower() for item in languages)
    primary = preferred[0] if preferred else "en-us"
    # Names containing a parenthesised language are localized by macOS.  The
    # capture was taken on a zh-CN system, so retain those rows only for a
    # Chinese system locale; the portable system voices remain valid for every
    # locale.  Chrome's provider voices are browser-specific and are never
    # inserted into an Edge profile.
    local_voices = tuple(
        row
        for row in MACOS_LOCAL_SPEECH_VOICES
        if primary.startswith("zh-") or " (" not in row[1]
    )
    provider_voices = (
        CHROME_MAC_REMOTE_SPEECH_VOICES
        if browser == "chrome"
        else ()
    )
    voices = local_voices + tuple(provider_voices)
    default_name = _MAC_DEFAULT_VOICE_NAMES.get(primary)
    default_index = next(
        (
            index
            for index, row in enumerate(voices)
            if row[0].lower() == primary and row[1] == default_name
        ),
        -1,
    )
    if default_index < 0:
        primary_base = primary.split("-", 1)[0]
        default_index = next(
            (
                index
                for index, row in enumerate(voices)
                if row[0].lower().split("-", 1)[0] == primary_base
            ),
            0,
        )
    ordered_voices = (
        (voices[default_index],)
        + voices[:default_index]
        + voices[default_index + 1 :]
    )
    return SpeechProfile(
        voices=tuple(
            SpeechVoiceProfile(
                voice_uri=voice_uri,
                name=name,
                lang=language,
                local_service=local_service,
                is_default=index == 0,
            )
            for index, (
                language,
                name,
                voice_uri,
                local_service,
                _captured_default,
            ) in enumerate(ordered_voices)
        )
    )


def _mac_media_profile(media: object) -> object:
    lists = MAC_CHROMIUM150_MEDIA_LISTS
    return replace(
        media,
        supported_constraints=lists["supported_constraints"],
        can_play_probably_types=lists["can_play_probably_types"],
        can_play_maybe_types=lists["can_play_maybe_types"],
        media_source_types=lists["media_source_types"],
        media_recorder_types=lists["media_recorder_types"],
        decoding_supported_types=lists["decoding_supported_types"],
        decoding_smooth_types=lists["decoding_smooth_types"],
        decoding_power_efficient_types=(
            lists["decoding_power_efficient_types"]
        ),
        encoding_supported_types=lists["encoding_supported_types"],
        encoding_smooth_types=lists["encoding_smooth_types"],
        encoding_power_efficient_types=(
            lists["encoding_power_efficient_types"]
        ),
        image_decoder_types=lists["image_decoder_types"],
        audio_decoder_codecs=lists["audio_decoder_codecs"],
        audio_encoder_codecs=lists["audio_encoder_codecs"],
        video_decoder_codecs=lists["video_decoder_codecs"],
        video_encoder_codecs=lists["video_encoder_codecs"],
        rtc_audio_codecs=tuple(
            RtcCodecProfile(mime, rate, channels, fmtp)
            for mime, rate, channels, fmtp in MAC_CHROMIUM150_RTC_AUDIO_CODECS
        ),
        rtc_video_codecs=tuple(
            RtcCodecProfile(mime, rate, channels, fmtp)
            for mime, rate, channels, fmtp in MAC_CHROMIUM150_RTC_VIDEO_CODECS
        ),
        rtc_header_extensions=tuple(
            RtcHeaderExtensionProfile(kind, uri)
            for kind, uri in MAC_CHROMIUM150_RTC_HEADER_EXTENSIONS
        ),
    )


def _android_media_profile(
    media: object,
    device: dict[str, object],
    chromium_major: int,
) -> object:
    """Build the Android Chromium codec surface for one device tier."""

    supported_constraints = [
        "aspectRatio", "autoGainControl", "brightness", "channelCount",
        "colorTemperature", "contrast", "deviceId", "displaySurface",
        "echoCancellation", "exposureCompensation", "exposureMode",
        "exposureTime", "facingMode", "focusDistance", "focusMode",
        "frameRate", "groupId", "height", "iso", "latency",
        "noiseSuppression", "pan", "pointsOfInterest", "resizeMode",
        "sampleRate", "sampleSize", "saturation", "sharpness",
        "suppressLocalAudioPlayback", "tilt", "torch", "voiceIsolation",
        "whiteBalanceMode", "width", "zoom",
    ]
    if int(chromium_major) >= 141:
        supported_constraints.insert(
            supported_constraints.index("sampleRate"),
            "restrictOwnAudio",
        )
    power_efficient = [
        'audio/webm; codecs="opus"',
        'video/webm; codecs="vp09.00.10.08"',
    ]
    if str(device.get("mediaTier", "")) == "av1-hardware":
        power_efficient.append('video/webm; codecs="av01.0.04M.08"')

    return replace(
        media,
        supported_constraints=tuple(supported_constraints),
        can_play_probably_types=(
            "audio/mpeg",
            'audio/ogg; codecs="vorbis"',
            'audio/webm; codecs="opus"',
            'video/webm; codecs="vp8"',
            'video/webm; codecs="vp09.00.10.08"',
            'video/webm; codecs="av01.0.04M.08"',
        ),
        can_play_maybe_types=(),
        media_source_types=(
            "audio/mpeg",
            'audio/webm; codecs="opus"',
            'video/webm; codecs="vp8"',
            'video/webm; codecs="vp09.00.10.08"',
            'video/webm; codecs="av01.0.04M.08"',
        ),
        media_recorder_types=(
            'audio/webm; codecs="opus"',
            'video/webm; codecs="vp8"',
            'video/webm; codecs="av01.0.04M.08"',
        ),
        decoding_supported_types=(
            'audio/webm; codecs="opus"',
            'video/webm; codecs="vp09.00.10.08"',
            'video/webm; codecs="av01.0.04M.08"',
        ),
        decoding_smooth_types=(
            'audio/webm; codecs="opus"',
            'video/webm; codecs="vp09.00.10.08"',
            'video/webm; codecs="av01.0.04M.08"',
        ),
        decoding_power_efficient_types=tuple(power_efficient),
        encoding_supported_types=(
            'audio/webm; codecs="opus"',
            'video/webm; codecs="vp8"',
            'video/webm; codecs="av01.0.04M.08"',
        ),
        encoding_smooth_types=(
            'audio/webm; codecs="opus"',
            'video/webm; codecs="vp8"',
            'video/webm; codecs="av01.0.04M.08"',
        ),
        encoding_power_efficient_types=(),
        audio_decoder_codecs=("opus",),
        audio_encoder_codecs=("opus",),
        video_decoder_codecs=("vp8", "vp9", "av1"),
        video_encoder_codecs=("vp8", "av1"),
    )


def _font_profile(selected: dict[str, object]) -> FontProfile:
    return FontProfile(
        families=tuple(str(item) for item in selected.get("families", ())),
        allow_unknown_families=bool(selected.get("allowUnknownFamilies", False)),
        local_fonts=tuple(
            LocalFontProfile(
                postscript_name=str(item[0]),
                full_name=str(item[1]),
                family=str(item[2]),
                style=str(item[3]),
            )
            for item in selected.get("localFonts", ())
        ),
        metrics=tuple(
            FontMetricProfile(
                family=str(item[0]),
                width_scale=float(item[1]),
                monospace=bool(item[2]),
            )
            for item in selected.get("metrics", ())
        ),
    )


def _battery_profile(
    rng: random.Random,
    hardware: dict[str, object],
    screen: dict[str, object],
    gpu: dict[str, object],
) -> BatteryProfile:
    tags = {
        str(item).lower()
        for source in (hardware.get("tags", ()), screen.get("tags", ()))
        for item in source
    }
    portable = bool(tags & {"laptop", "surface", "touch", "convertible"})
    portable = portable or str(gpu.get("tier", "")).lower() == "laptop"
    if not portable or rng.random() < 0.25:
        return BatteryProfile(
            charging=True,
            charging_time=0.0,
            discharging_time=float("inf"),
            level=1.0,
        )

    level = round(rng.uniform(0.18, 0.96), 2)
    if rng.random() < 0.35:
        return BatteryProfile(
            charging=True,
            charging_time=round((1.0 - level) * rng.uniform(3_600, 10_800), 0),
            discharging_time=float("inf"),
            level=level,
        )
    return BatteryProfile(
        charging=False,
        charging_time=float("inf"),
        discharging_time=round(level * rng.uniform(7_200, 28_800), 0),
        level=level,
    )


def _media_devices(
    platform_name: str,
    hardware: dict[str, object],
) -> tuple[MediaDeviceProfile, ...]:
    """Return the pre-permission device shape without invented identifiers.

    Chromium does not expose stable labels or hardware identifiers before the
    caller grants media permission.  Empty strings therefore model the real
    privacy boundary; unlike the old ``Edge Sandbox Device`` defaults, they
    are not synthetic device placeholders.
    """

    devices = [
        MediaDeviceProfile("", "audioinput", "", ""),
    ]
    hardware_tags = {str(item).lower() for item in hardware.get("tags", ())}
    portable_windows = platform_name == "windows" and bool(
        hardware_tags & {"laptop", "touch", "convertible", "surface"}
    )
    mac_camera = platform_name == "macos" and "camera" in hardware_tags
    if platform_name == "android" or portable_windows or mac_camera:
        devices.append(MediaDeviceProfile("", "videoinput", "", ""))
    devices.append(MediaDeviceProfile("", "audiooutput", "", ""))
    return tuple(devices)


def _storage_values(
    rng: random.Random,
    physical_memory_gb: int,
) -> tuple[int, int]:
    """Choose Chromium's static per-origin storage estimate.

    The exposed quota is not the host disk capacity and is not correlated
    with physical RAM.  Current Chromium's static quota policy exposes the
    origin's usage plus 10 GiB of available space.  Keep usage variable while
    preserving that relationship exactly.
    """

    del physical_memory_gb
    usage_bytes = rng.randrange(16 * 1024**2, 192 * 1024**2)
    quota_bytes = usage_bytes + 10 * 1024**3
    return quota_bytes, usage_bytes


def _network_profile(seed: int, platform_name: str) -> NetworkProfile:
    """Choose a browser-observed NetworkInformation snapshot.

    Network quality is independent from CPU/GPU selection. A separate random
    stream keeps the existing hardware catalog sequence stable. Values follow
    Chromium's observable 50 ms / 50 Kbit/s quantization: RTT covers 0..600 ms
    and downlink covers 0.05..10.00 Mbit/s. The effective type is derived from
    the same RTT/downlink pair instead of being selected independently.
    """

    rng = random.Random(seed ^ 0x4E4554574F524B)
    rtt = rng.randint(0, 12) * 50
    downlink = rng.randint(1, 200) / 20.0
    if rtt >= 2_000 or downlink <= 0.05:
        effective_type = "slow-2g"
    elif rtt >= 1_400 or downlink <= 0.07:
        effective_type = "2g"
    elif rtt >= 270 or downlink <= 0.7:
        effective_type = "3g"
    else:
        effective_type = "4g"
    save_data = rng.random() < 0.12
    connection_type = (
        rng.choices(
            ("wifi", "cellular", "ethernet", "bluetooth"),
            weights=(55, 40, 4, 1),
            k=1,
        )[0]
        if platform_name == "android"
        else rng.choices(
            ("wifi", "ethernet", "cellular", "bluetooth"),
            weights=(55, 40, 4, 1),
            k=1,
        )[0]
    )
    return NetworkProfile(
        effective_type=effective_type,
        rtt=rtt,
        downlink=downlink,
        save_data=save_data,
        connection_type=connection_type,
        # Pixel/Chromium HTTPS evidence exposes Infinity for this legacy ECT
        # property when the maximum link speed is not known.
        downlink_max=float("inf"),
    )


def _user_activation_values(seed: int) -> tuple[bool, bool]:
    """Choose one valid UserActivation state for this isolated evaluation."""

    value = random.Random(seed ^ 0x5553455241435449).random()
    if value < 0.12:
        return True, True
    if value < 0.32:
        return True, False
    return False, False


def _document_has_focus_value(seed: int, configured: bool | None) -> bool:
    """Choose the initial document focus without perturbing other catalogs."""

    if configured is not None:
        return configured
    return random.Random(seed ^ 0x444F43554D454E54).random() < 0.5


def _media_preference_values(
    seed: int,
    platform_name: str,
    hardware: dict[str, object],
    screen: dict[str, object],
    network: NetworkProfile,
) -> tuple[dict[str, object], str]:
    """Build one internally consistent display/accessibility preference set."""

    rng = random.Random(seed ^ 0x4D45444941505245)
    tags = {str(item).lower() for item in hardware.get("tags", ())}
    forced_colors = platform_name == "windows" and rng.random() < 0.04
    if forced_colors:
        contrast = "custom"
    else:
        contrast = rng.choices(
            ("no-preference", "more", "less"),
            weights=(94, 4, 2),
            k=1,
        )[0]
    if platform_name == "android":
        pointer = "coarse"
        hover = "none"
    else:
        pointer = "fine"
        hover = "hover"
    color_gamut = (
        str(screen.get("colorGamut", "p3"))
        if platform_name == "macos"
        else "srgb"
    )
    dynamic_range = (
        str(screen.get("dynamicRange", "standard"))
        if platform_name == "macos"
        else "standard"
    )
    # A folded posture changes the active screen/hinge geometry.  Until the
    # selected catalog row materializes that alternate geometry, expose only
    # the coherent continuous posture even for fold-capable hardware.
    posture = "continuous"
    return (
        {
            "color_scheme": rng.choices(("light", "dark"), weights=(68, 32), k=1)[0],
            "contrast": contrast,
            "reduced_motion": rng.random() < 0.10,
            "reduced_transparency": rng.random() < (0.06 if platform_name == "macos" else 0.025),
            "reduced_data": bool(network.save_data),
            "forced_colors": forced_colors,
            "inverted_colors": rng.random() < 0.01,
            "monochrome_bits": 0,
            "color_gamut": color_gamut,
            "pointer": pointer,
            "any_pointer": pointer,
            "hover": hover,
            "any_hover": hover,
            "display_mode": "browser",
            "dynamic_range": dynamic_range,
            "video_dynamic_range": dynamic_range,
            "scripting": "enabled",
        },
        posture,
    )


def _v8_heap_limit_for_profile(
    physical_memory_gb: int,
    platform_name: str,
) -> int:
    return v8_150_precise_heap_size_limit(
        physical_memory_gb,
        platform_name,
    )


def _memory_values(
    rng: random.Random,
    physical_memory_gb: int,
    platform_name: str,
) -> tuple[str, int, int, int]:
    """Select one indivisible, evidence-backed V8 memory snapshot."""

    heap_limit = _v8_heap_limit_for_profile(
        physical_memory_gb,
        platform_name,
    )
    snapshot = choose_v8_memory_snapshot(rng, platform_name)
    return (
        snapshot.id,
        heap_limit,
        snapshot.total_js_heap_size,
        snapshot.used_js_heap_size,
    )


def _chromium_device_memory_gb(
    physical_memory_gb: int | float,
    chromium_major: int,
) -> float:
    """Return Chromium's coarse JS-visible device-memory bucket.

    The physical machine value remains available separately as
    ``physicalRamHintGb``.  Chromium rounds physical RAM to the nearest power
    of two (ties round down), then applies the browser-version ceiling: 8 GiB
    for the legacy surface and 32 GiB for the updated desktop surface.
    """

    physical = float(physical_memory_gb)
    if physical <= 0:
        raise ValueError("physical_memory_gb must be positive")
    lower = 1.0
    while lower * 2.0 <= physical:
        lower *= 2.0
    upper = lower * 2.0
    bucket = lower if physical - lower <= upper - physical else upper
    if chromium_major >= 147:
        return max(2.0, min(bucket, 32.0))
    return min(bucket, 8.0)


def _windows_platform_version(
    rng: random.Random,
    gpu: dict[str, object],
    hardware: dict[str, object],
) -> str:
    """Choose a Windows UA-CH version compatible with selected hardware."""

    tags = {str(item).lower() for item in hardware.get("tags", ())}
    physical_memory_gb = int(hardware.get("physicalRamHintGb", 0))
    architecture = str(gpu.get("architecture", "")).lower()
    model = str(gpu.get("model", "")).lower()
    if "arm64" in tags:
        return "15.0.0"
    if (
        physical_memory_gb < 4
        or str(gpu.get("tier", "")).lower() == "legacy"
        or tags & {"legacy", "obsolete", "lowend", "netbook"}
        or architecture == "gen-9"
        or any(
            name in model
            for name in (
                "iris(r) plus graphics 640",
                "iris(r) plus graphics 650",
                "hd graphics 620",
                "hd graphics 615",
            )
        )
    ):
        return "10.0.0"
    # Both supported Windows generations can use the frozen NT 10.0 UA.
    # Modern installations are weighted toward Windows 11 without deleting
    # the still-real Windows 10 population.
    return rng.choices(("15.0.0", "10.0.0"), weights=(4, 1), k=1)[0]


def _chromium_mobile_device_memory_gb(
    physical_memory_gb: int | float,
) -> float:
    """Return Android Chromium's coarse 1/2/4/8 GiB bucket."""

    physical = float(physical_memory_gb)
    if physical <= 0:
        raise ValueError("physical_memory_gb must be positive")
    lower = 1.0
    while lower * 2.0 <= physical:
        lower *= 2.0
    upper = lower * 2.0
    bucket = lower if physical - lower <= upper - physical else upper
    return min(bucket, 8.0)


def _accept_language(languages: Sequence[str]) -> str:
    values = []
    for index, language in enumerate(languages):
        if index == 0:
            values.append(language)
        else:
            # Chromium decrements q by 0.1 until it reaches 0.1, then keeps
            # subsequent entries at 0.1.
            quality = max(0.1, 1.0 - index * 0.1)
            values.append(f"{language};q={quality:.1f}")
    return ",".join(values)


def get_random_fp_details(
    country_code: str,
    user_agent: str | None = None,
    *,
    seed: int | None = None,
    time_zone: str | None = None,
    include_virtual_gpu: bool = False,
    include_external_mac_screen: bool = False,
    body_child_element_count: int | None = 2,
    body_client_height: float | None = 0.0,
    document_has_focus: bool | None = None,
    document_visibility_state: str | None = "visible",
    is_popup: bool | None = False,
) -> RandomFingerprint:
    """Return a country-aware Windows, macOS, or Android Mobile profile.

    ``country_code`` controls the linked locale, navigator language list,
    IANA time zone, Accept-Language value, and speech voices.  ``user_agent``
    controls browser and platform: Windows UA selects Windows hardware; Mac UA
    selects an Apple-silicon Mac with a complete Chromium-150 Metal capability
    record, a compatible Retina screen, and macOS fonts. Android Mobile UA
    selects a complete phone record with linked SoC GPU, RAM, screen, DPR,
    Android model, and AOSP fonts. If omitted, the fixed Chrome 150 Windows UA
    is used.
    ``body_child_element_count`` materializes that many real placeholder DIV
    nodes in the default BODY used by standalone ``evaluate``. An explicit
    ``body_client_height`` controls the matching BODY geometry observation.
    Both default to the fixed workload profile values 2 and 0.
    ``seed`` makes every catalog choice deterministic.  Chromium's frozen
    low-entropy Mac UA retains ``MacIntel`` while UA-CH is set to the selected
    Apple-silicon hardware architecture.
    """

    country = _require_country_code(country_code)
    resolved_seed = _resolve_seed(seed)
    rng = random.Random(resolved_seed)
    user_agent_profile, platform_name = _resolve_user_agent(user_agent)
    if body_child_element_count is not None:
        if (
            isinstance(body_child_element_count, bool)
            or not isinstance(body_child_element_count, int)
            or not 0 <= body_child_element_count <= 10_000
        ):
            raise ValueError("body_child_element_count must be an integer from 0 to 10000")
    if body_client_height is not None:
        if isinstance(body_client_height, bool):
            raise ValueError("body_client_height must be a finite non-negative number")
        body_client_height = float(body_client_height)
        if not math.isfinite(body_client_height) or not 0 <= body_client_height <= 10_000_000:
            raise ValueError("body_client_height must be a finite non-negative number")
    if document_has_focus is not None and not isinstance(document_has_focus, bool):
        raise ValueError("document_has_focus must be a bool or None")
    if document_visibility_state is not None and document_visibility_state not in {
        "visible",
        "hidden",
    }:
        raise ValueError("document_visibility_state must be visible, hidden, or None")
    if is_popup is not None and not isinstance(is_popup, bool):
        raise ValueError("is_popup must be a bool or None")
    selected_document_has_focus = _document_has_focus_value(
        resolved_seed,
        document_has_focus,
    )

    languages = tuple(choose_language_list(rng, country, include_secondary=True))
    if not languages:
        raise ValueError(f"no language profile available for {country}")
    selected_time_zone = choose_timezone_for_country(
        rng,
        country,
        timezone_hint=time_zone,
    )
    reference_location = get_time_zone_reference_location(
        selected_time_zone,
        country,
    )
    if reference_location is None:
        raise ValueError(
            f"no evidence-backed reference location for {country}/{selected_time_zone}"
        )

    selected_fonts: dict[str, object]
    windows_platform_version: str | None = None
    if platform_name == "windows":
        ua_tags = {
            str(item).lower() for item in user_agent_profile.get("tags", ())
        }
        arm64 = "arm64" in ua_tags
        gpu = _choose_gpu(
            rng,
            arm64=arm64,
            include_virtual_gpu=include_virtual_gpu,
        )
        hardware = dict(
            choose_pc_navigator_hardware_profile_for_gpu(
                rng,
                gpu,
                tag="arm64" if arm64 else "windows",
            )
        )
        # Keep the physical RAM choice distinct from Chromium's privacy-
        # coarsened navigator.deviceMemory value.  The sandbox still accepts
        # arbitrary explicit caller profiles; only generated profiles follow
        # Chromium's observable buckets.
        hardware["deviceMemory"] = _chromium_device_memory_gb(
            float(hardware["physicalRamHintGb"]),
            int(user_agent_profile.get("chromiumMajor", 0)),
        )
        screen = choose_pc_screen_profile_for_hardware(
            rng,
            hardware,
            tag="windows",
            gpu_profile=gpu,
        )
        windows_platform_version = _windows_platform_version(
            rng,
            gpu,
            hardware,
        )
        screen = materialize_pc_screen_profile_for_windows(
            screen,
            windows_platform_version,
            color_depth=choose_windows_screen_depth(rng),
        )
        selected_fonts = build_windows_font_profile(
            languages[0],
            windows_platform_version,
        )
    elif platform_name == "macos":
        gpu = choose_mac_gpu_candidate(
            rng,
            get_mac_gpu_candidates(verified_only=True),
        )
        memory_choices = tuple(int(item) for item in gpu.get("memoryChoicesGb", ()))
        if not memory_choices:
            raise ValueError("selected Mac GPU has no memory choices")
        # A chip such as M3/M4 is shared by portable Macs and iMacs. Select
        # the concrete device screen first; deriving the form factor from the
        # GPU's entire list of possible products incorrectly turned iMac
        # profiles into battery-powered laptops.
        screen = choose_mac_screen_profile_for_gpu(
            rng,
            gpu,
            include_external=include_external_mac_screen,
        )
        portable = bool(screen.get("hostPortable", screen.get("portable", False)))
        host_device_class = str(
            screen.get("hostDeviceClass", screen.get("deviceClass", "mac"))
        )
        memory_gb = rng.choice(memory_choices)
        hardware_tags = [
            "mac",
            "laptop" if portable else "desktop",
            "notouch",
            f"host-{host_device_class}",
        ]
        if bool(screen.get("hostHasCamera", portable)):
            hardware_tags.append("camera")
        hardware = {
            "id": (
                f"{gpu.get('id', 'mac')}_{memory_gb}gb_"
                f"host_{host_device_class}"
            ),
            "hardwareConcurrency": int(gpu.get("cpuCores", 8)),
            "deviceMemory": _chromium_device_memory_gb(
                memory_gb,
                int(user_agent_profile.get("chromiumMajor", 0)),
            ),
            "maxTouchPoints": 0,
            "physicalRamHintGb": memory_gb,
            "tags": tuple(hardware_tags),
        }
        selected_fonts = build_mac_font_profile(
            languages[0],
            str(gpu.get("macosPlatformVersion", "15.5.0")),
        )
    else:
        ua_data_raw = user_agent_profile.get("userAgentData", {})
        requested_model = (
            str(ua_data_raw.get("model", ""))
            if isinstance(ua_data_raw, dict)
            else ""
        )
        requested_android_version = (
            str(ua_data_raw.get("platformVersion", ""))
            if isinstance(ua_data_raw, dict)
            else ""
        )
        frozen_android_ua = bool(user_agent_profile.get("androidUaIsFrozen"))
        try:
            parsed_android_major = int(
                requested_android_version.split(".", 1)[0]
            )
        except ValueError as error:
            raise ValueError("Android UA has no numeric platform version") from error
        requested_android_major = None if frozen_android_ua else parsed_android_major
        requested_device_model = None if frozen_android_ua else requested_model
        device = choose_android_device_profile(
            rng,
            get_android_device_profiles(
                requested_device_model,
                android_version=requested_android_major,
                minimum_android_version=10,
            ),
        )
        selected_android_major = choose_android_version_for_device(
            rng,
            device,
            requested_android_major,
            minimum_android_version=10,
        )
        device = materialize_android_device_profile(
            device,
            selected_android_major,
        )
        memory_choices = tuple(
            int(item) for item in device.get("physicalMemoryChoicesGb", ())
        )
        if not memory_choices:
            raise ValueError("selected Android device has no memory choices")
        physical_memory_gb = rng.choice(memory_choices)
        gpu_raw = device.get("gpu", {})
        if not isinstance(gpu_raw, dict):
            raise ValueError("selected Android device has no GPU record")
        gpu = dict(gpu_raw)
        gpu.update(
            id=f"{device.get('id', 'android')}-gpu",
            tier=("legacy" if "legacy" in device.get("tags", ()) else "mobile"),
        )
        hardware = {
            "id": f"{device.get('id', 'android')}_{physical_memory_gb}gb",
            "hardwareConcurrency": int(device.get("hardwareConcurrency", 8)),
            "deviceMemory": _chromium_mobile_device_memory_gb(
                physical_memory_gb
            ),
            "maxTouchPoints": int(device.get("maxTouchPoints", 5)),
            "physicalRamHintGb": physical_memory_gb,
            "tags": (*tuple(device.get("tags", ())), "android", "touch"),
        }
        screen = device
        selected_fonts = build_android_font_profile(
            languages[0],
            selected_android_major,
            str(device.get("oem", "aosp")),
        )
    if platform_name == "macos":
        speech = {
            "id": (
                "macos-chromium150-local-plus-google-voices"
                if str(user_agent_profile.get("browser", "")) == "chrome"
                else "macos-chromium150-local-voices"
            )
        }
    elif platform_name == "android":
        # A clean Android Chromium document can have an empty voice list until
        # the platform speech service reports voices. Do not inject Windows
        # Microsoft voices into that lifecycle state.
        speech = {
            "id": "android-clean-profile-empty-voices",
            "voices": (),
        }
    else:
        speech = choose_speech_synthesis_voice_profile(
            rng,
            country,
            languages,
        )

    ua_data_raw = user_agent_profile.get("userAgentData", {})
    if not isinstance(ua_data_raw, dict):
        raise ValueError("selected user-agent profile has no UA-CH data")
    ua_data_values = dict(ua_data_raw)
    if platform_name == "windows":
        if windows_platform_version is None:
            raise RuntimeError("Windows platform version was not selected")
        ua_data_values["platformVersion"] = windows_platform_version
    elif platform_name == "macos":
        ua_data_values.update(
            {
                "platform": "macOS",
                "architecture": str(gpu.get("cpuArchitecture", "arm")),
                "bitness": str(gpu.get("cpuBitness", "64")),
                "model": "",
                "platformVersion": str(
                    gpu.get("macosPlatformVersion", "15.5.0")
                ),
                "wow64": False,
                "formFactors": ("Desktop",),
            }
        )
    elif platform_name == "android":
        android_platform_version = str(device.get("androidVersion", ""))
        android_version_parts = android_platform_version.split(".")[:3]
        if android_platform_version:
            android_platform_version = ".".join(
                android_version_parts + ["0"] * (3 - len(android_version_parts))
            )
        ua_data_values.update(
            {
                "platform": "Android",
                # Android Chromium 151 returns empty architecture/bitness in
                # UA-CH high entropy values on the Pixel 4 HTTPS capture.
                "architecture": "",
                "bitness": "",
                "model": str(screen.get("model", "")),
                "platformVersion": android_platform_version,
                "wow64": False,
                "mobile": True,
                "formFactors": ("Mobile",),
            }
        )
    screen_profile, window_profile = _screen_profiles(screen)
    # This workload explicitly models a non-rendered top-level viewport.
    # Preserve physical Screen and outer window geometry, while the inner and
    # visual viewport surfaces remain fixed at zero for every generated worker.
    screen_profile = replace(
        screen_profile,
        viewport_width=0.0,
        viewport_height=0.0,
    )
    window_profile = replace(
        window_profile,
        inner_width=0.0,
        inner_height=0.0,
    )

    webgl_data = gpu.get("webgl", {})
    if not isinstance(webgl_data, dict):
        raise ValueError("selected GPU profile has no WebGL data")
    # Canvas output is tied to the selected platform/font/rendering stack. A
    # seed-derived salt or arbitrary global text scale would create values that
    # do not correspond to any sourced device profile.
    canvas_salt = ""
    text_width_scale = 1.0

    locale_profile = LocaleProfile(
        locale=languages[0],
        time_zone=selected_time_zone,
        time_zone_offset_minutes=_time_zone_offset_minutes(selected_time_zone),
    )
    network_profile = _network_profile(resolved_seed, platform_name)
    has_been_active, is_active = _user_activation_values(resolved_seed)
    media_preference_values, device_posture = _media_preference_values(
        resolved_seed,
        platform_name,
        hardware,
        screen,
        network_profile,
    )
    navigator_profile = NavigatorProfile(
        user_agent=str(user_agent_profile.get("userAgent", "")),
        app_version=str(user_agent_profile.get("appVersion", "")),
        app_code_name="Mozilla",
        app_name="Netscape",
        platform=str(user_agent_profile.get("platform", "Win32")),
        product="Gecko",
        product_sub="20030107",
        vendor="Google Inc.",
        vendor_sub="",
        language=languages[0],
        languages=languages,
        hardware_concurrency=int(hardware.get("hardwareConcurrency", 8)),
        device_memory_gb=float(hardware.get("deviceMemory", 8)),
        max_touch_points=int(hardware.get("maxTouchPoints", 0)),
        cookie_enabled=True,
        on_line=True,
        webdriver=False,
        pdf_viewer_enabled=platform_name != "android",
        do_not_track=None,
        user_activation_has_been_active=has_been_active,
        user_activation_is_active=is_active,
        user_agent_data=_user_agent_data_profile(ua_data_values),
        network=network_profile,
    )
    speech_profile = (
        _mac_speech_profile(
            languages,
            str(user_agent_profile.get("browser", "")),
        )
        if platform_name == "macos"
        else _speech_profile(speech)
    )
    battery_profile = _battery_profile(rng, hardware, screen, gpu)
    timing_profile = TimingProfile(clock_step_ms=1, random_seed=resolved_seed)

    # Start from the project's fully typed Chrome 150 desktop profile so every
    # configurable surface is explicit.  Platform-linked values are replaced
    # below; nothing is allowed to fall through to the native DLL's synthetic
    # fallback devices or rendering identifiers.
    base_arguments = {
        "locale": languages[0],
        "hardware_concurrency": int(hardware["hardwareConcurrency"]),
        "device_memory_gb": float(hardware["deviceMemory"]),
        "screen_width": int(screen_profile.width or 0),
        "screen_height": int(screen_profile.height or 0),
        "avail_height": int(screen_profile.avail_height or 0),
        "inner_width": float(window_profile.inner_width or 0),
        "inner_height": float(window_profile.inner_height or 0),
        "device_pixel_ratio": float(screen_profile.device_pixel_ratio or 1),
        "font_families": tuple(str(item) for item in selected_fonts["families"]),
        "local_fonts": tuple(
            LocalFontProfile(str(item[0]), str(item[1]), str(item[2]), str(item[3]))
            for item in selected_fonts["localFonts"]
        ),
        "font_metrics": tuple(
            FontMetricProfile(str(item[0]), float(item[1]), bool(item[2]))
            for item in selected_fonts["metrics"]
        ),
        "allow_unknown_font_families": False,
    }
    if platform_name == "windows":
        base_profile = windows_edge_150_profile(
            windows_platform_version=str(ua_data_values["platformVersion"]),
            time_zone=selected_time_zone,
            time_zone_offset_minutes=locale_profile.time_zone_offset_minutes,
            **base_arguments,
        )
    elif platform_name == "macos":
        base_profile = mac_edge_150_profile(
            macos_platform_version=str(ua_data_values["platformVersion"]),
            time_zone=selected_time_zone,
            time_zone_offset_minutes=locale_profile.time_zone_offset_minutes,
            **base_arguments,
        )
    else:
        # The typed profile is platform-neutral; use the complete Chromium
        # baseline, then replace every Android-visible navigator, screen,
        # font, media, WebGL, WebGPU, audio, and UA-CH surface below.
        base_profile = windows_edge_150_profile(
            time_zone=selected_time_zone,
            time_zone_offset_minutes=locale_profile.time_zone_offset_minutes,
            **base_arguments,
        )
    physical_memory_gb = int(
        hardware.get("physicalRamHintGb", hardware.get("deviceMemory", 8))
    )
    quota_bytes, usage_bytes = _storage_values(rng, physical_memory_gb)
    memory_snapshot_profile_id, heap_limit, total_heap, used_heap = _memory_values(
        rng,
        physical_memory_gb,
        platform_name,
    )
    if platform_name == "android" and device.get("jsHeapSizeLimit") is not None:
        # Preserve the connected-device value only for that exact device row;
        # other Android devices continue through the V8 memory calculation.
        heap_limit = int(device["jsHeapSizeLimit"])
    canvas_profile = replace(
        base_profile.canvas,
        data_url_salt=canvas_salt,
        text_width_scale=text_width_scale,
    )
    if platform_name == "macos":
        canvas_profile = replace(
            canvas_profile,
            **MAC_CHROMIUM150_CANVAS,
        )
    shared_profile = replace(
        base_profile,
        locale=locale_profile,
        navigator=navigator_profile,
        screen=screen_profile,
        window=window_profile,
        canvas=canvas_profile,
        css=(
            _mac_css_for_screen(base_profile.css, screen)
            if platform_name == "macos"
            else _windows_css_for_screen_and_locale(
                base_profile.css,
                screen,
                languages[0],
                int(user_agent_profile.get("chromiumMajor", 150)),
            )
            if platform_name == "windows"
            else _android_css_for_device(
                base_profile.css,
                screen,
                languages[0],
                int(user_agent_profile.get("chromiumMajor", 150)),
            )
        ),
        document=(
            DocumentProfile(
                body_child_element_count=body_child_element_count,
                body_client_height=body_client_height,
                has_focus=selected_document_has_focus,
                visibility_state=document_visibility_state,
                is_popup=is_popup,
            )
            if any(
                value is not None
                for value in (
                    body_child_element_count,
                    body_client_height,
                    selected_document_has_focus,
                    document_visibility_state,
                    is_popup,
                )
            )
            else None
        ),
        audio=replace(
            base_profile.audio,
            noise_seed=resolved_seed,
        ),
        storage=replace(
            base_profile.storage,
            quota_bytes=quota_bytes,
            usage_bytes=usage_bytes,
            persisted=False,
        ),
        speech=speech_profile,
        fonts=_font_profile(selected_fonts),
        plugins=(
            replace(base_profile.plugins, plugins=())
            if platform_name == "android"
            else base_profile.plugins
        ),
        media=replace(
            base_profile.media,
            devices=_media_devices(platform_name, hardware),
        ),
        battery=battery_profile,
        # This is the selected IANA zone's published reference location, not
        # an assertion about a real user's measured position.  Permission is
        # still prompt, matching an untouched browser profile.
        geolocation=GeolocationProfile(
            latitude=reference_location[0],
            longitude=reference_location[1],
            altitude=None,
            accuracy=10_000.0,
            altitude_accuracy=None,
            heading=None,
            speed=None,
        ),
        hardware_devices=replace(
            base_profile.hardware_devices,
            gamepads=(),
            usb_devices=(),
            hid_devices=(),
            serial_ports=(),
            bluetooth_devices=(),
            bluetooth_available=(
                True
                if platform_name in {"macos", "android"}
                else (
                    bool(
                        {str(item).lower() for item in hardware.get("tags", ())}
                        & {"laptop", "touch", "convertible", "surface"}
                    )
                    or rng.random() < 0.55
                )
            ),
            keyboard_layout=tuple(
                KeyboardLayoutEntryProfile(code, value)
                for code, value in keyboard_layout_for_profile(
                    platform_name,
                    country,
                )
            ),
            device_posture=device_posture,
            midi_inputs=(),
            midi_outputs=(),
        ),
        media_preferences=replace(
            base_profile.media_preferences,
            **media_preference_values,
        ),
        timing=timing_profile,
        memory=replace(
            base_profile.memory,
            performance_js_heap_size_limit=heap_limit,
            performance_total_js_heap_size=total_heap,
            performance_used_js_heap_size=used_heap,
            console_js_heap_size_limit=heap_limit,
            console_total_js_heap_size=total_heap,
            console_used_js_heap_size=used_heap,
        ),
    )

    if platform_name == "windows":
        windows_sensor_available = bool(
            {str(item).lower() for item in hardware.get("tags", ())}
            & {"touch", "convertible", "surface"}
        )
        profile = replace(
            shared_profile,
            id=f"random-windows-{country.lower()}-{resolved_seed:016x}",
            webgl=replace(
                base_profile.webgl,
                unmasked_vendor=str(webgl_data.get("unmaskedVendor", "")),
                unmasked_renderer=str(webgl_data.get("unmaskedRenderer", "")),
                webgl1_extensions=_WINDOWS_WEBGL1_EXTENSIONS,
                webgl2_extensions=_WINDOWS_WEBGL2_EXTENSIONS,
                compressed_texture_formats=tuple(
                    int(item) for item in _WINDOWS_COMPRESSED_TEXTURE_FORMATS
                ),
                webgl2_max_samples=16,
                webgl2_max_combined_vertex_uniform_components=212_988,
                aliased_point_size_max=1024.0,
            ),
            webgpu=replace(
                base_profile.webgpu,
                available=bool(gpu.get("webgpuSupported", True)),
                vendor=str(gpu.get("vendor", "")),
                architecture=str(gpu.get("webgpuArchitecture", "")),
                # Keep a valid internal adapter identity. GPUAdapterInfo masks
                # device/description to empty strings while developerFeatures
                # is false, matching the sourced browser observation.
                device=str(gpu.get("deviceId", "")),
                description=str(gpu.get("model", "")),
                developer_features=False,
                subgroup_min_size=32,
                subgroup_max_size=(
                    64 if str(gpu.get("vendor", "")).lower() == "amd" else 32
                ),
                is_fallback_adapter=False,
                features=("bgra8unorm-storage", "texture-compression-bc"),
                max_compute_workgroup_storage_size=16_384,
            ),
            audio=replace(
                shared_profile.audio,
                sample_rate=rng.choice((44_100.0, 48_000.0, 48_000.0, 48_000.0)),
                max_channel_count=2,
                base_latency=0.01,
                output_latency=0.0,
            ),
            media_preferences=replace(
                shared_profile.media_preferences,
                color_gamut="srgb",
                dynamic_range="standard",
                video_dynamic_range="standard",
            ),
            sensors=replace(
                shared_profile.sensors,
                available=windows_sensor_available,
            ),
        )
    elif platform_name == "macos":
        mac_webgl_capabilities, mac_webgpu_capabilities = (
            build_mac_graphics_capabilities(gpu)
        )
        mac_webgl_profile = replace(
            base_profile.webgl,
            unmasked_vendor=str(webgl_data.get("unmaskedVendor", "")),
            unmasked_renderer=str(webgl_data.get("unmaskedRenderer", "")),
            **mac_webgl_capabilities,
        )
        profile = replace(
            shared_profile,
            id=f"random-macos-{country.lower()}-{resolved_seed:016x}",
            webgl=mac_webgl_profile,
            webgpu=replace(
                base_profile.webgpu,
                available=True,
                device=str(gpu.get("deviceMarker", "") or gpu.get("model", "")),
                description=str(gpu.get("model", "")),
                **mac_webgpu_capabilities,
            ),
            audio=replace(
                shared_profile.audio,
                sample_rate=float(MAC_CHROMIUM150_AUDIO["sample_rate"]),
                max_channel_count=int(
                    MAC_CHROMIUM150_AUDIO["max_channel_count"]
                ),
                base_latency=float(MAC_CHROMIUM150_AUDIO["base_latency"]),
                output_latency=float(MAC_CHROMIUM150_AUDIO["output_latency"]),
            ),
            media=_mac_media_profile(shared_profile.media),
            permissions=replace(
                shared_profile.permissions,
                speaker_selection="unsupported",
                top_level_storage_access="invalid-origin",
                window_management="prompt",
            ),
            sensors=replace(shared_profile.sensors, available=False),
            hardware_devices=replace(
                shared_profile.hardware_devices,
                keyboard_layout=tuple(
                    KeyboardLayoutEntryProfile(code, value)
                    for code, value in MAC_CHROMIUM150_KEYBOARD_LAYOUT
                ),
            ),
            media_preferences=replace(
                shared_profile.media_preferences,
                color_gamut=str(screen.get("colorGamut", "p3")),
                dynamic_range=str(screen.get("dynamicRange", "standard")),
                video_dynamic_range=str(screen.get("dynamicRange", "standard")),
            ),
        )
    else:
        android_webgl_capabilities, android_webgpu_capabilities = (
            build_android_graphics_capabilities(
                device,
                int(user_agent_profile.get("chromiumMajor", 150)),
            )
        )
        profile = replace(
            shared_profile,
            id=f"random-android-{country.lower()}-{resolved_seed:016x}",
            webgl=replace(
                base_profile.webgl,
                unmasked_vendor=str(webgl_data.get("unmaskedVendor", "")),
                unmasked_renderer=str(webgl_data.get("unmaskedRenderer", "")),
                **android_webgl_capabilities,
            ),
            webgpu=replace(
                base_profile.webgpu,
                **android_webgpu_capabilities,
            ),
            audio=replace(
                shared_profile.audio,
                sample_rate=48_000.0,
                max_channel_count=2,
                base_latency=0.0026666666666666666,
                output_latency=0.0,
            ),
            media=_android_media_profile(
                shared_profile.media,
                device,
                int(user_agent_profile.get("chromiumMajor", 150)),
            ),
            permissions=replace(
                shared_profile.permissions,
                # Untouched Pixel 4 Chromium 151 HTTPS permission states.
                # Sensor permissions are granted, user-data permissions stay
                # prompt, and unsupported/invalid descriptors preserve their
                # browser rejection branches.
                accelerometer="granted",
                background_sync="granted",
                camera="prompt",
                clipboard_read="prompt",
                clipboard_write="granted",
                geolocation="prompt",
                gyroscope="granted",
                magnetometer="granted",
                microphone="prompt",
                midi="prompt",
                notifications="prompt",
                payment_handler="granted",
                persistent_storage="prompt",
                speaker_selection="unsupported",
                storage_access="granted",
                top_level_storage_access="invalid-origin",
                window_management="denied",
            ),
            sensors=replace(shared_profile.sensors, available=True),
            media_preferences=replace(
                shared_profile.media_preferences,
                color_gamut="srgb",
                dynamic_range="standard",
                video_dynamic_range="standard",
            ),
        )

    headers = dict(user_agent_profile.get("headers", {}))
    headers["user-agent"] = str(user_agent_profile.get("userAgent", ""))
    headers["accept-language"] = _accept_language(languages)
    if platform_name == "windows":
        headers["sec-ch-ua-platform-version"] = (
            f'"{ua_data_values["platformVersion"]}"'
        )
    elif platform_name == "macos":
        headers["sec-ch-ua-platform"] = '"macOS"'
        headers["sec-ch-ua-arch"] = f'"{ua_data_values["architecture"]}"'
        headers["sec-ch-ua-bitness"] = f'"{ua_data_values["bitness"]}"'
        headers["sec-ch-ua-platform-version"] = (
            f'"{ua_data_values["platformVersion"]}"'
        )
    elif platform_name == "android":
        headers["sec-ch-ua-platform"] = '"Android"'
        headers["sec-ch-ua-arch"] = '""'
        headers["sec-ch-ua-bitness"] = '""'
        headers["sec-ch-ua-model"] = f'"{ua_data_values["model"]}"'
        headers["sec-ch-ua-platform-version"] = (
            f'"{ua_data_values["platformVersion"]}"'
        )
        headers["sec-ch-ua-mobile"] = "?1"
    fingerprint = RandomFingerprint(
        profile=profile,
        seed=resolved_seed,
        country_code=country,
        time_zone=selected_time_zone,
        browser=str(user_agent_profile.get("browser", "")),
        platform=platform_name,
        request_headers=tuple((str(key), str(value)) for key, value in headers.items()),
        user_agent_profile_id=str(user_agent_profile.get("id", "")),
        navigator_hardware_profile_id=str(hardware.get("id", "")),
        screen_profile_id=str(screen.get("id", "")),
        webgl_gpu_profile_id=str(gpu.get("id", "")),
        speech_voice_profile_id=str(speech.get("id", "")),
        font_profile_id=str(selected_fonts.get("id", "")),
        cpu_logical_processors=int(hardware.get("hardwareConcurrency", 0)),
        device_memory_gb=float(hardware.get("deviceMemory", 0)),
        physical_memory_gb=int(
            hardware.get("physicalRamHintGb", hardware.get("deviceMemory", 0))
        ),
        memory_snapshot_profile_id=memory_snapshot_profile_id,
        gpu_model=str(gpu.get("model", "")),
        gpu_core_count=(
            int(gpu["gpuCores"]) if "gpuCores" in gpu else None
        ),
        gpu_core_unit=str(gpu.get("gpuCoreUnit", "")),
        geolocation_reference=reference_location,
        resource_load=_build_resource_load_profile(rng),
    )
    selection_issues: list[str] = []
    if int(hardware.get("hardwareConcurrency", 0)) != fingerprint.cpu_logical_processors:
        selection_issues.append("selected hardware CPU count changed during composition")
    if int(hardware.get("physicalRamHintGb", 0)) != fingerprint.physical_memory_gb:
        selection_issues.append("selected hardware memory changed during composition")
    if platform_name == "macos":
        memory_choices = {int(item) for item in gpu.get("memoryChoicesGb", ())}
        if fingerprint.physical_memory_gb not in memory_choices:
            selection_issues.append("Mac memory is unavailable for the selected chip")
        if fingerprint.cpu_logical_processors != int(gpu.get("cpuCores", 0)):
            selection_issues.append("Mac CPU count is unavailable for the selected chip")
        screen_class = str(screen.get("deviceClass", "")).lower()
        host_screen_class = str(
            screen.get("hostDeviceClass", screen_class)
        ).lower()
        allowed_screen_classes = {
            str(item).lower() for item in gpu.get("screenClasses", ())
        }
        if host_screen_class not in allowed_screen_classes:
            selection_issues.append("Mac host device is unavailable for the selected chip")
        if screen_class not in allowed_screen_classes and screen_class != "external":
            selection_issues.append("Mac screen is unavailable for the selected chip")
        hardware_tags = {str(item).lower() for item in hardware.get("tags", ())}
        if bool(screen.get("hostPortable", screen.get("portable", False))) != (
            "laptop" in hardware_tags
        ):
            selection_issues.append("Mac form factor conflicts with selected host")
        if profile.webgpu.available is not True:
            selection_issues.append("Apple-silicon Mac lost its WebGPU adapter")
    elif platform_name == "windows":
        hardware_tags = {str(item).lower() for item in hardware.get("tags", ())}
        gpu_form_factor = str(gpu.get("formFactor", "mixed")).lower()
        hardware_portable = bool(
            hardware_tags & {"laptop", "touch", "convertible", "surface"}
        )
        if gpu_form_factor == "portable" and not hardware_portable:
            selection_issues.append("portable Windows GPU is paired with desktop hardware")
        if gpu_form_factor == "desktop" and hardware_portable:
            selection_issues.append("desktop Windows GPU is paired with portable hardware")
        if float(hardware.get("physicalRamHintGb", 0)) < 2:
            selection_issues.append("64-bit Windows profile is below 2 GiB RAM")
        if profile.webgpu.available is not bool(gpu.get("webgpuSupported", True)):
            selection_issues.append("Windows WebGPU availability conflicts with the adapter")
        platform_major = int(str(profile.navigator.user_agent_data.platform_version).split(".", 1)[0])
        expected_taskbar_height = 48 if platform_major >= 13 else 40
        observed_taskbar_height = int(profile.screen.height or 0) - int(
            profile.screen.avail_height or 0
        )
        if observed_taskbar_height != expected_taskbar_height:
            selection_issues.append(
                "Windows taskbar work area conflicts with UA-CH platform version"
            )
    elif platform_name == "android":
        if profile.webgpu.available is not bool(device.get("webgpuSupported", False)):
            selection_issues.append("Android WebGPU availability conflicts with the adapter")
    if selection_issues:
        raise RuntimeError(
            "inconsistent catalog selection: " + "; ".join(selection_issues)
        )
    validate_random_fp(fingerprint)
    return fingerprint


def get_random_fp(
    country_code: str,
    user_agent: str | None = None,
    *,
    seed: int | None = None,
    time_zone: str | None = None,
    include_virtual_gpu: bool = False,
    include_external_mac_screen: bool = False,
    body_child_element_count: int | None = 2,
    body_client_height: float | None = 0.0,
    document_has_focus: bool | None = None,
    document_visibility_state: str | None = "visible",
    is_popup: bool | None = False,
) -> EdgeProfile:
    """Return the typed profile expected by ``EdgeSandbox(profile=...)``.

    The default standalone BODY state is fixed at two children and a zero
    client height unless the caller explicitly overrides it.
    """

    return get_random_fp_details(
        country_code,
        user_agent,
        seed=seed,
        time_zone=time_zone,
        include_virtual_gpu=include_virtual_gpu,
        include_external_mac_screen=include_external_mac_screen,
        body_child_element_count=body_child_element_count,
        body_client_height=body_client_height,
        document_has_focus=document_has_focus,
        document_visibility_state=document_visibility_state,
        is_popup=is_popup,
    ).profile


def _javascript_number(value: float | int | None) -> str:
    if value is None:
        return "undefined"
    number = float(value)
    if number.is_integer():
        return str(int(number))
    return str(number)


def _font_probe_family(fingerprint: RandomFingerprint) -> str:
    fonts = fingerprint.profile.fonts
    if fonts and fonts.families:
        return str(fonts.families[-1])
    return "Arial"


def _javascript_string_literal(value: str) -> str:
    escaped = (
        value.replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("\r", "\\r")
        .replace("\n", "\\n")
        .replace("\u2028", "\\u2028")
        .replace("\u2029", "\\u2029")
    )
    return f'"{escaped}"'


def _expected_observations(fingerprint: RandomFingerprint) -> tuple[tuple[str, str], ...]:
    profile = fingerprint.profile
    navigator = profile.navigator
    screen = profile.screen
    window = profile.window
    webgl = profile.webgl
    webgpu = profile.webgpu
    speech = profile.speech
    ua_data = navigator.user_agent_data
    if ua_data is None:
        raise ValueError("profile has no user-agent data")
    brands = ua_data.brands or ()
    voices = speech.voices or ()
    return (
        ("userAgent", str(navigator.user_agent)),
        ("appVersion", str(navigator.app_version)),
        ("platform", str(navigator.platform)),
        ("language", str(navigator.language)),
        ("languages", ",".join(navigator.languages or ())),
        ("hardwareConcurrency", str(navigator.hardware_concurrency)),
        ("deviceMemory", _javascript_number(navigator.device_memory_gb)),
        ("maxTouchPoints", str(navigator.max_touch_points)),
        ("uaPlatform", str(ua_data.platform)),
        ("uaArchitecture", str(ua_data.architecture)),
        ("uaBitness", str(ua_data.bitness)),
        ("uaBrands", ",".join(f"{item.brand}:{item.version}" for item in brands)),
        ("timeZone", fingerprint.time_zone),
        ("screenWidth", str(screen.width)),
        ("screenHeight", str(screen.height)),
        ("availWidth", str(screen.avail_width)),
        ("availHeight", str(screen.avail_height)),
        ("colorDepth", str(screen.color_depth)),
        ("pixelDepth", str(screen.pixel_depth)),
        ("innerWidth", _javascript_number(window.inner_width)),
        ("innerHeight", _javascript_number(window.inner_height)),
        ("outerWidth", _javascript_number(window.outer_width)),
        ("outerHeight", _javascript_number(window.outer_height)),
        ("devicePixelRatio", _javascript_number(screen.device_pixel_ratio)),
        ("webglVendor", str(webgl.unmasked_vendor)),
        ("webglRenderer", str(webgl.unmasked_renderer)),
        ("webgpuVendor", str(webgpu.vendor)),
        ("webgpuArchitecture", str(webgpu.architecture)),
        (
            "webgpuDevice",
            str(webgpu.device) if webgpu.developer_features else "",
        ),
        (
            "webgpuDescription",
            str(webgpu.description) if webgpu.developer_features else "",
        ),
        ("voiceName", voices[0].name if voices else ""),
        ("voiceLanguage", voices[0].lang if voices else ""),
        ("fontProbeFamily", _font_probe_family(fingerprint)),
        ("fontAvailable", "true"),
    )


def _observation_matches(name: str, wanted: str, actual: str) -> bool:
    if wanted == actual:
        return True
    if name != "timeZone":
        return False
    return (
        _ICU_TIME_ZONE_ALIASES.get(wanted) == actual
        or _ICU_TIME_ZONE_ALIASES.get(actual) == wanted
    )


_FINGERPRINT_PROBE = r"""
(async () => {
  const canvas = document.createElement("canvas");
  const gl = canvas.getContext("webgl");
  const debug = gl.getExtension("WEBGL_debug_renderer_info");
  const adapter = await navigator.gpu.requestAdapter();
  const voices = speechSynthesis.getVoices();
  const hints = await navigator.userAgentData.getHighEntropyValues([
    "architecture", "bitness"
  ]);
  return [
    navigator.userAgent,
    navigator.appVersion,
    navigator.platform,
    navigator.language,
    navigator.languages.join(","),
    navigator.hardwareConcurrency,
    navigator.deviceMemory,
    navigator.maxTouchPoints,
    navigator.userAgentData.platform,
    hints.architecture,
    hints.bitness,
    navigator.userAgentData.brands.map(item => item.brand + ":" + item.version).join(","),
    Intl.DateTimeFormat().resolvedOptions().timeZone,
    screen.width,
    screen.height,
    screen.availWidth,
    screen.availHeight,
    screen.colorDepth,
    screen.pixelDepth,
    innerWidth,
    innerHeight,
    outerWidth,
    outerHeight,
    devicePixelRatio,
    gl.getParameter(debug.UNMASKED_VENDOR_WEBGL),
    gl.getParameter(debug.UNMASKED_RENDERER_WEBGL),
    adapter.info.vendor,
    adapter.info.architecture,
    adapter.info.device,
    adapter.info.description,
    voices.length ? voices[0].name : "",
    voices.length ? voices[0].lang : "",
    __FONT_PROBE_FAMILY__,
    document.fonts.check('12px "' + __FONT_PROBE_FAMILY__ + '"')
  ].join("\x1f");
})()
"""


def verify_random_fp(
    fingerprint: RandomFingerprint,
    *,
    library: Path | None = None,
) -> FingerprintVerification:
    """Start one worker and assert that all selected values were installed."""

    try:
        from edge_sandbox import EdgeSandbox
    except ImportError:
        from examples.run_sandbox import EdgeSandbox  # type: ignore

    expected = _expected_observations(fingerprint)
    with EdgeSandbox(library=library, profile=fingerprint.profile) as sandbox:
        probe = _FINGERPRINT_PROBE.replace(
            "__FONT_PROBE_FAMILY__",
            _javascript_string_literal(_font_probe_family(fingerprint)),
        )
        raw = sandbox.evaluate(
            probe,
            source_url=f"demo://random-fingerprint/{fingerprint.country_code.lower()}.js",
        )
    values = tuple(raw.split(_PROBE_SEPARATOR))
    if len(values) != len(expected):
        raise AssertionError(
            f"fingerprint probe returned {len(values)} values; expected {len(expected)}"
        )
    observations = tuple(
        (name, actual) for (name, _), actual in zip(expected, values)
    )
    mismatches = tuple(
        (name, wanted, actual)
        for (name, wanted), actual in zip(expected, values)
        if not _observation_matches(name, wanted, actual)
    )
    if mismatches:
        details = "; ".join(
            f"{name}: expected {wanted!r}, got {actual!r}"
            for name, wanted, actual in mismatches
        )
        raise AssertionError(details)
    return FingerprintVerification(
        fingerprint=fingerprint,
        observations=observations,
    )


def test_random_fp_combinations(
    country_codes: Iterable[str] = DEFAULT_TEST_COUNTRIES,
    user_agent: str | None = None,
    *,
    combinations_per_country: int = 1,
    seed: int = 0x150,
    library: Path | None = None,
) -> tuple[FingerprintVerification, ...]:
    """Verify several deterministic random combinations in isolated workers."""

    if combinations_per_country < 1:
        raise ValueError("combinations_per_country must be at least 1")
    master = random.Random(_resolve_seed(seed))
    output = []
    for country_code in country_codes:
        for _ in range(combinations_per_country):
            fingerprint = get_random_fp_details(
                country_code,
                user_agent,
                seed=master.randrange(0, 1 << 63),
            )
            output.append(verify_random_fp(fingerprint, library=library))
    return tuple(output)


__all__ = [
    "DEFAULT_ANDROID_EDGE_USER_AGENT",
    "DEFAULT_MAC_USER_AGENT",
    "DEFAULT_TEST_COUNTRIES",
    "DEFAULT_WINDOWS_USER_AGENT",
    "FingerprintVerification",
    "RandomFingerprint",
    "ResourceLoadProfile",
    "audit_random_fp",
    "get_random_fp",
    "get_random_fp_details",
    "test_random_fp_combinations",
    "validate_random_fp",
    "verify_random_fp",
]
