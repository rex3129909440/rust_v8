# edge_sandbox

完整中文使用手册：[`docs/SANDBOX_USAGE_ZH.md`](docs/SANDBOX_USAGE_ZH.md)

`edge_sandbox` is a browser-free Microsoft Edge HTTPS Window runtime. It links
V8 directly, then installs Edge-specific DOM, BOM, WebGPU, WebXR, media,
storage, cryptography, CSS and related APIs with handwritten Rust callbacks and
state stores.

The runtime identity is a secure page at `https://sandbox.test/`:

- `isSecureContext === true`
- `location.href === "https://sandbox.test/"`
- `origin === "https://sandbox.test"`
- 1232 own Window properties in Edge order

Microsoft Edge 150, explicitly launched through Playwright with the `msedge`
channel against an HTTPS-routed page, is the compatibility authority. V8
intrinsics are replaced or amended whenever their observable surface conflicts
with Edge; `console`, WebAssembly streaming members and Location exotic
properties are examples.

## Compatibility fingerprints

The regression test covers the complete observable Window surface, including
property order and descriptors, native function names/lengths/stringification,
prototype inheritance, symbols, object tags and primitive values:

- Window names: `1232`, hash `60594b80`
- Window descriptors: hash `946e759f`
- prototype-bearing constructors: `961`, hash `3918685b`
- callable globals: `1024`, hash `a8b3e550`
- object globals: `53`, hash `59942bf5`
- primitive globals: `155`, hash `907ccdc7`

Navigator and console have dedicated fingerprints. Stateful behavior checks
cover secure WebXR layers and sessions, Storage, Crypto/SubtleCrypto,
AudioContext/AudioSinkInfo, Edge document token methods, DOM token-list
setters, pointer-raw-update event handlers, CSS paint worklets, deterministic
CSS box geometry, ResizeObserver/IntersectionObserver delivery and WebAssembly
streaming compilation/instantiation. Geometry checks also cover Range
DOMRect/DOMRectList results, positioned overflow scrolling and rendered image
coordinates/sizes, document/shadow hit testing and scroll-into-view alignment.
Streaming tests also cover typed-array Response bodies, MIME enforcement and
already-consumed body rejection. The targeted Edge HTTPS evidence set contains
246 behavior rows.

The relationship audit compares 4362 Edge/local observations with zero
differences. It covers constructor and prototype chains, constructor backlinks,
function/object aliases, `instanceof`, symbols and hidden
`WindowProperties`. A separate 232-case DOM factory audit covers HTML, SVG and
MathML element names with zero differences; non-element node factories and the
legacy `Document.createEvent()` mapping are regression-tested as well.

## Implementation constraints

The browser API layer is split into 1182 explicitly declared Rust modules under
`src/web`; registration is explicit and does not loop over API names. The
browser API implementation does not embed serialized API snapshots, deserialize
API definitions, or generate JavaScript from captured definitions. The finished
executable does not launch a browser or use CDP. Serialization is used only by
the host/worker binary control protocol and never to define a Web API.

## Process isolation

The native library executes V8 in a dedicated worker process. The controller and worker
exchange bounded, length-prefixed binary messages rather than JSON. The
controller enforces a wall-clock deadline and resident-memory ceiling, kills an
unresponsive or oversized worker, and starts a clean replacement on the next
request. V8 heap, source and output limits remain enforced inside the worker.

On Windows the worker is assigned to a Job Object limited to one process, with
kill-on-close and a process-memory ceiling. On Linux the worker receives a
parent-death signal policy and a seccomp filter that denies network socket
operations. Worker stdin, stdout and stderr are private pipes and its inherited
environment is reduced to the entries needed by the runtime.

FFI embedders use the `edge_sandbox_create_self_hosted*` entry points. On
Windows the operating-system DLL loader enters an export in the same deployed
DLL; on Linux the already-loaded SO forks directly. Neither path requires a
project-provided worker executable. The legacy Rust executable constructors
remain available only for custom Rust hosts.

## Typed Python profile

Install the published package with:

```powershell
python -m pip install rexisohe-sandbox
```

The PyPI distribution name is `rexisohe-sandbox`; the Python import package is
`edge_sandbox`:

The Python binding accepts nested dataclasses and writes each field through the
native typed profile builder. Profile configuration is never serialized as
JSON. Omitted values keep the fixed Chrome 150 defaults, and the complete
profile is validated before the isolated worker starts.

The typed profile covers Navigator and Client Hints, locale/time zone,
screen/orientation/VisualViewport, Canvas and every `TextMetrics` number,
WebGL/WebGPU limits and capabilities, WebAudio, storage and heap-memory values,
speech voices, installed/local fonts, media devices and supported media types,
WebRTC RTP capabilities and SDP, permissions, battery, geolocation, CSS media
preferences, plugins/MIME types, gamepads, USB/HID/Serial/Bluetooth/MIDI
devices, keyboard layout, device posture, sensors, XR session modes and the
deterministic clock/random seed. Structured inventories use dedicated native
append functions; they are not JSON strings or generic API shells.

```python
from edge_sandbox.edge_profile import (
    EdgeProfile,
    LocaleProfile,
    NavigatorProfile,
    ScreenProfile,
    WebAudioProfile,
    WindowProfile,
)
from edge_sandbox import EdgeSandbox

profile = EdgeProfile(
    locale=LocaleProfile(
        locale="fr-FR",
        time_zone="Europe/Paris",
        time_zone_offset_minutes=-120,
    ),
    navigator=NavigatorProfile(
        language="fr-FR",
        languages=("fr-FR", "fr"),
        hardware_concurrency=16,
        webdriver=False,
    ),
    screen=ScreenProfile(
        width=1920,
        height=1080,
        avail_width=1900,
        avail_height=1040,
        device_pixel_ratio=1.25,
    ),
    window=WindowProfile(
        inner_width=1536,
        inner_height=864,
        outer_width=1920,
        outer_height=1040,
    ),
    audio=WebAudioProfile(
        sample_rate=96_000,
        max_channel_count=6,
        base_latency=0.004,
        output_latency=0.017,
    ),
)

with EdgeSandbox(profile=profile) as sandbox:
    value = sandbox.evaluate(
        "[navigator.language, screen.width, "
        "new AudioContext().sampleRate].join('|')"
    )
```

The same validated snapshot is shared by Window, same-origin iframe, Worker and
Worklet realms. Native tracing remains host-controlled and does not change
profile-visible descriptors or function stringification.

## Typed Python runtime options

Page HTML, offline network responses, deterministic execution and process
limits and iframe preload hooks also use dedicated typed C ABI fields. They are
transferred to the isolated worker with the bounded binary protocol; no option
is encoded as a JSON string.

```python
from edge_sandbox.edge_runtime_options import (
    DeterministicExecution,
    EdgeRunOptions,
    IframeHook,
    NetworkReplayEntry,
    PageInit,
    SandboxLimits,
)
from edge_sandbox import EdgeSandbox

options = EdgeRunOptions(
    page=PageInit(
        url="https://example.test/app/index.html",
        html='<main id="app"><a id="next" href="../next">Next</a></main>',
    ),
    network_replay=(
        NetworkReplayEntry(
            url="https://api.example.test/data",
            body="typed response",
            headers=(("content-type", "text/plain"),),
        ),
    ),
    deterministic=DeterministicExecution(
        clock_epoch_ms=1_893_456_000_000,
        random_seed=150,
    ),
    limits=SandboxLimits(timeout_ms=3_000),
)

with EdgeSandbox(options=options) as sandbox:
    value = sandbox.evaluate(
        'fetch("https://api.example.test/data").then(async response => '
        '[await response.text(), document.getElementById("next").href, '
        'location.href].join("|"))'
    )
```

### Iframe preload hooks

`IframeHook` source runs inside every newly created or navigated iframe realm
after the browser APIs are installed and before any script from the iframe
document executes. It is independent from native trace and may directly
replace realm-local functions.

The host passes a private V8-native object to each hook as the local binding
`__edgev8`. It is never installed on Window, so all of these remain true after
the hook and while the iframe page runs:

```javascript
"__edgev8" in window === false
typeof window.__edgev8 === "undefined"
Reflect.ownKeys(window).includes("__edgev8") === false
```

`__edgev8.proxy(function, nativeName)` returns the same function and registers
native source text for it. The method name does not mean JavaScript `Proxy`;
the sandbox creates no Proxy object. `protectPrototypeFunction(prototype,
propertyName)` protects a function after it has already been assigned.

```python
from edge_sandbox import EdgeRunOptions, EdgeSandbox, IframeHook

iframe_xhr_hook = IframeHook(
    name="xhr-hook",
    source=r'''
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
    const body = arguments[0];
    const metadata = requestMetadata.get(this) ?? {};
    parent.__iframeHookRequests ??= [];
    parent.__iframeHookRequests.push({
      method: metadata.method,
      url: metadata.url,
      body
    });
    return Reflect.apply(originalSend, this, arguments);
  },
  "send"
);
''',
)

options = EdgeRunOptions(iframe_hooks=(iframe_xhr_hook,))

with EdgeSandbox(options=options) as sandbox:
    sandbox.evaluate(ips_javascript)
    requests = sandbox.network_requests()  # Trace is not required.
```

The protected wrappers retain their actual identity and behavior while:

```javascript
Function.prototype.toString.call(XMLHttpRequest.prototype.open)
// function open() { [native code] }
```

Because the hook deliberately replaces a function, its identity is expected to
differ from the original function. The protection API only controls native
source stringification; it does not silently rewrite the wrapper's `name`,
`length`, prototype or property descriptor.

`evaluate()` waits for a top-level Promise. A fulfilled Promise returns its
fulfillment value; a rejection raises `SandboxExecutionError`. A Promise that
remains pending after the configured task-turn bound is rejected by the host
instead of being returned as `[object Promise]`.

A reusable no-command-line wrapper is available in
`examples/run_typed_page.py`.

## Concurrent Python worker pool

`EdgeSandboxPool` keeps one Python facade over multiple independent operating-
system processes and V8 isolates. `submit()` schedules unrelated scripts in
parallel. Each task can supply a different typed fingerprint; a native worker
is reused only when its fingerprint and runtime options exactly match the next
task.

```python
from edge_sandbox.edge_profile import EdgeProfile, NavigatorProfile
from edge_sandbox import EdgeSandboxPool

profiles = (
    EdgeProfile(navigator=NavigatorProfile(user_agent="Pool-UA-A")),
    EdgeProfile(navigator=NavigatorProfile(user_agent="Pool-UA-B")),
)

with EdgeSandboxPool(
    workers=2,
    timeout_ms=2_000,
    close_worker_after_network_requests=True,
) as sandbox:
    tasks = tuple(
        sandbox.submit(source, profile=profile)
        for source, profile in zip((javascript_a, javascript_b), profiles)
    )
    values = tuple(task.result() for task in tasks)
    requests = sandbox.network_requests()
```

Captured requests include `task_id` and `worker_id` in addition to method,
URL, ordered headers and exact body bytes. With
`close_worker_after_network_requests=True`, reading the completed task's
requests closes its idle worker (or marks a currently busy reused worker for
closure as soon as that task finishes). A timed-out or failed evaluation always
discards its worker immediately, releasing the V8 heap and DOM state; the pool
creates a clean replacement for later work.

## Build and verify

```powershell
cargo fmt --all
cargo check --lib --message-format short
cargo test --lib --message-format short
cargo build --release --lib --message-format short
```

The production output is one native library: `edge_sandbox.dll` on Windows or
`libedge_sandbox.so` on Linux. No project worker executable is built or
deployed. Windows launches the DLL worker entry with the system DLL loader;
Linux forks directly into the loaded shared object.

## Run from Python

```python
from edge_sandbox import EdgeSandbox

with EdgeSandbox() as sandbox:
    print(sandbox.evaluate("Object.getOwnPropertyNames(window).length"))
```

Tracing is disabled by default and controlled by the Rust host, so it does not
add properties to Window or replace any JavaScript-visible value. The trace
path uses V8 host callback trampolines and ObjectTemplate interceptors that
return `kNo`, leaving receiver selection, property lookup, return slots and
exceptions to V8. It records Window/property reads, writes, WebIDL getters and
setters, function calls, and construction, including values returned by another
API. Object labels use V8 identity-hash indexes rather than scanning previously
seen APIs.

Tracing preserves Edge-observable function strings, descriptors, prototype and
constructor relationships, `instanceof`, object tags, aliases, own-key
enumeration and exotic object brands for Array, TypedArray, Map, Set, Date,
RegExp and Promise values. Objects and functions supplied by user code retain
their identity when they round-trip through API calls, assignments and property
descriptors. Each line contains a sequence number, operation, API path,
receiver, arguments and result without JSON serialization:

```text
TRACE	7	call	window.document.createElement	receiver=[object HTMLDocument]	args="div"	result=[object HTMLDivElement]
```

Embedders can call `enable_native_trace()`, `disable_native_trace()`,
`clear_native_trace()` and `native_trace()` on `EdgeRuntime`; the former
`*_proxy_trace` names remain compatibility aliases. Entries are exposed as
`TraceEntry` values.

The Python binding locates only the native library. Its deprecated `worker=`
keyword is ignored and can be removed from existing callers.

## Python platform wheels

`.github/workflows/python-wheels.yml` builds installable Python wheels on
native Windows x64, macOS arm64, macOS x64, and a manylinux 2.28 x64 build
container. Every job installs its freshly built wheel and starts a real
isolated worker for a JavaScript smoke test before uploading the artifact.

The binding uses `ctypes` over the stable C ABI, so the wheels use a
`py3-none-<platform>` tag rather than being tied to one CPython minor version.
The shared library is stored inside `edge_sandbox/_native` and is discovered
automatically after `pip install`; source-tree execution continues to search
`target/release` and `target/debug`.

When running an uninstalled checkout directly, the equivalent development-only
import is `from examples.run_sandbox import EdgeSandbox`.

See `docs/GITHUB_ACTIONS_WHEELS_ZH.md` for the build matrix, manual/tag
triggers, installation example, Linux compatibility policy, and binary-size
notes.
