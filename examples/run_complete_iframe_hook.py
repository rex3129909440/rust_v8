"""Complete, import-only Python example for a configured iframe XHR hook."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

try:
    from .edge_profile import (
        BatteryProfile,
        CanvasProfile,
        EdgeProfile,
        GeolocationProfile,
        LocaleProfile,
        MediaPreferencesProfile,
        MemoryProfile,
        NavigatorProfile,
        NetworkProfile,
        PermissionsProfile,
        ScreenProfile,
        StorageProfile,
        UserAgentBrandProfile,
        UserAgentDataProfile,
        WebAudioProfile,
        WebGlProfile,
        WebGpuProfile,
    )
    from .edge_runtime_options import (
        DeterministicExecution,
        EdgeRunOptions,
        IframeHook,
        NetworkReplayEntry,
        PageInit,
        SandboxLimits,
    )
    from .run_sandbox import CapturedConsoleOutput, CapturedNetworkRequest, EdgeSandbox
except ImportError:
    from edge_profile import (
        BatteryProfile,
        CanvasProfile,
        EdgeProfile,
        GeolocationProfile,
        LocaleProfile,
        MediaPreferencesProfile,
        MemoryProfile,
        NavigatorProfile,
        NetworkProfile,
        PermissionsProfile,
        ScreenProfile,
        StorageProfile,
        UserAgentBrandProfile,
        UserAgentDataProfile,
        WebAudioProfile,
        WebGlProfile,
        WebGpuProfile,
    )
    from edge_runtime_options import (
        DeterministicExecution,
        EdgeRunOptions,
        IframeHook,
        NetworkReplayEntry,
        PageInit,
        SandboxLimits,
    )
    from run_sandbox import CapturedConsoleOutput, CapturedNetworkRequest, EdgeSandbox


CHROME_150_USER_AGENT = (
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
    "AppleWebKit/537.36 (KHTML, like Gecko) "
    "Chrome/150.0.0.0 Safari/537.36"
)


IFRAME_XHR_HOOK = IframeHook(
    name="iframe-xhr-hook",
    source=r"""
const originalOpen = XMLHttpRequest.prototype.open;
const originalSend = XMLHttpRequest.prototype.send;
const requestMetadata = new WeakMap();

XMLHttpRequest.prototype.open = __edgev8.proxy(
  function open(method, url) {
    requestMetadata.set(this, {
      method: String(method),
      url: String(url)
    });
    return Reflect.apply(originalOpen, this, arguments);
  },
  "open"
);

XMLHttpRequest.prototype.send = __edgev8.proxy(
  function send() {
    const metadata = requestMetadata.get(this) ?? {};
    const originalBody = arguments[0];
    const outgoingBody = metadata.url?.endsWith("/tl")
      ? "hooked:" + String(originalBody ?? "")
      : originalBody;
    // srcdoc/about:blank iframe 与父页面同源，可把业务记录交给父页面。
    // 真正导出请求仍使用 sandbox.network_requests()，不需要 Trace。
    parent.__iframeHookCalls ??= [];
    parent.__iframeHookCalls.push({
      method: metadata.method,
      url: metadata.url,
      originalBody,
      outgoingBody
    });

    return Reflect.apply(originalSend, this, [outgoingBody]);
  },
  "send"
);
""",
)


IFRAME_TEXT_ENCODER_HOOK = IframeHook(
    name="iframe-text-encoder-hook",
    source=r"""
const originalTextEncoderEncode = TextEncoder.prototype.encode;

TextEncoder.prototype.encode = __edgev8.proxy(
  function encode() {
    const encoded = Reflect.apply(originalTextEncoderEncode, this, arguments);

    // stdout() 会保留 arguments 的逐项类型和值，并把 Uint8Array 原样导出为 bytes。
    console.log(
      "TextEncoder.prototype.encode",
      arguments,
      encoded,
      { input: arguments[0], byteLength: encoded.byteLength }
    );
    return encoded;
  },
  "encode"
);
""",
)

IFRAME_DEBUG_HOOK = IframeHook(
    name="iframe-debug-hook",
    source=r"""console.debug=function(){};
    console.log=function(){}
""",
)
IFRAME_JSON_HOOK = IframeHook(
    name="iframe-json-hook",
    source=r"""
    const json_parse = JSON.parse;

    JSON.parse = __edgev8.proxy(
      function parse() {
        const encoded = Reflect.apply(json_parse, this, arguments);

        // stdout() 会保留 arguments 的逐项类型和值，并把 Uint8Array 原样导出为 bytes。
        console.log(
          "JSON.parse",
          arguments,
          encoded,
        );
        return encoded;
      },
      "parse"
    );
    """,
)


DEMO_JAVASCRIPT = r"""
(async () => {
  const bootstrapText = await fetch("/api/bootstrap").then(
    response => response.text()
  );

  const frame = document.createElement("iframe");
  frame.id = "ips-frame";
  frame.srcdoc = `<script>
    new TextEncoder().encode("iframe-hook-中文");
    const request = new XMLHttpRequest();
    request.open("POST", "/tl");
    request.send("payload");
  <\/script>`;
  document.body.appendChild(frame);

  const audioContext = new AudioContext();
  const hookCall = __iframeHookCalls[0];
  return [
    navigator.userAgent,
    navigator.language,
    navigator.hardwareConcurrency,
    navigator.deviceMemory,
    screen.width + "x" + screen.height,
    devicePixelRatio,
    audioContext.sampleRate,
    Date.now(),
    bootstrapText,
    hookCall.method,
    hookCall.url,
    hookCall.originalBody,
    hookCall.outgoingBody,
    frame.contentWindow.Function.prototype.toString.call(
      frame.contentWindow.XMLHttpRequest.prototype.send
    ),
    "__edgev8" in frame.contentWindow,
    Reflect.ownKeys(frame.contentWindow).includes("__edgev8")
  ].join("|");
})()
"""


@dataclass(frozen=True)
class CompleteDemoResult:
    value: str
    requests: tuple[CapturedNetworkRequest, ...]
    tl_request: CapturedNetworkRequest | None
    stdout: tuple[CapturedConsoleOutput, ...]


def build_fingerprint() -> EdgeProfile:
    """Return one complete, internally consistent Chrome 150 fingerprint."""

    return EdgeProfile(
        id="complete-demo-chrome-150-win11",
        locale=LocaleProfile(
            locale="en-US",
            time_zone='Etc/GMT-8',
            time_zone_offset_minutes=-480,
        ),
        navigator=NavigatorProfile(
            user_agent=CHROME_150_USER_AGENT,
            app_version=CHROME_150_USER_AGENT.removeprefix("Mozilla/"),
            app_code_name="Mozilla",
            app_name="Netscape",
            platform="Win32",
            product="Gecko",
            product_sub="20030107",
            vendor="Google Inc.",
            vendor_sub="",
            language="en-US",
            languages=("en-US", "en"),
            hardware_concurrency=16,
            device_memory_gb=8.0,
            max_touch_points=0,
            cookie_enabled=True,
            on_line=True,
            webdriver=False,
            pdf_viewer_enabled=True,
            do_not_track=None,
            user_agent_data=UserAgentDataProfile(
                brands=(
                    UserAgentBrandProfile("Not_A Brand", "99", "99.0.0.0"),
                    UserAgentBrandProfile("Chromium", "150", "150.0.0.0"),
                    UserAgentBrandProfile("Google Chrome", "150", "150.0.0.0"),
                ),
                mobile=False,
                platform="Windows",
                architecture="x86",
                bitness="64",
                model="",
                platform_version="19.0.0",
                ua_full_version="150.0.0.0",
                wow64=False,
                form_factors=("Desktop",),
            ),
            network=NetworkProfile(
                effective_type="4g",
                rtt=50,
                downlink=10.0,
                save_data=False,
            ),
        ),
        screen=ScreenProfile(
            width=1920,
            height=1080,
            avail_width=1920,
            avail_height=1040,
            avail_left=0,
            avail_top=0,
            color_depth=24,
            pixel_depth=24,
            viewport_width=0,
            viewport_height=0,
            outer_width=0,
            outer_height=0,
            screen_x=0.0,
            screen_y=0.0,
            device_pixel_ratio=1.25,
            orientation_type="landscape-primary",
            orientation_angle=0,
            visual_viewport_scale=1.0,
        ),
        canvas=CanvasProfile(
            data_url_salt="complete-demo-canvas-150",
            text_width_scale=1.0,
        ),
        webgl=WebGlProfile(
            vendor="WebKit",
            renderer="WebKit WebGL",
            unmasked_vendor="Google Inc. (NVIDIA)",
            unmasked_renderer=(
                "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 "
                "vs_5_0 ps_5_0, D3D11)"
            ),
            max_texture_size=16384,
            max_cube_map_texture_size=16384,
            max_renderbuffer_size=16384,
            max_viewport_width=32767,
            max_viewport_height=32767,
            max_vertex_attribs=16,
            max_texture_image_units=16,
            max_combined_texture_image_units=32,
            context_alpha=True,
            context_antialias=True,
            context_depth=True,
            context_stencil=False,
            context_power_preference="default",
        ),
        webgpu=WebGpuProfile(
            vendor="nvidia",
            architecture="ampere",
            device="0x2503",
            description="NVIDIA GeForce RTX 3060",
            max_texture_dimension_2d=16384,
            max_bind_groups=4,
        ),
        audio=WebAudioProfile(
            sample_rate=48_000.0,
            max_channel_count=2,
            base_latency=0.01,
            output_latency=0.004,
            noise_seed=0x150150,
        ),
        storage=StorageProfile(
            quota_bytes=64 * 1024 * 1024 * 1024,
            usage_bytes=128 * 1024 * 1024,
            persisted=False,
        ),
        permissions=PermissionsProfile(
            camera="prompt",
            microphone="prompt",
            geolocation="prompt",
            notifications="prompt",
            clipboard_read="prompt",
            clipboard_write="granted",
        ),
        battery=BatteryProfile(
            charging=True,
            charging_time=0.0,
            discharging_time=float("inf"),
            level=1.0,
        ),
        geolocation=GeolocationProfile(
            latitude=31.2304,
            longitude=121.4737,
            altitude=4.0,
            accuracy=25.0,
            altitude_accuracy=10.0,
            heading=0.0,
            speed=0.0,
        ),
        media_preferences=MediaPreferencesProfile(
            color_scheme="light",
            contrast="no-preference",
            reduced_motion=False,
            reduced_data=False,
            forced_colors=False,
            inverted_colors=False,
            color_gamut="srgb",
            pointer="fine",
            any_pointer="fine",
            hover="hover",
            any_hover="hover",
            display_mode="browser",
            dynamic_range="standard",
            scripting="enabled",
        ),
        memory=MemoryProfile(
            performance_js_heap_size_limit=4_294_705_152,
            performance_total_js_heap_size=13061022,
            performance_used_js_heap_size=12562246,
            console_js_heap_size_limit=4_294_705_152,
            console_total_js_heap_size=13061022,
            console_used_js_heap_size=12562246,
        ),
    )


def build_runtime_options() -> EdgeRunOptions:
    """Return page, replay, hook, deterministic and process-limit settings."""

    return EdgeRunOptions(
        page=PageInit(
            url="https://www.wizzair.com/149e9513-01fa-4fb0-aad4-566afd725d1b/2d206a39-8ed7-437e-a3be-862e0f06eea3/fp?x-kpsdk-v=j-1.2.543",
            html=(
                    '<!DOCTYPE html><html><head></head><body><script>window.KPSDK={};KPSDK.now=typeof performance!==\'undefined\'&&performance.now?performance.now.bind(performance):Date.now.bind(Date);KPSDK.start=KPSDK.now();</script><script></script></body></html>'
            ),
            referrer="https://www.wizzair.com",
            content_type="text/html; charset=utf-8",
        ),
        network_replay=(
            NetworkReplayEntry(
                url="https://demo.example.test/api/bootstrap",
                method="GET",
                status=200,
                status_text="OK",
                headers=(("content-type", "text/plain; charset=utf-8"),),
                body=b"bootstrap-ok",
            ),
            NetworkReplayEntry(
                url="https://demo.example.test/tl",
                method="POST",
                status=204,
                status_text="No Content",
                headers=(("content-type", "text/plain"),),
                body=b"",
            ),
        ),
        # iframe_hooks=(IFRAME_XHR_HOOK, IFRAME_TEXT_ENCODER_HOOK, IFRAME_DEBUG_HOOK),
        iframe_hooks=(IFRAME_DEBUG_HOOK, IFRAME_JSON_HOOK),
        deterministic=DeterministicExecution(
            clock_epoch_ms=1_893_456_000_000,
            clock_step_ms=1,
            random_seed=150,
            max_task_turns=2_048,
        ),
        limits=SandboxLimits(
            timeout_ms=5_000,
            max_heap_bytes=256 * 1024 * 1024,
            max_resident_bytes=768 * 1024 * 1024,
            max_source_bytes=4 * 1024 * 1024,
            max_output_bytes=4 * 1024 * 1024,
        ),
    )


def create_complete_sandbox(
    *,
    library: Path | None = None,
    worker: Path | None = None,
    profile: EdgeProfile | None = None,
    options: EdgeRunOptions | None = None,
) -> EdgeSandbox:
    """Create one configured process-isolated sandbox; the caller closes it."""

    return EdgeSandbox(
        library=library,
        worker=worker,
        profile=profile or build_fingerprint(),
        options=options or build_runtime_options(),
    )


def run_complete_demo(
    javascript: str = DEMO_JAVASCRIPT,
    *,
    library: Path | None = None,
    worker: Path | None = None,
    profile: EdgeProfile | None = None,
    options: EdgeRunOptions | None = None,
) -> CompleteDemoResult:
    """Evaluate JavaScript, export requests, then synchronously close Worker."""

    with create_complete_sandbox(
        library=library,
        worker=worker,
        profile=profile,
        options=options,
    ) as sandbox:
        # Native Trace is intentionally not enabled.
        value = sandbox.evaluate(javascript)
        requests = sandbox.network_requests()
        stdout = sandbox.stdout()
        tl_request = next(
            (request for request in requests if request.url.endswith("/tl")),
            None,
        )
        sandbox.clear_network_requests()
        sandbox.clear_stdout()
        return CompleteDemoResult(
            value=value,
            requests=requests,
            tl_request=tl_request,
            stdout=stdout,
        )
