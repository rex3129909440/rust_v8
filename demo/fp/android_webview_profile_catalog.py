"""Android System WebView-specific profile evidence."""

from __future__ import annotations


WEBVIEW_150_CAN_PLAY_PROBABLY: tuple[str, ...] = (
    "audio/mpeg",
    "video/webm; codecs=vorbis",
    "audio/aac",
    'video/mp4; codecs="avc1.42c00d"',
    'video/mp4; codecs="avc1.42E01E"',
    'video/webm; codecs="vorbis,vp9"',
    'video/webm; codecs="vp8, vorbis"',
    'audio/wav; codecs="1"',
    "video/mp4; codecs=mp4a.40.2",
    "video/ogg; codecs=opus",
    'audio/ogg; codecs="vorbis"',
    'audio/mp4; codecs="mp4a.40.2"',
    'video/mp4; codecs="avc1.64001E, mp4a.40.2"',
)

WEBVIEW_150_CAN_PLAY_MAYBE: tuple[str, ...] = (
    "audio/x-mpegurl",
    "audio/x-m4a",
    "audio/wav",
    "video/mp4",
    "audio/webm",
)

WEBVIEW_150_REQUEST_INIT: tuple[str, ...] = (
    "attributionReporting",
    "body",
    "cache",
    "credentials",
    "duplex",
    "headers",
    "integrity",
    "keepalive",
    "method",
    "mode",
    "priority",
    "privateToken",
    "redirect",
    "referrer",
    "referrerPolicy",
    "signal",
)

__all__ = [
    "WEBVIEW_150_CAN_PLAY_MAYBE",
    "WEBVIEW_150_CAN_PLAY_PROBABLY",
    "WEBVIEW_150_REQUEST_INIT",
]
