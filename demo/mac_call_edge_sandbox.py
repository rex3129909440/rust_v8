"""Python 调用示例：接口与 call_edge_sandbox.py 一致，默认使用 Mac profile。"""

from __future__ import annotations

from dataclasses import dataclass, replace
from pathlib import Path

from examples.edge_profile import EdgeProfile
from examples.edge_runtime_options import EdgeRunOptions
from examples.mac_edge_profile import mac_edge_150_profile
from examples.run_complete_iframe_hook import DEMO_JAVASCRIPT, build_runtime_options
from examples.run_sandbox import (
    CapturedConsoleOutput,
    CapturedNetworkRequest,
    EdgeSandbox,
)


IPS_JAVASCRIPT_PATH = Path(__file__).with_name("ips.js")


def build_evaluation_runtime_options(
    *, timeout_ms: int = 10_000
) -> EdgeRunOptions:
    """返回完整页面/iframe Hook 配置，并保持 Edge 风格的推进时钟。"""

    options = build_runtime_options()
    return replace(
        options,
        deterministic=replace(options.deterministic, clock_epoch_ms=None),
        limits=replace(options.limits, timeout_ms=timeout_ms),
    )


@dataclass(frozen=True)
class SandboxCallResult:
    """沙箱返回值、网络请求及控制台输出。"""

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
    """使用独立沙箱执行 JavaScript；未传 profile 时安装 Apple M2 Mac profile。"""

    configured_profile = (
        profile if profile is not None else mac_edge_150_profile()
    )
    configured_options = (
        options if options is not None else build_evaluation_runtime_options()
    )

    with EdgeSandbox(
        library=library,
        worker=worker,
        profile=configured_profile,
        options=configured_options,
    ) as sandbox:
        value = sandbox.evaluate(javascript, source_url=source_url)
        requests = sandbox.network_requests()
        stdout = sandbox.stdout()
        tl_requests = tuple(
            request for request in requests if request.url.endswith("/tl")
        )
        sandbox.clear_network_requests()
        sandbox.clear_stdout()

    return SandboxCallResult(
        value=value,
        requests=requests,
        tl_requests=tl_requests,
        stdout=stdout,
    )


def call_built_in_demo() -> SandboxCallResult:
    """使用默认 Mac profile 运行内置 iframe/XHR/fetch 示例。"""

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
    """读取 demo/ips.js，并使用默认或用户传入的 Mac profile 执行。"""

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
    """把返回对象转换为便于业务代码读取的结构。"""

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


# 自定义 Mac 字体或屏幕等参数时：
#
# from examples.mac_edge_profile import mac_edge_150_profile
# from demo.mac_call_edge_sandbox import call_ips_file
#
# profile = mac_edge_150_profile(
#     font_families=("SF Pro Text", "Helvetica Neue", "Menlo"),
# )
# result = call_ips_file(profile=profile)
