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

import random
import re
import secrets
import sys
from dataclasses import dataclass, replace
from functools import lru_cache
from pathlib import Path
from typing import Iterable, Sequence
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
    get_android_device_profiles,
)
from android_font_profile_catalog import build_android_font_profile  # noqa: E402
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
from screen_profile_catalog import choose_pc_screen_profile_for_hardware  # noqa: E402
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

if SOURCE_CHECKOUT:
    from examples.edge_profile import (  # type: ignore
        BatteryProfile,
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
    "Mozilla/5.0 (Linux; Android 15; K) "
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
    gpu_model: str
    gpu_core_count: int | None
    gpu_core_unit: str
    geolocation_reference: tuple[float, float]


@dataclass(frozen=True, slots=True)
class FingerprintVerification:
    """Result of reading one generated fingerprint from a real sandbox."""

    fingerprint: RandomFingerprint
    observations: tuple[tuple[str, str], ...]


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
    if not 140 <= chromium_major <= 150:
        raise ValueError("user_agent Chromium major must be between 140 and 150")

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
        if edge_match is None or not 140 <= int(edge_match.group(1)) <= 150:
            raise ValueError("user_agent Edge major must be between 140 and 150")

    ua_options: dict[str, object] = {}
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
            r"\bAndroid\s+[\d.]+;\s*([^;)]+?)(?:\s+Build/[^)]*)?\)",
            ua_string,
        )
        ua_options.update(
            architecture="arm",
            bitness="64",
            platform="Android",
            platformVersion=(android_match.group(1) if android_match else ""),
            model=(model_match.group(1).strip() if model_match else "K"),
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
                if str(ua_data.get("architecture", "")).lower() == "arm"
                else "x64",
            ),
            "userAgent": ua_string,
            "appVersion": ua_string.removeprefix("Mozilla/"),
            "platform": get_base_platform(ua_string),
            "userAgentData": ua_data,
            "headers": headers,
            "chromiumMajor": chromium_major,
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


def _media_devices(platform_name: str) -> tuple[MediaDeviceProfile, ...]:
    """Return the pre-permission device shape without invented identifiers.

    Chromium does not expose stable labels or hardware identifiers before the
    caller grants media permission.  Empty strings therefore model the real
    privacy boundary; unlike the old ``Edge Sandbox Device`` defaults, they
    are not synthetic device placeholders.
    """

    devices = [
        MediaDeviceProfile("", "audioinput", "", ""),
    ]
    if platform_name in {"macos", "android"}:
        devices.append(MediaDeviceProfile("", "videoinput", "", ""))
    devices.append(MediaDeviceProfile("", "audiooutput", "", ""))
    return tuple(devices)


def _storage_values(
    rng: random.Random,
    physical_memory_gb: int,
) -> tuple[int, int]:
    """Choose a deterministic, internally valid browser storage estimate."""

    # Disk capacity cannot be inferred from navigator.deviceMemory.  Use a
    # broad desktop quota pool and only bias larger-memory machines away from
    # the smallest entry.  Both values are real byte counts, not sentinel
    # placeholders.
    quota_choices = [64, 128, 256, 512]
    if physical_memory_gb >= 32:
        quota_choices = quota_choices[1:]
    quota_bytes = rng.choice(quota_choices) * 1024**3
    usage_bytes = rng.randrange(24, 768) * 1024**2
    return quota_bytes, min(usage_bytes, quota_bytes // 64)


def _network_profile(seed: int, platform_name: str) -> NetworkProfile:
    """Choose a browser-observed NetworkInformation snapshot.

    Network quality is independent from CPU/GPU selection. A separate random
    stream keeps the existing hardware catalog sequence stable. The pool
    contains Edge 150 observations from Windows and macOS plus the project's
    original Windows baseline.
    """

    observations = (
        ("4g", 100, 1.7, False),
        ("4g", 100, 10.0, False),
        ("4g", 50, 10.0, False),
    )
    index = (seed ^ 0x4E4554574F524B) % len(observations)
    effective_type, rtt, downlink, save_data = observations[index]
    return NetworkProfile(
        effective_type=effective_type,
        rtt=rtt,
        downlink=downlink,
        save_data=save_data,
    )


def _memory_values(rng: random.Random) -> tuple[int, int, int]:
    """Create a coherent performance.memory snapshot for one evaluation."""

    heap_limit = 4_294_705_152
    total_heap = rng.randrange(12, 49) * 1024**2
    used_heap = int(total_heap * rng.uniform(0.58, 0.91))
    return heap_limit, total_heap, used_heap


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
    ``seed`` makes every catalog choice deterministic.  Chromium's frozen
    low-entropy Mac UA retains ``MacIntel`` while UA-CH is set to the selected
    Apple-silicon hardware architecture.
    """

    country = _require_country_code(country_code)
    resolved_seed = _resolve_seed(seed)
    rng = random.Random(resolved_seed)
    user_agent_profile, platform_name = _resolve_user_agent(user_agent)

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
        selected_fonts = build_windows_font_profile(languages[0])
    elif platform_name == "macos":
        gpu = choose_mac_gpu_candidate(
            rng,
            get_mac_gpu_candidates(verified_only=True),
        )
        memory_choices = tuple(int(item) for item in gpu.get("memoryChoicesGb", ()))
        if not memory_choices:
            raise ValueError("selected Mac GPU has no memory choices")
        screen_classes = tuple(str(item) for item in gpu.get("screenClasses", ()))
        portable = any(
            item.startswith(("air", "pro", "intel13", "intel16"))
            for item in screen_classes
        )
        memory_gb = rng.choice(memory_choices)
        hardware = {
            "id": f"{gpu.get('id', 'mac')}_{memory_gb}gb",
            "hardwareConcurrency": int(gpu.get("cpuCores", 8)),
            "deviceMemory": _chromium_device_memory_gb(
                memory_gb,
                int(user_agent_profile.get("chromiumMajor", 0)),
            ),
            "maxTouchPoints": 0,
            "physicalRamHintGb": memory_gb,
            "tags": ("mac", "laptop" if portable else "desktop", "notouch"),
        }
        screen = choose_mac_screen_profile_for_gpu(
            rng,
            gpu,
            include_external=include_external_mac_screen,
        )
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
        device = choose_android_device_profile(
            rng,
            get_android_device_profiles(requested_model),
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
        selected_fonts = build_android_font_profile(languages[0])
    if platform_name == "macos":
        speech = {
            "id": (
                "macos-chromium150-local-plus-google-voices"
                if str(user_agent_profile.get("browser", "")) == "chrome"
                else "macos-chromium150-local-voices"
            )
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
    if platform_name == "macos":
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
        ua_data_values.update(
            {
                "platform": "Android",
                "architecture": "arm",
                "bitness": "64",
                "model": str(screen.get("model", "")),
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
        pdf_viewer_enabled=True,
        do_not_track=None,
        user_agent_data=_user_agent_data_profile(ua_data_values),
        network=_network_profile(resolved_seed, platform_name),
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
    heap_limit, total_heap, used_heap = _memory_values(rng)
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
            else base_profile.css
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
        media=replace(
            base_profile.media,
            devices=_media_devices(platform_name),
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
            keyboard_layout=tuple(
                KeyboardLayoutEntryProfile(code, value)
                for code, value in keyboard_layout_for_profile(
                    platform_name,
                    country,
                )
            ),
            midi_inputs=(),
            midi_outputs=(),
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
                aliased_point_size_max=1024.0,
            ),
            webgpu=replace(
                base_profile.webgpu,
                vendor=str(gpu.get("vendor", "")),
                architecture=str(gpu.get("webgpuArchitecture", "")),
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
                base_profile.media_preferences,
                color_gamut="srgb",
                dynamic_range="standard",
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
                window_management="prompt",
            ),
            hardware_devices=replace(
                shared_profile.hardware_devices,
                keyboard_layout=tuple(
                    KeyboardLayoutEntryProfile(code, value)
                    for code, value in MAC_CHROMIUM150_KEYBOARD_LAYOUT
                ),
            ),
            memory=replace(
                shared_profile.memory,
                performance_js_heap_size_limit=4_395_630_592,
                console_js_heap_size_limit=4_395_630_592,
            ),
            media_preferences=replace(
                shared_profile.media_preferences,
                color_gamut=str(screen.get("colorGamut", "p3")),
                dynamic_range=str(screen.get("dynamicRange", "standard")),
            ),
        )
    else:
        profile = replace(
            shared_profile,
            id=f"random-android-{country.lower()}-{resolved_seed:016x}",
            webgl=replace(
                base_profile.webgl,
                unmasked_vendor=str(webgl_data.get("unmaskedVendor", "")),
                unmasked_renderer=str(webgl_data.get("unmaskedRenderer", "")),
                webgl1_extensions=_ANDROID_WEBGL1_EXTENSIONS,
                webgl2_extensions=_ANDROID_WEBGL2_EXTENSIONS,
                compressed_texture_formats=tuple(
                    int(item) for item in _ANDROID_COMPRESSED_TEXTURE_FORMATS
                ),
                webgl2_max_samples=4,
                aliased_point_size_max=1024.0,
            ),
            webgpu=replace(
                base_profile.webgpu,
                vendor=str(gpu.get("vendor", "")),
                architecture=str(gpu.get("webgpuArchitecture", "")),
                device="",
                description=str(gpu.get("model", "")),
                developer_features=False,
                subgroup_min_size=32,
                subgroup_max_size=32,
                is_fallback_adapter=False,
                features=(
                    "bgra8unorm-storage",
                    "texture-compression-etc2",
                    "texture-compression-astc",
                ),
                max_compute_workgroup_storage_size=16_384,
            ),
            audio=replace(
                shared_profile.audio,
                sample_rate=48_000.0,
                max_channel_count=2,
                base_latency=0.01,
                output_latency=0.0,
            ),
            media_preferences=replace(
                base_profile.media_preferences,
                color_gamut="srgb",
                dynamic_range="standard",
            ),
        )

    headers = dict(user_agent_profile.get("headers", {}))
    headers["user-agent"] = str(user_agent_profile.get("userAgent", ""))
    headers["accept-language"] = _accept_language(languages)
    if platform_name == "macos":
        headers["sec-ch-ua-platform"] = '"macOS"'
        headers["sec-ch-ua-arch"] = f'"{ua_data_values["architecture"]}"'
        headers["sec-ch-ua-bitness"] = f'"{ua_data_values["bitness"]}"'
        headers["sec-ch-ua-platform-version"] = (
            f'"{ua_data_values["platformVersion"]}"'
        )
    elif platform_name == "android":
        headers["sec-ch-ua-platform"] = '"Android"'
        headers["sec-ch-ua-arch"] = '"arm"'
        headers["sec-ch-ua-bitness"] = '"64"'
        headers["sec-ch-ua-model"] = f'"{ua_data_values["model"]}"'
        headers["sec-ch-ua-platform-version"] = (
            f'"{ua_data_values["platformVersion"]}"'
        )
        headers["sec-ch-ua-mobile"] = "?1"
    return RandomFingerprint(
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
        gpu_model=str(gpu.get("model", "")),
        gpu_core_count=(
            int(gpu["gpuCores"]) if "gpuCores" in gpu else None
        ),
        gpu_core_unit=str(gpu.get("gpuCoreUnit", "")),
        geolocation_reference=reference_location,
    )


def get_random_fp(
    country_code: str,
    user_agent: str | None = None,
    *,
    seed: int | None = None,
    time_zone: str | None = None,
    include_virtual_gpu: bool = False,
    include_external_mac_screen: bool = False,
) -> EdgeProfile:
    """Return the typed profile expected by ``EdgeSandbox(profile=...)``."""

    return get_random_fp_details(
        country_code,
        user_agent,
        seed=seed,
        time_zone=time_zone,
        include_virtual_gpu=include_virtual_gpu,
        include_external_mac_screen=include_external_mac_screen,
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
    "get_random_fp",
    "get_random_fp_details",
    "test_random_fp_combinations",
    "verify_random_fp",
]
