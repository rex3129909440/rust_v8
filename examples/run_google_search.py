
import random
import re
import string
import time
from urllib.parse import quote
import never_primp as primp
from curl_cffi import requests
from loguru import logger



import ctypes
import os
from dataclasses import dataclass
from pathlib import Path
from types import TracebackType
from typing import Self


PROJECT_ROOT = Path(__file__).resolve().parents[1]
ABI_VERSION = 1



class _NativeBuffer(ctypes.Structure):
    _fields_ = [
        ("data", ctypes.POINTER(ctypes.c_ubyte)),
        ("len", ctypes.c_size_t),
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
            headers.append(
                (
                    bytes(take(name_len)).decode("utf-8"),
                    bytes(take(value_len)).decode("utf-8"),
                )
            )
        body = bytes(take(body_len))
        requests.append(
            CapturedNetworkRequest(
                sequence=sequence,
                source={1: "XMLHttpRequest", 2: "fetch"}.get(
                    source_id, f"unknown:{source_id}"
                ),
                method=method,
                url=url,
                headers=tuple(headers),
                body=body,
            )
        )
    if offset != len(view):
        raise SandboxExecutionError("trailing bytes in ESNR network request buffer")
    return tuple(requests)


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
    """Locate the single self-hosting native library."""

    del worker
    if library is not None:
        resolved_library = library.resolve()
        if not resolved_library.is_file():
            raise FileNotFoundError(f"沙箱动态库不存在：{resolved_library}")
        return resolved_library, None

    library_name = _artifact_name()
    for profile in ("release", "debug"):
        profile_dir = PROJECT_ROOT / "target" / profile
        candidate_library = profile_dir / library_name
        if candidate_library.is_file():
            return candidate_library, None

    raise FileNotFoundError("未找到 edge-sandbox 动态库；请先构建项目")


class EdgeSandbox:
    """Native Python binding for an OS-process-isolated edge-sandbox runtime."""

    def __init__(
        self,
        *,
        library: Path | None = None,
        worker: Path | None = None,
    ) -> None:
        library_path, _ = find_native_artifacts(library, worker)
        self._library = ctypes.CDLL(str(library_path))
        self._configure_native_api()
        self._handle: int | None = None

        actual_abi = self._library.edge_sandbox_abi_version()
        if actual_abi != ABI_VERSION:
            raise SandboxExecutionError(
                f"原生 ABI 版本不匹配：Python={ABI_VERSION}, 动态库={actual_abi}"
            )

        error = _NativeBuffer()
        handle = self._library.edge_sandbox_create_self_hosted(ctypes.byref(error))
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

        for name in (
            "edge_sandbox_enable_native_trace",
            "edge_sandbox_disable_native_trace",
            "edge_sandbox_clear_native_trace",
        ):
            function = getattr(library, name)
            function.argtypes = [ctypes.c_void_p, buffer_pointer]
            function.restype = ctypes.c_bool

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

        library.edge_sandbox_buffer_free.argtypes = [buffer_pointer]
        library.edge_sandbox_buffer_free.restype = None

    def _require_handle(self) -> int:
        if self._handle is None:
            raise SandboxExecutionError("沙箱已经关闭")
        return self._handle

    def _consume_buffer(self, buffer: _NativeBuffer) -> str:
        return self._consume_binary_buffer(buffer).decode("utf-8")

    def _consume_binary_buffer(self, buffer: _NativeBuffer) -> bytes:
        try:
            if not buffer.data or buffer.len == 0:
                return b""
            return ctypes.string_at(buffer.data, buffer.len)
        finally:
            self._library.edge_sandbox_buffer_free(ctypes.byref(buffer))

    def evaluate(self, source: str) -> str:
        """Evaluate JavaScript and return its display value."""

        source_bytes = source.encode("utf-8")
        result = _NativeBuffer()
        error = _NativeBuffer()
        succeeded = self._library.edge_sandbox_evaluate(
            self._require_handle(),
            source_bytes,
            len(source_bytes),
            ctypes.byref(result),
            ctypes.byref(error),
        )
        if not succeeded:
            self._consume_buffer(result)
            raise SandboxExecutionError(self._consume_buffer(error))

        self._consume_buffer(error)
        return self._consume_buffer(result)

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
        error = _NativeBuffer()
        succeeded = self._library.edge_sandbox_clear_network_requests(
            self._require_handle(), ctypes.byref(error)
        )
        if not succeeded:
            raise SandboxExecutionError(self._consume_buffer(error))
        self._consume_buffer(error)

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

    def native_trace(self) -> tuple[str, ...]:
        result = _NativeBuffer()
        error = _NativeBuffer()
        succeeded = self._library.edge_sandbox_native_trace(
            self._require_handle(),
            ctypes.byref(result),
            ctypes.byref(error),
        )
        if not succeeded:
            self._consume_buffer(result)
            raise SandboxExecutionError(self._consume_buffer(error))

        self._consume_buffer(error)
        text = self._consume_buffer(result)
        return tuple(text.splitlines()) if text else ()

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

def _extract_p_and_js(text: str) -> tuple[str, str]:
    p_match = re.findall(r"p='(.*?)';var g='knitsail'", text)
    if not p_match:
        raise ValueError("p value was not found in Google response")
    js_match = re.findall(r"==\n(.*?)</script>", text, re.S)
    if not js_match:
        raise ValueError("knitsail JavaScript was not found in Google response")
    sei_match = re.findall(r"var eid='(.*?)';var ss_cgi", text, re.S)

    if not sei_match:
        raise ValueError("knitsail JavaScript was not found in Google response")
    return p_match[0], js_match[0]


PROXY = (
    f"http://client-flyscoot_area-us_session-{''.join(random.choices(string.digits+string.ascii_uppercase,k=8))}-10:"
    "Hkkejo8Aafa@proxy.iproyal.net:9000"
)
session = primp.Client(impersonate="chrome_148", timeout=30)

headers = {
    'accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7',
    'accept-language': 'en-US,en;q=0.9',
    'available-dictionary': ':TqhDfEMVU1QbUn0JXuNaKOa9wILaJpWlLhyyeJhPhro=:',
    'downlink': '1.55',
    'priority': 'u=0, i',
    'referer': 'https://www.google.com.hk/',
    'rtt': '450',
    'sec-ch-prefers-color-scheme': 'light',
    'sec-ch-ua': '"Not;A=Brand";v="8", "Chromium";v="150", "Google Chrome";v="150"',
    'sec-ch-ua-arch': '"x86"',
    'sec-ch-ua-bitness': '"64"',
    'sec-ch-ua-form-factors': '"Desktop"',
    'sec-ch-ua-full-version': '"150.0.7871.115"',
    'sec-ch-ua-full-version-list': '"Not;A=Brand";v="8.0.0.0", "Chromium";v="150.0.7871.115", "Google Chrome";v="150.0.7871.115"',
    'sec-ch-ua-mobile': '?0',
    'sec-ch-ua-model': '""',
    'sec-ch-ua-platform': '"Windows"',
    'sec-ch-ua-platform-version': '"15.0.0"',
    'sec-ch-ua-wow64': '?0',
    'sec-fetch-dest': 'document',
    'sec-fetch-mode': 'navigate',
    'sec-fetch-site': 'same-origin',
    'sec-fetch-user': '?1',
    'upgrade-insecure-requests': '1',
    'user-agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36',
    'x-browser-channel': 'stable',
    'x-browser-copyright': 'Copyright 2026 Google LLC. All Rights Reserved.',
    'x-browser-validation': '9VleGbLep3vn3Yse2zzlhFbTCd4=',
    'x-browser-year': '2026',
}

params = {
    'q': "openai site:openai.com",
}

response = session.get('https://www.google.com.hk/search', params=params,headers=headers)

# print(response.text)
p, js = _extract_p_and_js(response.text)

DEMO_JAVASCRIPT = f'''
  ;(() => {{

      location.href = "{response.url}";

      {js}

      const p = "{p}";

      const l = function (a) {{
          return a[Symbol.iterator].call(a);
      }};

      const a = l(
          window["knitsail"].a(p, function () {{}}, false)
      ).next().value;

      const sgss = a([{{ q: "{params['q']}" }}]);
      return sgss;
  }})()
  '''
with EdgeSandbox() as sandbox:
    value = sandbox.evaluate(DEMO_JAVASCRIPT)
    print(f"JavaScript 返回值：{value}")
    session.cookies.set("SG_SS", value)
    response = session.get('https://www.google.com.hk/search', params=params,headers=headers)
    print([response.status_code, len(response.text)])
    print(response.text)
