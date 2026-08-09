"""Coherent macOS fingerprint presets for the Edge sandbox.

The values below are a practical Apple-silicon test preset, not a claim that
every Mac exposes the same hardware values. Callers can replace the hardware
and screen parameters with evidence from their target machine.
"""

from __future__ import annotations

import os
import sys
from dataclasses import replace
from datetime import datetime
from pathlib import Path
from zoneinfo import ZoneInfo, ZoneInfoNotFoundError


PROJECT_ROOT = Path(__file__).resolve().parents[1]
project_root_text = str(PROJECT_ROOT)
if project_root_text not in sys.path:
    sys.path.insert(0, project_root_text)

try:
    from .edge_profile import (
        BatteryProfile,
        CanvasProfile,
        CssProfile,
        DocumentProfile,
        EdgeProfile,
        FontMetricProfile,
        FontProfile,
        HardwareDevicesProfile,
        KeyboardLayoutEntryProfile,
        LocalFontProfile,
        LocaleProfile,
        MediaDeviceProfile,
        MediaPreferencesProfile,
        MediaProfile,
        MemoryProfile,
        MimeTypeProfile,
        NavigatorProfile,
        NetworkProfile,
        PermissionsProfile,
        PerformanceProfile,
        PluginListProfile,
        PluginProfile,
        RtcCodecProfile,
        RtcHeaderExtensionProfile,
        ScreenProfile,
        SensorsProfile,
        SpeechProfile,
        SpeechVoiceProfile,
        StorageProfile,
        UserAgentBrandProfile,
        UserAgentDataProfile,
        WebAudioProfile,
        WebGlProfile,
        WebGpuProfile,
        WindowProfile,
        XrProfile,
    )
except ImportError:
    from edge_profile import (
        BatteryProfile,
        CanvasProfile,
        CssProfile,
        DocumentProfile,
        EdgeProfile,
        FontMetricProfile,
        FontProfile,
        HardwareDevicesProfile,
        KeyboardLayoutEntryProfile,
        LocalFontProfile,
        LocaleProfile,
        MediaDeviceProfile,
        MediaPreferencesProfile,
        MediaProfile,
        MemoryProfile,
        MimeTypeProfile,
        NavigatorProfile,
        NetworkProfile,
        PermissionsProfile,
        PerformanceProfile,
        PluginListProfile,
        PluginProfile,
        RtcCodecProfile,
        RtcHeaderExtensionProfile,
        ScreenProfile,
        SensorsProfile,
        SpeechProfile,
        SpeechVoiceProfile,
        StorageProfile,
        UserAgentBrandProfile,
        UserAgentDataProfile,
        WebAudioProfile,
        WebGlProfile,
        WebGpuProfile,
        WindowProfile,
        XrProfile,
    )

if (PROJECT_ROOT / "demo" / "fp" / "mac_chromium150_capture_catalog.py").is_file():
    from demo.fp.mac_chromium150_capture_catalog import (  # type: ignore
        CHROME_MAC_REMOTE_SPEECH_VOICES,
        MAC_CHROMIUM150_AUDIO,
        MAC_CHROMIUM150_CANVAS,
        MAC_CHROMIUM150_KEYBOARD_LAYOUT,
        MAC_CHROMIUM150_MEDIA_LISTS,
        MAC_CHROMIUM150_RTC_AUDIO_CODECS,
        MAC_CHROMIUM150_RTC_HEADER_EXTENSIONS,
        MAC_CHROMIUM150_RTC_VIDEO_CODECS,
        MAC_CHROMIUM150_WEBGL1_EXTENSIONS,
        MAC_CHROMIUM150_WEBGL2_EXTENSIONS,
        MAC_CHROMIUM150_WEBGPU_FEATURES,
        MAC_CHROMIUM150_WEBGPU_LIMITS,
        MACOS_LOCAL_SPEECH_VOICES,
    )
else:  # Installed wheel.
    from edge_sandbox.country_profiles.fp.mac_chromium150_capture_catalog import (
        CHROME_MAC_REMOTE_SPEECH_VOICES,
        MAC_CHROMIUM150_AUDIO,
        MAC_CHROMIUM150_CANVAS,
        MAC_CHROMIUM150_KEYBOARD_LAYOUT,
        MAC_CHROMIUM150_MEDIA_LISTS,
        MAC_CHROMIUM150_RTC_AUDIO_CODECS,
        MAC_CHROMIUM150_RTC_HEADER_EXTENSIONS,
        MAC_CHROMIUM150_RTC_VIDEO_CODECS,
        MAC_CHROMIUM150_WEBGL1_EXTENSIONS,
        MAC_CHROMIUM150_WEBGL2_EXTENSIONS,
        MAC_CHROMIUM150_WEBGPU_FEATURES,
        MAC_CHROMIUM150_WEBGPU_LIMITS,
        MACOS_LOCAL_SPEECH_VOICES,
    )


EDGE_150_FULL_VERSION = "150.0.0.0"
APPLE_GPU_NAME = "Apple M5 Pro"
APPLE_GPU_FAMILY = "apple10"
APPLE_WEBGPU_ARCHITECTURE = "metal-3"
MAC_PHYSICAL_MEMORY_GB = 24
MAC_HARDWARE_CONCURRENCY = 15
# Chromium 147+ desktop builds expose the updated 2/4/8/16/32-GiB buckets.
MAC_DEVICE_MEMORY_GB = 16.0
MAC_EDGE_150_USER_AGENT = (
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
    "AppleWebKit/537.36 (KHTML, like Gecko) "
    f"Chrome/{EDGE_150_FULL_VERSION} Safari/537.36"
)

_WINDOWS_TIME_ZONE_TO_IANA = {
    "China Standard Time": "Asia/Shanghai",
    "Tokyo Standard Time": "Asia/Tokyo",
    "Korea Standard Time": "Asia/Seoul",
    "Singapore Standard Time": "Asia/Singapore",
    "India Standard Time": "Asia/Kolkata",
    "Arabian Standard Time": "Asia/Dubai",
    "Russian Standard Time": "Europe/Moscow",
    "Turkey Standard Time": "Europe/Istanbul",
    "Israel Standard Time": "Asia/Jerusalem",
    "South Africa Standard Time": "Africa/Johannesburg",
    "W. Europe Standard Time": "Europe/Berlin",
    "Central Europe Standard Time": "Europe/Budapest",
    "Romance Standard Time": "Europe/Paris",
    "GMT Standard Time": "Europe/London",
    # Chromium/V8 canonicalizes the Windows UTC zone to the IANA primary ID.
    "UTC": "UTC",
    "Eastern Standard Time": "America/New_York",
    "Central Standard Time": "America/Chicago",
    "Mountain Standard Time": "America/Denver",
    "US Mountain Standard Time": "America/Phoenix",
    "Pacific Standard Time": "America/Los_Angeles",
    "Alaskan Standard Time": "America/Anchorage",
    "Hawaiian Standard Time": "Pacific/Honolulu",
    "AUS Eastern Standard Time": "Australia/Sydney",
    "E. Australia Standard Time": "Australia/Brisbane",
    "New Zealand Standard Time": "Pacific/Auckland",
}


def _current_offset_minutes() -> int:
    offset = datetime.now().astimezone().utcoffset()
    return -int(offset.total_seconds() / 60) if offset is not None else 0


def _fixed_offset_time_zone(offset_minutes: int) -> str:
    utc_minutes = -offset_minutes
    sign = "+" if utc_minutes >= 0 else "-"
    absolute = abs(utc_minutes)
    return f"GMT{sign}{absolute // 60:02d}:{absolute % 60:02d}"


def _windows_local_time_zone() -> str | None:
    if sys.platform != "win32":
        return None
    try:
        import winreg

        with winreg.OpenKey(
            winreg.HKEY_LOCAL_MACHINE,
            r"SYSTEM\CurrentControlSet\Control\TimeZoneInformation",
        ) as key:
            windows_name = str(winreg.QueryValueEx(key, "TimeZoneKeyName")[0]).rstrip(
                "\0"
            )
    except (OSError, ImportError):
        return None
    return 'America/New_York'


def local_time_zone() -> tuple[str, int]:
    """Return the host IANA time-zone ID and current JS offset in minutes."""

    candidates = (
        os.environ.get("TZ"),
        getattr(datetime.now().astimezone().tzinfo, "key", None),
        _windows_local_time_zone(),
    )
    for candidate in candidates:
        if not candidate:
            continue
        try:
            offset = datetime.now(ZoneInfo(candidate)).utcoffset()
        except ZoneInfoNotFoundError:
            continue
        offset_minutes = -int(offset.total_seconds() / 60) if offset is not None else 0
        return candidate, offset_minutes
    offset_minutes = _current_offset_minutes()
    return _fixed_offset_time_zone(offset_minutes), offset_minutes


def _time_zone_offset(time_zone: str) -> int:
    try:
        offset = datetime.now(ZoneInfo(time_zone)).utcoffset()
    except ZoneInfoNotFoundError as error:
        raise ValueError(f"unknown IANA time zone: {time_zone}") from error
    return -int(offset.total_seconds() / 60) if offset is not None else 0


# BC/S3TC/BPTC, ETC2/EAC, and linear/sRGB ASTC formats exposed by an
# Apple8 macOS device through Chromium 150's ANGLE Metal backend.
APPLE_M2_COMPRESSED_TEXTURE_FORMATS = (
    *range(0x83F0, 0x83F4),
    *range(0x8C4C, 0x8C50),
    *range(0x8E8C, 0x8E90),
    *range(0x9270, 0x927A),
    *range(0x93B0, 0x93BE),
    *range(0x93D0, 0x93DE),
)

MAC_FONT_FAMILIES = (
    "-apple-system",
    "BlinkMacSystemFont",
    "SF Pro Display",
    "SF Pro Text",
    "Helvetica Neue",
    "Helvetica",
    "Arial",
    "Times",
    "Times New Roman",
    "Menlo",
    "Monaco",
    "Apple Color Emoji",
)

MAC_LOCAL_FONTS = (
    LocalFontProfile(
        "SFProText-Regular", "SF Pro Text Regular", "SF Pro Text", "Regular"
    ),
    LocalFontProfile(
        "SFProDisplay-Regular",
        "SF Pro Display Regular",
        "SF Pro Display",
        "Regular",
    ),
    LocalFontProfile(
        "HelveticaNeue", "Helvetica Neue", "Helvetica Neue", "Regular"
    ),
    LocalFontProfile("Menlo-Regular", "Menlo Regular", "Menlo", "Regular"),
)

MAC_FONT_METRICS = (
    FontMetricProfile("SF Pro Text", 0.965),
    FontMetricProfile("SF Pro Display", 0.985),
    FontMetricProfile("Helvetica Neue", 0.972),
    FontMetricProfile("Helvetica", 0.98),
    FontMetricProfile("Menlo", 1.0, True),
    FontMetricProfile("Monaco", 1.0, True),
)

MAC_CSS = CssProfile(
    body="display:block;margin:8px",
    input_common=(
        "font-family:Arial;font-size:13.3333px;font-weight:400;"
        "line-height:normal;color:rgb(0, 0, 0);appearance:auto"
    ),
    input_hidden="display:none;width:auto;height:auto;padding:0;border-width:0",
    input_search="display:inline-block;box-sizing:border-box;width:177px;height:21px;padding:1px 2px;border-width:2px",
    input_checkbox_radio="display:inline-block;box-sizing:border-box;width:13px;height:13px;padding:0;border-width:0",
    input_range="display:inline-block;box-sizing:content-box;width:129px;height:16px;padding:0;border-width:0",
    input_color="display:inline-block;box-sizing:border-box;width:50px;height:27px;padding:1px 2px;border-width:1px",
    input_date="display:inline-block;box-sizing:content-box;width:113.328125px;height:19px;padding:0 0 0 1px;border-width:2px",
    input_time="display:inline-block;box-sizing:content-box;width:70px;height:20px;padding:0 0 0 1px;border-width:2px",
    input_datetime_local="display:inline-block;box-sizing:content-box;width:159.328125px;height:19px;padding:0 0 0 1px;border-width:2px",
    input_month="display:inline-block;box-sizing:content-box;width:107.328125px;height:19px;padding:0 0 0 1px;border-width:2px",
    input_week="display:inline-block;box-sizing:content-box;width:135.328125px;height:19px;padding:0 0 0 1px;border-width:2px",
    input_image="display:inline-block;box-sizing:content-box;width:0;height:0;padding:0;border-width:0",
    input_button="display:inline-block;box-sizing:border-box;width:16px;height:21px;padding:1px 6px;border-width:2px",
    input_submit_reset="display:inline-block;box-sizing:border-box;width:42.671875px;height:23px;padding:1px 6px;border-width:2px",
    input_file="display:inline-block;box-sizing:content-box;width:253px;height:23px;padding:0;border-width:0",
    input_text=(
        "display:inline-block;box-sizing:content-box;width:139px;height:15.5px;"
        "padding:1px 2px;border-width:2px;border-style:inset;"
        "border-color:rgb(118, 118, 118);background-color:rgb(255, 255, 255)"
    ),
)


def mac_css_for_device_pixel_ratio(device_pixel_ratio: float) -> CssProfile:
    """Return Chromium 150 macOS control geometry for the display scale."""

    if float(device_pixel_ratio) >= 1.5:
        width, height = 139.0, 15.5
    else:
        width, height = 145.0, 15.0
    return replace(
        MAC_CSS,
        input_text=(
            "display:inline-block;box-sizing:content-box;"
            f"width:{width:g}px;height:{height:g}px;"
            "padding:1px 2px;border-width:2px;border-style:inset;"
            "border-color:rgb(118, 118, 118);"
            "background-color:rgb(255, 255, 255)"
        ),
    )


def mac_edge_150_profile(
    *,
    macos_platform_version: str = "26.5.2",
    locale: str = "en-US",
    time_zone: str | None = None,
    time_zone_offset_minutes: int | None = None,
    hardware_concurrency: int = MAC_HARDWARE_CONCURRENCY,
    device_memory_gb: float = MAC_DEVICE_MEMORY_GB,
    screen_width: int = 1512,
    screen_height: int = 982,
    avail_height: int = 879,
    inner_width: float = 0.0,
    inner_height: float = 0.0,
    device_pixel_ratio: float = 2.0,
    font_families: tuple[str, ...] | None = None,
    local_fonts: tuple[LocalFontProfile, ...] | None = None,
    font_metrics: tuple[FontMetricProfile, ...] | None = None,
    allow_unknown_font_families: bool = False,
    local_fonts_permission: str = "prompt",
) -> EdgeProfile:
    """Return the regression-covered Apple-silicon Mac Edge 150 fingerprint.

    Chromium's compatibility-facing ``navigator.platform`` remains
    ``"MacIntel"`` while UA-CH reports ``"arm"``. An Intel preset is not
    offered because this project does not have an evidence-backed Intel Mac
    capture to regress against. The default is the captured 15-core M5 Pro
    profile with a 16-GiB Chromium device-memory value. When no
    time zone is supplied, the caller's host time zone is detected for every
    new profile instead of being frozen in the preset.
    """

    if not macos_platform_version:
        raise ValueError("macos_platform_version must not be empty")
    if screen_width <= 0 or screen_height <= 0 or avail_height <= 0:
        raise ValueError("Mac screen dimensions must be positive")
    if inner_width < 0 or inner_height < 0 or device_pixel_ratio <= 0:
        raise ValueError("Mac window dimensions and DPR are outside supported bounds")

    resolved_font_families = (
        MAC_FONT_FAMILIES if font_families is None else tuple(font_families)
    )
    resolved_local_fonts = MAC_LOCAL_FONTS if local_fonts is None else tuple(local_fonts)
    resolved_font_metrics = MAC_FONT_METRICS if font_metrics is None else tuple(font_metrics)
    if time_zone is None:
        resolved_time_zone, detected_offset = local_time_zone()
    else:
        resolved_time_zone = time_zone
        detected_offset = _time_zone_offset(time_zone)
    resolved_time_zone_offset = (
        detected_offset
        if time_zone_offset_minutes is None
        else time_zone_offset_minutes
    )
    if any(
        not isinstance(family, str) or not family or "\0" in family
        for family in resolved_font_families
    ):
        raise ValueError("font_families contains an invalid family name")
    if any(
        not isinstance(font, LocalFontProfile)
        or not font.postscript_name
        or not font.full_name
        or not font.family
        or not font.style
        or any(
            "\0" in value
            for value in (
                font.postscript_name,
                font.full_name,
                font.family,
                font.style,
            )
        )
        for font in resolved_local_fonts
    ):
        raise ValueError("local_fonts contains an invalid font record")

    renderer = (
        f"ANGLE (Apple, ANGLE Metal Renderer: {APPLE_GPU_NAME}, "
        "Unspecified Version)"
    )

    profile = EdgeProfile(
        id="macos-chrome-150-apple-m5-pro",
        locale=LocaleProfile(
            locale=locale,
            time_zone=resolved_time_zone,
            time_zone_offset_minutes=resolved_time_zone_offset,
        ),
        navigator=NavigatorProfile(
            user_agent=MAC_EDGE_150_USER_AGENT,
            app_version=MAC_EDGE_150_USER_AGENT.removeprefix("Mozilla/"),
            app_code_name="Mozilla",
            app_name="Netscape",
            platform="MacIntel",
            product="Gecko",
            product_sub="20030107",
            vendor="Google Inc.",
            vendor_sub="",
            language=locale,
            languages=(locale, "en"),
            hardware_concurrency=hardware_concurrency,
            device_memory_gb=device_memory_gb,
            max_touch_points=0,
            cookie_enabled=True,
            on_line=True,
            webdriver=False,
            pdf_viewer_enabled=True,
            do_not_track=None,
            user_activation_has_been_active=False,
            user_activation_is_active=False,
            user_agent_data=UserAgentDataProfile(
                brands=(
                    UserAgentBrandProfile("Not;A=Brand", "8", "8.0.0.0"),
                    UserAgentBrandProfile(
                        "Chromium", "150", EDGE_150_FULL_VERSION
                    ),
                    UserAgentBrandProfile(
                        "Google Chrome", "150", EDGE_150_FULL_VERSION
                    ),
                ),
                mobile=False,
                platform="macOS",
                architecture="arm",
                bitness="64",
                model="",
                platform_version=macos_platform_version,
                ua_full_version=EDGE_150_FULL_VERSION,
                wow64=False,
                form_factors=("Desktop",),
            ),
            network=NetworkProfile(
                effective_type="4g",
                rtt=50,
                downlink=10.0,
                save_data=False,
            ),
        ),
        screen=ScreenProfile(
            width=screen_width,
            height=screen_height,
            avail_width=screen_width,
            avail_height=avail_height,
            avail_left=0,
            avail_top=25,
            color_depth=30,
            pixel_depth=30,
            viewport_width=0,
            viewport_height=0,
            outer_width=0,
            outer_height=0,
            screen_x=0.0,
            screen_y=25.0,
            device_pixel_ratio=device_pixel_ratio,
            orientation_type="landscape-primary",
            orientation_angle=0,
            visual_viewport_offset_left=0.0,
            visual_viewport_offset_top=0.0,
            visual_viewport_page_left=0.0,
            visual_viewport_page_top=0.0,
            visual_viewport_scale=1.0,
        ),
        window=WindowProfile(
            inner_width=0,
            inner_height=0,
            outer_width=float(screen_width),
            outer_height=float(avail_height),
        ),
        canvas=CanvasProfile(
            data_url_salt="",
            **MAC_CHROMIUM150_CANVAS,
        ),
        webgl=WebGlProfile(
            vendor="WebKit",
            renderer="WebKit WebGL",
            unmasked_vendor="Google Inc. (Apple)",
            unmasked_renderer=renderer,
            webgl1_version="WebGL 1.0 (OpenGL ES 2.0 Chromium)",
            webgl1_shading_language_version=(
                "WebGL GLSL ES 1.0 (OpenGL ES GLSL ES 1.0 Chromium)"
            ),
            webgl2_version="WebGL 2.0 (OpenGL ES 3.0 Chromium)",
            webgl2_shading_language_version=(
                "WebGL GLSL ES 3.00 (OpenGL ES GLSL ES 3.0 Chromium)"
            ),
            webgl1_extensions=MAC_CHROMIUM150_WEBGL1_EXTENSIONS,
            webgl2_extensions=MAC_CHROMIUM150_WEBGL2_EXTENSIONS,
            compressed_texture_formats=APPLE_M2_COMPRESSED_TEXTURE_FORMATS,
            max_texture_size=16_384,
            max_cube_map_texture_size=16_384,
            max_renderbuffer_size=16_384,
            max_viewport_width=16_384,
            max_viewport_height=16_384,
            max_vertex_attribs=16,
            max_vertex_uniform_vectors=1_024,
            max_varying_vectors=30,
            max_fragment_uniform_vectors=1_024,
            max_vertex_texture_image_units=16,
            max_texture_image_units=16,
            max_combined_texture_image_units=32,
            subpixel_bits=4,
            webgl2_max_3d_texture_size=2_048,
            webgl2_max_array_texture_layers=2_048,
            webgl2_max_draw_buffers=8,
            webgl2_max_color_attachments=8,
            webgl2_max_samples=8,
            webgl2_max_vertex_uniform_components=4_096,
            webgl2_max_fragment_uniform_components=4_096,
            webgl2_max_varying_components=120,
            webgl2_max_vertex_output_components=120,
            webgl2_max_fragment_input_components=120,
            webgl2_max_vertex_uniform_blocks=16,
            webgl2_max_fragment_uniform_blocks=16,
            webgl2_max_combined_uniform_blocks=32,
            webgl2_max_uniform_buffer_bindings=32,
            webgl2_max_uniform_block_size=16_384,
            webgl2_max_combined_vertex_uniform_components=69_632,
            webgl2_max_combined_fragment_uniform_components=69_632,
            webgl2_max_transform_feedback_separate_attribs=4,
            webgl2_max_transform_feedback_interleaved_components=128,
            webgl2_max_transform_feedback_separate_components=4,
            webgl2_max_program_texel_offset=7,
            webgl2_max_elements_vertices=2_147_483_647,
            webgl2_max_elements_indices=2_147_483_647,
            webgl2_max_element_index=4_294_967_294,
            webgl2_max_texture_lod_bias=15.0,
            max_anisotropy=16.0,
            aliased_point_size_min=1.0,
            aliased_point_size_max=511.0,
            aliased_line_width_min=1.0,
            aliased_line_width_max=1.0,
            shader_precision_range_min=127,
            shader_precision_range_max=127,
            shader_precision_bits=23,
            context_alpha=True,
            context_antialias=True,
            context_depth=True,
            context_desynchronized=False,
            context_fail_if_major_performance_caveat=False,
            context_premultiplied_alpha=True,
            context_preserve_drawing_buffer=False,
            context_stencil=False,
            context_xr_compatible=False,
            context_power_preference="default",
        ),
        webgpu=WebGpuProfile(
            available=True,
            vendor="apple",
            architecture=APPLE_WEBGPU_ARCHITECTURE,
            device=APPLE_GPU_NAME,
            description=f"{APPLE_GPU_NAME} Metal adapter",
            developer_features=False,
            subgroup_min_size=32,
            subgroup_max_size=32,
            is_fallback_adapter=False,
            features=MAC_CHROMIUM150_WEBGPU_FEATURES,
            max_texture_dimension_1d=16_384,
            max_texture_dimension_2d=16_384,
            max_texture_dimension_3d=2_048,
            max_texture_array_layers=2_048,
            max_bind_groups=4,
            max_bind_groups_plus_vertex_buffers=24,
            max_bindings_per_bind_group=1_000,
            max_dynamic_uniform_buffers_per_pipeline_layout=10,
            max_dynamic_storage_buffers_per_pipeline_layout=8,
            max_sampled_textures_per_shader_stage=48,
            max_samplers_per_shader_stage=16,
            max_storage_buffers_per_shader_stage=10,
            max_storage_textures_per_shader_stage=8,
            max_uniform_buffers_per_shader_stage=12,
            max_uniform_buffer_binding_size=65_536,
            max_storage_buffer_binding_size=4_294_967_292,
            min_uniform_buffer_offset_alignment=256,
            min_storage_buffer_offset_alignment=256,
            max_vertex_buffers=8,
            max_buffer_size=4_294_967_292,
            max_vertex_attributes=30,
            max_vertex_buffer_array_stride=2_048,
            max_inter_stage_shader_variables=28,
            max_color_attachments=8,
            max_color_attachment_bytes_per_sample=128,
            max_compute_workgroup_storage_size=32_768,
            max_compute_invocations_per_workgroup=1_024,
            max_compute_workgroup_size_x=1_024,
            max_compute_workgroup_size_y=1_024,
            max_compute_workgroup_size_z=64,
            max_compute_workgroups_per_dimension=65_535,
            max_immediate_size=int(
                MAC_CHROMIUM150_WEBGPU_LIMITS["max_immediate_size"]
            ),
            max_storage_buffers_in_fragment_stage=10,
            max_storage_textures_in_fragment_stage=8,
            max_storage_buffers_in_vertex_stage=10,
            max_storage_textures_in_vertex_stage=8,
        ),
        audio=WebAudioProfile(
            sample_rate=float(MAC_CHROMIUM150_AUDIO["sample_rate"]),
            max_channel_count=int(MAC_CHROMIUM150_AUDIO["max_channel_count"]),
            base_latency=float(MAC_CHROMIUM150_AUDIO["base_latency"]),
            output_latency=float(MAC_CHROMIUM150_AUDIO["output_latency"]),
            noise_seed=0x4D41434F53,
        ),
        storage=StorageProfile(
            quota_bytes=128 * 1024 * 1024 * 1024,
            usage_bytes=96 * 1024 * 1024,
            persisted=False,
        ),
        speech=SpeechProfile(
            voices=(
                SpeechVoiceProfile(
                    voice_uri="com.apple.voice.compact.en-US.Samantha",
                    name="Samantha",
                    lang="en-US",
                    local_service=True,
                    is_default=True,
                ),
                SpeechVoiceProfile(
                    voice_uri="com.apple.voice.compact.en-US.Alex",
                    name="Alex",
                    lang="en-US",
                    local_service=True,
                    is_default=False,
                ),
            )
        ),
        fonts=FontProfile(
            families=resolved_font_families,
            allow_unknown_families=allow_unknown_font_families,
            local_fonts=resolved_local_fonts,
            metrics=resolved_font_metrics,
        ),
        css=mac_css_for_device_pixel_ratio(device_pixel_ratio),
        document=DocumentProfile(
            body_child_element_count=0,
            body_client_height=0.0,
            has_focus=True,
            visibility_state="visible",
            is_popup=False,
        ),
        media=MediaProfile(
            devices=(
                MediaDeviceProfile("default", "audioinput", "", "mac-audio"),
                MediaDeviceProfile("default", "audiooutput", "", "mac-audio"),
                MediaDeviceProfile("facetime-camera", "videoinput", "", "mac-camera"),
            ),
            supported_constraints=(
                "aspectRatio",
                "autoGainControl",
                "backgroundBlur",
                "channelCount",
                "deviceId",
                "displaySurface",
                "echoCancellation",
                "facingMode",
                "frameRate",
                "groupId",
                "height",
                "latency",
                "logicalSurface",
                "noiseSuppression",
                "resizeMode",
                "sampleRate",
                "sampleSize",
                "suppressLocalAudioPlayback",
                "voiceIsolation",
                "width",
            ),
            can_play_probably_types=(
                "audio/mp4",
                "audio/mpeg",
                "audio/wav",
                "audio/webm",
                "video/mp4",
                "video/webm",
            ),
            can_play_maybe_types=("audio/*", "video/*"),
            media_source_types=(
                "audio/mp4;*",
                "audio/webm;*",
                "video/mp4;*",
                "video/webm;*",
            ),
            media_recorder_types=(
                "",
                "audio/mp4",
                "audio/webm",
                "video/mp4",
                "video/webm",
            ),
            decoding_supported_types=("*",),
            decoding_smooth_types=("*",),
            decoding_power_efficient_types=(
                "audio/mp4",
                "video/mp4",
            ),
            encoding_supported_types=(
                "audio/mp4",
                "audio/webm",
                "video/mp4",
                "video/webm",
            ),
            encoding_smooth_types=(
                "audio/mp4",
                "audio/webm",
                "video/mp4",
                "video/webm",
            ),
            encoding_power_efficient_types=("audio/mp4", "video/mp4"),
            image_decoder_types=(
                "image/avif",
                "image/gif",
                "image/jpeg",
                "image/png",
                "image/webp",
            ),
            audio_decoder_codecs=("mp4a.*", "flac", "mp3", "opus", "vorbis"),
            audio_encoder_codecs=("mp4a.*", "flac", "opus"),
            video_decoder_codecs=(
                "av01.*",
                "avc1.*",
                "h264",
                "vp8",
                "vp09.*",
                "vp9",
            ),
            video_encoder_codecs=("avc1.*", "h264", "vp8", "vp09.*", "vp9"),
            rtc_audio_codecs=(
                RtcCodecProfile("audio/opus", 48_000, channels=2),
                RtcCodecProfile("audio/red", 48_000, channels=2),
                RtcCodecProfile("audio/G722", 8_000, channels=1),
                RtcCodecProfile("audio/PCMU", 8_000, channels=1),
                RtcCodecProfile("audio/PCMA", 8_000, channels=1),
            ),
            rtc_video_codecs=(
                RtcCodecProfile("video/VP8", 90_000),
                RtcCodecProfile(
                    "video/VP9", 90_000, sdp_fmtp_line="profile-id=0"
                ),
                RtcCodecProfile(
                    "video/AV1", 90_000, sdp_fmtp_line="profile=0;level-idx=5;tier=0"
                ),
                RtcCodecProfile(
                    "video/H264",
                    90_000,
                    sdp_fmtp_line=(
                        "level-asymmetry-allowed=1;packetization-mode=1;"
                        "profile-level-id=42001f"
                    ),
                ),
            ),
            rtc_header_extensions=(
                RtcHeaderExtensionProfile(
                    "audio", "urn:ietf:params:rtp-hdrext:ssrc-audio-level"
                ),
                RtcHeaderExtensionProfile(
                    "video", "urn:ietf:params:rtp-hdrext:sdes:mid"
                ),
                RtcHeaderExtensionProfile(
                    "video",
                    "http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time",
                ),
            ),
            rtc_offer_sdp="",
            rtc_answer_sdp="",
        ),
        permissions=PermissionsProfile(
            accelerometer="granted",
            background_sync="granted",
            camera="prompt",
            clipboard_read="prompt",
            clipboard_write="granted",
            microphone="prompt",
            geolocation="prompt",
            gyroscope="granted",
            notifications="prompt",
            local_fonts=local_fonts_permission,
            magnetometer="granted",
            midi="prompt",
            payment_handler="granted",
            persistent_storage="granted",
            speaker_selection="prompt",
            storage_access="granted",
            top_level_storage_access="granted",
            window_management="granted",
        ),
        battery=BatteryProfile(
            charging=False,
            charging_time=float("inf"),
            discharging_time=14_400.0,
            level=0.87,
        ),
        media_preferences=MediaPreferencesProfile(
            color_scheme="light",
            contrast="no-preference",
            reduced_motion=False,
            reduced_transparency=False,
            reduced_data=False,
            forced_colors=False,
            inverted_colors=False,
            monochrome_bits=0,
            color_gamut="p3",
            pointer="fine",
            any_pointer="fine",
            hover="hover",
            any_hover="hover",
            display_mode="browser",
            dynamic_range="high",
            video_dynamic_range="high",
            scripting="enabled",
        ),
        plugins=PluginListProfile(
            plugins=tuple(
                PluginProfile(
                    name=name,
                    filename="internal-pdf-viewer",
                    description="Portable Document Format",
                    mime_types=(
                        MimeTypeProfile(
                            "application/pdf", "pdf", "Portable Document Format"
                        ),
                        MimeTypeProfile(
                            "text/pdf", "pdf", "Portable Document Format"
                        ),
                    ),
                )
                for name in (
                    "PDF Viewer",
                    "Chrome PDF Viewer",
                    "Chromium PDF Viewer",
                    "Microsoft Edge PDF Viewer",
                    "WebKit built-in PDF",
                )
            )
        ),
        hardware_devices=HardwareDevicesProfile(
            gamepads=(),
            usb_devices=(),
            hid_devices=(),
            serial_ports=(),
            bluetooth_available=True,
            bluetooth_devices=(),
            keyboard_layout=(),
            device_posture="continuous",
            midi_inputs=(),
            midi_outputs=(),
            midi_sysex_enabled=False,
        ),
        sensors=SensorsProfile(
            available=False,
            accelerometer=(0.0, 0.0, 0.0),
            gravity=(0.0, 0.0, 0.0),
            linear_acceleration=(0.0, 0.0, 0.0),
            gyroscope=(0.0, 0.0, 0.0),
            absolute_orientation_quaternion=(0.0, 0.0, 0.0, 1.0),
            relative_orientation_quaternion=(0.0, 0.0, 0.0, 1.0),
        ),
        xr=XrProfile(supported_session_modes=("inline",)),
        memory=MemoryProfile(
            performance_js_heap_size_limit=4_395_630_592,
            performance_total_js_heap_size=189_287_527,
            performance_used_js_heap_size=180_511_835,
            console_js_heap_size_limit=4_395_630_592,
            console_total_js_heap_size=189_287_527,
            console_used_js_heap_size=180_511_835,
        ),
        performance=PerformanceProfile(
            entries=None,
            evaluated_script_content_encoding="zstd",
        ),
    )
    primary_language = locale.replace("_", "-").lower()
    local_voice_rows = tuple(
        row
        for row in MACOS_LOCAL_SPEECH_VOICES
        if primary_language.startswith("zh-") or " (" not in row[1]
    )
    voice_rows = local_voice_rows + tuple(CHROME_MAC_REMOTE_SPEECH_VOICES)
    preferred_voice = {
        "en-us": "Samantha",
        "en-gb": "Daniel",
        "zh-cn": "婷婷",
        "zh-tw": "美嘉",
        "yue-hk": "善怡",
    }.get(primary_language)
    default_index = next(
        (
            index
            for index, row in enumerate(voice_rows)
            if row[0].lower() == primary_language and row[1] == preferred_voice
        ),
        -1,
    )
    if default_index < 0:
        default_index = next(
            (
                index
                for index, row in enumerate(voice_rows)
                if row[0].lower() == primary_language
            ),
            0,
        )
    ordered_voice_rows = (
        (voice_rows[default_index],)
        + voice_rows[:default_index]
        + voice_rows[default_index + 1 :]
    )
    captured_media = MAC_CHROMIUM150_MEDIA_LISTS
    return replace(
        profile,
        speech=SpeechProfile(
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
                ) in enumerate(ordered_voice_rows)
            )
        ),
        media=replace(
            profile.media,
            devices=(
                MediaDeviceProfile("", "audioinput", "", ""),
                MediaDeviceProfile("", "videoinput", "", ""),
                MediaDeviceProfile("", "audiooutput", "", ""),
            ),
            supported_constraints=captured_media["supported_constraints"],
            can_play_probably_types=captured_media["can_play_probably_types"],
            can_play_maybe_types=captured_media["can_play_maybe_types"],
            media_source_types=captured_media["media_source_types"],
            media_recorder_types=captured_media["media_recorder_types"],
            decoding_supported_types=captured_media["decoding_supported_types"],
            decoding_smooth_types=captured_media["decoding_smooth_types"],
            decoding_power_efficient_types=(
                captured_media["decoding_power_efficient_types"]
            ),
            encoding_supported_types=captured_media["encoding_supported_types"],
            encoding_smooth_types=captured_media["encoding_smooth_types"],
            encoding_power_efficient_types=(
                captured_media["encoding_power_efficient_types"]
            ),
            image_decoder_types=captured_media["image_decoder_types"],
            audio_decoder_codecs=captured_media["audio_decoder_codecs"],
            audio_encoder_codecs=captured_media["audio_encoder_codecs"],
            video_decoder_codecs=captured_media["video_decoder_codecs"],
            video_encoder_codecs=captured_media["video_encoder_codecs"],
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
        ),
        permissions=replace(
            profile.permissions,
            window_management="prompt",
        ),
        hardware_devices=replace(
            profile.hardware_devices,
            keyboard_layout=tuple(
                KeyboardLayoutEntryProfile(code, value)
                for code, value in MAC_CHROMIUM150_KEYBOARD_LAYOUT
            ),
        ),
        memory=replace(
            profile.memory,
            performance_js_heap_size_limit=4_395_630_592,
            console_js_heap_size_limit=4_395_630_592,
        ),
    )
