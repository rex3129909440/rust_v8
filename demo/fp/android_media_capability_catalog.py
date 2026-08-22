"""Android Chromium/WebView media capability profiles.

The lists intentionally keep four browser APIs separate:

* HTMLMediaElement.canPlayType reports recognized playback formats.
* MediaSource reports appendable stream formats.
* MediaRecorder reports recordable output formats.
* MediaCapabilities reports support, smoothness and hardware efficiency.

The baseline is Android's published codec support.  WebView-specific output
was verified on the authorized Pixel 4 / Android 11 / WebView 150 HTTPS probe.
Hardware AV1 efficiency is selected only for catalog rows explicitly tagged
``av1-hardware``; software AV1 remains supported and smooth but not efficient.
"""

from __future__ import annotations

from collections.abc import Mapping


AAC = 'audio/mp4; codecs="mp4a.40.2"'
OPUS = 'audio/webm; codecs="opus"'
VORBIS = 'audio/ogg; codecs="vorbis"'
FLAC = "audio/flac"
WAV_PCM = 'audio/wav; codecs="1"'
H264_BASELINE = 'video/mp4; codecs="avc1.42E01E"'
H264_MAIN = 'video/mp4; codecs="avc1.4D401F"'
H264_HIGH = 'video/mp4; codecs="avc1.640028"'
H264_HIGH_4K = 'video/mp4; codecs="avc1.640033"'
HEVC_MAIN = 'video/mp4; codecs="hvc1.1.6.L93.B0"'
VP8 = 'video/webm; codecs="vp8"'
VP9_8 = 'video/webm; codecs="vp09.00.10.08"'
VP9_8_4K = 'video/webm; codecs="vp09.00.50.08"'
VP9_10 = 'video/webm; codecs="vp09.02.10.10"'
AV1_8 = 'video/webm; codecs="av01.0.04M.08"'
AV1_8_1080 = 'video/webm; codecs="av01.0.08M.08"'
AV1_10 = 'video/webm; codecs="av01.0.08M.10"'
AV1_10_4K = 'video/webm; codecs="av01.0.12M.10"'


def android_supported_constraints(chromium_major: int) -> tuple[str, ...]:
    constraints = [
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
        constraints.insert(constraints.index("sampleRate"), "restrictOwnAudio")
    return tuple(constraints)


def _android_major(device: Mapping[str, object]) -> int:
    try:
        return int(str(device.get("androidVersion", "10")).split(".", 1)[0])
    except ValueError:
        return 10


def build_android_media_capabilities(
    device: Mapping[str, object],
    chromium_major: int,
    *,
    webview: bool = False,
) -> dict[str, tuple[object, ...]]:
    """Return one internally coherent Android media capability record."""

    android_major = _android_major(device)
    supports_opus = android_major >= 5
    supports_hevc = android_major >= 5
    supports_vp9 = android_major >= 5
    supports_av1 = android_major >= 10
    av1_hardware = str(device.get("mediaTier", "")) == "av1-hardware"

    can_play = [
        "audio/mpeg", "audio/aac", "audio/x-mpeg", "audio/x-m4a",
        VORBIS, AAC, FLAC, WAV_PCM, "audio/wave; codecs=\"1\"",
        H264_BASELINE, H264_MAIN, H264_HIGH, H264_HIGH_4K, VP8,
    ]
    media_source = ["audio/mpeg", AAC, H264_BASELINE, H264_MAIN, H264_HIGH, H264_HIGH_4K, VP8]
    media_recorder = [AAC, H264_BASELINE, H264_MAIN, H264_HIGH, VP8]
    decoding = [AAC, FLAC, H264_BASELINE, H264_MAIN, H264_HIGH, H264_HIGH_4K, VP8]
    power_efficient = [AAC, FLAC, H264_BASELINE, H264_MAIN, H264_HIGH, H264_HIGH_4K, VP8]

    if supports_opus:
        can_play.append(OPUS)
        media_source.append(OPUS)
        media_recorder.append(OPUS)
        decoding.append(OPUS)
        power_efficient.append(OPUS)
    if supports_hevc:
        can_play.append(HEVC_MAIN)
        media_source.append(HEVC_MAIN)
        media_recorder.append(HEVC_MAIN)
        decoding.append(HEVC_MAIN)
        power_efficient.append(HEVC_MAIN)
    if supports_vp9:
        can_play.extend((VP9_8, VP9_10))
        media_source.extend((VP9_8, VP9_8_4K, VP9_10))
        decoding.extend((VP9_8, VP9_8_4K, VP9_10))
        power_efficient.extend((VP9_8, VP9_8_4K, VP9_10))
    if supports_av1:
        can_play.extend((AV1_8, AV1_10))
        media_source.extend((AV1_8, AV1_8_1080, AV1_10, AV1_10_4K))
        media_recorder.extend((AV1_8, AV1_8_1080, AV1_10, AV1_10_4K))
        decoding.extend((AV1_8, AV1_8_1080, AV1_10, AV1_10_4K))
        if av1_hardware:
            power_efficient.extend((AV1_8, AV1_8_1080, AV1_10, AV1_10_4K))

    # Pixel 4 WebView 150 returned false for every tested `webrtc`
    # encodingInfo configuration even though MediaRecorder advertised several
    # output MIME types.  Keep those two APIs independent.
    encoding_supported: tuple[str, ...] = () if webview else tuple(media_recorder)
    encoding_smooth: tuple[str, ...] = () if webview else tuple(media_recorder)
    encoding_efficient: tuple[str, ...] = ()

    audio_decoders = ["mp3", "mp4a.40.2", "flac", "vorbis"]
    audio_encoders = ["mp4a.40.2"]
    if supports_opus:
        audio_decoders.append("opus")
        audio_encoders.append("opus")
    video_decoders = ["vp8", "avc1.42E01E", "avc1.4D401F", "avc1.640028"]
    video_encoders = ["vp8", "avc1.42E01E", "avc1.4D401F", "avc1.640028"]
    if supports_hevc:
        video_decoders.append("hvc1.1.6.L93.B0")
        video_encoders.append("hvc1.1.6.L93.B0")
    if supports_vp9:
        video_decoders.extend(("vp09.00.10.08", "vp09.02.10.10"))
    if supports_av1:
        video_decoders.extend(("av01.0.04M.08", "av01.0.08M.10"))
        video_encoders.extend(("av01.0.04M.08", "av01.0.08M.10"))

    return {
        "supported_constraints": android_supported_constraints(chromium_major),
        "can_play_probably_types": tuple(dict.fromkeys(can_play)),
        "can_play_maybe_types": (
            "audio/x-mpegurl", "audio/wave", "audio/wav", "audio/webm", "video/mp4",
        ),
        "media_source_types": tuple(dict.fromkeys(media_source)),
        "media_recorder_types": tuple(dict.fromkeys(media_recorder)),
        "decoding_supported_types": tuple(dict.fromkeys(decoding)),
        "decoding_smooth_types": tuple(dict.fromkeys(decoding)),
        "decoding_power_efficient_types": tuple(dict.fromkeys(power_efficient)),
        "encoding_supported_types": encoding_supported,
        "encoding_smooth_types": encoding_smooth,
        "encoding_power_efficient_types": encoding_efficient,
        "audio_decoder_codecs": tuple(audio_decoders),
        "audio_encoder_codecs": tuple(audio_encoders),
        "video_decoder_codecs": tuple(video_decoders),
        "video_encoder_codecs": tuple(video_encoders),
    }


__all__ = [
    "AAC", "OPUS", "VORBIS", "FLAC", "WAV_PCM", "H264_BASELINE",
    "H264_MAIN", "H264_HIGH", "H264_HIGH_4K", "HEVC_MAIN", "VP8",
    "VP9_8", "VP9_8_4K", "VP9_10", "AV1_8", "AV1_8_1080",
    "AV1_10", "AV1_10_4K", "android_supported_constraints",
    "build_android_media_capabilities",
]
