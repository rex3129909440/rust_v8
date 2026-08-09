#!/usr/bin/env python3
"""Call edge-sandbox through its native library and read JavaScript results."""

from __future__ import annotations

import ctypes
import os
import struct
from dataclasses import dataclass
from pathlib import Path
from types import TracebackType
from typing import Iterable, Iterator, Self

try:
    from .edge_profile import EdgeProfile, ProfileField, WebAudioProfile
    from .edge_runtime_options import EdgeRunOptions
except ImportError:
    from edge_profile import EdgeProfile, ProfileField, WebAudioProfile
    from edge_runtime_options import EdgeRunOptions


PROJECT_ROOT = Path(__file__).resolve().parents[1]
ABI_VERSION = 1
PROFILE_SCHEMA_VERSION = 10
OPTIONS_SCHEMA_VERSION = 2

DEMO_JAVASCRIPT = r"""
(() => {
  const element = document.createElement("div");
  element.id = "python-demo";
  element.textContent = "called from Python";
  document.body.appendChild(element);

  return [
    element.tagName,
    document.getElementById("python-demo").id,
    document.all.namedItem("python-demo") === element,
    Object.getOwnPropertyNames(window).length
  ].join("|");
})()
""".strip()


class _NativeBuffer(ctypes.Structure):
    _fields_ = [
        ("data", ctypes.POINTER(ctypes.c_ubyte)),
        ("len", ctypes.c_size_t),
    ]


class _NativeStringView(ctypes.Structure):
    _fields_ = [
        ("data", ctypes.POINTER(ctypes.c_ubyte)),
        ("len", ctypes.c_size_t),
    ]


class _NativePerformanceEntryProfile(ctypes.Structure):
    _fields_ = [
        ("name", _NativeStringView),
        ("entry_type", _NativeStringView),
        ("initiator_type", _NativeStringView),
        ("delivery_type", _NativeStringView),
        ("next_hop_protocol", _NativeStringView),
        ("render_blocking_status", _NativeStringView),
        ("content_type", _NativeStringView),
        ("content_encoding", _NativeStringView),
        ("worker_matched_source_type", _NativeStringView),
        ("worker_final_source_type", _NativeStringView),
        ("navigation_type", _NativeStringView),
        ("start_time", ctypes.c_double),
        ("duration", ctypes.c_double),
        ("worker_start", ctypes.c_double),
        ("worker_router_evaluation_start", ctypes.c_double),
        ("worker_cache_lookup_start", ctypes.c_double),
        ("redirect_start", ctypes.c_double),
        ("redirect_end", ctypes.c_double),
        ("fetch_start", ctypes.c_double),
        ("domain_lookup_start", ctypes.c_double),
        ("domain_lookup_end", ctypes.c_double),
        ("connect_start", ctypes.c_double),
        ("secure_connection_start", ctypes.c_double),
        ("connect_end", ctypes.c_double),
        ("request_start", ctypes.c_double),
        ("response_start", ctypes.c_double),
        ("first_interim_response_start", ctypes.c_double),
        ("final_response_headers_start", ctypes.c_double),
        ("response_end", ctypes.c_double),
        ("unload_event_start", ctypes.c_double),
        ("unload_event_end", ctypes.c_double),
        ("dom_interactive", ctypes.c_double),
        ("dom_content_loaded_event_start", ctypes.c_double),
        ("dom_content_loaded_event_end", ctypes.c_double),
        ("dom_complete", ctypes.c_double),
        ("load_event_start", ctypes.c_double),
        ("load_event_end", ctypes.c_double),
        ("critical_ch_restart", ctypes.c_double),
        ("activation_start", ctypes.c_double),
        ("paint_time", ctypes.c_double),
        ("presentation_time", ctypes.c_double),
        ("transfer_size", ctypes.c_uint64),
        ("encoded_body_size", ctypes.c_uint64),
        ("decoded_body_size", ctypes.c_uint64),
        ("redirect_count", ctypes.c_uint32),
        ("response_status", ctypes.c_uint16),
        ("has_transfer_size", ctypes.c_uint8),
        ("has_encoded_body_size", ctypes.c_uint8),
        ("has_decoded_body_size", ctypes.c_uint8),
        ("has_response_status", ctypes.c_uint8),
        ("reserved", ctypes.c_uint8 * 8),
    ]


class _NativeWebAudioProfile(ctypes.Structure):
    _fields_ = [
        ("sample_rate", ctypes.c_double),
        ("base_latency", ctypes.c_double),
        ("output_latency", ctypes.c_double),
        ("noise_seed", ctypes.c_uint64),
        ("max_channel_count", ctypes.c_uint32),
        ("channel_noise_amplitude", ctypes.c_float),
        ("frequency_noise_amplitude", ctypes.c_float),
        ("time_domain_noise_amplitude", ctypes.c_float),
    ]


class _NativeDeterministicOptions(ctypes.Structure):
    _fields_ = [
        ("clock_epoch_ms", ctypes.c_int64),
        ("clock_step_ms", ctypes.c_uint64),
        ("random_seed", ctypes.c_uint64),
        ("max_task_turns", ctypes.c_uint32),
        ("has_clock_epoch_ms", ctypes.c_uint8),
        ("has_random_seed", ctypes.c_uint8),
        ("reserved", ctypes.c_uint8 * 6),
    ]


class _NativeSandboxLimits(ctypes.Structure):
    _fields_ = [
        ("timeout_ms", ctypes.c_uint64),
        ("max_heap_bytes", ctypes.c_uint64),
        ("max_resident_bytes", ctypes.c_uint64),
        ("max_source_bytes", ctypes.c_uint64),
        ("max_output_bytes", ctypes.c_uint64),
    ]


class SandboxExecutionError(RuntimeError):
    """Raised when the native sandbox cannot complete an operation."""


@dataclass(frozen=True)
class CapturedNetworkRequest:
    sequence: int
    source: str
    method: str
    url: str
    headers: tuple[tuple[str, str], ...]
    body: bytes


@dataclass(frozen=True)
class CapturedConsoleValue:
    """One typed JavaScript value captured from a console call."""

    kind: str
    value: object
    type_name: str | None = None
    truncated: bool = False


@dataclass(frozen=True)
class CapturedConsoleOutput:
    """One console message captured without enabling native trace."""

    sequence: int
    level: str
    frame_url: str
    text: str
    arguments: tuple[CapturedConsoleValue, ...]


def _decode_network_requests(data: bytes) -> tuple[CapturedNetworkRequest, ...]:
    view = memoryview(data)
    offset = 0

    def take(size: int) -> memoryview:
        nonlocal offset
        end = offset + size
        if size < 0 or end > len(view):
            raise SandboxExecutionError("invalid ESNR network request buffer")
        value = view[offset:end]
        offset = end
        return value

    def unsigned(size: int) -> int:
        return int.from_bytes(take(size), "little")

    if bytes(take(4)) != b"ESNR":
        raise SandboxExecutionError("invalid ESNR network request signature")
    version = unsigned(2)
    if version != 1:
        raise SandboxExecutionError(f"unsupported ESNR version: {version}")
    take(2)
    count = unsigned(4)
    requests: list[CapturedNetworkRequest] = []
    for _ in range(count):
        sequence = unsigned(8)
        source_id = unsigned(1)
        take(3)
        method_len = unsigned(4)
        url_len = unsigned(4)
        header_count = unsigned(4)
        body_len = unsigned(8)
        method = bytes(take(method_len)).decode("utf-8")
        url = bytes(take(url_len)).decode("utf-8")
        headers: list[tuple[str, str]] = []
        for _ in range(header_count):
            name_len = unsigned(4)
            value_len = unsigned(4)
            name = bytes(take(name_len)).decode("utf-8")
            value = bytes(take(value_len)).decode("utf-8")
            headers.append((name, value))
        body = bytes(take(body_len))
        source = {1: "XMLHttpRequest", 2: "fetch"}.get(
            source_id, f"unknown:{source_id}"
        )
        requests.append(
            CapturedNetworkRequest(
                sequence=sequence,
                source=source,
                method=method,
                url=url,
                headers=tuple(headers),
                body=body,
            )
        )
    if offset != len(view):
        raise SandboxExecutionError("trailing bytes in ESNR network request buffer")
    return tuple(requests)


def _decode_stdout(data: bytes) -> tuple[CapturedConsoleOutput, ...]:
    view = memoryview(data)
    offset = 0

    def take(size: int) -> memoryview:
        nonlocal offset
        end = offset + size
        if size < 0 or end > len(view):
            raise SandboxExecutionError("invalid ESSO stdout buffer")
        value = view[offset:end]
        offset = end
        return value

    def unsigned(size: int) -> int:
        return int.from_bytes(take(size), "little")

    def text() -> str:
        return bytes(take(unsigned(4))).decode("utf-8")

    def value() -> CapturedConsoleValue:
        tag = unsigned(1)
        truncated = bool(unsigned(1))
        take(2)
        if tag == 0:
            return CapturedConsoleValue("undefined", None, truncated=truncated)
        if tag == 1:
            return CapturedConsoleValue("null", None, truncated=truncated)
        if tag == 2:
            return CapturedConsoleValue(
                "boolean", bool(unsigned(1)), truncated=truncated
            )
        if tag == 3:
            return CapturedConsoleValue(
                "number", struct.unpack("<d", take(8))[0], truncated=truncated
            )
        if tag == 4:
            return CapturedConsoleValue("string", text(), truncated=truncated)
        if tag == 5:
            return CapturedConsoleValue("bigint", text(), truncated=truncated)
        if tag == 6:
            type_name = text()
            payload = bytes(take(unsigned(8)))
            return CapturedConsoleValue(
                "bytes", payload, type_name=type_name, truncated=truncated
            )
        if tag == 7:
            type_name = text()
            values = tuple(value() for _ in range(unsigned(4)))
            return CapturedConsoleValue(
                "sequence", values, type_name=type_name, truncated=truncated
            )
        if tag == 8:
            type_name = text()
            return CapturedConsoleValue(
                "other", text(), type_name=type_name, truncated=truncated
            )
        if tag == 9:
            type_name = text()
            entries = tuple((text(), value()) for _ in range(unsigned(4)))
            return CapturedConsoleValue(
                "object", entries, type_name=type_name, truncated=truncated
            )
        raise SandboxExecutionError(f"unknown ESSO value tag: {tag}")

    if bytes(take(4)) != b"ESSO":
        raise SandboxExecutionError("invalid ESSO stdout signature")
    version = unsigned(2)
    if version != 1:
        raise SandboxExecutionError(f"unsupported ESSO version: {version}")
    take(2)
    count = unsigned(4)
    levels = {
        1: "debug",
        2: "info",
        3: "log",
        4: "warn",
        5: "error",
        6: "dir",
        7: "dirxml",
        8: "table",
        9: "trace",
    }
    entries: list[CapturedConsoleOutput] = []
    for _ in range(count):
        sequence = unsigned(8)
        level_id = unsigned(1)
        take(3)
        frame_url = text()
        rendered = text()
        arguments = tuple(value() for _ in range(unsigned(4)))
        entries.append(
            CapturedConsoleOutput(
                sequence=sequence,
                level=levels.get(level_id, f"unknown:{level_id}"),
                frame_url=frame_url,
                text=rendered,
                arguments=arguments,
            )
        )
    if offset != len(view):
        raise SandboxExecutionError("trailing bytes in ESSO stdout buffer")
    return tuple(entries)


def _artifact_name() -> str:
    if os.name == "nt":
        return "edge_sandbox.dll"
    if os.uname().sysname == "Darwin":
        return "libedge_sandbox.dylib"
    return "libedge_sandbox.so"


def find_native_artifacts(
    library: Path | None = None,
    worker: Path | None = None,
) -> tuple[Path, None]:
    """Locate the single self-hosting edge-sandbox native library.

    ``worker`` remains accepted for source compatibility, but is ignored.
    """

    del worker
    if library is not None:
        resolved_library = library.resolve()
        if not resolved_library.is_file():
            raise FileNotFoundError(f"沙箱动态库不存在：{resolved_library}")
        return resolved_library, None

    library_name = _artifact_name()
    # In a source checkout, prefer the freshly built artifact over the staged
    # wheel payload so local regression tests cannot silently load a stale DLL.
    # Installed wheels do not contain PROJECT_ROOT/target and therefore still
    # resolve to their packaged native library below.
    source_artifacts = tuple(
        candidate
        for profile in ("release", "debug")
        if (candidate := PROJECT_ROOT / "target" / profile / library_name).is_file()
    )
    if source_artifacts:
        return max(source_artifacts, key=lambda item: item.stat().st_mtime_ns), None

    packaged_library = Path(__file__).resolve().parent / "_native" / library_name
    if packaged_library.is_file():
        return packaged_library, None

    raise FileNotFoundError("未找到 edge-sandbox 动态库；请先构建项目")


class EdgeSandbox:
    """Native Python binding for an OS-process-isolated edge-sandbox runtime."""

    def __init__(
        self,
        *,
        library: Path | None = None,
        worker: Path | None = None,
        profile: EdgeProfile | None = None,
        audio_profile: WebAudioProfile | None = None,
        options: EdgeRunOptions | None = None,
    ) -> None:
        if profile is not None and audio_profile is not None:
            raise ValueError("profile and audio_profile cannot be supplied together")
        library_path, _ = find_native_artifacts(library, worker)
        self._library = ctypes.CDLL(str(library_path))
        self._configure_native_api()
        self._handle: int | None = None

        actual_abi = self._library.edge_sandbox_abi_version()
        if actual_abi != ABI_VERSION:
            raise SandboxExecutionError(
                f"原生 ABI 版本不匹配：Python={ABI_VERSION}, 动态库={actual_abi}"
            )

        if (
            self._library.edge_sandbox_profile_schema_version()
            != PROFILE_SCHEMA_VERSION
        ):
            raise SandboxExecutionError(
                "native profile schema does not match the Python binding"
            )
        if (
            self._library.edge_sandbox_options_schema_version()
            != OPTIONS_SCHEMA_VERSION
        ):
            raise SandboxExecutionError(
                "native runtime-options schema does not match the Python binding"
            )

        error = _NativeBuffer()
        if options is not None:
            effective_profile = (
                EdgeProfile(audio=audio_profile)
                if audio_profile is not None
                else profile
            )
            handle = self._create_with_options(
                effective_profile,
                options,
                error,
            )
        elif profile is not None:
            handle = self._create_with_profile(profile, error)
        elif audio_profile is None:
            handle = self._library.edge_sandbox_create_self_hosted(ctypes.byref(error))
        else:
            native_audio_profile = _NativeWebAudioProfile(
                sample_rate=audio_profile.sample_rate,
                base_latency=audio_profile.base_latency,
                output_latency=audio_profile.output_latency,
                noise_seed=audio_profile.noise_seed,
                max_channel_count=audio_profile.max_channel_count,
                channel_noise_amplitude=audio_profile.channel_noise_amplitude,
                frequency_noise_amplitude=audio_profile.frequency_noise_amplitude,
                time_domain_noise_amplitude=audio_profile.time_domain_noise_amplitude,
            )
            handle = self._library.edge_sandbox_create_self_hosted_with_audio_profile(
                ctypes.byref(native_audio_profile),
                ctypes.byref(error),
            )
        if not handle:
            raise SandboxExecutionError(self._consume_buffer(error))
        self._handle = handle

    def _configure_native_api(self) -> None:
        library = self._library
        buffer_pointer = ctypes.POINTER(_NativeBuffer)

        library.edge_sandbox_abi_version.argtypes = []
        library.edge_sandbox_abi_version.restype = ctypes.c_uint32

        library.edge_sandbox_create_self_hosted.argtypes = [buffer_pointer]
        library.edge_sandbox_create_self_hosted.restype = ctypes.c_void_p

        library.edge_sandbox_create_self_hosted_with_audio_profile.argtypes = [
            ctypes.POINTER(_NativeWebAudioProfile),
            buffer_pointer,
        ]
        library.edge_sandbox_create_self_hosted_with_audio_profile.restype = ctypes.c_void_p

        library.edge_sandbox_profile_schema_version.argtypes = []
        library.edge_sandbox_profile_schema_version.restype = ctypes.c_uint32

        library.edge_sandbox_options_schema_version.argtypes = []
        library.edge_sandbox_options_schema_version.restype = ctypes.c_uint32
        library.edge_sandbox_options_create.argtypes = [buffer_pointer]
        library.edge_sandbox_options_create.restype = ctypes.c_void_p
        library.edge_sandbox_options_destroy.argtypes = [ctypes.c_void_p]
        library.edge_sandbox_options_destroy.restype = None
        library.edge_sandbox_options_set_profile.argtypes = [
            ctypes.c_void_p,
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_options_set_profile.restype = ctypes.c_bool
        library.edge_sandbox_options_set_page.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_options_set_page.restype = ctypes.c_bool
        library.edge_sandbox_options_clear_page.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_options_clear_page.restype = ctypes.c_bool
        library.edge_sandbox_options_clear_network_replay.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_options_clear_network_replay.restype = ctypes.c_bool
        library.edge_sandbox_options_clear_iframe_hooks.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_options_clear_iframe_hooks.restype = ctypes.c_bool
        library.edge_sandbox_options_append_iframe_hook.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_options_append_iframe_hook.restype = ctypes.c_bool
        library.edge_sandbox_options_append_network_replay.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_uint16,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_options_append_network_replay.restype = ctypes.c_bool
        library.edge_sandbox_options_append_network_replay_header.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint32,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_options_append_network_replay_header.restype = (
            ctypes.c_bool
        )
        library.edge_sandbox_options_set_deterministic.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(_NativeDeterministicOptions),
            buffer_pointer,
        ]
        library.edge_sandbox_options_set_deterministic.restype = ctypes.c_bool
        library.edge_sandbox_options_set_limits.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(_NativeSandboxLimits),
            buffer_pointer,
        ]
        library.edge_sandbox_options_set_limits.restype = ctypes.c_bool
        library.edge_sandbox_options_validate.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_options_validate.restype = ctypes.c_bool
        library.edge_sandbox_create_self_hosted_with_options.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_create_self_hosted_with_options.restype = ctypes.c_void_p

        library.edge_sandbox_profile_create.argtypes = [buffer_pointer]
        library.edge_sandbox_profile_create.restype = ctypes.c_void_p
        library.edge_sandbox_profile_destroy.argtypes = [ctypes.c_void_p]
        library.edge_sandbox_profile_destroy.restype = None
        library.edge_sandbox_profile_clear_performance_entries.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_performance_entries.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_performance_entry.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(_NativePerformanceEntryProfile),
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_performance_entry.restype = ctypes.c_bool

        library.edge_sandbox_profile_set_string.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint32,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_set_string.restype = ctypes.c_bool
        library.edge_sandbox_profile_clear_optional_string.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint32,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_optional_string.restype = ctypes.c_bool
        library.edge_sandbox_profile_clear_string_list.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint32,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_string_list.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_string.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint32,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_string.restype = ctypes.c_bool

        library.edge_sandbox_profile_set_u32.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint32,
            ctypes.c_uint32,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_set_u32.restype = ctypes.c_bool
        library.edge_sandbox_profile_set_i32.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint32,
            ctypes.c_int32,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_set_i32.restype = ctypes.c_bool
        library.edge_sandbox_profile_set_u64.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint32,
            ctypes.c_uint64,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_set_u64.restype = ctypes.c_bool
        library.edge_sandbox_profile_set_i64.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint32,
            ctypes.c_int64,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_set_i64.restype = ctypes.c_bool
        library.edge_sandbox_profile_set_f64.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint32,
            ctypes.c_double,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_set_f64.restype = ctypes.c_bool
        library.edge_sandbox_profile_set_f32.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint32,
            ctypes.c_float,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_set_f32.restype = ctypes.c_bool
        library.edge_sandbox_profile_set_bool.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint32,
            ctypes.c_bool,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_set_bool.restype = ctypes.c_bool

        library.edge_sandbox_profile_clear_ua_brands.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_ua_brands.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_ua_brand.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_ua_brand.restype = ctypes.c_bool
        library.edge_sandbox_profile_clear_speech_voices.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_speech_voices.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_speech_voice.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_bool,
            ctypes.c_bool,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_speech_voice.restype = ctypes.c_bool

        library.edge_sandbox_profile_clear_local_fonts.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_local_fonts.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_local_font.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_local_font.restype = ctypes.c_bool
        library.edge_sandbox_profile_clear_font_metrics.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_font_metrics.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_font_metric.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_double,
            ctypes.c_bool,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_font_metric.restype = ctypes.c_bool

        library.edge_sandbox_profile_clear_media_devices.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_media_devices.restype = ctypes.c_bool
        library.edge_sandbox_profile_clear_webgl_compressed_texture_formats.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_webgl_compressed_texture_formats.restype = (
            ctypes.c_bool
        )
        library.edge_sandbox_profile_append_webgl_compressed_texture_format.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint32,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_webgl_compressed_texture_format.restype = (
            ctypes.c_bool
        )
        library.edge_sandbox_profile_clear_rtc_audio_codecs.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_rtc_audio_codecs.restype = ctypes.c_bool
        library.edge_sandbox_profile_clear_rtc_video_codecs.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_rtc_video_codecs.restype = ctypes.c_bool
        library.edge_sandbox_profile_clear_rtc_header_extensions.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_rtc_header_extensions.restype = (
            ctypes.c_bool
        )
        library.edge_sandbox_profile_clear_plugins.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_plugins.restype = ctypes.c_bool

        library.edge_sandbox_profile_append_media_device.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_media_device.restype = ctypes.c_bool

        rtc_codec_arguments = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_uint32,
            ctypes.c_uint16,
            ctypes.c_bool,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_bool,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_rtc_audio_codec.argtypes = (
            rtc_codec_arguments
        )
        library.edge_sandbox_profile_append_rtc_audio_codec.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_rtc_video_codec.argtypes = (
            rtc_codec_arguments
        )
        library.edge_sandbox_profile_append_rtc_video_codec.restype = ctypes.c_bool

        library.edge_sandbox_profile_append_rtc_header_extension.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_rtc_header_extension.restype = (
            ctypes.c_bool
        )
        library.edge_sandbox_profile_append_plugin.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_plugin.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_plugin_mime_type.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint32,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_plugin_mime_type.restype = ctypes.c_bool

        library.edge_sandbox_profile_clear_gamepads.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_gamepads.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_gamepad.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_uint32,
            ctypes.c_bool,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_double),
            ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_double),
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_gamepad.restype = ctypes.c_bool

        library.edge_sandbox_profile_clear_usb_devices.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_usb_devices.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_usb_device.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint8,
            ctypes.c_uint8,
            ctypes.c_uint8,
            ctypes.c_uint8,
            ctypes.c_uint8,
            ctypes.c_uint8,
            ctypes.c_uint16,
            ctypes.c_uint16,
            ctypes.c_uint8,
            ctypes.c_uint8,
            ctypes.c_uint8,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_bool,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_bool,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_bool,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_usb_device.restype = ctypes.c_bool

        library.edge_sandbox_profile_clear_hid_devices.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_hid_devices.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_hid_device.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint16,
            ctypes.c_uint16,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_hid_device.restype = ctypes.c_bool

        library.edge_sandbox_profile_clear_serial_ports.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_serial_ports.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_serial_port.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint16,
            ctypes.c_uint16,
            ctypes.c_bool,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_serial_port.restype = ctypes.c_bool

        library.edge_sandbox_profile_clear_bluetooth_devices.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_bluetooth_devices.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_bluetooth_device.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_bool,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_bluetooth_device.restype = ctypes.c_bool
        library.edge_sandbox_profile_clear_keyboard_layout.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_keyboard_layout.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_keyboard_layout_entry.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_keyboard_layout_entry.restype = (
            ctypes.c_bool
        )
        library.edge_sandbox_profile_clear_midi_inputs.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_midi_inputs.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_midi_input.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_midi_input.restype = ctypes.c_bool
        library.edge_sandbox_profile_clear_midi_outputs.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_clear_midi_outputs.restype = ctypes.c_bool
        library.edge_sandbox_profile_append_midi_output.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_append_midi_output.restype = ctypes.c_bool
        library.edge_sandbox_profile_validate.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_profile_validate.restype = ctypes.c_bool
        library.edge_sandbox_create_self_hosted_with_profile.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_create_self_hosted_with_profile.restype = ctypes.c_void_p

        library.edge_sandbox_reinitialize_with_profile.argtypes = [
            ctypes.c_void_p,
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_reinitialize_with_profile.restype = ctypes.c_bool
        library.edge_sandbox_process_id.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(ctypes.c_uint32),
            buffer_pointer,
        ]
        library.edge_sandbox_process_id.restype = ctypes.c_bool

        library.edge_sandbox_destroy.argtypes = [ctypes.c_void_p]
        library.edge_sandbox_destroy.restype = None

        library.edge_sandbox_evaluate.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
            buffer_pointer,
        ]
        library.edge_sandbox_evaluate.restype = ctypes.c_bool
        library.edge_sandbox_evaluate_with_source_url.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
            buffer_pointer,
        ]
        library.edge_sandbox_evaluate_with_source_url.restype = ctypes.c_bool

        library.edge_sandbox_enable_native_trace.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_enable_native_trace.restype = ctypes.c_bool
        library.edge_sandbox_disable_native_trace.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_disable_native_trace.restype = ctypes.c_bool
        library.edge_sandbox_clear_native_trace.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_clear_native_trace.restype = ctypes.c_bool
        library.edge_sandbox_set_native_trace_exclusions.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(_NativeStringView),
            ctypes.c_size_t,
            buffer_pointer,
        ]
        library.edge_sandbox_set_native_trace_exclusions.restype = ctypes.c_bool

        library.edge_sandbox_native_trace.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
            buffer_pointer,
        ]
        library.edge_sandbox_native_trace.restype = ctypes.c_bool
        library.edge_sandbox_native_trace_matching.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
            buffer_pointer,
            buffer_pointer,
        ]
        library.edge_sandbox_native_trace_matching.restype = ctypes.c_bool
        library.edge_sandbox_network_requests.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
            buffer_pointer,
        ]
        library.edge_sandbox_network_requests.restype = ctypes.c_bool
        library.edge_sandbox_clear_network_requests.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_clear_network_requests.restype = ctypes.c_bool
        library.edge_sandbox_stdout.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
            buffer_pointer,
        ]
        library.edge_sandbox_stdout.restype = ctypes.c_bool
        library.edge_sandbox_clear_stdout.argtypes = [
            ctypes.c_void_p,
            buffer_pointer,
        ]
        library.edge_sandbox_clear_stdout.restype = ctypes.c_bool

        library.edge_sandbox_buffer_free.argtypes = [buffer_pointer]
        library.edge_sandbox_buffer_free.restype = None

    def _profile_call(self, name: str, *arguments: object) -> None:
        error = _NativeBuffer()
        function = getattr(self._library, name)
        if not function(*arguments, ctypes.byref(error)):
            raise SandboxExecutionError(self._consume_buffer(error))
        self._consume_buffer(error)

    def _profile_set_string(
        self, handle: int, field: ProfileField, value: str | None
    ) -> None:
        if value is None:
            return
        encoded = value.encode("utf-8")
        self._profile_call(
            "edge_sandbox_profile_set_string",
            handle,
            int(field),
            encoded,
            len(encoded),
        )

    def _profile_set_number(
        self,
        native_name: str,
        handle: int,
        field: ProfileField,
        value: int | float | None,
    ) -> None:
        if value is not None:
            self._profile_call(native_name, handle, int(field), value)

    def _profile_set_bool(
        self, handle: int, field: ProfileField, value: bool | None
    ) -> None:
        if value is not None:
            self._profile_call(
                "edge_sandbox_profile_set_bool", handle, int(field), value
            )

    def _profile_set_string_list(
        self,
        handle: int,
        field: ProfileField,
        values: tuple[str, ...] | None,
    ) -> None:
        if values is None:
            return
        self._profile_call(
            "edge_sandbox_profile_clear_string_list", handle, int(field)
        )
        for value in values:
            encoded = value.encode("utf-8")
            self._profile_call(
                "edge_sandbox_profile_append_string",
                handle,
                int(field),
                encoded,
                len(encoded),
            )

    def _profile_append_performance_entry(
        self, handle: int, entry: object
    ) -> None:
        buffers: list[ctypes.Array[ctypes.c_ubyte]] = []

        def view(value: str) -> _NativeStringView:
            encoded = value.encode("utf-8")
            buffer = (ctypes.c_ubyte * len(encoded)).from_buffer_copy(encoded)
            buffers.append(buffer)
            return _NativeStringView(
                ctypes.cast(buffer, ctypes.POINTER(ctypes.c_ubyte)),
                len(encoded),
            )

        native = _NativePerformanceEntryProfile(
            name=view(entry.name),
            entry_type=view(entry.entry_type),
            initiator_type=view(entry.initiator_type),
            delivery_type=view(entry.delivery_type),
            next_hop_protocol=view(entry.next_hop_protocol),
            render_blocking_status=view(entry.render_blocking_status),
            content_type=view(entry.content_type),
            content_encoding=view(entry.content_encoding),
            worker_matched_source_type=view(entry.worker_matched_source_type),
            worker_final_source_type=view(entry.worker_final_source_type),
            navigation_type=view(entry.navigation_type),
            start_time=entry.start_time,
            duration=entry.duration,
            worker_start=entry.worker_start,
            worker_router_evaluation_start=entry.worker_router_evaluation_start,
            worker_cache_lookup_start=entry.worker_cache_lookup_start,
            redirect_start=entry.redirect_start,
            redirect_end=entry.redirect_end,
            fetch_start=entry.fetch_start,
            domain_lookup_start=entry.domain_lookup_start,
            domain_lookup_end=entry.domain_lookup_end,
            connect_start=entry.connect_start,
            secure_connection_start=entry.secure_connection_start,
            connect_end=entry.connect_end,
            request_start=entry.request_start,
            response_start=entry.response_start,
            first_interim_response_start=entry.first_interim_response_start,
            final_response_headers_start=entry.final_response_headers_start,
            response_end=entry.response_end,
            unload_event_start=entry.unload_event_start,
            unload_event_end=entry.unload_event_end,
            dom_interactive=entry.dom_interactive,
            dom_content_loaded_event_start=entry.dom_content_loaded_event_start,
            dom_content_loaded_event_end=entry.dom_content_loaded_event_end,
            dom_complete=entry.dom_complete,
            load_event_start=entry.load_event_start,
            load_event_end=entry.load_event_end,
            critical_ch_restart=entry.critical_ch_restart,
            activation_start=entry.activation_start,
            paint_time=entry.paint_time,
            presentation_time=entry.presentation_time,
            transfer_size=entry.transfer_size or 0,
            encoded_body_size=entry.encoded_body_size or 0,
            decoded_body_size=entry.decoded_body_size or 0,
            redirect_count=entry.redirect_count,
            response_status=entry.response_status or 0,
            has_transfer_size=entry.transfer_size is not None,
            has_encoded_body_size=entry.encoded_body_size is not None,
            has_decoded_body_size=entry.decoded_body_size is not None,
            has_response_status=entry.response_status is not None,
        )
        self._profile_call(
            "edge_sandbox_profile_append_performance_entry",
            handle,
            ctypes.byref(native),
        )

    def _apply_profile(self, handle: int, profile: EdgeProfile) -> None:
        field = ProfileField
        self._profile_set_string(handle, field.ID, profile.id)

        locale = profile.locale
        if locale is not None:
            self._profile_set_string(handle, field.LOCALE, locale.locale)
            self._profile_set_string(handle, field.TIME_ZONE, locale.time_zone)
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.TIME_ZONE_OFFSET_MINUTES,
                locale.time_zone_offset_minutes,
            )

        navigator = profile.navigator
        if navigator is not None:
            self._profile_set_string(
                handle, field.NAVIGATOR_USER_AGENT, navigator.user_agent
            )
            self._profile_set_string(
                handle, field.NAVIGATOR_APP_VERSION, navigator.app_version
            )
            self._profile_set_string(
                handle, field.NAVIGATOR_APP_CODE_NAME, navigator.app_code_name
            )
            self._profile_set_string(
                handle, field.NAVIGATOR_APP_NAME, navigator.app_name
            )
            self._profile_set_string(
                handle, field.NAVIGATOR_PLATFORM, navigator.platform
            )
            self._profile_set_string(
                handle, field.NAVIGATOR_PRODUCT, navigator.product
            )
            self._profile_set_string(
                handle, field.NAVIGATOR_PRODUCT_SUB, navigator.product_sub
            )
            self._profile_set_string(
                handle, field.NAVIGATOR_VENDOR, navigator.vendor
            )
            self._profile_set_string(
                handle, field.NAVIGATOR_VENDOR_SUB, navigator.vendor_sub
            )
            self._profile_set_string(
                handle, field.NAVIGATOR_LANGUAGE, navigator.language
            )
            self._profile_set_string_list(
                handle, field.NAVIGATOR_LANGUAGES, navigator.languages
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.HARDWARE_CONCURRENCY,
                navigator.hardware_concurrency,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.DEVICE_MEMORY_GB,
                navigator.device_memory_gb,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.MAX_TOUCH_POINTS,
                navigator.max_touch_points,
            )
            self._profile_set_bool(
                handle, field.NAVIGATOR_COOKIE_ENABLED, navigator.cookie_enabled
            )
            self._profile_set_bool(
                handle, field.NAVIGATOR_ON_LINE, navigator.on_line
            )
            self._profile_set_bool(
                handle, field.NAVIGATOR_WEBDRIVER, navigator.webdriver
            )
            self._profile_set_bool(
                handle,
                field.NAVIGATOR_PDF_VIEWER_ENABLED,
                navigator.pdf_viewer_enabled,
            )
            self._profile_set_string(
                handle, field.NAVIGATOR_DO_NOT_TRACK, navigator.do_not_track
            )
            self._profile_set_bool(
                handle,
                field.NAVIGATOR_USER_ACTIVATION_HAS_BEEN_ACTIVE,
                navigator.user_activation_has_been_active,
            )
            self._profile_set_bool(
                handle,
                field.NAVIGATOR_USER_ACTIVATION_IS_ACTIVE,
                navigator.user_activation_is_active,
            )

            user_agent_data = navigator.user_agent_data
            if user_agent_data is not None:
                if user_agent_data.brands is not None:
                    self._profile_call(
                        "edge_sandbox_profile_clear_ua_brands", handle
                    )
                    for brand in user_agent_data.brands:
                        brand_name = brand.brand.encode("utf-8")
                        version = brand.version.encode("utf-8")
                        full_version = brand.full_version.encode("utf-8")
                        self._profile_call(
                            "edge_sandbox_profile_append_ua_brand",
                            handle,
                            brand_name,
                            len(brand_name),
                            version,
                            len(version),
                            full_version,
                            len(full_version),
                        )
                self._profile_set_bool(
                    handle, field.UA_MOBILE, user_agent_data.mobile
                )
                self._profile_set_string(
                    handle, field.UA_PLATFORM, user_agent_data.platform
                )
                self._profile_set_string(
                    handle, field.UA_ARCHITECTURE, user_agent_data.architecture
                )
                self._profile_set_string(
                    handle, field.UA_BITNESS, user_agent_data.bitness
                )
                self._profile_set_string(
                    handle, field.UA_MODEL, user_agent_data.model
                )
                self._profile_set_string(
                    handle,
                    field.UA_PLATFORM_VERSION,
                    user_agent_data.platform_version,
                )
                self._profile_set_string(
                    handle, field.UA_FULL_VERSION, user_agent_data.ua_full_version
                )
                self._profile_set_bool(
                    handle, field.UA_WOW64, user_agent_data.wow64
                )
                self._profile_set_string_list(
                    handle, field.UA_FORM_FACTORS, user_agent_data.form_factors
                )

            network = navigator.network
            if network is not None:
                self._profile_set_string(
                    handle, field.NETWORK_EFFECTIVE_TYPE, network.effective_type
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_u32",
                    handle,
                    field.NETWORK_RTT,
                    network.rtt,
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.NETWORK_DOWNLINK,
                    network.downlink,
                )
                self._profile_set_bool(
                    handle, field.NETWORK_SAVE_DATA, network.save_data
                )

        screen = profile.screen
        if screen is not None:
            self._profile_set_number(
                "edge_sandbox_profile_set_i32", handle, field.SCREEN_WIDTH, screen.width
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.SCREEN_HEIGHT,
                screen.height,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.SCREEN_AVAIL_WIDTH,
                screen.avail_width if screen.avail_width is not None else screen.width,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.SCREEN_AVAIL_HEIGHT,
                screen.avail_height if screen.avail_height is not None else screen.height,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.SCREEN_AVAIL_LEFT,
                screen.avail_left,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.SCREEN_AVAIL_TOP,
                screen.avail_top,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.SCREEN_COLOR_DEPTH,
                screen.color_depth,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.SCREEN_PIXEL_DEPTH,
                screen.pixel_depth,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.SCREEN_X,
                screen.screen_x,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.SCREEN_Y,
                screen.screen_y,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.SCREEN_DEVICE_PIXEL_RATIO,
                screen.device_pixel_ratio,
            )
            self._profile_set_string(
                handle,
                field.SCREEN_ORIENTATION_TYPE,
                screen.orientation_type,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.SCREEN_ORIENTATION_ANGLE,
                screen.orientation_angle,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.VISUAL_VIEWPORT_OFFSET_LEFT,
                screen.visual_viewport_offset_left,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.VISUAL_VIEWPORT_OFFSET_TOP,
                screen.visual_viewport_offset_top,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.VISUAL_VIEWPORT_PAGE_LEFT,
                screen.visual_viewport_page_left,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.VISUAL_VIEWPORT_PAGE_TOP,
                screen.visual_viewport_page_top,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.VISUAL_VIEWPORT_SCALE,
                screen.visual_viewport_scale,
            )

        window = profile.window
        legacy_inner_width = screen.viewport_width if screen is not None else None
        legacy_inner_height = screen.viewport_height if screen is not None else None
        legacy_outer_width = screen.outer_width if screen is not None else None
        legacy_outer_height = screen.outer_height if screen is not None else None
        inner_width = (
            window.inner_width
            if window is not None and window.inner_width is not None
            else legacy_inner_width
        )
        inner_height = (
            window.inner_height
            if window is not None and window.inner_height is not None
            else legacy_inner_height
        )
        outer_width = (
            window.outer_width
            if window is not None and window.outer_width is not None
            else legacy_outer_width
        )
        outer_height = (
            window.outer_height
            if window is not None and window.outer_height is not None
            else legacy_outer_height
        )
        screen_width = None if screen is None else screen.width or screen.avail_width
        screen_height = None if screen is None else screen.height or screen.avail_height
        if inner_width is None:
            inner_width = screen_width if screen_width is not None else outer_width
        if outer_width is None:
            outer_width = screen_width if screen_width is not None else inner_width
        if inner_height is None:
            inner_height = screen_height if screen_height is not None else outer_height
        if outer_height is None:
            outer_height = screen_height if screen_height is not None else inner_height
        for field_id, value in (
            (field.WINDOW_INNER_WIDTH, inner_width),
            (field.WINDOW_INNER_HEIGHT, inner_height),
            (field.WINDOW_OUTER_WIDTH, outer_width),
            (field.WINDOW_OUTER_HEIGHT, outer_height),
        ):
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field_id,
                value,
            )

        canvas = profile.canvas
        if canvas is not None:
            self._profile_set_string(
                handle, field.CANVAS_DATA_URL_SALT, canvas.data_url_salt
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.CANVAS_TEXT_WIDTH_SCALE,
                canvas.text_width_scale,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.CANVAS_ACTUAL_BOUNDING_BOX_LEFT,
                canvas.actual_bounding_box_left,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.CANVAS_ACTUAL_BOUNDING_BOX_RIGHT_SCALE,
                canvas.actual_bounding_box_right_scale,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.CANVAS_FONT_BOUNDING_BOX_ASCENT,
                canvas.font_bounding_box_ascent,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.CANVAS_FONT_BOUNDING_BOX_DESCENT,
                canvas.font_bounding_box_descent,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.CANVAS_ACTUAL_BOUNDING_BOX_ASCENT,
                canvas.actual_bounding_box_ascent,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.CANVAS_ACTUAL_BOUNDING_BOX_DESCENT,
                canvas.actual_bounding_box_descent,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.CANVAS_HANGING_BASELINE,
                canvas.hanging_baseline,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.CANVAS_ALPHABETIC_BASELINE,
                canvas.alphabetic_baseline,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.CANVAS_IDEOGRAPHIC_BASELINE,
                canvas.ideographic_baseline,
            )

        webgl = profile.webgl
        if webgl is not None:
            self._profile_set_string(handle, field.WEBGL_VENDOR, webgl.vendor)
            self._profile_set_string(handle, field.WEBGL_RENDERER, webgl.renderer)
            self._profile_set_string(
                handle, field.WEBGL_UNMASKED_VENDOR, webgl.unmasked_vendor
            )
            self._profile_set_string(
                handle, field.WEBGL_UNMASKED_RENDERER, webgl.unmasked_renderer
            )
            self._profile_set_string(
                handle, field.WEBGL1_VERSION, webgl.webgl1_version
            )
            self._profile_set_string(
                handle,
                field.WEBGL1_SHADING_LANGUAGE_VERSION,
                webgl.webgl1_shading_language_version,
            )
            self._profile_set_string(
                handle, field.WEBGL2_VERSION, webgl.webgl2_version
            )
            self._profile_set_string(
                handle,
                field.WEBGL2_SHADING_LANGUAGE_VERSION,
                webgl.webgl2_shading_language_version,
            )
            self._profile_set_string(
                handle,
                field.WEBGL_CONTEXT_POWER_PREFERENCE,
                webgl.context_power_preference,
            )
            self._profile_set_string_list(
                handle, field.WEBGL1_EXTENSIONS, webgl.webgl1_extensions
            )
            self._profile_set_string_list(
                handle, field.WEBGL2_EXTENSIONS, webgl.webgl2_extensions
            )
            if webgl.compressed_texture_formats is not None:
                self._profile_call(
                    "edge_sandbox_profile_clear_webgl_compressed_texture_formats",
                    handle,
                )
                for texture_format in webgl.compressed_texture_formats:
                    self._profile_call(
                        "edge_sandbox_profile_append_webgl_compressed_texture_format",
                        handle,
                        texture_format,
                    )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_MAX_TEXTURE_SIZE,
                webgl.max_texture_size,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_MAX_CUBE_MAP_TEXTURE_SIZE,
                webgl.max_cube_map_texture_size,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_MAX_RENDERBUFFER_SIZE,
                webgl.max_renderbuffer_size,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_MAX_VIEWPORT_WIDTH,
                webgl.max_viewport_width,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_MAX_VIEWPORT_HEIGHT,
                webgl.max_viewport_height,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_MAX_VERTEX_ATTRIBS,
                webgl.max_vertex_attribs,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_MAX_VERTEX_UNIFORM_VECTORS,
                webgl.max_vertex_uniform_vectors,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_MAX_VARYING_VECTORS,
                webgl.max_varying_vectors,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_MAX_FRAGMENT_UNIFORM_VECTORS,
                webgl.max_fragment_uniform_vectors,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_MAX_VERTEX_TEXTURE_IMAGE_UNITS,
                webgl.max_vertex_texture_image_units,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_MAX_TEXTURE_IMAGE_UNITS,
                webgl.max_texture_image_units,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_MAX_COMBINED_TEXTURE_IMAGE_UNITS,
                webgl.max_combined_texture_image_units,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_SUBPIXEL_BITS,
                webgl.subpixel_bits,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_3D_TEXTURE_SIZE,
                webgl.webgl2_max_3d_texture_size,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_ARRAY_TEXTURE_LAYERS,
                webgl.webgl2_max_array_texture_layers,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_DRAW_BUFFERS,
                webgl.webgl2_max_draw_buffers,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_COLOR_ATTACHMENTS,
                webgl.webgl2_max_color_attachments,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_SAMPLES,
                webgl.webgl2_max_samples,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_VERTEX_UNIFORM_COMPONENTS,
                webgl.webgl2_max_vertex_uniform_components,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_FRAGMENT_UNIFORM_COMPONENTS,
                webgl.webgl2_max_fragment_uniform_components,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_VARYING_COMPONENTS,
                webgl.webgl2_max_varying_components,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_VERTEX_OUTPUT_COMPONENTS,
                webgl.webgl2_max_vertex_output_components,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_FRAGMENT_INPUT_COMPONENTS,
                webgl.webgl2_max_fragment_input_components,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_VERTEX_UNIFORM_BLOCKS,
                webgl.webgl2_max_vertex_uniform_blocks,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_FRAGMENT_UNIFORM_BLOCKS,
                webgl.webgl2_max_fragment_uniform_blocks,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_COMBINED_UNIFORM_BLOCKS,
                webgl.webgl2_max_combined_uniform_blocks,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_UNIFORM_BUFFER_BINDINGS,
                webgl.webgl2_max_uniform_buffer_bindings,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_UNIFORM_BLOCK_SIZE,
                webgl.webgl2_max_uniform_block_size,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS,
                webgl.webgl2_max_combined_vertex_uniform_components,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS,
                webgl.webgl2_max_combined_fragment_uniform_components,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS,
                webgl.webgl2_max_transform_feedback_separate_attribs,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS,
                webgl.webgl2_max_transform_feedback_interleaved_components,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS,
                webgl.webgl2_max_transform_feedback_separate_components,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_PROGRAM_TEXEL_OFFSET,
                webgl.webgl2_max_program_texel_offset,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_ELEMENTS_VERTICES,
                webgl.webgl2_max_elements_vertices,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL2_MAX_ELEMENTS_INDICES,
                webgl.webgl2_max_elements_indices,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGL2_MAX_ELEMENT_INDEX,
                webgl.webgl2_max_element_index,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.WEBGL2_MAX_TEXTURE_LOD_BIAS,
                webgl.webgl2_max_texture_lod_bias,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_SHADER_PRECISION_RANGE_MIN,
                webgl.shader_precision_range_min,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_SHADER_PRECISION_RANGE_MAX,
                webgl.shader_precision_range_max,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_i32",
                handle,
                field.WEBGL_SHADER_PRECISION_BITS,
                webgl.shader_precision_bits,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.WEBGL_MAX_ANISOTROPY,
                webgl.max_anisotropy,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.WEBGL_ALIASED_POINT_SIZE_MIN,
                webgl.aliased_point_size_min,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.WEBGL_ALIASED_POINT_SIZE_MAX,
                webgl.aliased_point_size_max,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.WEBGL_ALIASED_LINE_WIDTH_MIN,
                webgl.aliased_line_width_min,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.WEBGL_ALIASED_LINE_WIDTH_MAX,
                webgl.aliased_line_width_max,
            )
            self._profile_set_bool(
                handle, field.WEBGL_CONTEXT_ALPHA, webgl.context_alpha
            )
            self._profile_set_bool(
                handle,
                field.WEBGL_CONTEXT_ANTIALIAS,
                webgl.context_antialias,
            )
            self._profile_set_bool(
                handle, field.WEBGL_CONTEXT_DEPTH, webgl.context_depth
            )
            self._profile_set_bool(
                handle,
                field.WEBGL_CONTEXT_DESYNCHRONIZED,
                webgl.context_desynchronized,
            )
            self._profile_set_bool(
                handle,
                field.WEBGL_CONTEXT_FAIL_IF_MAJOR_PERFORMANCE_CAVEAT,
                webgl.context_fail_if_major_performance_caveat,
            )
            self._profile_set_bool(
                handle,
                field.WEBGL_CONTEXT_PREMULTIPLIED_ALPHA,
                webgl.context_premultiplied_alpha,
            )
            self._profile_set_bool(
                handle,
                field.WEBGL_CONTEXT_PRESERVE_DRAWING_BUFFER,
                webgl.context_preserve_drawing_buffer,
            )
            self._profile_set_bool(
                handle, field.WEBGL_CONTEXT_STENCIL, webgl.context_stencil
            )
            self._profile_set_bool(
                handle,
                field.WEBGL_CONTEXT_XR_COMPATIBLE,
                webgl.context_xr_compatible,
            )

        webgpu = profile.webgpu
        if webgpu is not None:
            self._profile_set_bool(
                handle,
                field.WEBGPU_AVAILABLE,
                webgpu.available,
            )
            self._profile_set_string(handle, field.WEBGPU_VENDOR, webgpu.vendor)
            self._profile_set_string(
                handle, field.WEBGPU_ARCHITECTURE, webgpu.architecture
            )
            self._profile_set_string(handle, field.WEBGPU_DEVICE, webgpu.device)
            self._profile_set_string(
                handle, field.WEBGPU_DESCRIPTION, webgpu.description
            )
            self._profile_set_bool(
                handle,
                field.WEBGPU_DEVELOPER_FEATURES,
                webgpu.developer_features,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_SUBGROUP_MIN_SIZE,
                webgpu.subgroup_min_size,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_SUBGROUP_MAX_SIZE,
                webgpu.subgroup_max_size,
            )
            self._profile_set_bool(
                handle,
                field.WEBGPU_IS_FALLBACK_ADAPTER,
                webgpu.is_fallback_adapter,
            )
            self._profile_set_string_list(
                handle, field.WEBGPU_FEATURES, webgpu.features
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_TEXTURE_DIMENSION_1D,
                webgpu.max_texture_dimension_1d,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_TEXTURE_DIMENSION_2D,
                webgpu.max_texture_dimension_2d,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_TEXTURE_DIMENSION_3D,
                webgpu.max_texture_dimension_3d,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_TEXTURE_ARRAY_LAYERS,
                webgpu.max_texture_array_layers,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_BIND_GROUPS,
                webgpu.max_bind_groups,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_BIND_GROUPS_PLUS_VERTEX_BUFFERS,
                webgpu.max_bind_groups_plus_vertex_buffers,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_BINDINGS_PER_BIND_GROUP,
                webgpu.max_bindings_per_bind_group,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_DYNAMIC_UNIFORM_BUFFERS_PER_PIPELINE_LAYOUT,
                webgpu.max_dynamic_uniform_buffers_per_pipeline_layout,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_DYNAMIC_STORAGE_BUFFERS_PER_PIPELINE_LAYOUT,
                webgpu.max_dynamic_storage_buffers_per_pipeline_layout,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_SAMPLED_TEXTURES_PER_SHADER_STAGE,
                webgpu.max_sampled_textures_per_shader_stage,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_SAMPLERS_PER_SHADER_STAGE,
                webgpu.max_samplers_per_shader_stage,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_STORAGE_BUFFERS_PER_SHADER_STAGE,
                webgpu.max_storage_buffers_per_shader_stage,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_STORAGE_TEXTURES_PER_SHADER_STAGE,
                webgpu.max_storage_textures_per_shader_stage,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_UNIFORM_BUFFERS_PER_SHADER_STAGE,
                webgpu.max_uniform_buffers_per_shader_stage,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u64",
                handle,
                field.WEBGPU_MAX_UNIFORM_BUFFER_BINDING_SIZE,
                webgpu.max_uniform_buffer_binding_size,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u64",
                handle,
                field.WEBGPU_MAX_STORAGE_BUFFER_BINDING_SIZE,
                webgpu.max_storage_buffer_binding_size,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MIN_UNIFORM_BUFFER_OFFSET_ALIGNMENT,
                webgpu.min_uniform_buffer_offset_alignment,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MIN_STORAGE_BUFFER_OFFSET_ALIGNMENT,
                webgpu.min_storage_buffer_offset_alignment,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_VERTEX_BUFFERS,
                webgpu.max_vertex_buffers,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u64",
                handle,
                field.WEBGPU_MAX_BUFFER_SIZE,
                webgpu.max_buffer_size,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_VERTEX_ATTRIBUTES,
                webgpu.max_vertex_attributes,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_VERTEX_BUFFER_ARRAY_STRIDE,
                webgpu.max_vertex_buffer_array_stride,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_INTER_STAGE_SHADER_VARIABLES,
                webgpu.max_inter_stage_shader_variables,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_COLOR_ATTACHMENTS,
                webgpu.max_color_attachments,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_COLOR_ATTACHMENT_BYTES_PER_SAMPLE,
                webgpu.max_color_attachment_bytes_per_sample,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_COMPUTE_WORKGROUP_STORAGE_SIZE,
                webgpu.max_compute_workgroup_storage_size,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_COMPUTE_INVOCATIONS_PER_WORKGROUP,
                webgpu.max_compute_invocations_per_workgroup,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_COMPUTE_WORKGROUP_SIZE_X,
                webgpu.max_compute_workgroup_size_x,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_COMPUTE_WORKGROUP_SIZE_Y,
                webgpu.max_compute_workgroup_size_y,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_COMPUTE_WORKGROUP_SIZE_Z,
                webgpu.max_compute_workgroup_size_z,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_COMPUTE_WORKGROUPS_PER_DIMENSION,
                webgpu.max_compute_workgroups_per_dimension,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_IMMEDIATE_SIZE,
                webgpu.max_immediate_size,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_STORAGE_BUFFERS_IN_FRAGMENT_STAGE,
                webgpu.max_storage_buffers_in_fragment_stage,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_STORAGE_TEXTURES_IN_FRAGMENT_STAGE,
                webgpu.max_storage_textures_in_fragment_stage,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_STORAGE_BUFFERS_IN_VERTEX_STAGE,
                webgpu.max_storage_buffers_in_vertex_stage,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.WEBGPU_MAX_STORAGE_TEXTURES_IN_VERTEX_STAGE,
                webgpu.max_storage_textures_in_vertex_stage,
            )

        audio = profile.audio
        if audio is not None:
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.AUDIO_SAMPLE_RATE,
                audio.sample_rate,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.AUDIO_MAX_CHANNEL_COUNT,
                audio.max_channel_count,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.AUDIO_BASE_LATENCY,
                audio.base_latency,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.AUDIO_OUTPUT_LATENCY,
                audio.output_latency,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u64",
                handle,
                field.AUDIO_NOISE_SEED,
                audio.noise_seed,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f32",
                handle,
                field.AUDIO_CHANNEL_NOISE_AMPLITUDE,
                audio.channel_noise_amplitude,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f32",
                handle,
                field.AUDIO_FREQUENCY_NOISE_AMPLITUDE,
                audio.frequency_noise_amplitude,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f32",
                handle,
                field.AUDIO_TIME_DOMAIN_NOISE_AMPLITUDE,
                audio.time_domain_noise_amplitude,
            )

        storage = profile.storage
        if storage is not None:
            self._profile_set_number(
                "edge_sandbox_profile_set_u64",
                handle,
                field.STORAGE_QUOTA_BYTES,
                storage.quota_bytes,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u64",
                handle,
                field.STORAGE_USAGE_BYTES,
                storage.usage_bytes,
            )
            self._profile_set_bool(
                handle, field.STORAGE_PERSISTED, storage.persisted
            )

        speech = profile.speech
        if speech is not None and speech.voices is not None:
            self._profile_call("edge_sandbox_profile_clear_speech_voices", handle)
            for voice in speech.voices:
                voice_uri = voice.voice_uri.encode("utf-8")
                name = voice.name.encode("utf-8")
                lang = voice.lang.encode("utf-8")
                self._profile_call(
                    "edge_sandbox_profile_append_speech_voice",
                    handle,
                    voice_uri,
                    len(voice_uri),
                    name,
                    len(name),
                    lang,
                    len(lang),
                    voice.local_service,
                    voice.is_default,
                )

        fonts = profile.fonts
        if fonts is not None:
            self._profile_set_string_list(
                handle, field.FONT_FAMILIES, fonts.families
            )
            self._profile_set_bool(
                handle,
                field.FONT_ALLOW_UNKNOWN_FAMILIES,
                fonts.allow_unknown_families,
            )
            if fonts.local_fonts is not None:
                self._profile_call(
                    "edge_sandbox_profile_clear_local_fonts", handle
                )
                for local_font in fonts.local_fonts:
                    postscript_name = local_font.postscript_name.encode("utf-8")
                    full_name = local_font.full_name.encode("utf-8")
                    family = local_font.family.encode("utf-8")
                    style = local_font.style.encode("utf-8")
                    self._profile_call(
                        "edge_sandbox_profile_append_local_font",
                        handle,
                        postscript_name,
                        len(postscript_name),
                        full_name,
                        len(full_name),
                        family,
                        len(family),
                        style,
                        len(style),
                    )
            if fonts.metrics is not None:
                self._profile_call(
                    "edge_sandbox_profile_clear_font_metrics", handle
                )
                for metric in fonts.metrics:
                    family = metric.family.encode("utf-8")
                    self._profile_call(
                        "edge_sandbox_profile_append_font_metric",
                        handle,
                        family,
                        len(family),
                        metric.width_scale,
                        metric.monospace,
                    )

        css = profile.css
        if css is not None:
            self._profile_set_string(handle, field.CSS_BODY, css.body)
            self._profile_set_string(
                handle, field.CSS_INPUT_COMMON, css.input_common
            )
            self._profile_set_string(
                handle, field.CSS_INPUT_HIDDEN, css.input_hidden
            )
            self._profile_set_string(
                handle, field.CSS_INPUT_SEARCH, css.input_search
            )
            self._profile_set_string(
                handle,
                field.CSS_INPUT_CHECKBOX_RADIO,
                css.input_checkbox_radio,
            )
            self._profile_set_string(
                handle, field.CSS_INPUT_RANGE, css.input_range
            )
            self._profile_set_string(
                handle, field.CSS_INPUT_COLOR, css.input_color
            )
            self._profile_set_string(handle, field.CSS_INPUT_DATE, css.input_date)
            self._profile_set_string(handle, field.CSS_INPUT_TIME, css.input_time)
            self._profile_set_string(
                handle,
                field.CSS_INPUT_DATETIME_LOCAL,
                css.input_datetime_local,
            )
            self._profile_set_string(
                handle, field.CSS_INPUT_MONTH, css.input_month
            )
            self._profile_set_string(handle, field.CSS_INPUT_WEEK, css.input_week)
            self._profile_set_string(
                handle, field.CSS_INPUT_IMAGE, css.input_image
            )
            self._profile_set_string(
                handle, field.CSS_INPUT_BUTTON, css.input_button
            )
            self._profile_set_string(
                handle,
                field.CSS_INPUT_SUBMIT_RESET,
                css.input_submit_reset,
            )
            self._profile_set_string(handle, field.CSS_INPUT_FILE, css.input_file)
            self._profile_set_string(handle, field.CSS_INPUT_TEXT, css.input_text)

        document = profile.document
        if document is not None:
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.DOCUMENT_BODY_CHILD_ELEMENT_COUNT,
                document.body_child_element_count,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.DOCUMENT_BODY_CLIENT_HEIGHT,
                document.body_client_height,
            )

        media = profile.media
        if media is not None:
            if media.devices is not None:
                self._profile_call(
                    "edge_sandbox_profile_clear_media_devices", handle
                )
                for device in media.devices:
                    device_id = device.device_id.encode("utf-8")
                    kind = device.kind.encode("utf-8")
                    label = device.label.encode("utf-8")
                    group_id = device.group_id.encode("utf-8")
                    self._profile_call(
                        "edge_sandbox_profile_append_media_device",
                        handle,
                        device_id,
                        len(device_id),
                        kind,
                        len(kind),
                        label,
                        len(label),
                        group_id,
                        len(group_id),
                    )
            self._profile_set_string_list(
                handle,
                field.MEDIA_SUPPORTED_CONSTRAINTS,
                media.supported_constraints,
            )
            self._profile_set_string_list(
                handle,
                field.MEDIA_CAN_PLAY_PROBABLY_TYPES,
                media.can_play_probably_types,
            )
            self._profile_set_string_list(
                handle,
                field.MEDIA_CAN_PLAY_MAYBE_TYPES,
                media.can_play_maybe_types,
            )
            self._profile_set_string_list(
                handle, field.MEDIA_SOURCE_TYPES, media.media_source_types
            )
            self._profile_set_string_list(
                handle, field.MEDIA_RECORDER_TYPES, media.media_recorder_types
            )
            self._profile_set_string_list(
                handle,
                field.MEDIA_DECODING_SUPPORTED_TYPES,
                media.decoding_supported_types,
            )
            self._profile_set_string_list(
                handle,
                field.MEDIA_DECODING_SMOOTH_TYPES,
                media.decoding_smooth_types,
            )
            self._profile_set_string_list(
                handle,
                field.MEDIA_DECODING_POWER_EFFICIENT_TYPES,
                media.decoding_power_efficient_types,
            )
            self._profile_set_string_list(
                handle,
                field.MEDIA_ENCODING_SUPPORTED_TYPES,
                media.encoding_supported_types,
            )
            self._profile_set_string_list(
                handle,
                field.MEDIA_ENCODING_SMOOTH_TYPES,
                media.encoding_smooth_types,
            )
            self._profile_set_string_list(
                handle,
                field.MEDIA_ENCODING_POWER_EFFICIENT_TYPES,
                media.encoding_power_efficient_types,
            )
            self._profile_set_string_list(
                handle,
                field.IMAGE_DECODER_TYPES,
                media.image_decoder_types,
            )
            self._profile_set_string_list(
                handle,
                field.AUDIO_DECODER_CODECS,
                media.audio_decoder_codecs,
            )
            self._profile_set_string_list(
                handle,
                field.AUDIO_ENCODER_CODECS,
                media.audio_encoder_codecs,
            )
            self._profile_set_string_list(
                handle,
                field.VIDEO_DECODER_CODECS,
                media.video_decoder_codecs,
            )
            self._profile_set_string_list(
                handle,
                field.VIDEO_ENCODER_CODECS,
                media.video_encoder_codecs,
            )
            self._profile_set_string(
                handle, field.RTC_OFFER_SDP, media.rtc_offer_sdp
            )
            self._profile_set_string(
                handle, field.RTC_ANSWER_SDP, media.rtc_answer_sdp
            )
            if media.rtc_audio_codecs is not None:
                self._profile_call(
                    "edge_sandbox_profile_clear_rtc_audio_codecs", handle
                )
                for codec in media.rtc_audio_codecs:
                    mime_type = codec.mime_type.encode("utf-8")
                    fmtp = (
                        codec.sdp_fmtp_line.encode("utf-8")
                        if codec.sdp_fmtp_line is not None
                        else b""
                    )
                    self._profile_call(
                        "edge_sandbox_profile_append_rtc_audio_codec",
                        handle,
                        mime_type,
                        len(mime_type),
                        codec.clock_rate,
                        codec.channels or 0,
                        codec.channels is not None,
                        fmtp,
                        len(fmtp),
                        codec.sdp_fmtp_line is not None,
                    )
            if media.rtc_video_codecs is not None:
                self._profile_call(
                    "edge_sandbox_profile_clear_rtc_video_codecs", handle
                )
                for codec in media.rtc_video_codecs:
                    mime_type = codec.mime_type.encode("utf-8")
                    fmtp = (
                        codec.sdp_fmtp_line.encode("utf-8")
                        if codec.sdp_fmtp_line is not None
                        else b""
                    )
                    self._profile_call(
                        "edge_sandbox_profile_append_rtc_video_codec",
                        handle,
                        mime_type,
                        len(mime_type),
                        codec.clock_rate,
                        codec.channels or 0,
                        codec.channels is not None,
                        fmtp,
                        len(fmtp),
                        codec.sdp_fmtp_line is not None,
                    )
            if media.rtc_header_extensions is not None:
                self._profile_call(
                    "edge_sandbox_profile_clear_rtc_header_extensions", handle
                )
                for extension in media.rtc_header_extensions:
                    kind = extension.kind.encode("utf-8")
                    uri = extension.uri.encode("utf-8")
                    self._profile_call(
                        "edge_sandbox_profile_append_rtc_header_extension",
                        handle,
                        kind,
                        len(kind),
                        uri,
                        len(uri),
                    )

        permissions = profile.permissions
        if permissions is not None:
            self._profile_set_string(
                handle,
                field.PERMISSION_ACCELEROMETER,
                permissions.accelerometer,
            )
            self._profile_set_string(
                handle,
                field.PERMISSION_BACKGROUND_SYNC,
                permissions.background_sync,
            )
            self._profile_set_string(
                handle, field.PERMISSION_CAMERA, permissions.camera
            )
            self._profile_set_string(
                handle,
                field.PERMISSION_CLIPBOARD_READ,
                permissions.clipboard_read,
            )
            self._profile_set_string(
                handle,
                field.PERMISSION_CLIPBOARD_WRITE,
                permissions.clipboard_write,
            )
            self._profile_set_string(
                handle,
                field.PERMISSION_GEOLOCATION,
                permissions.geolocation,
            )
            self._profile_set_string(
                handle, field.PERMISSION_GYROSCOPE, permissions.gyroscope
            )
            self._profile_set_string(
                handle,
                field.PERMISSION_LOCAL_FONTS,
                permissions.local_fonts,
            )
            self._profile_set_string(
                handle,
                field.PERMISSION_MAGNETOMETER,
                permissions.magnetometer,
            )
            self._profile_set_string(
                handle,
                field.PERMISSION_MICROPHONE,
                permissions.microphone,
            )
            self._profile_set_string(
                handle, field.PERMISSION_MIDI, permissions.midi
            )
            self._profile_set_string(
                handle,
                field.PERMISSION_NOTIFICATIONS,
                permissions.notifications,
            )
            self._profile_set_string(
                handle,
                field.PERMISSION_PAYMENT_HANDLER,
                permissions.payment_handler,
            )
            self._profile_set_string(
                handle,
                field.PERMISSION_PERSISTENT_STORAGE,
                permissions.persistent_storage,
            )
            self._profile_set_string(
                handle,
                field.PERMISSION_SPEAKER_SELECTION,
                permissions.speaker_selection,
            )
            self._profile_set_string(
                handle,
                field.PERMISSION_STORAGE_ACCESS,
                permissions.storage_access,
            )
            self._profile_set_string(
                handle,
                field.PERMISSION_TOP_LEVEL_STORAGE_ACCESS,
                permissions.top_level_storage_access,
            )
            self._profile_set_string(
                handle,
                field.PERMISSION_WINDOW_MANAGEMENT,
                permissions.window_management,
            )

        battery = profile.battery
        if battery is not None:
            self._profile_set_bool(
                handle, field.BATTERY_CHARGING, battery.charging
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.BATTERY_CHARGING_TIME,
                battery.charging_time,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.BATTERY_DISCHARGING_TIME,
                battery.discharging_time,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.BATTERY_LEVEL,
                battery.level,
            )

        geolocation = profile.geolocation
        if geolocation is not None:
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.GEOLOCATION_LATITUDE,
                geolocation.latitude,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.GEOLOCATION_LONGITUDE,
                geolocation.longitude,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.GEOLOCATION_ALTITUDE,
                geolocation.altitude,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.GEOLOCATION_ACCURACY,
                geolocation.accuracy,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.GEOLOCATION_ALTITUDE_ACCURACY,
                geolocation.altitude_accuracy,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.GEOLOCATION_HEADING,
                geolocation.heading,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_f64",
                handle,
                field.GEOLOCATION_SPEED,
                geolocation.speed,
            )

        preferences = profile.media_preferences
        if preferences is not None:
            self._profile_set_string(
                handle,
                field.MEDIA_PREFERENCE_COLOR_SCHEME,
                preferences.color_scheme,
            )
            self._profile_set_string(
                handle,
                field.MEDIA_PREFERENCE_CONTRAST,
                preferences.contrast,
            )
            self._profile_set_bool(
                handle,
                field.MEDIA_PREFERENCE_REDUCED_MOTION,
                preferences.reduced_motion,
            )
            self._profile_set_bool(
                handle,
                field.MEDIA_PREFERENCE_REDUCED_TRANSPARENCY,
                preferences.reduced_transparency,
            )
            self._profile_set_bool(
                handle,
                field.MEDIA_PREFERENCE_REDUCED_DATA,
                preferences.reduced_data,
            )
            self._profile_set_bool(
                handle,
                field.MEDIA_PREFERENCE_FORCED_COLORS,
                preferences.forced_colors,
            )
            self._profile_set_bool(
                handle,
                field.MEDIA_PREFERENCE_INVERTED_COLORS,
                preferences.inverted_colors,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u32",
                handle,
                field.MEDIA_PREFERENCE_MONOCHROME_BITS,
                preferences.monochrome_bits,
            )
            self._profile_set_string(
                handle,
                field.MEDIA_PREFERENCE_COLOR_GAMUT,
                preferences.color_gamut,
            )
            self._profile_set_string(
                handle,
                field.MEDIA_PREFERENCE_POINTER,
                preferences.pointer,
            )
            self._profile_set_string(
                handle,
                field.MEDIA_PREFERENCE_ANY_POINTER,
                preferences.any_pointer,
            )
            self._profile_set_string(
                handle, field.MEDIA_PREFERENCE_HOVER, preferences.hover
            )
            self._profile_set_string(
                handle,
                field.MEDIA_PREFERENCE_ANY_HOVER,
                preferences.any_hover,
            )
            self._profile_set_string(
                handle,
                field.MEDIA_PREFERENCE_DISPLAY_MODE,
                preferences.display_mode,
            )
            self._profile_set_string(
                handle,
                field.MEDIA_PREFERENCE_DYNAMIC_RANGE,
                preferences.dynamic_range,
            )
            self._profile_set_string(
                handle,
                field.MEDIA_PREFERENCE_VIDEO_DYNAMIC_RANGE,
                preferences.video_dynamic_range,
            )
            self._profile_set_string(
                handle,
                field.MEDIA_PREFERENCE_SCRIPTING,
                preferences.scripting,
            )

        plugins = profile.plugins
        if plugins is not None and plugins.plugins is not None:
            self._profile_call("edge_sandbox_profile_clear_plugins", handle)
            for plugin_index, plugin in enumerate(plugins.plugins):
                name = plugin.name.encode("utf-8")
                filename = plugin.filename.encode("utf-8")
                description = plugin.description.encode("utf-8")
                self._profile_call(
                    "edge_sandbox_profile_append_plugin",
                    handle,
                    name,
                    len(name),
                    filename,
                    len(filename),
                    description,
                    len(description),
                )
                for mime in plugin.mime_types:
                    mime_type = mime.mime_type.encode("utf-8")
                    suffixes = mime.suffixes.encode("utf-8")
                    mime_description = mime.description.encode("utf-8")
                    self._profile_call(
                        "edge_sandbox_profile_append_plugin_mime_type",
                        handle,
                        plugin_index,
                        mime_type,
                        len(mime_type),
                        suffixes,
                        len(suffixes),
                        mime_description,
                        len(mime_description),
                    )

        hardware = profile.hardware_devices
        if hardware is not None:
            if hardware.gamepads is not None:
                self._profile_call("edge_sandbox_profile_clear_gamepads", handle)
                for gamepad in hardware.gamepads:
                    gamepad_id = gamepad.id.encode("utf-8")
                    mapping = gamepad.mapping.encode("utf-8")
                    axes_type = ctypes.c_double * len(gamepad.axes)
                    buttons_type = ctypes.c_double * len(gamepad.buttons)
                    axes = axes_type(*gamepad.axes) if gamepad.axes else None
                    buttons = (
                        buttons_type(*gamepad.buttons)
                        if gamepad.buttons
                        else None
                    )
                    self._profile_call(
                        "edge_sandbox_profile_append_gamepad",
                        handle,
                        gamepad_id,
                        len(gamepad_id),
                        gamepad.index,
                        gamepad.connected,
                        mapping,
                        len(mapping),
                        axes,
                        len(gamepad.axes),
                        buttons,
                        len(gamepad.buttons),
                    )

            if hardware.usb_devices is not None:
                self._profile_call(
                    "edge_sandbox_profile_clear_usb_devices", handle
                )
                for device in hardware.usb_devices:
                    manufacturer = (
                        device.manufacturer_name.encode("utf-8")
                        if device.manufacturer_name is not None
                        else b""
                    )
                    product_name = (
                        device.product_name.encode("utf-8")
                        if device.product_name is not None
                        else b""
                    )
                    serial_number = (
                        device.serial_number.encode("utf-8")
                        if device.serial_number is not None
                        else b""
                    )
                    self._profile_call(
                        "edge_sandbox_profile_append_usb_device",
                        handle,
                        device.usb_version_major,
                        device.usb_version_minor,
                        device.usb_version_subminor,
                        device.device_class,
                        device.device_subclass,
                        device.device_protocol,
                        device.vendor_id,
                        device.product_id,
                        device.device_version_major,
                        device.device_version_minor,
                        device.device_version_subminor,
                        manufacturer,
                        len(manufacturer),
                        device.manufacturer_name is not None,
                        product_name,
                        len(product_name),
                        device.product_name is not None,
                        serial_number,
                        len(serial_number),
                        device.serial_number is not None,
                    )

            if hardware.hid_devices is not None:
                self._profile_call(
                    "edge_sandbox_profile_clear_hid_devices", handle
                )
                for device in hardware.hid_devices:
                    product_name = device.product_name.encode("utf-8")
                    self._profile_call(
                        "edge_sandbox_profile_append_hid_device",
                        handle,
                        device.vendor_id,
                        device.product_id,
                        product_name,
                        len(product_name),
                    )

            if hardware.serial_ports is not None:
                self._profile_call(
                    "edge_sandbox_profile_clear_serial_ports", handle
                )
                for port in hardware.serial_ports:
                    self._profile_call(
                        "edge_sandbox_profile_append_serial_port",
                        handle,
                        port.usb_vendor_id,
                        port.usb_product_id,
                        port.connected,
                    )

            self._profile_set_bool(
                handle,
                field.BLUETOOTH_AVAILABLE,
                hardware.bluetooth_available,
            )
            if hardware.bluetooth_devices is not None:
                self._profile_call(
                    "edge_sandbox_profile_clear_bluetooth_devices", handle
                )
                for device in hardware.bluetooth_devices:
                    device_id = device.id.encode("utf-8")
                    name = (
                        device.name.encode("utf-8")
                        if device.name is not None
                        else b""
                    )
                    self._profile_call(
                        "edge_sandbox_profile_append_bluetooth_device",
                        handle,
                        device_id,
                        len(device_id),
                        name,
                        len(name),
                        device.name is not None,
                    )
            if hardware.keyboard_layout is not None:
                self._profile_call(
                    "edge_sandbox_profile_clear_keyboard_layout", handle
                )
                for entry in hardware.keyboard_layout:
                    code = entry.code.encode("utf-8")
                    value = entry.value.encode("utf-8")
                    self._profile_call(
                        "edge_sandbox_profile_append_keyboard_layout_entry",
                        handle,
                        code,
                        len(code),
                        value,
                        len(value),
                    )
            self._profile_set_string(
                handle,
                field.DEVICE_POSTURE,
                hardware.device_posture,
            )
            if hardware.midi_inputs is not None:
                self._profile_call(
                    "edge_sandbox_profile_clear_midi_inputs", handle
                )
                for port in hardware.midi_inputs:
                    port_id = port.id.encode("utf-8")
                    manufacturer = port.manufacturer.encode("utf-8")
                    name = port.name.encode("utf-8")
                    version = port.version.encode("utf-8")
                    state = port.state.encode("utf-8")
                    connection = port.connection.encode("utf-8")
                    self._profile_call(
                        "edge_sandbox_profile_append_midi_input",
                        handle,
                        port_id,
                        len(port_id),
                        manufacturer,
                        len(manufacturer),
                        name,
                        len(name),
                        version,
                        len(version),
                        state,
                        len(state),
                        connection,
                        len(connection),
                    )
            if hardware.midi_outputs is not None:
                self._profile_call(
                    "edge_sandbox_profile_clear_midi_outputs", handle
                )
                for port in hardware.midi_outputs:
                    port_id = port.id.encode("utf-8")
                    manufacturer = port.manufacturer.encode("utf-8")
                    name = port.name.encode("utf-8")
                    version = port.version.encode("utf-8")
                    state = port.state.encode("utf-8")
                    connection = port.connection.encode("utf-8")
                    self._profile_call(
                        "edge_sandbox_profile_append_midi_output",
                        handle,
                        port_id,
                        len(port_id),
                        manufacturer,
                        len(manufacturer),
                        name,
                        len(name),
                        version,
                        len(version),
                        state,
                        len(state),
                        connection,
                        len(connection),
                    )
            self._profile_set_bool(
                handle,
                field.MIDI_SYSEX_ENABLED,
                hardware.midi_sysex_enabled,
            )

        sensors = profile.sensors
        if sensors is not None:
            self._profile_set_bool(
                handle,
                field.SENSORS_AVAILABLE,
                sensors.available,
            )
            if sensors.accelerometer is not None:
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_ACCELEROMETER_X,
                    sensors.accelerometer[0],
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_ACCELEROMETER_Y,
                    sensors.accelerometer[1],
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_ACCELEROMETER_Z,
                    sensors.accelerometer[2],
                )
            if sensors.gravity is not None:
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_GRAVITY_X,
                    sensors.gravity[0],
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_GRAVITY_Y,
                    sensors.gravity[1],
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_GRAVITY_Z,
                    sensors.gravity[2],
                )
            if sensors.linear_acceleration is not None:
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_LINEAR_ACCELERATION_X,
                    sensors.linear_acceleration[0],
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_LINEAR_ACCELERATION_Y,
                    sensors.linear_acceleration[1],
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_LINEAR_ACCELERATION_Z,
                    sensors.linear_acceleration[2],
                )
            if sensors.gyroscope is not None:
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_GYROSCOPE_X,
                    sensors.gyroscope[0],
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_GYROSCOPE_Y,
                    sensors.gyroscope[1],
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_GYROSCOPE_Z,
                    sensors.gyroscope[2],
                )
            if sensors.absolute_orientation_quaternion is not None:
                absolute = sensors.absolute_orientation_quaternion
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_ABSOLUTE_ORIENTATION_X,
                    absolute[0],
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_ABSOLUTE_ORIENTATION_Y,
                    absolute[1],
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_ABSOLUTE_ORIENTATION_Z,
                    absolute[2],
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_ABSOLUTE_ORIENTATION_W,
                    absolute[3],
                )
            if sensors.relative_orientation_quaternion is not None:
                relative = sensors.relative_orientation_quaternion
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_RELATIVE_ORIENTATION_X,
                    relative[0],
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_RELATIVE_ORIENTATION_Y,
                    relative[1],
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_RELATIVE_ORIENTATION_Z,
                    relative[2],
                )
                self._profile_set_number(
                    "edge_sandbox_profile_set_f64",
                    handle,
                    field.SENSOR_RELATIVE_ORIENTATION_W,
                    relative[3],
                )

        timing = profile.timing
        if timing is not None:
            self._profile_set_number(
                "edge_sandbox_profile_set_i64",
                handle,
                field.TIMING_CLOCK_EPOCH_MS,
                timing.clock_epoch_ms,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u64",
                handle,
                field.TIMING_CLOCK_STEP_MS,
                timing.clock_step_ms,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u64",
                handle,
                field.TIMING_RANDOM_SEED,
                timing.random_seed,
            )

        xr = profile.xr
        if xr is not None:
            self._profile_set_string_list(
                handle,
                field.XR_SUPPORTED_SESSION_MODES,
                xr.supported_session_modes,
            )

        memory = profile.memory
        if memory is not None:
            self._profile_set_number(
                "edge_sandbox_profile_set_u64",
                handle,
                field.PERFORMANCE_JS_HEAP_SIZE_LIMIT,
                memory.performance_js_heap_size_limit,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u64",
                handle,
                field.PERFORMANCE_TOTAL_JS_HEAP_SIZE,
                memory.performance_total_js_heap_size,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u64",
                handle,
                field.PERFORMANCE_USED_JS_HEAP_SIZE,
                memory.performance_used_js_heap_size,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u64",
                handle,
                field.CONSOLE_JS_HEAP_SIZE_LIMIT,
                memory.console_js_heap_size_limit,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u64",
                handle,
                field.CONSOLE_TOTAL_JS_HEAP_SIZE,
                memory.console_total_js_heap_size,
            )
            self._profile_set_number(
                "edge_sandbox_profile_set_u64",
                handle,
                field.CONSOLE_USED_JS_HEAP_SIZE,
                memory.console_used_js_heap_size,
            )

        performance = profile.performance
        if performance is not None:
            self._profile_set_string(
                handle,
                field.PERFORMANCE_EVALUATED_SCRIPT_CONTENT_ENCODING,
                performance.evaluated_script_content_encoding,
            )
            if performance.entries is not None:
                self._profile_call(
                    "edge_sandbox_profile_clear_performance_entries",
                    handle,
                )
                for entry in performance.entries:
                    self._profile_append_performance_entry(handle, entry)

        self._profile_call("edge_sandbox_profile_validate", handle)

    def _create_with_profile(
        self, profile: EdgeProfile, error: _NativeBuffer
    ) -> int | None:
        handle = self._create_profile_builder(profile)
        try:
            return self._library.edge_sandbox_create_self_hosted_with_profile(
                handle,
                ctypes.byref(error),
            )
        finally:
            self._library.edge_sandbox_profile_destroy(handle)

    def _create_profile_builder(self, profile: EdgeProfile) -> int:
        profile_error = _NativeBuffer()
        handle = self._library.edge_sandbox_profile_create(
            ctypes.byref(profile_error)
        )
        if not handle:
            raise SandboxExecutionError(self._consume_buffer(profile_error))
        self._consume_buffer(profile_error)
        try:
            self._apply_profile(handle, profile)
            return handle
        except BaseException:
            self._library.edge_sandbox_profile_destroy(handle)
            raise

    def _create_with_options(
        self,
        profile: EdgeProfile | None,
        options: EdgeRunOptions,
        error: _NativeBuffer,
    ) -> int | None:
        options_error = _NativeBuffer()
        options_handle = self._library.edge_sandbox_options_create(
            ctypes.byref(options_error)
        )
        if not options_handle:
            raise SandboxExecutionError(self._consume_buffer(options_error))
        self._consume_buffer(options_error)
        profile_handle: int | None = None
        try:
            if profile is not None:
                profile_handle = self._create_profile_builder(profile)
                self._profile_call(
                    "edge_sandbox_options_set_profile",
                    options_handle,
                    profile_handle,
                )

            if options.page is not None:
                url = options.page.url.encode("utf-8")
                html = options.page.html.encode("utf-8")
                referrer = options.page.referrer.encode("utf-8")
                content_type = options.page.content_type.encode("utf-8")
                self._profile_call(
                    "edge_sandbox_options_set_page",
                    options_handle,
                    url,
                    len(url),
                    html,
                    len(html),
                    referrer,
                    len(referrer),
                    content_type,
                    len(content_type),
                )

            self._profile_call(
                "edge_sandbox_options_clear_iframe_hooks",
                options_handle,
            )
            for hook in options.iframe_hooks:
                name = hook.name.encode("utf-8")
                source = hook.source.encode("utf-8")
                self._profile_call(
                    "edge_sandbox_options_append_iframe_hook",
                    options_handle,
                    name,
                    len(name),
                    source,
                    len(source),
                )

            self._profile_call(
                "edge_sandbox_options_clear_network_replay",
                options_handle,
            )
            for entry_index, entry in enumerate(options.network_replay):
                url = entry.url.encode("utf-8")
                method = entry.method.encode("utf-8")
                status_text = entry.status_text.encode("utf-8")
                body = entry.body_bytes()
                self._profile_call(
                    "edge_sandbox_options_append_network_replay",
                    options_handle,
                    url,
                    len(url),
                    method,
                    len(method),
                    entry.status,
                    status_text,
                    len(status_text),
                    body,
                    len(body),
                )
                for name, value in entry.headers:
                    encoded_name = name.encode("utf-8")
                    encoded_value = value.encode("utf-8")
                    self._profile_call(
                        "edge_sandbox_options_append_network_replay_header",
                        options_handle,
                        entry_index,
                        encoded_name,
                        len(encoded_name),
                        encoded_value,
                        len(encoded_value),
                    )

            deterministic = options.deterministic
            native_deterministic = _NativeDeterministicOptions(
                clock_epoch_ms=deterministic.clock_epoch_ms or 0,
                clock_step_ms=deterministic.clock_step_ms,
                random_seed=deterministic.random_seed or 0,
                max_task_turns=deterministic.max_task_turns,
                has_clock_epoch_ms=deterministic.clock_epoch_ms is not None,
                has_random_seed=deterministic.random_seed is not None,
            )
            self._profile_call(
                "edge_sandbox_options_set_deterministic",
                options_handle,
                ctypes.byref(native_deterministic),
            )

            limits = options.limits
            native_limits = _NativeSandboxLimits(
                timeout_ms=limits.timeout_ms or 0,
                max_heap_bytes=limits.max_heap_bytes or 0,
                max_resident_bytes=limits.max_resident_bytes or 0,
                max_source_bytes=limits.max_source_bytes or 0,
                max_output_bytes=limits.max_output_bytes or 0,
            )
            self._profile_call(
                "edge_sandbox_options_set_limits",
                options_handle,
                ctypes.byref(native_limits),
            )
            self._profile_call(
                "edge_sandbox_options_validate",
                options_handle,
            )
            return self._library.edge_sandbox_create_self_hosted_with_options(
                options_handle,
                ctypes.byref(error),
            )
        finally:
            if profile_handle is not None:
                self._library.edge_sandbox_profile_destroy(profile_handle)
            self._library.edge_sandbox_options_destroy(options_handle)

    def _require_handle(self) -> int:
        if self._handle is None:
            raise SandboxExecutionError("沙箱已经关闭")
        return self._handle

    def reinitialize_profile(self, profile: EdgeProfile) -> None:
        """Load a fresh profile/isolate without replacing the Worker PID."""

        profile_handle = self._create_profile_builder(profile)
        error = _NativeBuffer()
        try:
            succeeded = self._library.edge_sandbox_reinitialize_with_profile(
                self._require_handle(),
                profile_handle,
                ctypes.byref(error),
            )
        finally:
            self._library.edge_sandbox_profile_destroy(profile_handle)
        if not succeeded:
            raise SandboxExecutionError(self._consume_buffer(error))
        self._consume_buffer(error)

    def process_id(self) -> int:
        """Return the stable PID of this sandbox's isolated Worker."""

        process_id = ctypes.c_uint32()
        error = _NativeBuffer()
        succeeded = self._library.edge_sandbox_process_id(
            self._require_handle(),
            ctypes.byref(process_id),
            ctypes.byref(error),
        )
        if not succeeded:
            raise SandboxExecutionError(self._consume_buffer(error))
        self._consume_buffer(error)
        return int(process_id.value)

    def _consume_buffer(self, buffer: _NativeBuffer) -> str:
        return self._consume_binary_buffer(buffer).decode("utf-8")

    def _consume_binary_buffer(self, buffer: _NativeBuffer) -> bytes:
        try:
            if not buffer.data or buffer.len == 0:
                return b""
            return ctypes.string_at(buffer.data, buffer.len)
        finally:
            self._library.edge_sandbox_buffer_free(ctypes.byref(buffer))

    def evaluate(self, source: str, *, source_url: str | None = None) -> str:
        """Evaluate JavaScript and optionally name it for V8 stack traces."""

        source_bytes = source.encode("utf-8")
        result = _NativeBuffer()
        error = _NativeBuffer()
        if source_url is None:
            succeeded = self._library.edge_sandbox_evaluate(
                self._require_handle(),
                source_bytes,
                len(source_bytes),
                ctypes.byref(result),
                ctypes.byref(error),
            )
        else:
            source_url_bytes = source_url.encode("utf-8")
            succeeded = self._library.edge_sandbox_evaluate_with_source_url(
                self._require_handle(),
                source_bytes,
                len(source_bytes),
                source_url_bytes,
                len(source_url_bytes),
                ctypes.byref(result),
                ctypes.byref(error),
            )
        if not succeeded:
            self._consume_buffer(result)
            raise SandboxExecutionError(self._consume_buffer(error))

        self._consume_buffer(error)
        return self._consume_buffer(result)

    def _trace_control(self, native_name: str) -> None:
        error = _NativeBuffer()
        function = getattr(self._library, native_name)
        succeeded = function(self._require_handle(), ctypes.byref(error))
        if not succeeded:
            raise SandboxExecutionError(self._consume_buffer(error))
        self._consume_buffer(error)

    def enable_native_trace(self) -> None:
        self._trace_control("edge_sandbox_enable_native_trace")

    def disable_native_trace(self) -> None:
        self._trace_control("edge_sandbox_disable_native_trace")

    def clear_native_trace(self) -> None:
        self._trace_control("edge_sandbox_clear_native_trace")

    def set_native_trace_exclusions(self, exclusions: Iterable[str]) -> None:
        """Replace API-path exclusions applied before Trace entries are stored.

        Rules match exact API paths. A trailing ``*`` performs a prefix match.
        Passing an empty iterable clears the filter.
        """

        if isinstance(exclusions, (str, bytes)):
            raise TypeError("exclusions must be an iterable of strings")
        rules = tuple(exclusions)
        if any(not isinstance(rule, str) for rule in rules):
            raise TypeError("every native trace exclusion must be a string")

        buffers: list[ctypes.Array[ctypes.c_ubyte]] = []
        views = (_NativeStringView * len(rules))()
        for index, rule in enumerate(rules):
            encoded = rule.encode("utf-8")
            buffer = (ctypes.c_ubyte * len(encoded)).from_buffer_copy(encoded)
            buffers.append(buffer)
            views[index] = _NativeStringView(
                ctypes.cast(buffer, ctypes.POINTER(ctypes.c_ubyte)),
                len(encoded),
            )

        error = _NativeBuffer()
        succeeded = self._library.edge_sandbox_set_native_trace_exclusions(
            self._require_handle(),
            views if rules else None,
            len(rules),
            ctypes.byref(error),
        )
        if not succeeded:
            raise SandboxExecutionError(self._consume_buffer(error))
        self._consume_buffer(error)

    def native_trace(self) -> tuple[str, ...]:
        """Return all trace entries without expanding the whole trace in the Worker."""

        entries: list[str] = []
        for batch in self.native_trace_batches():
            entries.extend(batch)
        return tuple(entries)

    def native_trace_batches(
        self, batch_size: int = 8_192
    ) -> Iterator[tuple[str, ...]]:
        """Yield trace entries in bounded sequence ranges.

        Range filtering happens in the isolated Worker. This avoids the large
        temporary allocation caused by expanding and serializing every compact
        trace entry in one response.
        """

        if isinstance(batch_size, bool) or not isinstance(batch_size, int):
            raise TypeError("batch_size must be an integer")
        if batch_size <= 0:
            raise ValueError("batch_size must be positive")

        start = 1
        maximum_sequence = (1 << 64) - 1
        while start <= maximum_sequence:
            end = min(start + batch_size - 1, maximum_sequence)
            batch = self.native_trace_matching(f"@sequence:{start}..{end}")
            if not batch:
                break
            yield batch
            if len(batch) < batch_size or end == maximum_sequence:
                break
            start = end + 1

    def export_native_trace(
        self,
        path: str | os.PathLike[str],
        *,
        batch_size: int = 8_192,
        overwrite: bool = False,
    ) -> int:
        """Stream all trace entries to a UTF-8 file and return their count.

        Parent directories are created automatically. Existing files are kept
        unless ``overwrite=True`` is explicitly requested.
        """

        if isinstance(batch_size, bool) or not isinstance(batch_size, int):
            raise TypeError("batch_size must be an integer")
        if batch_size <= 0:
            raise ValueError("batch_size must be positive")

        destination = Path(path)
        destination.parent.mkdir(parents=True, exist_ok=True)
        mode = "w" if overwrite else "x"
        count = 0
        with destination.open(mode, encoding="utf-8", newline="\n") as output:
            for batch in self.native_trace_batches(batch_size=batch_size):
                output.writelines(f"{entry}\n" for entry in batch)
                count += len(batch)
        return count

    def native_trace_matching(self, needle: str) -> tuple[str, ...]:
        needle_bytes = needle.encode("utf-8")
        result = _NativeBuffer()
        error = _NativeBuffer()
        succeeded = self._library.edge_sandbox_native_trace_matching(
            self._require_handle(),
            needle_bytes,
            len(needle_bytes),
            ctypes.byref(result),
            ctypes.byref(error),
        )
        if not succeeded:
            self._consume_buffer(result)
            raise SandboxExecutionError(self._consume_buffer(error))

        self._consume_buffer(error)
        text = self._consume_buffer(result)
        return tuple(text.splitlines()) if text else ()

    def network_requests(self) -> tuple[CapturedNetworkRequest, ...]:
        """Return captured XHR/fetch requests without enabling API tracing."""

        result = _NativeBuffer()
        error = _NativeBuffer()
        succeeded = self._library.edge_sandbox_network_requests(
            self._require_handle(),
            ctypes.byref(result),
            ctypes.byref(error),
        )
        if not succeeded:
            self._consume_binary_buffer(result)
            raise SandboxExecutionError(self._consume_buffer(error))
        self._consume_buffer(error)
        return _decode_network_requests(self._consume_binary_buffer(result))

    def clear_network_requests(self) -> None:
        self._trace_control("edge_sandbox_clear_network_requests")

    def stdout(self) -> tuple[CapturedConsoleOutput, ...]:
        """Return typed console output without enabling native trace."""

        result = _NativeBuffer()
        error = _NativeBuffer()
        succeeded = self._library.edge_sandbox_stdout(
            self._require_handle(),
            ctypes.byref(result),
            ctypes.byref(error),
        )
        if not succeeded:
            self._consume_binary_buffer(result)
            raise SandboxExecutionError(self._consume_buffer(error))
        self._consume_buffer(error)
        return _decode_stdout(self._consume_binary_buffer(result))

    def clear_stdout(self) -> None:
        self._trace_control("edge_sandbox_clear_stdout")

    def close(self) -> None:
        if self._handle is not None:
            self._library.edge_sandbox_destroy(self._handle)
            self._handle = None

    def __enter__(self) -> Self:
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except (AttributeError, OSError):
            pass


def main() -> None:
    with EdgeSandbox() as sandbox:
        value = sandbox.evaluate(DEMO_JAVASCRIPT)
        print(f"JavaScript 返回值：{value}")


if __name__ == "__main__":
    main()
