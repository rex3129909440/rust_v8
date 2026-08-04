"""完整的 Python 调用示例；本文件不提供命令行入口。"""

from __future__ import annotations

from dataclasses import dataclass, replace
from pathlib import Path

from examples.edge_profile import EdgeProfile
from examples.edge_runtime_options import EdgeRunOptions
from examples.run_complete_iframe_hook import (
    DEMO_JAVASCRIPT,
    build_fingerprint,
    build_runtime_options,
)
from examples.run_sandbox import (
    CapturedConsoleOutput,
    CapturedNetworkRequest,
    EdgeSandbox,
)


IPS_JAVASCRIPT_PATH = Path(__file__).with_name("ips.js")


def build_evaluation_runtime_options(
    *, timeout_ms: int = 10_000
) -> EdgeRunOptions:
    """Return the full demo setup with an Edge-style advancing wall clock.

    A fixed ``clock_epoch_ms`` is useful for deterministic unit tests, but it
    intentionally stops wall-clock progress.  Browser scripts that wait until
    ``Date.now()`` advances can therefore run until the process timeout.  Keep
    the rest of the complete page, replay, iframe-hook and limit configuration
    while allowing Date/performance clocks to follow normal Edge semantics.
    """

    options = build_runtime_options()
    return replace(
        options,
        deterministic=replace(options.deterministic, clock_epoch_ms=None),
        limits=replace(options.limits, timeout_ms=timeout_ms),
    )


@dataclass(frozen=True)
class SandboxCallResult:
    """沙箱返回值和不开启 Trace 时导出的结构化请求。"""

    value: str
    requests: tuple[CapturedNetworkRequest, ...]
    tl_requests: tuple[CapturedNetworkRequest, ...]
    stdout: tuple[CapturedConsoleOutput, ...]


def call_javascript(
    javascript: str,
    *,
    source_url: str | None = None,
    profile: EdgeProfile | None = None,
    options: EdgeRunOptions | None = None,
    library: Path | None = None,
    worker: Path | None = None,
) -> SandboxCallResult:
    """创建独立 Worker，执行一次 JavaScript，导出请求并释放 Worker。"""

    configured_profile = profile if profile is not None else build_fingerprint()
    configured_options = (
        options if options is not None else build_evaluation_runtime_options()
    )

    with EdgeSandbox(
        library=library,
        worker=worker,
        profile=configured_profile,
        options=configured_options,
    ) as sandbox:
        sandbox.set_native_trace_exclusions(
            [
                "window.String",
                "window.Number",
                "window.Math",
                "window.Object",
                "window.Array",
                "window.isNaN",
                "random",
            ]
        )
        sandbox.enable_native_trace()
        try:
            value = sandbox.evaluate(javascript, source_url=source_url)
        finally:
            sandbox.disable_native_trace()
        requests = sandbox.network_requests()
        stdout = sandbox.stdout()
        tl_requests = tuple(
            request for request in requests if request.url.endswith("/tl")
        )
        count = sandbox.export_native_trace(
            r"D:\sandbox\edge_sandbox-main\demo\native-trace.log",
            batch_size=8192,
            overwrite=True,
        )

        print("导出数量:", count)
        # requests 已复制成 Python bytes，清理沙箱中的原生请求记录不影响返回值。
        sandbox.clear_network_requests()
        sandbox.clear_stdout()


    # 离开 with 后，EdgeSandbox 句柄和对应 OS Worker 已关闭。
    return SandboxCallResult(
        value=value,
        requests=requests,
        tl_requests=tl_requests,
        stdout=stdout,
    )


def call_built_in_demo() -> SandboxCallResult:
    """运行带 iframe XHR、fetch 和完整指纹读取的内置示例。"""

    return call_javascript(DEMO_JAVASCRIPT)


def call_ips_file(
    ips_path: Path = IPS_JAVASCRIPT_PATH,
    *,
    profile: EdgeProfile | None = None,
    options: EdgeRunOptions | None = None,
    library: Path | None = None,
    worker: Path | None = None,
    source_url: str | None = None,
) -> SandboxCallResult:
    """读取 demo/ips.js，并使用相同的 iframe Hook 初始化配置执行。"""

    javascript = ips_path.read_text(encoding="utf-8")
    return call_javascript(
        javascript,
        source_url=source_url or ips_path.resolve().as_uri(),
        profile=profile,
        options=options,
        library=library,
        worker=worker,
    )


def consume_result(result: SandboxCallResult) -> dict[str, object]:
    """演示调用方如何读取返回值以及 /tl 的 header/body。"""

    return {
        "javascript_value": result.value,
        "request_count": len(result.requests),
        "stdout": tuple(
            {
                "level": message.level,
                "frame_url": message.frame_url,
                "text": message.text,
                "arguments": message.arguments,
            }
            for message in result.stdout
        ),
        "tl": tuple(
            {
                "source": request.source,
                "method": request.method,
                "url": request.url,
                "headers": request.headers,
                "body": request.body,
            }
            for request in result.tl_requests
        ),
    }


# 在其他 Python 文件中这样调用：
#
# from demo.call_edge_sandbox import call_built_in_demo, call_ips_file
#
# built_in = call_built_in_demo()
# print(built_in.value)
# print(built_in.tl_requests[0].body)
#
# ips_result = call_ips_file()
# for request in ips_result.tl_requests:
#     print(request.method, request.url, request.headers, request.body)
