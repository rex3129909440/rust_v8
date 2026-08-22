"""Versioned desktop media and audio capability helpers.

These values describe browser/OS capability state. They do not generate media
device identifiers or permission grants.
"""

from __future__ import annotations

import random


CHROMIUM_SUPPORTED_CONSTRAINTS_SOURCE = (
    "https://chromium.googlesource.com/chromium/src/+/master/"
    "third_party/blink/renderer/modules/mediastream/"
    "media_track_supported_constraints.idl"
)
CHROMIUM_AUDIO_CONTEXT_SOURCE = (
    "https://chromium.googlesource.com/chromium/src/+/HEAD/"
    "third_party/blink/renderer/modules/webaudio/audio_context.cc"
)
CHROMIUM_AUDIO_LATENCY_SOURCE = (
    "https://chromium.googlesource.com/chromium/src/+/refs/tags/"
    "125.0.6422.219/media/base/audio_latency.cc"
)
MICROSOFT_AUDIO_PERIOD_SOURCE = (
    "https://learn.microsoft.com/windows/win32/api/audioclient/"
    "nf-audioclient-iaudioclient3-getsharedmodeengineperiod"
)
CHROMIUM_MAC_AUDIO_SOURCE = (
    "https://chromium.googlesource.com/chromium/src/media/+/master/"
    "audio/mac/audio_manager_mac.cc"
)


_DESKTOP_SUPPORTED_CONSTRAINTS = (
    "aspectRatio",
    "autoGainControl",
    "brightness",
    "channelCount",
    "colorTemperature",
    "contrast",
    "deviceId",
    "displaySurface",
    "echoCancellation",
    "exposureCompensation",
    "exposureMode",
    "exposureTime",
    "facingMode",
    "focusDistance",
    "focusMode",
    "frameRate",
    "groupId",
    "height",
    "iso",
    "latency",
    "noiseSuppression",
    "pan",
    "pointsOfInterest",
    "resizeMode",
    "restrictOwnAudio",
    "sampleRate",
    "sampleSize",
    "saturation",
    "sharpness",
    "suppressLocalAudioPlayback",
    "tilt",
    "torch",
    "voiceIsolation",
    "whiteBalanceMode",
    "width",
    "zoom",
)


def chromium_desktop_supported_constraints(
    chromium_major: int,
) -> tuple[str, ...]:
    """Return Chromium's own-property order for supported constraints."""

    if int(chromium_major) >= 141:
        return _DESKTOP_SUPPORTED_CONSTRAINTS
    return tuple(
        name for name in _DESKTOP_SUPPORTED_CONSTRAINTS
        if name != "restrictOwnAudio"
    )


def choose_windows_audio_capabilities(
    rng: random.Random,
) -> dict[str, float | int]:
    """Choose a linked Windows default-output sample-rate/period row.

    Chromium computes baseLatency as max(hardware period, 128 frames) divided
    by the sample rate.  The 48 kHz row is the normal 10 ms engine period; the
    44.1 kHz row uses Microsoft's documented 448-frame default-period example.
    """

    sample_rate, frames = rng.choices(
        ((48_000, 480), (44_100, 448)),
        weights=(85, 15),
        k=1,
    )[0]
    return {
        "sample_rate": float(sample_rate),
        "max_channel_count": 2,
        "base_latency": max(128, frames) / float(sample_rate),
        # Chromium quantizes the hardware-output latency. Before an output
        # position is reported, a fresh context exposes zero.
        "output_latency": 0.0,
    }


__all__ = [
    "CHROMIUM_AUDIO_CONTEXT_SOURCE",
    "CHROMIUM_AUDIO_LATENCY_SOURCE",
    "CHROMIUM_MAC_AUDIO_SOURCE",
    "CHROMIUM_SUPPORTED_CONSTRAINTS_SOURCE",
    "MICROSOFT_AUDIO_PERIOD_SOURCE",
    "choose_windows_audio_capabilities",
    "chromium_desktop_supported_constraints",
]
