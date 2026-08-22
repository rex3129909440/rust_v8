# Edge Sandbox 当前完整使用手册

本文以当前工作区源码和原生库为准，覆盖Python导入、单实例、HTML页面、
固定/随机profile、preload、iframe hook、stdout、请求捕获、网络回放、Trace、
进程隔离、并发Worker池、构建和常见错误。历史修复过程仍保存在其它审计文档中；
实际调用优先参考本文。

## 1. 当前架构

Edge Sandbox通过稳定C ABI把Rust/V8浏览器沙箱暴露给Python。

- Windows部署 `edge_sandbox.dll`；
- Linux部署 `libedge_sandbox.so`；
- macOS部署 `libedge_sandbox.dylib`；
- 动态库自行创建隔离子进程，不依赖单独Worker EXE；
- `worker=` 仅兼容旧代码，当前会被忽略；
- 每个 `EdgeSandbox` 句柄对应一个隔离运行时；
- `EdgeSandboxPool` 在一个Python进程内管理多个独立Worker。

当前schema：

```text
profile schema = 15
runtime-options schema = 4
```

Python绑定与动态库的schema必须一致。

## 2. 关键文件

```text
examples/run_sandbox.py              Python单实例绑定
examples/edge_profile.py             所有typed profile定义
examples/edge_runtime_options.py     页面、回放、hook、限制
examples/edge_sandbox_pool.py        Python并发Worker池
demo/android_call_edge_sandbox.py    Android/WebView入口和硬编码profile
demo/w6_sandbox_executor_api.py      纯沙箱执行与/tl导出FastAPI
demo/success_profile_sandbox_test.py 固定profile离线入口
target/release/edge_sandbox.dll      Windows release库
```

源码目录中不传 `library` 时，会从 `target/release` 和 `target/debug` 选择修改时间
较新的动态库。稳定回归和生产建议显式传release路径，避免误加载debug。

## 3. 安装与导入

### 3.1 源码目录

```python
from pathlib import Path
from examples import EdgeSandbox

library = Path(r"D:\sandbox\edge_sandbox-main\target\release\edge_sandbox.dll")

with EdgeSandbox(library=library) as sandbox:
    print(sandbox.evaluate("1 + 2"))  # 3
```

### 3.2 wheel/PyPI安装后

```python
from edge_sandbox import EdgeSandbox

with EdgeSandbox() as sandbox:
    print(sandbox.evaluate("1 + 2"))
```

wheel内包含当前平台动态库，通常无需手动传 `library`。

## 4. 单实例与返回值

```python
from examples import EdgeSandbox

source = """
(() => {
  const element = document.createElement("div");
  element.id = "result";
  element.textContent = "hello";
  document.body.appendChild(element);
  return document.getElementById("result").textContent;
})()
"""

with EdgeSandbox() as sandbox:
    print(sandbox.evaluate(source))  # hello
```

返回值是JavaScript最终值的文本形式：

| JavaScript | Python |
|---|---|
| `undefined` | `"undefined"` |
| `null` | `"null"` |
| `true` | `"true"` |
| `123` | `"123"` |
| `"abc"` | `"abc"` |

同一个实例内状态持续存在：

```python
with EdgeSandbox() as sandbox:
    sandbox.evaluate("globalThis.counter = 10")
    assert sandbox.evaluate("++counter") == "11"
```

优先使用 `with`；退出时会关闭隔离进程并释放原生内存。也可显式调用
`sandbox.close()`。

## 5. source URL和堆栈

```python
result = sandbox.evaluate(
    javascript,
    source_url="https://sandbox.test/assets/app.js",
)
```

`source_url` 会写入V8 `ScriptOrigin`、异常堆栈和资源
`PerformanceEntry.name`。

## 6. typed运行选项

```python
from examples import DeterministicExecution, EdgeRunOptions, PageInit, SandboxLimits

options = EdgeRunOptions(
    page=PageInit(
        url="https://sandbox.test/page",
        html="<!doctype html><html><body><main id='app'></main></body></html>",
        referrer="https://sandbox.test/start",
        content_type="text/html",
    ),
    cross_origin_isolated=False,
    deterministic=DeterministicExecution(
        clock_epoch_ms=None,
        clock_step_ms=1,
        random_seed=None,
        max_task_turns=1024,
    ),
    limits=SandboxLimits(
        timeout_ms=30_000,
        max_heap_bytes=512 * 1024 * 1024,
        max_resident_bytes=1024 * 1024 * 1024,
        max_source_bytes=4 * 1024 * 1024,
        max_output_bytes=16 * 1024 * 1024,
    ),
)

with EdgeSandbox(options=options) as sandbox:
    print(sandbox.evaluate("document.getElementById('app').tagName"))
```

配置通过typed C ABI和有界二进制IPC传输，不使用JSON字符串。

## 7. HTML、parser脚本与preload

```python
from demo.android_call_edge_sandbox import call_javascript

result = call_javascript(
    javascript,
    profile=profile,
    source_url="https://sandbox.test/ips.js",
    parser_inserted=True,
    timeout_ms=30_000,
)
```

### 7.1 当前preload时序

`preload_javascript` 不再作为额外 `<script>` 插入HTML：

```text
创建 Window + 空 document/html/head/body
→ 执行root preload
→ 解析配置HTML
→ 按parser顺序执行内联和外部脚本
```

preload阶段：

- `document.body` 已存在；
- 页面HTML节点尚不存在；
- `document.body.childElementCount` 初始为0；
- 不会增加额外 `<script>` 节点；
- 只进入root Window，不复制到iframe。

```python
preload = """
Object.defineProperty(window, "m42", {
  value: "configured", writable: true, enumerable: true, configurable: true
});
Object.defineProperty(window, "localJS", {
  value: Object.create(null), writable: true, enumerable: true, configurable: true
});
"""

result = call_javascript(
    javascript,
    profile=profile,
    preload_javascript=preload,
    parser_inserted=True,
)
```

若HTML脚本随后执行 `window.KPSDK = {}`，初始化属性顺序为：

```text
m42 → localJS → KPSDK
```

## 8. profile配置

### 8.1 配置组

`EdgeProfile` 包括：

```text
locale, navigator, screen, window, canvas, webgl, webgpu, audio,
storage, speech, fonts, css, document, media, permissions, battery,
geolocation, media_preferences, plugins, hardware_devices, sensors,
timing, xr, memory, performance
```

定义位于 `examples/edge_profile.py`，全部是typed dataclass。

### 8.2 完全硬编码的WebView 136 profile

三套固定profile完整展开在 `demo/android_call_edge_sandbox.py`：

```python
from demo.android_call_edge_sandbox import build_webview_136_success_reference_profile

profile = build_webview_136_success_reference_profile(sample_index=1)
```

- `sample_index` 只能为1、2、3；
- 每套都是完整 `EdgeProfile(...)` 字面量；
- 不调用 `build_android_profile()`；
- 不进入国家、设备、GPU、字体或内存随机池；
- 每次返回同一个不可变对象；
- 所有叶子值都可在该文件直接修改。

当前 `demo/w6_app.py` 默认使用：

```python
SANDBOX_SUCCESS_SAMPLE_INDEX = 1
```

修改为2或3即可切换固定样本。

### 8.3 随机Android profile

固定样本验证完成后，可显式进入随机入口：

```python
from demo.android_call_edge_sandbox import build_android_profile

profile = build_android_profile(
    "US",
    "Mozilla/5.0 (...) Chrome/150.0.0.0 Mobile Safari/537.36",
    seed=12345,       # 固定seed可复现；None才是真随机
    chromium_major=150,
)
```

自定义App UA包含Android版本时，设备、GPU、内存、CPU、screen/DPR和字体按兼容
整机档案选择。Chromium 136–139允许Android 8/9；140及以上要求Android 10+。

### 8.4 media_preferences

```python
MediaPreferencesProfile(
    color_scheme="light",          # light | dark
    contrast="no-preference",      # no-preference | more | less | custom
    reduced_motion=False,
    reduced_transparency=False,
    reduced_data=False,
    forced_colors=False,
    inverted_colors=False,
    monochrome_bits=0,              # 0..64
    color_gamut="p3",              # srgb | p3 | rec2020
    pointer="fine",                 # none | coarse | fine
    any_pointer="fine",             # none | coarse | fine
    hover="hover",                  # none | hover
    any_hover="hover",              # none | hover
    display_mode="browser",         # browser/fullscreen/standalone/minimal-ui/window-controls-overlay
    dynamic_range="standard",       # standard | high
    video_dynamic_range="standard", # standard | high
    scripting="enabled",            # none | initial-only | enabled
)
```

### 8.5 device_posture

```python
profile.hardware_devices.device_posture  # continuous | folded
```

它驱动 `(device-posture: ...)` media query。API是否暴露仍受版本surface控制；当前
WebView 136成功Navigator表没有 `navigator.devicePosture`，仅改值不会让不存在的API出现。

## 9. WebView 136独立surface

WebView不是普通Android Chrome别名。当前独立对齐：

- Window/Navigator/Worker/Prototype键表；
- `Object.keys(iframe.contentWindow)` 201项；
- `Object.getOwnPropertyNames(iframe.contentWindow)` 871项；
- root/iframe `element.style` 647项；
- Android系统颜色、控件几何、字体别名和部分布局。

桌面Edge 150的 `element.style` 保持735项。

## 10. iframe hook

```python
from examples import IframeHook

hook = IframeHook(
    name="iframe-text-encoder-hook",
    source=r"""
const original = TextEncoder.prototype.encode;
TextEncoder.prototype.encode = __edgev8.proxy(function encode() {
  const output = Reflect.apply(original, this, arguments);
  console.log("TextEncoder.prototype.encode", arguments, output);
  return output;
}, "encode");
""",
)

result = call_javascript(
    javascript,
    profile=profile,
    parser_inserted=True,
    iframe_hooks=(hook,),
)
```

- hook在每个iframe Realm的页面脚本前执行；
- root preload与iframe hook隔离；
- `__edgev8` 是native私有参数，不是Window属性；
- 不使用JavaScript `new Proxy`；
- hook不要求开启Trace。

## 11. stdout

console输出进入typed缓冲，不自动打印到宿主终端：

```python
for entry in result.stdout:
    print(entry.sequence, entry.level, entry.frame_url, entry.text)
    for argument in entry.arguments:
        print(argument.kind, argument.value)
```

直接调用：

```python
with EdgeSandbox() as sandbox:
    sandbox.evaluate("console.log('value', new Uint8Array([1,2,3]))")
    messages = sandbox.stdout()
    sandbox.clear_stdout()
```

## 12. 请求捕获

请求捕获不依赖Trace：

```python
with EdgeSandbox(profile=profile, options=options) as sandbox:
    sandbox.evaluate(javascript)
    for request in sandbox.network_requests():
        print(request.sequence, request.source)
        print(request.method, request.url)
        print(request.headers)
        print(request.body)  # bytes
    sandbox.clear_network_requests()
```

默认离线运行时只捕获，不发送真实网络流量。

## 13. network replay

```python
from examples import EdgeRunOptions, NetworkReplayEntry

options = EdgeRunOptions(
    network_replay=(
        NetworkReplayEntry(
            url="https://sandbox.test/api/data",
            method="GET",
            status=200,
            status_text="OK",
            headers=(("content-type", "application/json"),),
            body=b'{"ok":true}',
        ),
    ),
)
```

replay按规范化method+URL精确匹配，可用于fetch、XHR、iframe、Worker和外部脚本。
需要继续响应回调的流程必须配置正确status、headers和body。

## 14. Native Trace

Trace默认关闭：

```python
from pathlib import Path

with EdgeSandbox(profile=profile) as sandbox:
    sandbox.set_native_trace_exclusions((
        "window.String",
        "window.Number",
        "window.Math*",
    ))
    sandbox.enable_native_trace()
    try:
        sandbox.evaluate(javascript)
    finally:
        sandbox.disable_native_trace()

    count = sandbox.export_native_trace(
        Path("build/native-trace.log"),
        batch_size=8192,
        overwrite=True,
    )
```

尾部 `*` 表示前缀过滤。大trace优先使用 `export_native_trace()`，不要一次性展开全部。

## 15. 超时与资源限制

```python
options = EdgeRunOptions(
    limits=SandboxLimits(
        timeout_ms=5_000,
        max_heap_bytes=256 * 1024 * 1024,
        max_resident_bytes=768 * 1024 * 1024,
        max_source_bytes=2 * 1024 * 1024,
        max_output_bytes=8 * 1024 * 1024,
    )
)
```

超时会终止隔离Worker并抛出 `SandboxExecutionError`。

## 16. 多Worker并发

```python
from examples import EdgeSandboxPool

with EdgeSandboxPool(
    workers=10,
    timeout_ms=30_000,
    one_shot_workers=True,
    prewarm=True,
) as pool:
    results = pool.evaluate_many(
        sources,
        source_urls=source_urls,
        profiles=profiles,
    )
```

- 一个Worker同时只执行一个顶层任务；
- 多Worker并行执行互不相关脚本；
- 每个任务可传不同profile；
- `one_shot_workers=True` 时每个Worker执行一次后销毁并补充；
- `close_worker_after_network_requests=True` 时读取请求后关闭对应Worker；
- `completed_worker_process_id(task_id)` 可取得执行任务的OS PID。

## 17. 固定profile离线测试

```python
from pathlib import Path
from demo.success_profile_sandbox_test import call_file_with_success_profile

result = call_file_with_success_profile(
    Path("user-test/ips.js"),
    sample_index=1,
    parser_inserted=True,
    iframe_hooks=(hook,),
)

paths = tuple(request.url for request in result.requests)
has_error = any(path.rstrip("/").endswith("/error") for path in paths)
has_tl = any(path.rstrip("/").endswith("/tl") for path in paths)
```

该模块只运行沙箱并捕获请求，不向真实第三方发送请求。

## 18. API能力概览

主要功能族：

- Window、Location、History、Screen、VisualViewport；
- EventTarget和常见Event；
- Document、Node、Element、HTML元素、选择器和DOM集合；
- HTML parser、srcdoc、iframe Realm和同源/跨源控制；
- CSSStyleDeclaration、getComputedStyle、matchMedia、布局和Rect；
- Cookie、Storage、IndexedDB、Cache、URL和Blob URL；
- TextEncoder/TextDecoder、Crypto/SubtleCrypto；
- XMLHttpRequest、fetch、Headers、Request和Response；
- Canvas、WebGL1/2、WebGPU；
- WebAudio、媒体能力和部分媒体事件；
- Navigator、Permissions、Battery、Geolocation、Sensors、XR；
- Worker/SharedWorker/ServiceWorker形态和离线加载；
- Performance Timeline、Date/performance时钟和memory；
- typed stdout和native Trace。

这是浏览器语义沙箱，不是完整网络浏览器：没有真实页面渲染器、真实GPU驱动或默认外网传输。

## 19. 在其它设备复制w6调试环境

当前源码结构下，Windows最小运行集如下：

```text
edge_sandbox-main/
├─ demo/
│  ├─ __init__.py
│  ├─ w6_app.py
│  ├─ dv.py
│  ├─ encrypt_dt.js
│  ├─ android_call_edge_sandbox.py
│  ├─ call_edge_sandbox.py
│  ├─ get_random_fp.py
│  └─ fp/                         # 整个目录，包含31个catalog模块
├─ examples/                     # 整个Python目录，不需要复制_native
└─ target/
   └─ release/
      └─ edge_sandbox.dll
```

Linux把最后一个文件替换为 `libedge_sandbox.so`，macOS替换为
`libedge_sandbox.dylib`。当前 `w6_app.py` 会根据操作系统和文件自身位置寻找动态库，
不再依赖原机器的绝对路径。

运行所需第三方Python包：

```text
PyExecJS
rex_tls（使用你当前安装来源或对应wheel）
requests
loguru
py-mini-racer
tzdata（Windows建议安装）
```

`PyExecJS` 还需要可用JavaScript运行时，通常安装Node.js。Python建议3.11或更高。

不需要复制：

```text
src, tests, tools, docs, build, dist, wheelhouse*, target/debug, user-test
```

只有需要在目标设备重新编译动态库时，才需要额外复制 `src`、`Cargo.toml`、
`Cargo.lock` 并安装Rust工具链。

## 20. 构建

```powershell
# Windows debug
cargo build

# Windows release
cargo build --release --lib
```

```bash
# Linux/macOS release
cargo build --release --lib
```

产物：

```text
Windows: target/release/edge_sandbox.dll
Linux:   target/release/libedge_sandbox.so
macOS:   target/release/libedge_sandbox.dylib
```

release启用优化，构建明显慢于debug。Windows动态库被Python加载后无法覆盖，构建前必须
停止占用DLL的进程。

## 21. 常见错误

### schema不一致

```text
native profile schema does not match the Python binding
native runtime-options schema does not match the Python binding
```

Python和动态库不是同一源码版本，或长驻进程仍加载旧DLL。确认导入文件和library路径，
重新构建并重启宿主。

### 执行超时

```text
JavaScript execution exceeded the configured timeout
```

检查无限循环、长任务、Trace和debug构建。性能测试使用release。

### hook没有stdout

- 确认 `iframe_hooks` 已传入；
- hook内确实调用 `console.log`；
- 从 `result.stdout` 或 `sandbox.stdout()`读取；
- console不会自动打印到终端。

### preload看到完整HTML

说明仍加载旧DLL。当前正确行为是preload阶段body为空，之后才解析HTML。

### DLL拒绝覆盖

说明Python、PyCharm、FastAPI或其它宿主仍加载DLL。先停止进程，不要强制删除使用中的DLL。

## 22. 缓存清理

可安全重建：

```text
target/debug
demo/__pycache__
examples/__pycache__
tests/__pycache__
tools/__pycache__
```

不要把 `target/release`、`build`、`dist`、`wheelhouse*`、`user-test` 当普通缓存删除；
其中可能包含当前动态库、CI产物、wheel或黑盒证据。

## 23. 当前回归基线

```text
profile schema = 15
options schema = 4
WebView Object.keys(iframe.contentWindow) = 201项
WebView Object.getOwnPropertyNames(iframe.contentWindow) = 871项
WebView root/iframe element.style = 647项
Desktop element.style = 735项
root preload先于HTML解析
iframe TextEncoder hook可从stdout读取
```

后续修复应先在固定profile上复现，通过后再进入随机profile组合测试。

## 24. W6纯沙箱执行API

当前W6执行、`/tl`导出、可选console、DV/DT、4C4G并发与V8内存配置的完整调用
方法统一记录在：

[`W6_SANDBOX_EXECUTOR_API_ZH.md`](W6_SANDBOX_EXECUTOR_API_ZH.md)
