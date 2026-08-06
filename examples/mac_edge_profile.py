"""Coherent macOS fingerprint presets for the Edge sandbox.

The values below are a practical Apple-silicon test preset, not a claim that
every Mac exposes the same hardware values. Callers can replace the hardware
and screen parameters with evidence from their target machine.
"""

from __future__ import annotations

import os
import sys
from datetime import datetime
from zoneinfo import ZoneInfo, ZoneInfoNotFoundError

try:
    from .edge_profile import (
        BatteryProfile,
        CanvasProfile,
        CssProfile,
        EdgeProfile,
        FontMetricProfile,
        FontProfile,
        HardwareDevicesProfile,
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
        EdgeProfile,
        FontMetricProfile,
        FontProfile,
        HardwareDevicesProfile,
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


EDGE_150_FULL_VERSION = "150.0.0.0"
APPLE_GPU_NAME = "Apple M2 Pro"
APPLE_GPU_FAMILY = "apple8"
MAC_PHYSICAL_MEMORY_GB = 32
MAC_HARDWARE_CONCURRENCY = 10
# Chromium 147+ desktop builds expose the updated 2/4/8/16/32-GiB buckets.
MAC_DEVICE_MEMORY_GB = 32.0
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
    return _WINDOWS_TIME_ZONE_TO_IANA.get(windows_name)


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


# ETC2/EAC plus linear and sRGB ASTC formats exposed by Apple8/Metal.
APPLE_M2_COMPRESSED_TEXTURE_FORMATS = (
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
    input_text="display:inline-block;box-sizing:content-box;width:169px;height:15px;padding:1px 2px;border-width:2px",
)


def mac_edge_150_profile(
    *,
    macos_platform_version: str = "15.5.0",
    locale: str = "en-US",
    time_zone: str | None = None,
    time_zone_offset_minutes: int | None = None,
    hardware_concurrency: int = MAC_HARDWARE_CONCURRENCY,
    device_memory_gb: float = MAC_DEVICE_MEMORY_GB,
    screen_width: int = 1512,
    screen_height: int = 982,
    avail_height: int = 944,
    inner_width: float = 1440.0,
    inner_height: float = 820.0,
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
    capture to regress against. The default hardware is a base 10-core,
    32-GiB M2 Pro; Chromium exposes its device-memory bucket as 32 GiB. When no
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

    return EdgeProfile(
        id="macos-chrome-150-apple-m2-pro",
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
            color_depth=24,
            pixel_depth=24,
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
            inner_width=inner_width,
            inner_height=inner_height,
            outer_width=float(screen_width),
            outer_height=float(avail_height),
        ),
        canvas=CanvasProfile(
            data_url_salt="macos-edge-150-apple-m2-metal",
            text_width_scale=1.0,
            actual_bounding_box_left=0.0,
            actual_bounding_box_right_scale=1.0,
            font_bounding_box_ascent=12.0,
            font_bounding_box_descent=3.0,
            actual_bounding_box_ascent=8.0,
            actual_bounding_box_descent=2.0,
            hanging_baseline=9.6,
            alphabetic_baseline=0.0,
            ideographic_baseline=-1.2,
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
            webgl1_extensions=(
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
                "WEBGL_color_buffer_float",
                "WEBGL_compressed_texture_astc",
                "WEBGL_compressed_texture_etc",
                "WEBGL_debug_renderer_info",
                "WEBGL_depth_texture",
                "WEBGL_draw_buffers",
                "WEBGL_lose_context",
                "WEBGL_multi_draw",
            ),
            webgl2_extensions=(
                "EXT_color_buffer_float",
                "EXT_color_buffer_half_float",
                "EXT_float_blend",
                "EXT_texture_filter_anisotropic",
                "OES_draw_buffers_indexed",
                "OES_texture_float_linear",
                "WEBGL_compressed_texture_astc",
                "WEBGL_compressed_texture_etc",
                "WEBGL_debug_renderer_info",
                "WEBGL_lose_context",
                "WEBGL_multi_draw",
            ),
            compressed_texture_formats=APPLE_M2_COMPRESSED_TEXTURE_FORMATS,
            max_texture_size=16_384,
            max_cube_map_texture_size=16_384,
            max_renderbuffer_size=16_384,
            max_viewport_width=32_767,
            max_viewport_height=32_767,
            max_vertex_attribs=16,
            max_vertex_uniform_vectors=4_096,
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
            webgl2_max_samples=4,
            webgl2_max_vertex_uniform_components=16_384,
            webgl2_max_fragment_uniform_components=4_096,
            webgl2_max_varying_components=120,
            webgl2_max_vertex_output_components=120,
            webgl2_max_fragment_input_components=120,
            webgl2_max_vertex_uniform_blocks=12,
            webgl2_max_fragment_uniform_blocks=12,
            webgl2_max_combined_uniform_blocks=24,
            webgl2_max_uniform_buffer_bindings=24,
            webgl2_max_uniform_block_size=65_536,
            webgl2_max_combined_vertex_uniform_components=212_992,
            webgl2_max_combined_fragment_uniform_components=200_704,
            webgl2_max_transform_feedback_separate_attribs=4,
            webgl2_max_transform_feedback_interleaved_components=120,
            webgl2_max_transform_feedback_separate_components=4,
            webgl2_max_program_texel_offset=7,
            webgl2_max_elements_vertices=2_147_483_647,
            webgl2_max_elements_indices=2_147_483_647,
            webgl2_max_element_index=4_294_967_294,
            webgl2_max_texture_lod_bias=2.0,
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
            vendor="apple",
            architecture=APPLE_GPU_FAMILY,
            device=APPLE_GPU_NAME,
            description=f"{APPLE_GPU_NAME} Metal adapter",
            developer_features=False,
            subgroup_min_size=32,
            subgroup_max_size=32,
            is_fallback_adapter=False,
            features=(
                "bgra8unorm-storage",
                "texture-compression-astc",
                "texture-compression-etc2",
            ),
            max_texture_dimension_1d=16_384,
            max_texture_dimension_2d=16_384,
            max_texture_dimension_3d=2_048,
            max_texture_array_layers=2_048,
            max_bind_groups=4,
            max_bind_groups_plus_vertex_buffers=24,
            max_bindings_per_bind_group=1_000,
            max_dynamic_uniform_buffers_per_pipeline_layout=8,
            max_dynamic_storage_buffers_per_pipeline_layout=4,
            max_sampled_textures_per_shader_stage=16,
            max_samplers_per_shader_stage=16,
            max_storage_buffers_per_shader_stage=8,
            max_storage_textures_per_shader_stage=4,
            max_uniform_buffers_per_shader_stage=12,
            max_uniform_buffer_binding_size=65_536,
            max_storage_buffer_binding_size=134_217_728,
            min_uniform_buffer_offset_alignment=256,
            min_storage_buffer_offset_alignment=256,
            max_vertex_buffers=8,
            max_buffer_size=268_435_456,
            max_vertex_attributes=16,
            max_vertex_buffer_array_stride=2_048,
            max_inter_stage_shader_variables=16,
            max_color_attachments=8,
            max_color_attachment_bytes_per_sample=32,
            max_compute_workgroup_storage_size=32_768,
            max_compute_invocations_per_workgroup=256,
            max_compute_workgroup_size_x=256,
            max_compute_workgroup_size_y=256,
            max_compute_workgroup_size_z=64,
            max_compute_workgroups_per_dimension=65_535,
            max_immediate_size=0,
            max_storage_buffers_in_fragment_stage=8,
            max_storage_textures_in_fragment_stage=4,
            max_storage_buffers_in_vertex_stage=8,
            max_storage_textures_in_vertex_stage=4,
        ),
        audio=WebAudioProfile(
            sample_rate=48_000.0,
            max_channel_count=2,
            base_latency=128.0 / 48_000.0,
            output_latency=256.0 / 48_000.0,
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
        css=MAC_CSS,
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
            accelerometer=(0.0, 0.0, 0.0),
            gravity=(0.0, 0.0, 0.0),
            linear_acceleration=(0.0, 0.0, 0.0),
            gyroscope=(0.0, 0.0, 0.0),
            absolute_orientation_quaternion=(0.0, 0.0, 0.0, 1.0),
            relative_orientation_quaternion=(0.0, 0.0, 0.0, 1.0),
        ),
        xr=XrProfile(supported_session_modes=("inline",)),
        memory=MemoryProfile(
            performance_js_heap_size_limit=4_294_705_152,
            performance_total_js_heap_size=13_061_022,
            performance_used_js_heap_size=12_562_246,
            console_js_heap_size_limit=4_294_705_152,
            console_total_js_heap_size=13_061_022,
            console_used_js_heap_size=12_562_246,
        ),
    )
