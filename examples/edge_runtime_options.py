"""Typed runtime inputs for the native Edge sandbox.

These dataclasses are transferred through dedicated C ABI fields and the
sandbox's bounded binary IPC protocol. They are never serialized as JSON.
"""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(frozen=True)
class PageInit:
    """A fetched HTTPS page to materialize before JavaScript evaluation."""

    url: str = "https://sandbox.test/"
    html: str = ""
    referrer: str = ""
    content_type: str = "text/html"


@dataclass(frozen=True)
class NetworkReplayEntry:
    """One offline response visible to fetch, XHR, frames and module loaders."""

    url: str
    body: bytes | str = b""
    method: str = "GET"
    status: int = 200
    status_text: str = "OK"
    headers: tuple[tuple[str, str], ...] = ()

    def body_bytes(self) -> bytes:
        return self.body.encode("utf-8") if isinstance(self.body, str) else bytes(self.body)


@dataclass(frozen=True)
class IframeHook:
    """One preload script executed in every iframe before its page scripts."""

    name: str
    source: str


@dataclass(frozen=True)
class DeterministicExecution:
    """Clock, random and event-loop bounds for one isolated runtime."""

    clock_epoch_ms: int | None = None
    clock_step_ms: int = 1
    random_seed: int | None = None
    max_task_turns: int = 1024


@dataclass(frozen=True)
class SandboxLimits:
    """OS-process and V8 limits; omitted values use isolated defaults."""

    timeout_ms: int | None = None
    max_heap_bytes: int | None = None
    max_resident_bytes: int | None = None
    max_source_bytes: int | None = None
    max_output_bytes: int | None = None


@dataclass(frozen=True)
class EdgeRunOptions:
    """Non-fingerprint inputs used to construct the isolated runtime."""

    page: PageInit | None = None
    network_replay: tuple[NetworkReplayEntry, ...] = ()
    iframe_hooks: tuple[IframeHook, ...] = ()
    deterministic: DeterministicExecution = field(
        default_factory=DeterministicExecution
    )
    limits: SandboxLimits = field(default_factory=SandboxLimits)
