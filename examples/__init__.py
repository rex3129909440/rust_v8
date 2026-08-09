"""Python API for the native Edge sandbox."""

from .edge_profile import (
    EdgeProfile,
    PerformanceEntryProfile,
    PerformanceProfile,
    ProfileField,
    WebAudioProfile,
)
from .edge_runtime_options import (
    DeterministicExecution,
    EdgeRunOptions,
    IframeHook,
    NetworkReplayEntry,
    PageInit,
    SandboxLimits,
)
from .edge_sandbox_pool import EdgeSandboxPool, PooledNetworkRequest, SandboxTask
from .mac_edge_profile import mac_edge_150_profile
from .windows_edge_profile import windows_edge_150_profile
from .run_sandbox import (
    CapturedConsoleOutput,
    CapturedConsoleValue,
    CapturedNetworkRequest,
    EdgeSandbox,
    SandboxExecutionError,
    find_native_artifacts,
)

__all__ = [
    "CapturedConsoleOutput",
    "CapturedConsoleValue",
    "CapturedNetworkRequest",
    "create_country_profile",
    "create_country_profile_details",
    "DeterministicExecution",
    "EdgeProfile",
    "EdgeRunOptions",
    "EdgeSandbox",
    "EdgeSandboxPool",
    "IframeHook",
    "NetworkReplayEntry",
    "PageInit",
    "PerformanceEntryProfile",
    "PerformanceProfile",
    "PooledNetworkRequest",
    "ProfileField",
    "SandboxExecutionError",
    "SandboxLimits",
    "SandboxTask",
    "WebAudioProfile",
    "find_native_artifacts",
    "mac_edge_150_profile",
    "windows_edge_150_profile",
]


def __getattr__(name: str):
    """Load country-profile composition lazily to avoid source-tree cycles."""

    if name in {"create_country_profile", "create_country_profile_details"}:
        from .country_profile import (
            create_country_profile,
            create_country_profile_details,
        )

        globals().update(
            create_country_profile=create_country_profile,
            create_country_profile_details=create_country_profile_details,
        )
        return globals()[name]
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
