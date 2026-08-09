# Edge Sandbox 完整使用手册

本文档说明如何构建、配置和嵌入 `edge_sandbox`，以及如何使用单实例、HTML/DOM、指纹配置、离线网络、请求导出、原生 API Trace、进程隔离和多 Worker 并发执行。

## 1. 项目定位

`edge_sandbox` 是一个不启动真实浏览器的 Edge HTTPS Window 运行时。它直接链接 V8，并通过 Rust 原生回调实现 DOM、BOM、事件、Worker、WebAudio、WebGPU、WebXR、媒体、存储、密码学、CSS 等浏览器接口。

兼容性基准来自 Microsoft Edge 150 的 HTTPS 页面证据。默认页面身份是：

```text
isSecureContext === true
location.href === "https://sandbox.test/"
origin === "https://sandbox.test"
```

默认 User-Agent 是固定的 Chrome 150 字符串，不包含 `Edg/`：

```text
Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36
```

这里的“Edge 兼容”指 Window 属性、描述符、原型链、对象标签、API 行为和 HTTPS 页面语义以 Edge 证据为准；默认 UA 则按照项目要求使用 Chrome 150。

## 2. 核心能力概览

| 能力 | 当前行为 |
|---|---|
| JavaScript | V8 150，支持普通脚本和顶层 Promise 结果 |
| Window | 1232 个自有属性，按 Edge 证据顺序安装 |
| HTML | 可在创建沙箱时解析 HTML 并生成真实 Document 树 |
| DOM | Document、Node、Element、Range、Selection、Shadow DOM、集合与查询等 |
| `document.all` | 连接真实 DOM，保留 HTMLAllCollection 特殊形态 |
| Events | Event、EventTarget、Window/Document/Node 传播、监听选项和 Shadow DOM 路径 |
| Worker | Dedicated、Shared、Service Worker 相关 Realm、消息、计时器和结构化克隆 |
| Navigator | 固定默认值，并支持完整的强类型指纹覆盖 |
| 时间 | Date、performance、事件时间、计时器和动画帧共享时钟模型 |
| 网络响应 | 离线 NetworkReplay，不直接访问互联网 |
| 请求导出 | 不开启 Trace 也会记录 XHR/fetch 的 method、URL、headers 和原始 body |
| API Trace | V8 原生拦截，不使用 JavaScript `new Proxy` |
| 进程隔离 | JavaScript 在独立 OS 进程中执行，带超时、堆和常驻内存限制 |
| Python 并发 | 一个 `EdgeSandboxPool` 可管理多个独立进程/V8 Worker |
| 每任务指纹 | Pool 的每次 `submit()`/`evaluate()` 都可指定不同指纹 |
| DLL/FFI | 强类型 C ABI、二进制 IPC、ESNR 请求记录，不使用 JSON 配置包 |

项目不是浏览器自动化工具，不会启动 Edge、Chrome、CDP 或 Playwright。它也不是网络客户端；需要真实发送请求时，应先从沙箱导出请求，再由调用方自己的 HTTP 客户端发送。

## 3. 运行架构

常用的 Python 隔离调用链如下：

```text
Python 应用
  └─ EdgeSandbox / EdgeSandboxPool
       └─ edge_sandbox.dll / libedge_sandbox.so
            └─ 动态库自承载的独立 Worker 进程
                 └─ V8 Isolate + Window + DOM + Web APIs
```

控制器与 Worker 使用有长度边界的二进制 IPC。JavaScript 源码、执行结果、配置和请求记录不会通过 JSON 信封传输。

Windows 的部署包只需 `edge_sandbox.dll`：控制器调用 Windows 自带的 DLL 加载宿主进入 DLL 导出的 Worker 入口，不需要项目自带的 EXE。Linux 的部署包只需 `libedge_sandbox.so`，已加载的 SO 直接 `fork()` 出 Worker。两个平台仍然是真实 OS 进程隔离，不是在 Python 线程里伪造 Worker。

单个 `EdgeSandbox` 对应一个独立 Worker 进程和一个 V8 Isolate。`EdgeSandboxPool` 是一个 Python 外观对象，内部管理多个 `EdgeSandbox`，因此多个互不相关的脚本可以真正并行执行。

## 4. 构建

### 4.1 开发构建

```powershell
cd D:\sandbox\edge_sandbox-main
cargo build
```

### 4.2 Windows Release DLL

```powershell
cargo build --release --lib
Copy-Item -LiteralPath target\release\edge_sandbox.dll `
  -Destination dist\windows-x64\edge_sandbox.dll -Force
```

Windows 下的唯一部署产物是：

```text
target/release/edge_sandbox.dll
```

`autobins = false` 使普通 Cargo 构建不再生成项目自带的 `edge-sandbox.exe`。替换 DLL 后必须重新启动已经加载过旧 DLL 的 Python/宿主进程；Windows 会继续让存量进程使用内存中已加载的旧模块。

### 4.3 Linux Release SO

应在真实 Linux、安装了 Linux 发行版的 WSL2，或者已经完整配置好的 Linux 交叉编译环境中执行：

```bash
cargo build --release --lib
mkdir -p dist/linux-x64
cp target/release/libedge_sandbox.so dist/linux-x64/libedge_sandbox.so
```

Linux 的唯一部署产物是：

```text
target/release/libedge_sandbox.so
```

不能因为 Windows 上存在 `clang.exe` 就认为可以直接生成 `.so`。至少还需要 Linux Rust target、ELF 链接器、目标系统库，以及与目标三元组匹配的 `rusty_v8` 静态库。当前开发机仅安装了 `x86_64-pc-windows-msvc` target，WSL 没有 Linux 发行版，Docker/Podman 也未安装，因此当前状态只能直接构建 DLL，不能立即构建可发布的 SO。机器已有 Zig，但尚未配置为本项目的 Linux 交叉链接器。

Release 启用了优化、Thin-LTO、单 codegen unit 和符号剥离，构建时间会明显长于 Debug；构建耗时不等于脚本运行耗时。

## 5. Python 单实例快速开始

Python 绑定位于 `examples/run_sandbox.py`。项目没有要求通过命令行包装 Python 调用。

```python
from examples.run_sandbox import EdgeSandbox

javascript = """
(() => {
  const element = document.createElement("div");
  element.id = "result";
  element.textContent = "hello";
  document.body.appendChild(element);
  return document.getElementById("result").textContent;
})()
"""

with EdgeSandbox() as sandbox:
    result = sandbox.evaluate(javascript)
    print(result)  # hello
```

`with` 块结束时会销毁原生句柄并关闭隔离 Worker。也可以手动调用：

```python
sandbox = EdgeSandbox()
try:
    result = sandbox.evaluate("1 + 1")
finally:
    sandbox.close()
```

建议 Python 3.11 或更高版本。

## 6. `evaluate()` 的返回和状态语义

```python
value = sandbox.evaluate(source)
```

Python 层返回 JavaScript 最终值的文本表示：

| JavaScript 结果 | Python 返回示例 |
|---|---|
| `undefined` | `"undefined"` |
| `null` | `"null"` |
| `true` | `"true"` |
| `123` | `"123"` |
| `"text"` | `"text"` |
| 其他对象 | 对应的显示字符串 |

顶层值是 Promise 时，沙箱会执行微任务和受限的任务轮次：

```python
result = sandbox.evaluate(
    'Promise.resolve("fulfilled")'
)
```

- Promise fulfilled：返回 fulfillment value。
- Promise rejected：抛出 `SandboxExecutionError`。
- 在允许的任务轮次内仍 pending：抛出 pending 错误，不返回 `[object Promise]`。

同一个单实例中的状态会持续存在：

```python
with EdgeSandbox() as sandbox:
    sandbox.evaluate("globalThis.counter = 10")
    print(sandbox.evaluate("++counter"))  # 11
```

持续状态包括 Window 全局变量、DOM、Cookie、Storage、加载的脚本和未清除的请求记录。互不信任的任务不应共享单实例，应使用独立实例或 Pool 中的独立 Worker。

## 7. 页面 URL、`location.href` 和 HTML 初始化

### 7.1 推荐方式：在创建时配置页面

```python
from examples.edge_runtime_options import EdgeRunOptions, PageInit
from examples.run_sandbox import EdgeSandbox

options = EdgeRunOptions(
    page=PageInit(
        url="https://example.test/app/index.html",
        html="""
            <!doctype html>
            <html>
              <head><title>Example</title></head>
              <body>
                <main id="app" class="ready">content</main>
              </body>
            </html>
        """,
        referrer="https://referrer.test/start",
        content_type="text/html",
    )
)

with EdgeSandbox(options=options) as sandbox:
    result = sandbox.evaluate(
        "[location.href, document.title, "
        "document.getElementById('app').textContent].join('|')"
    )
```

`PageInit.url` 必须是没有用户名/密码的 HTTPS URL。支持的页面类型包括：

- `text/html`
- `application/xhtml+xml`
- `application/xml`
- `text/xml`
- `image/svg+xml`

HTML 上限为 4 MiB；URL 和 referrer 上限为 16 KiB。

### 7.2 脚本运行时修改地址

```python
target_url = "https://example.test/search?q=edge"
javascript = f'''
(() => {{
  location.href = {target_url!r};
  return location.href;
}})()
'''
```

赋值必须位于合法 JavaScript 语句中。不要把 Python 的 `f'''...'''` 片段误写进 JavaScript 源码，否则会产生类似 `SyntaxError: Unexpected identifier 'location'` 的错误。

如果 URL 会参与相对地址解析、Cookie 域和路径、资源加载或 DOM 初始化，优先使用 `PageInit`，不要等脚本开始后再修改。

## 8. HTML 和 DOM 功能

`PageInit.html` 会在脚本执行前解析为 Document。生成的节点会进入真实 DOM 状态，因此下列接口共享同一棵树：

```javascript
document.documentElement
document.head
document.body
document.all
document.getElementById()
document.getElementsByName()
document.getElementsByTagName()
document.getElementsByClassName()
document.querySelector()
document.querySelectorAll()
```

动态节点示例：

```javascript
const list = document.createElement("ul");
for (const text of ["a", "b", "c"]) {
  const item = document.createElement("li");
  item.className = "entry";
  item.textContent = text;
  list.append(item);
}
document.body.append(list);

return [
  document.querySelectorAll(".entry").length,
  document.getElementsByTagName("li").length,
  document.all.length
].join("|");
```

DOM 行为覆盖的主要类别包括：

- Document、DocumentFragment、Node、Element、Text、Comment、Attr。
- HTML、SVG、MathML 元素工厂和对应原型链。
- 节点插入、删除、替换、克隆、导入、采用和规范化。
- live collection、static NodeList、HTMLCollection、NamedNodeMap、DOMTokenList。
- `innerHTML`、`outerHTML`、fragment parsing、DOMParser、XMLSerializer。
- Range、StaticRange、Selection、TreeWalker、NodeIterator。
- Shadow DOM、slot 分配、开放/关闭 ShadowRoot、composed path。
- 表单、label、datalist、模板内容和属性反射。
- CSSStyleDeclaration、style attribute、CSSOM、adoptedStyleSheets。
- 几何、滚动、命中测试、ResizeObserver、IntersectionObserver。
- Cookie、CookieStore 和 `document.cookie` 的域/路径/过期状态。

沙箱不进行 GPU 像素级页面渲染。布局、几何、Canvas、WebGL 等由可配置且确定性的状态模型提供，不等价于在桌面上打开真实浏览器窗口。

## 9. Event 和 EventTarget

Window、Document 和 DOM Node 使用同一套事件传播模型：

```javascript
const log = [];
window.addEventListener("demo", () => log.push("window-capture"), true);
document.addEventListener("demo", () => log.push("document"));
document.body.addEventListener("demo", () => log.push("body"));

document.body.dispatchEvent(new Event("demo", {
  bubbles: true,
  composed: true
}));

return log.join(",");
```

支持的行为包括监听器顺序、capture/bubble、`once`、`passive`、AbortSignal、`stopPropagation()`、`stopImmediatePropagation()`、属性事件处理器、异常隔离以及 Shadow DOM composed/non-composed 路径。

## 10. 指纹配置

### 10.1 强类型、部分覆盖

指纹通过 `EdgeProfile` 及其嵌套 dataclass 配置。未填写字段继续使用固定 Chrome 150 默认值，不需要构造完整配置。

```python
from examples.edge_profile import (
    EdgeProfile,
    LocaleProfile,
    NavigatorProfile,
    ScreenProfile,
    UserAgentDataProfile,
    WebAudioProfile,
    WindowProfile,
)
from examples.run_sandbox import EdgeSandbox

profile = EdgeProfile(
    id="desktop-a",
    locale=LocaleProfile(
        locale="en-US",
        time_zone="America/New_York",
        time_zone_offset_minutes=300,
    ),
    navigator=NavigatorProfile(
        user_agent="Mozilla/5.0 (...) Chrome/150.0.0.0 Safari/537.36",
        platform="Win32",
        language="en-US",
        languages=("en-US", "en"),
        hardware_concurrency=8,
        device_memory_gb=8.0,
        webdriver=False,
        user_agent_data=UserAgentDataProfile(
            platform="Windows",
            architecture="x86",
            bitness="64",
            mobile=False,
        ),
    ),
    screen=ScreenProfile(
        width=1920,
        height=1080,
        avail_width=1920,
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
        sample_rate=48_000,
        max_channel_count=2,
        base_latency=0.01,
        output_latency=0.02,
    ),
)

with EdgeSandbox(profile=profile) as sandbox:
    print(sandbox.evaluate(
        "[navigator.userAgent, navigator.platform, "
        "screen.width, new AudioContext().sampleRate].join('|')"
    ))
```

`ScreenProfile` 表示物理屏幕，`WindowProfile` 表示 Window 视口。如果配置了 `screen.width/height` 但未显式配置 Window 尺寸，`innerWidth`、`innerHeight`、`outerWidth`、`outerHeight` 会从 Screen 尺寸自动派生。Window 中显式填写的 `0` 是有效指纹值，不会被默认值覆盖。旧的 `ScreenProfile.viewport_width/viewport_height/outer_width/outer_height` 仍保留兼容，新代码建议使用 `WindowProfile`。

DOM 布局使用最终 Window 视口。例如 input 的 `width:50vw`、`height:10vh` 和 `width:50%` 会在每次读取 Rect 时按当前视口重新计算；没有相对尺寸样式的默认 text input 依 Edge 语义保持固有 `177×21` CSS px。

`hardware_concurrency` 和 `device_memory_gb` 是用户指纹字段。沙箱不再对它们施加浏览器档位白名单或人为范围：`hardware_concurrency` 通过 C ABI 的 `u32` 字段传递，`device_memory_gb` 通过 `f64` 字段传递。调用方应根据自己的目标证据填写；preset 中的值只是默认值，不是全局限制。

随机 profile 生成器会额外保证浏览器版本一致性，但不会把两者混为一谈：`hardwareConcurrency` 按所选真实硬件行可覆盖 32、64、96、128、192 等逻辑处理器数；Edge/Chromium 147–150 的桌面 `deviceMemory` 只生成 2/4/8/16/32，Android 只生成 1/2/4/8。这个桶规则仅用于随机生成器，不会限制用户显式传入的自定义值。

### 10.2 可配置类别

| 类别 | Python 类型示例 |
|---|---|
| Locale/时区 | `LocaleProfile` |
| Navigator/UA/Client Hints | `NavigatorProfile`、`UserAgentDataProfile` |
| 网络指纹 | `NetworkProfile` |
| Screen/Window/VisualViewport | `ScreenProfile`、`WindowProfile` |
| Canvas/TextMetrics | `CanvasProfile` |
| WebGL/WebGPU | `WebGlProfile`、`WebGpuProfile` |
| WebAudio | `WebAudioProfile` |
| Storage/Heap | `StorageProfile`、`MemoryProfile` |
| Speech/Fonts | `SpeechProfile`、`FontProfile` |
| Media/WebRTC/WebCodecs | `MediaProfile`、`RtcCodecProfile` |
| Permissions | `PermissionsProfile` |
| Battery/Geolocation | `BatteryProfile`、`GeolocationProfile` |
| CSS media preferences | `MediaPreferencesProfile` |
| 初始 Document/BODY | `DocumentProfile` |
| Plugins/MIME types | `PluginListProfile` |
| Gamepad/USB/HID/Serial | `HardwareDevicesProfile` |
| Bluetooth/MIDI/Keyboard | `HardwareDevicesProfile` |
| Sensors/XR | `SensorsProfile`、`XrProfile` |
| Timing/random | `TimingProfile` |

指纹在 Worker 创建时固定。不要试图在同一个 V8 环境中热替换完整指纹；需要每次执行不同指纹时使用 `EdgeSandboxPool`，Pool 会按配置选择、复用或重建 Worker。

### 10.3 国家随机 profile 与资源加载上下文

`create_country_profile_details()` 除了返回强类型指纹，还会返回与该 seed 绑定的 `resource_load`。它只随机生成 URL-safe 的不透明资源标识；页面的 HTTPS origin、目录和版本仍由调用方传入，因此同一次执行中的页面地址、脚本地址和版本不会互相冲突。

```python
from edge_sandbox import create_country_profile_details
from edge_sandbox.edge_runtime_options import EdgeRunOptions, PageInit
from edge_sandbox.run_sandbox import EdgeSandbox

fingerprint = create_country_profile_details("US", seed=803431)
page_url = "https://page.example.test/a/b/fp?x-kpsdk-v=j-1.2.594"
script_url = fingerprint.resource_load.script_url(
    page_url,
    "j-1.2.594",
)

options = EdgeRunOptions(
    page=PageInit(url=page_url, html="<!doctype html><body></body>")
)
with EdgeSandbox(profile=fingerprint.profile, options=options) as sandbox:
    value = sandbox.evaluate(
        "performance.getEntriesByType('resource').at(-1).name",
        source_url=script_url,
    )
    assert value == script_url
```

同一个 seed 会生成相同资源上下文，不同 seed 会生成不同上下文。`script_url()` 保证脚本与页面同源、继承页面目录、去除 fragment，并把调用方提供的版本写入资源查询参数。

随机 profile 同时生成相互关联的 `navigator.connection`、显示/无障碍媒体偏好、设备姿态和 `navigator.userActivation` 状态。RTT 使用 50ms 桶并覆盖 0–600ms，downlink 使用 0.05Mbps 桶；`effectiveType` 由同一组 RTT/downlink 推导，`saveData` 与 `(prefers-reduced-data: reduce)` 保持一致。桌面姿态固定为 `continuous`，只有目录中标记为可折叠的 Android 设备才可能生成 `folded`。

不加载 page HTML、只调用一次 `evaluate()` 时，国家随机 profile 默认固定初始 BODY 为2个子元素、`clientHeight=0`，因此以下调用不需要额外传入这两个参数：

```python
from edge_sandbox import create_country_profile_details
from edge_sandbox.run_sandbox import EdgeSandbox

fingerprint = create_country_profile_details(
    "US",
    seed=803431,
)

with EdgeSandbox(profile=fingerprint.profile) as sandbox:
    value = sandbox.evaluate(
        "[document.body.childElementCount, "
        "document.body.children[0].tagName, "
        "document.body.clientHeight].join('|')"
    )
    assert value == "2|DIV|0"
```

`body_child_element_count=N` 指定脚本运行前 BODY 应达到的真实子元素总数，不是伪造 getter 返回值；DOM 查询、`children`、`document.all` 和节点遍历都能观察到这些节点。国家随机 profile 未传参数时使用固定的 `2/0`；`body_client_height` 只覆盖 BODY 的初始几何读数。显式传入其他数值可覆盖，显式传入 `None` 可恢复正常 HTML/CSS 布局计算。如果同时加载 page HTML，沙箱只在现有 BODY 元素数量不足目标值时补足占位 `DIV`，不会删除页面原有节点。

同一入口还支持 `document_has_focus`、`document_visibility_state`（`"visible"` 或 `"hidden"`）和 `is_popup`。这三个值分别驱动 `document.hasFocus()`、同源的 `document.hidden/visibilityState/webkitVisibilityState` 与初始 Performance 可见性条目，以及六个 `BarProp.visible`。国家随机 profile 未显式传入 `document_has_focus` 时按独立 seed 流以 50/50 抽取 `True/False`；显式传入布尔值可固定覆盖。同一 seed 可复现，并且不会改变 GPU、屏幕、语言等既有随机序列。可见性和弹窗状态默认仍为 `"visible"/False`，不会把互相关联的 API 独立随机化。

当 `evaluate(..., source_url=...)` 收到 HTTPS 脚本 URL 时，沙箱会在脚本开始执行前加入一条 `PerformanceResourceTiming`：`entryType="resource"`、`initiatorType="script"`、`responseStatus=200`、`contentType="text/javascript"`。Rust 会对本次传入的完整 UTF-8 源码执行真实 HTTP 内容编码，`decodedBodySize` 是源码字节数，`encodedBodySize` 是压缩结果字节数，`transferSize=encodedBodySize+300`。未传入编码时默认使用 `zstd`，可通过 `PerformanceProfile.evaluated_script_content_encoding` 切换为 `gzip`、`deflate`、`br`、`zstd` 或空字符串（不压缩）。因此脚本内部第一次调用 `performance.getEntriesByType("resource")` 就能看到该地址和大小；`source_url` 同时继续作为异常堆栈的 ScriptOrigin。

如果调用方已经从真实页面响应中取得完整脚本 URL，可以直接把真实 URL 作为 `source_url`；随机资源上下文用于离线测试或需要每个一次性 Worker 独立资源身份的场景，不能用固定 `xxx` 占位地址代替完整 URL。

### 10.4 Mac Edge 150 preset

Apple Silicon Mac 预设位于 `examples/mac_edge_profile.py`：

```python
from examples.mac_edge_profile import mac_edge_150_profile
from examples.run_sandbox import EdgeSandbox

profile = mac_edge_150_profile()

with EdgeSandbox(profile=profile) as sandbox:
    print(sandbox.evaluate(
        "[Intl.DateTimeFormat().resolvedOptions().timeZone, "
        "new Date().getTimezoneOffset(), navigator.hardwareConcurrency, "
        "navigator.deviceMemory, innerWidth, innerHeight].join('|')"
    ))
```

当前 preset 是一组一致的 Apple M2 Pro 测试配置：

- `hardwareConcurrency === 10`。
- `deviceMemory === 32`，对应 32GB 统一内存配置。
- 不传 `time_zone` 时，每次创建 profile 都读取 Python 宿主的本机 IANA 时区；当前开发机得到 `Asia/Shanghai` 和 JavaScript offset `-480`。
- 默认 Window 为 `1440×820` CSS px；传入 `inner_width=0, inner_height=0` 时仍会保留用户显式配置的 `0`。
- Window、同源 iframe 和 Worker 使用同一份时区与硬件配置。

用户可以覆盖为任意目标值，不需要使用预设档位：

```python
profile = mac_edge_150_profile(
    hardware_concurrency=37,
    device_memory_gb=31.5,
    time_zone="America/Los_Angeles",
    inner_width=1600,
    inner_height=950,
)
```

完整 Mac 字体、GPU、媒体与权限说明见 `docs/MAC_EDGE_PROFILE_ZH.md`。

### 10.5 `performance.memory` / `console.memory`

这三个字段不能作为彼此独立的随机数：

- `usedJSHeapSize` 是 V8 当前存活对象大小。
- `totalJSHeapSize` 对应 V8 已提交的物理堆内存。
- `jsHeapSizeLimit` 是 V8 为当前 isolate 保留的最大堆容量；始终满足 `used <= total <= limit`。

国家随机 profile 使用项目内置 V8 `15.0.245.2` 的资源约束算法计算精确上限。桌面 64 位默认值如下：

| 物理内存 | `jsHeapSizeLimit` |
| ---: | ---: |
| 1 GiB | 562036736 |
| 2 GiB | 1124073472 |
| 3 GiB | 1711276032 |
| 4 GiB | 2248146944 |
| 6 GiB | 3321888768 |
| 8 GiB 及以上 | 4395630592 |

Android 使用 V8 独立的 1:4 old-generation 比例和低端 young-generation 配置，不能复用桌面表：

| Android 物理内存 | `jsHeapSizeLimit` |
| ---: | ---: |
| 2 GiB | 549453824 |
| 3 GiB | 830472192 |
| 4 GiB | 1098907648 |
| 6 GiB | 1635778560 |
| 8 GiB | 2248146944 |
| 12 GiB | 3321888768 |
| 16 GiB 及以上 | 4395630592 |

Blink 在非精确模式下还会把三个字段映射到 100 个指数桶：首桶为 `10000000`，末桶为 `3760000000`，并缓存 20 分钟。站点隔离的 HTTPS 页面使用精确模式，缓存窗口为 50ms。随机 profile 默认模拟后者；`demo.fp.v8_memory_profile_catalog.BLINK_MEMORY_BUCKETS` 保留了全部 100 个非精确模式结果，`quantize_blink_memory_size()` 可用于明确需要非精确页面的测试。

`totalJSHeapSize` 与 `usedJSHeapSize` 由页面代码、DOM/API 初始化、资源加载、分配和 GC 决定，不存在按 RAM/CPU/GPU 枚举的有限“设备表”。随机生成器因此只抽取完整的实测快照对，不再使用“随机总量 × 随机百分比”。当前目录包含：

- Windows Edge/Chromium 148 页面样本：`98833423 / 62981207`。
- Mac M5 Edge 150 完整采集：`189287527 / 180511835`。
- 内置 V8 15.0.245.2 在 0、1000、5000、10000、25000、50000、100000、250000、500000 个保留对象负载下直接读取的 9 组 `HeapStatistics`，范围为 `8388608 / 7002608` 到 `74006528 / 56718360`。

这些负载快照不是根据比例插值出来的值。Windows 候选池为 10 组、macOS 为 10 组、Android 为 9 组；浏览器页面样本只进入对应操作系统，嵌入 V8 的 9 组进入三个平台。完整逐行数值和来源见 `demo/fp/v8_memory_profile_catalog.py`。

每次 `create_country_profile_details()` 的 `memory_snapshot_profile_id` 会注明采用哪一条证据；同一一次性 Worker 内保持一份一致快照，`performance.memory` 和 `console.memory` 使用相同三元组。用户也可以通过 `MemoryProfile` 显式覆盖六个 typed 字段，但必须保持大小关系；这些字段仍通过独立 C ABI 数字 setter 传入，不使用 JSON 字符串。

### 10.6 `performance.getEntries()` typed profile

`PerformanceProfile` 可以按顺序配置根 Window 的完整初始 Performance Timeline。配置通过二进制 C ABI 结构传入，不使用 JSON 字符串。`entries=None` 保留页面、replay 和 `evaluate(source_url=...)` 自动生成的记录；显式传入 tuple（包括空 tuple）则使用用户给出的精确初始记录，并抑制自动 navigation、visibility、resource 和 paint 重复项。脚本之后主动调用 `performance.mark()` / `measure()` 仍会正常追加记录。

自动 `evaluate` 资源的编码可独立配置：

```python
from dataclasses import replace
from examples.edge_profile import PerformanceProfile

profile = replace(
    profile,
    performance=PerformanceProfile(
        entries=None,
        evaluated_script_content_encoding="br",
    ),
)
```

gzip/deflate 使用 `flate2`，br 使用 `brotli`，zstd 使用 `zstd`；只有创建自动资源记录且编码非空时才执行压缩。`network_replay` 若带 `Content-Encoding` 响应头，也会对 replay 中的解码后 body 使用对应算法计算两种大小。精确复现某个远端响应时，服务器的压缩级别、字典、分块和版本同样会影响结果；已有 `PerformanceEntryProfile` 的显式 byte-size 字段仍是精确回填真实网络证据的入口。

```python
from dataclasses import replace

from edge_sandbox import (
    EdgeSandbox,
    PerformanceEntryProfile,
    PerformanceProfile,
    create_country_profile_details,
)
from edge_sandbox.edge_runtime_options import EdgeRunOptions, PageInit

page_url = "https://example.test/page"
script_url = "https://example.test/ips.js?v=1"
javascript = "JSON.stringify(performance.getEntries().map(e => e.toJSON()))"

navigation = PerformanceEntryProfile(
    name=page_url,
    entry_type="navigation",
    duration=3368.9,
    initiator_type="navigation",
    next_hop_protocol="h2",
    content_type="text/html",
    content_encoding="zstd",
    encoded_body_size=587,
    decoded_body_size=847,
    # 省略时由 Rust 按 encodedBodySize + 300 得到 887。
    transfer_size=None,
    response_status=429,
    response_end=425.9,
    dom_complete=3368.9,
    load_event_end=3368.9,
)
visible = PerformanceEntryProfile(
    name="visible",
    entry_type="visibility-state",
)
resource = PerformanceEntryProfile(
    name=script_url,
    entry_type="resource",
    start_time=431.3,
    duration=2121.2,
    initiator_type="script",
    next_hop_protocol="h2",
    content_type="text/javascript",
    content_encoding="zstd",
    encoded_body_size=291181,
    decoded_body_size=609863,
    transfer_size=None,  # 自动得到 291481。
    response_status=200,
    response_end=2552.5,
)

base = create_country_profile_details("US", seed=101).profile
profile = replace(
    base,
    performance=PerformanceProfile(entries=(navigation, visible, resource)),
)
options = EdgeRunOptions(
    page=PageInit(url=page_url, html="<!doctype html><body></body>")
)

with EdgeSandbox(profile=profile, options=options) as sandbox:
    print(sandbox.evaluate(javascript, source_url=script_url))
```

支持的单一 `content_encoding` 为 `""`、`"gzip"`、`"deflate"`、`"br"`、`"zstd"`。Chromium 对这些格式执行内容解码，但不会重新决定服务器的压缩结果：

- `encoded_body_size` 是线上响应在 Content-Encoding 解码前的 body 字节数；
- `decoded_body_size` 是解码后的 HTML/JavaScript 字节数；
- 普通非缓存同源响应的 `transfer_size` 为 `encoded_body_size + 300`；显式值优先，可表达缓存、重新验证或真实采集值；
- 压缩条目必须提供真实 encoded 和 decoded 大小，沙箱不会用任意压缩级别伪造外部服务器结果；
- `navigation.name` 应与 `PageInit.url` 一致，脚本 `resource.name` 应与 `evaluate(..., source_url=...)` 一致。

## 11. 时间、随机数和确定性执行

```python
from examples.edge_runtime_options import (
    DeterministicExecution,
    EdgeRunOptions,
)

options = EdgeRunOptions(
    deterministic=DeterministicExecution(
        clock_epoch_ms=1_893_456_000_000,
        clock_step_ms=1,
        random_seed=150,
        max_task_turns=1024,
    )
)
```

- `clock_epoch_ms`：固定 Date epoch；为 `None` 时使用运行时默认时钟。
- `clock_step_ms`：确定性时钟推进步长，最大一天。
- `random_seed`：固定 `Math.random()` 和相关确定性随机源。
- `max_task_turns`：一次 evaluate 最多执行的任务轮次，范围 1–65536。

Date、`performance.now()`、Event timestamp、timer、requestAnimationFrame 和 Worker 时钟使用关联模型。不要用宿主 Python 的耗时直接推断 JavaScript 内部的确定性时钟值。

普通运行模式下，`performance.now()` 使用不受系统墙钟调整影响的 Rust 单调时钟。当前沙箱公开的 `crossOriginIsolated` 为 `false`，因此按 Chromium 的非隔离路径收敛到 100 微秒网格：同一 sandbox 内使用一个秘密阈值，对当前单调时刻与当前 realm 的时间原点分别做确定性抖动收敛后再相减。它不是每次调用添加随机数，也不是简单执行 `floor(relativeTime / 0.1) * 0.1`。

- 根 Window、每个 iframe 和每个 Worker 都有自己的时间原点；同一 realm 内结果保证不倒退。
- `Event.timeStamp`、RAF callback timestamp、Gamepad/Sensor 时间戳复用同一 realm 的 Performance 时钟。
- `Date.now()` 是整数毫秒墙钟；`performance.timeOrigin + performance.now()` 与它近似对应，但不承诺逐次严格相等。
- 确定性模式仍按 `clock_step_ms` 推进任务时间；相同配置会得到相同的时间收敛阈值。

## 12. 资源限制和超时

```python
from examples.edge_runtime_options import EdgeRunOptions, SandboxLimits

options = EdgeRunOptions(
    limits=SandboxLimits(
        timeout_ms=3_000,
        max_heap_bytes=512 * 1024 * 1024,
        max_resident_bytes=768 * 1024 * 1024,
        max_source_bytes=4 * 1024 * 1024,
        max_output_bytes=2 * 1024 * 1024,
    )
)
```

隔离运行时默认限制：

| 限制 | 默认值 | 允许范围 |
|---|---:|---:|
| `timeout_ms` | 30,000 ms | 10 ms–300 s |
| `max_heap_bytes` | 512 MiB | 16 MiB–8 GiB |
| `max_resident_bytes` | 768 MiB | 64 MiB–16 GiB |
| `max_source_bytes` | 1 MiB | 1 KiB–64 MiB |
| `max_output_bytes` | 1 MiB | 1 KiB–64 MiB |

超时由控制器使用墙钟时间执行，不依赖 JavaScript 事件循环。无限循环示例：

```python
from examples.run_sandbox import SandboxExecutionError

try:
    sandbox.evaluate("while (true) {}")
except SandboxExecutionError as error:
    print(error)
```

隔离控制器会终止无响应 Worker；同一个控制器可以在后续请求中创建干净 Worker。Pool 遇到超时或原生错误时会丢弃该 Worker，避免把可能损坏或占用过多内存的实例放回池中。

## 13. 离线网络回放

沙箱本身不访问互联网。`fetch()`、XHR、iframe、模块或资源加载需要由调用方提供 `NetworkReplayEntry`。

```python
from examples.edge_runtime_options import (
    EdgeRunOptions,
    NetworkReplayEntry,
    PageInit,
)
from examples.run_sandbox import EdgeSandbox

options = EdgeRunOptions(
    page=PageInit(url="https://app.example/index.html"),
    network_replay=(
        NetworkReplayEntry(
            url="https://api.example/data",
            method="GET",
            status=200,
            status_text="OK",
            headers=(("content-type", "text/plain; charset=utf-8"),),
            body=b"offline response",
        ),
    ),
)

with EdgeSandbox(options=options) as sandbox:
    result = sandbox.evaluate(
        'fetch("https://api.example/data")'
        '.then(response => response.text())'
    )
```

匹配键是 method（忽略大小写）和完整 URL。没有匹配项时，普通 HTTP(S) fetch 会以离线不可用错误拒绝，而不是偷偷发起真实网络请求。

单个配置最多 1024 条 replay；单个 body 最大 16 MiB，header 数量最多 256。`data:` URL 可由运行时直接解码，不需要 replay。

## 14. 请求数据导出

### 14.1 请求记录与 Trace 独立

XHR 和 fetch 的请求记录默认始终开启，不需要调用 `enable_native_trace()`。记录过程不安装 JavaScript Proxy。

```python
from examples.run_sandbox import EdgeSandbox

javascript = """
const xhr = new XMLHttpRequest();
xhr.open("POST", "https://collector.example/tl");
xhr.setRequestHeader("X-Test", "value");
xhr.send(new Uint8Array([0, 1, 2, 255]));
"""

with EdgeSandbox() as sandbox:
    sandbox.evaluate(javascript)
    requests = sandbox.network_requests()

    for request in requests:
        print(request.sequence)
        print(request.source)
        print(request.method)
        print(request.url)
        print(request.headers)
        print(request.body)  # bytes，不是 JSON 字符串

    sandbox.clear_network_requests()
```

`CapturedNetworkRequest` 字段：

| 字段 | 类型 | 含义 |
|---|---|---|
| `sequence` | `int` | 当前 Worker 内的请求顺序 |
| `source` | `str` | `XMLHttpRequest` 或 `fetch` |
| `method` | `str` | 大写请求方法 |
| `url` | `str` | 解析后的完整 URL |
| `headers` | `tuple[(str, str), ...]` | 有序请求头 |
| `body` | `bytes` | 原始请求体字节 |

重复的 XHR header 会按浏览器语义合并。字符串、TypedArray、ArrayBuffer、Blob 和 URLSearchParams 会保留或转换为对应 body 字节，并在适用时补充 Content-Type。

当前结构化请求导出覆盖 XHR 和 fetch。它不表示真实网络栈最终附加的所有传输层 header，也不把 WebSocket 帧、RTCDataChannel 数据或任意设备 API 统一伪装成 HTTP 请求。

### 14.2 ESNR 二进制格式

DLL 的 `edge_sandbox_network_requests()` 返回版本化的 ESNR 二进制，而不是 JSON：

```text
4 bytes   magic = "ESNR"
u16 LE    version = 1
u16 LE    reserved
u32 LE    request count
```

每条记录包含：

```text
u64 sequence
u8  source (1 = XHR, 2 = fetch)
3 bytes reserved
u32 method length
u32 URL length
u32 header count
u64 body length
method bytes
URL bytes
repeated header name/value lengths and bytes
body bytes
```

Python 绑定已经自动解码 ESNR，普通调用方不需要手动解析。

### 14.3 初始化时 Hook iframe 内的函数

`IframeHook` 会在每一个新建、导航后的 iframe Realm 中执行。执行顺序是：安装该
Realm 的 Web API → 执行 Hook → 执行 iframe HTML 中的脚本。因此 `ips.js` 即使在
iframe 第一段同步脚本中调用 XHR，也会先经过 Hook。该功能不需要开启 Trace。

```python
from examples.edge_runtime_options import EdgeRunOptions, IframeHook
from examples.run_sandbox import EdgeSandbox

xhr_hook = IframeHook(
    name="iframe-xhr",
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

options = EdgeRunOptions(iframe_hooks=(xhr_hook,))

with EdgeSandbox(options=options) as sandbox:
    sandbox.evaluate(ips_javascript)
    requests = sandbox.network_requests()
```

`__edgev8` 是宿主只传给 Hook 函数的私有局部绑定，不是 Window 属性：

```javascript
"__edgev8" in window === false
typeof window.__edgev8 === "undefined"
Reflect.ownKeys(window).includes("__edgev8") === false
```

它提供两个 V8 原生方法：

- `__edgev8.proxy(function, nativeName)`：登记并返回同一个函数。
- `__edgev8.protectPrototypeFunction(prototype, propertyName)`：保护已经赋给原型属性的函数。

这里的 `proxy` 只是底层保护方法名，不创建 JavaScript `Proxy`。被保护函数的行为和
身份不变，并且：

```javascript
Function.prototype.toString.call(XMLHttpRequest.prototype.open)
// function open() { [native code] }
```

保护接口不会替用户修改函数的 `name`、`length`、原型或属性描述符；Hook 函数应当按
目标 API 的实际签名声明。Hook 会对嵌套 iframe 和每次 iframe 导航重新安装。

## 15. 原生 API Trace

Trace 是可选的 API 访问审计功能。不开启时不会记录 Trace，也不会承担多百万条事件的存储开销。

```python
with EdgeSandbox() as sandbox:
    # 精确排除两个 API；规则必须在需要过滤的执行之前设置。
    sandbox.set_native_trace_exclusions([
        "window.String",
        "window.Number",
    ])

    # 只记录 enable_native_trace() 之后发生的 API 交互。
    sandbox.clear_native_trace()
    sandbox.enable_native_trace()
    try:
        sandbox.evaluate(
            "document.createElement('div'); navigator.userAgent"
        )
    finally:
        sandbox.disable_native_trace()

    all_entries = sandbox.native_trace()
    navigator_entries = sandbox.native_trace_matching("navigator")

    for entry in navigator_entries:
        print(entry)

    sandbox.clear_native_trace()
```

`set_native_trace_exclusions()` 的规则由调用方决定，并在 Worker 的原生记录入口执行：

- `"window.String"`：只排除完全相同的 API 路径。
- `"window.console.*"`：末尾 `*` 表示按前缀排除。
- `[]`：清空全部排除规则。
- 规则只影响设置之后新产生的记录，不会删除此前已经记录的 Trace。
- 被排除的记录不会进入字符串表或 Trace 条目数组，因此不会占用后续导出内存。

该接口传递强类型 UTF-8 字符串视图，不使用 JSON 配置。Trace 中的数组参数会在不调用用户 getter、也不触发 Proxy trap 的前提下展开，例如 `args=["alpha",2,[true,null]]`。数组输出限制为 3 层、每层 32 项和 512 个字符；循环引用显示为 `[Circular]`。

最小开启方式就是：

```python
sandbox.enable_native_trace()
```

读取结果使用 `native_trace()`。当前 Python 绑定会自动按 sequence 范围分批从 Worker 拉取，避免 Worker 为全量导出一次性展开所有紧凑记录。需要边读边处理、避免 Python 控制器也持有全部结果时，使用：

```python
for batch in sandbox.native_trace_batches(batch_size=8192):
    for entry in batch:
        print(entry)
```

需要直接生成指定文件时，使用 `export_native_trace()`：

```python
count = sandbox.export_native_trace(
    "output/native-trace.log",
    batch_size=8192,
    overwrite=True,
)
print(f"已导出 {count} 条 Trace")
```

该接口使用 UTF-8、每条记录一行，并按批次从隔离 Worker 流式写入文件，不会先在 Python 中构造完整 Trace。父目录不存在时会自动创建。默认 `overwrite=False`，目标文件已经存在时抛出 `FileExistsError`；明确传入 `overwrite=True` 才会覆盖。

只关心部分 API 时优先使用 `native_trace_matching("关键词")`，筛选在隔离 Worker 内完成。`disable_native_trace()` 只停止继续记录，不会清空已有记录；`clear_native_trace()` 才会清空。

实现特性：

- 使用 V8 原生 API 回调和拦截路径。
- 不使用 JavaScript `new Proxy` 包装 Window 或 API。
- 不改变函数的 `name`、`length`、描述符、原型或 `[native code]` 字符串形态。
- 用户自己创建的 Proxy 不会被 Trace 为了格式化而额外触发 trap。
- `native_trace_matching()` 在隔离 Worker 内过滤，适合大规模 Trace。

Trace 与请求导出是两套独立功能。只需要 method、URL、header 和 body 时不要开启 Trace。

Rust 层兼容旧代码的 `enable_proxy_trace()` 等名称仍是 native trace 的别名，但实现不依赖 JavaScript Proxy。Python `EdgeSandbox` 推荐并直接提供 `set_native_trace_exclusions()`、`enable_native_trace()`、`disable_native_trace()`、`native_trace()`、`native_trace_batches()`、`export_native_trace()`、`native_trace_matching()` 和 `clear_native_trace()`。

## 16. 多 Worker 并发

### 16.1 基本结构

```python
from examples.edge_sandbox_pool import EdgeSandboxPool

with EdgeSandboxPool(
    workers=10,
    timeout_ms=30_000,
    close_worker_after_network_requests=True,
) as sandbox:
    tasks = [sandbox.submit(source) for source in javascript_sources]
    results = [task.result() for task in tasks]
```

这里是 10 个 Python 调度线程控制 10 个独立 OS 进程/V8 Worker，不是循环在同一个 Isolate 中执行 10 次。

### 16.2 每个任务使用不同指纹

```python
from examples.edge_profile import EdgeProfile, NavigatorProfile
from examples.edge_sandbox_pool import EdgeSandboxPool

profiles = [
    EdgeProfile(
        id=f"profile-{index}",
        navigator=NavigatorProfile(user_agent=f"Configured-UA-{index}"),
    )
    for index in range(4)
]

with EdgeSandboxPool(workers=4, timeout_ms=5_000) as sandbox:
    tasks = [
        sandbox.submit("navigator.userAgent", profile=profile)
        for profile in profiles
    ]
    values = [task.result() for task in tasks]
```

Pool 只会把任务交给指纹和运行参数完全相同的空闲 Worker。没有匹配 Worker 时会创建新进程；达到容量上限且存在不兼容的空闲 Worker 时，会关闭旧 Worker 再创建符合新配置的 Worker。

### 16.3 `submit()`、`evaluate()` 和 `evaluate_many()`

| 方法 | 行为 |
|---|---|
| `submit(source, ...)` | 异步提交，返回 `SandboxTask` |
| `evaluate(source, ...)` | 同步等待一个任务 |
| `evaluate_many(sources, profiles=...)` | 批量提交并按输入顺序返回结果 |

`SandboxTask` 提供 `task_id`、`result()`、`done()`、`cancel()`、`cancelled()` 和 `exception()`。`cancel()` 只能取消尚未开始的 Future；已经进入 V8 的任务应依赖沙箱的原生 `timeout_ms` 终止。

### 16.4 按任务提取请求

Pool 中的请求增加了 `task_id` 和 `worker_id`：

```python
task = sandbox.submit(javascript, profile=profile)
value = task.result()
requests = sandbox.network_requests(task.task_id)

for request in requests:
    print(request.task_id, request.worker_id, request.url)
```

### 16.5 请求读取后释放 Worker

```python
with EdgeSandboxPool(
    workers=10,
    close_worker_after_network_requests=True,
) as sandbox:
    task = sandbox.submit(javascript)
    result = task.result()
    requests = sandbox.network_requests(task.task_id)
    # 对应空闲 Worker 已在 network_requests 返回前关闭。
```

具体语义：

- 任务完成后，Pool 先把请求复制到 Python 结构中。
- 调用 `network_requests(task_id)` 时返回该任务的请求。
- 开关为 `True` 时，对应空闲 Worker 同步关闭。
- 如果该 Worker 已被复用于另一个正在执行的任务，会标记为任务完成后关闭。
- Worker 关闭后，请求的 Python `bytes` 仍然有效。
- `clear_network_requests()` 用于释放保留在 Python 进程中的请求副本。

十 Worker 黑盒回归验证了每个响应完成后立即提取并关闭，最终 Pool 计数和 Windows 实际 Worker 进程数都为 0。

### 16.6 并发内存规划

每个 Worker 拥有独立 V8、Window、DOM 和原生状态，内存会近似随 Worker 数量增长。十个 Worker 不会共享同一套 DOM 堆。

推荐做法：

- 根据机器物理内存设置 `workers`。
- 为每个 Worker 设置 `max_heap_bytes` 和 `max_resident_bytes`。
- 为所有任务设置有限 `timeout_ms`。
- 请求取完即释放时开启 `close_worker_after_network_requests`。
- 要复用性能时关闭该开关，并定期清理请求和状态。
- 不要把需要共享同一 Window/DOM 的任务分配给不同 Worker；这类任务应固定在同一个单实例中串行执行。

## 17. 浏览器 Worker API 与 Python Pool 的区别

脚本中的：

```javascript
const worker = new Worker("worker.js");
```

表示浏览器语义的 Worker Realm。项目覆盖 DedicatedWorker、SharedWorker、ServiceWorker、消息、结构化克隆、Transferable、timer、importScripts、close/terminate 等行为。

`EdgeSandboxPool` 则是宿主侧调度器，为多个互不相关的顶层脚本创建多个隔离进程。它们用途不同：

- 页面内部 Worker：由 JavaScript API 创建，遵循浏览器 Worker/Realm 关系。
- Python Pool Worker：由宿主调度，拥有独立 Window/DOM/V8，适合并行沙箱任务。

## 18. 主要 Web API 功能族

以下列表用于理解覆盖范围，不代表真实硬件或真实网络被访问。

### 18.1 DOM、HTML 和 CSS

- Document、Node、Element、HTMLElement、SVGElement、MathMLElement。
- DOM 查询、集合、属性、命名空间、fragment 和序列化。
- Shadow DOM、Range、Selection、Traversal、Mutation/Resize/Intersection observers。
- CSSOM、style、media queries、几何、滚动和命中测试。
- iframe Realm、WindowProxy、同源/跨源访问白名单和导航状态。

### 18.2 存储和 Cookie

- localStorage/sessionStorage 风格状态。
- Cookie、`document.cookie`、CookieStore。
- StorageManager、quota/persisted 等指纹状态。

### 18.3 密码学与编码

- Crypto、SubtleCrypto。
- Digest、HMAC、PBKDF2、HKDF、AES-GCM/CTR/CBC/KW 等测试覆盖算法。
- TextEncoder/TextDecoder、CompressionStream/DecompressionStream。
- Structured clone、ArrayBuffer transfer 和 MessagePort。

`structuredClone()`、`window.postMessage()`、`MessagePort.postMessage()`、Worker 与 BroadcastChannel 共用平台对象品牌检查。没有 WebIDL `[Serializable]` 能力的平台对象会同步抛出 `DOMException`，其 `name` 为 `DataCloneError`、`code` 为 `25`；例如 `navigator.plugins`、DOM 节点、`URL`、`Headers`。仅通过 `Object.create(PluginArray.prototype)` 制造的普通对象不具有内部平台品牌，仍按普通对象克隆。

### 18.4 音频、媒体和图形

- AudioContext、OfflineAudioContext、AudioNode、AudioParam、AudioWorklet。
- 可配置 sample rate、latency、channel count 和确定性噪声参数。
- Canvas、ImageBitmap、图像加载和可配置 TextMetrics。
- `URL.createObjectURL()` 使用当前 Window/iframe/Worker Realm 的序列化 origin，并生成 RFC 4122 version 4 形态的标识，例如 `blob:https://example.com/<uuid>`，不再把正常 HTTPS 页面错误地固定为 `blob:null/0000000000000001`。
- WebGL/WebGL2、WebGPU 能力和限制指纹。
- WebCodecs 能力、MediaDevices、WebRTC 能力和 SDP 配置。

### 18.5 Navigator、设备和环境

- Navigator/WorkerNavigator、UA Client Hints、language、hardware、memory。
- Permissions、Battery、Geolocation、Gamepad。
- USB、HID、Serial、Bluetooth、MIDI、Keyboard layout。
- Sensors、XR、device posture 和媒体偏好。

这些设备接口返回配置状态或沙箱对象，不会直接控制宿主真实设备。

## 19. Rust API

### 19.1 当前进程内运行

`EdgeRuntime` 直接在调用进程内持有 V8，适合测试和可信嵌入，但不提供崩溃隔离：

```rust
use edge_sandbox::{EdgeRuntime, Evaluation};

fn main() -> Result<(), String> {
    let mut runtime = EdgeRuntime::new()?;
    let value = runtime.evaluate("document.createElement('div').tagName")?;
    assert_eq!(value, Evaluation::String("DIV".to_owned()));
    Ok(())
}
```

### 19.2 Linux SO 自承载的独立进程

```rust
use edge_sandbox::{EdgeRuntimeOptions, IsolatedEdgeRuntime};

fn main() -> Result<(), String> {
    let runtime = IsolatedEdgeRuntime::self_hosted(EdgeRuntimeOptions::default())?;

    let value = runtime.evaluate("navigator.userAgent")?;
    println!("{value}");
    println!("worker pid = {}", runtime.process_id()?);
    println!("resident bytes = {:?}", runtime.resident_memory_bytes()?);
    Ok(())
}
```

这个 Rust 直接调用示例适用于 Linux/macOS 的 `fork()` 路径。Windows 的单 DLL 模式应调用第 20 节的 `edge_sandbox_create_self_hosted*` C ABI（Python 绑定已封装），以便导出入口位于真实 DLL 模块内。

主要 Rust 方法包括：

- `evaluate()`
- `enable_native_trace()` / `disable_native_trace()`
- `native_trace()` / `native_trace_matching()`
- `clear_native_trace()`
- `network_requests()` / `clear_network_requests()`
- `process_id()` / `resident_memory_bytes()`（隔离运行时）

## 20. DLL/C ABI

FFI 入口位于 `src/ffi.rs`。典型生命周期：

```text
edge_sandbox_options_create
  -> typed options/profile setters
  -> edge_sandbox_options_validate
  -> edge_sandbox_create_self_hosted_with_options
  -> edge_sandbox_evaluate
  -> edge_sandbox_network_requests / trace APIs
  -> edge_sandbox_buffer_free
  -> edge_sandbox_destroy
```

关键接口：

| API | 用途 |
|---|---|
| `edge_sandbox_create_self_hosted` | 仅使用当前 DLL/SO 和默认配置创建隔离沙箱 |
| `edge_sandbox_create_self_hosted_with_profile` | 仅使用当前 DLL/SO 和强类型指纹创建 |
| `edge_sandbox_create_self_hosted_with_options` | 仅使用当前 DLL/SO，按页面、回放、限制和指纹创建 |
| `edge_sandbox_create_self_hosted_with_audio_profile` | 仅使用当前 DLL/SO 和 WebAudio 指纹创建 |
| `edge_sandbox_evaluate` | 执行 UTF-8 JavaScript |
| `edge_sandbox_network_requests` | 获取 ESNR 二进制请求数据 |
| `edge_sandbox_clear_network_requests` | 清理请求记录 |
| `edge_sandbox_enable_native_trace` | 开启原生 Trace |
| `edge_sandbox_set_native_trace_exclusions` | 使用 UTF-8 字符串视图数组替换原生 Trace 排除规则 |
| `edge_sandbox_native_trace_matching` | 在 Worker 内筛选 Trace |
| `edge_sandbox_buffer_free` | 释放 DLL 返回的内存 |
| `edge_sandbox_destroy` | 销毁句柄并关闭 Worker |

调用方必须为每个成功返回的 `EdgeSandboxBuffer` 调用一次 `edge_sandbox_buffer_free()`，并且每个 handle 只能销毁一次。

ABI、Profile schema 和 Options schema 可分别通过版本函数检查。Python 绑定会在创建实例时自动检查版本一致性。

## 21. 无 CLI 部署

生产构建不生成也不需要项目 CLI/Worker EXE。调用方直接使用 Python 绑定、C ABI 或 Rust `IsolatedEdgeRuntime::self_hosted()`。二进制请求 body 在 Python 中保留为 `bytes`，C ABI 中保留为 ESNR 字节缓冲区。

## 22. 错误处理

Python 原生执行错误统一抛出 `SandboxExecutionError`。常见情况：

### 22.1 JavaScript 语法错误

```text
SyntaxError: Unexpected identifier 'location'
```

检查 Python f-string 是否生成了合法 JavaScript，语句之间是否有分号或换行，以及引号是否正确。

### 22.2 执行超时

```text
JavaScript execution exceeded the configured timeout
```

增加合理超时、减少脚本工作量，或检查无限循环。不要设置无限超时来处理不可信脚本。

### 22.3 离线 fetch

```text
Fetch for the 'https://...' URL scheme is unavailable in this offline runtime
```

为完整 method+URL 添加 `NetworkReplayEntry`，或仅捕获请求后由宿主发送。

### 22.4 顶层 Promise pending

增加 `max_task_turns`，或确保 Promise 能由沙箱的任务源完成。

### 22.5 Source/Output 超限

调整 `max_source_bytes` 或 `max_output_bytes`，但仍应设置合理上限。

### 22.6 Python 与 DLL schema 不一致

确保 `examples/run_sandbox.py` 与 DLL/SO 来自同一份项目版本，并重新构建动态库。

### 22.7 Pool Worker 未立即释放

检查：

- 是否设置 `close_worker_after_network_requests=True`。
- 是否已经调用对应 task 的 `network_requests(task_id)`。
- task 是否仍在执行。
- 调用方是否只调用了 `task.result()`，但尚未读取请求。

`task.result()` 返回表示 JavaScript 响应完成；按设计，启用开关后是在 `network_requests()` 成功提取请求时关闭 Worker。

### 22.8 `device_memory_gb must use a Chromium-exposed bucket`

这是旧 DLL 的指纹校验错误。当前源码已经移除 `hardware_concurrency` 和 `device_memory_gb` 的浏览器档位限制。如果仍看到该错误：

1. 确认使用的是重新构建后的 `dist/windows-x64/edge_sandbox.dll`。
2. 完全退出并重启已经加载旧 DLL 的 Python/宿主进程。
3. 确认 `examples/run_sandbox.py` 与 DLL 来自同一份源码和 schema 版本。

### 22.9 `failed to fill whole buffer: memory allocation ... failed`

如果错误发生在读取 `sandbox.native_trace()` 时，通常是旧 Python 绑定要求 Worker 一次性展开并序列化全部 Trace，瞬时内存超过 `max_resident_bytes`。当前 `examples/run_sandbox.py` 已将 `native_trace()` 改为自动分批读取；更新绑定后重新创建沙箱并重新执行脚本。超大 Trace 建议直接迭代 `native_trace_batches()`，不要再在调用方拼成一个巨大字符串。

如果错误发生在 `evaluate()` 阶段，则是记录本身已经占满 Worker 内存，应缩短开启 Trace 的代码区间、在阶段之间执行 `clear_native_trace()`，或在确认宿主内存足够后提高 `max_resident_bytes`。`navigator.deviceMemory` 只是脚本可见指纹，不会改变真实 Worker 内存限制。

## 23. 安全建议

- 对不可信代码优先使用 `IsolatedEdgeRuntime`、DLL/Python `EdgeSandbox`，不要直接使用进程内 `EdgeRuntime`。
- 始终配置有限的墙钟超时、V8 heap、常驻内存、源码和输出大小。
- 根据物理内存限制 Pool Worker 数量。
- 不要把宿主秘密写入 JavaScript 全局或错误信息。
- NetworkReplay 内容应视为传入沙箱的数据。
- 请求 body 导出后可能包含敏感信息，应按应用安全策略处理和清除。
- Worker 进程禁用真实网络并不等于整个宿主 Python 应用禁网；宿主 HTTP 客户端仍由调用方负责。

Windows 使用 Job Object 限制 Worker 为单进程并配置 kill-on-close。Linux 使用 parent-death 策略和禁止网络 socket 的 seccomp 过滤。进程隔离能限制 V8 崩溃、超时和内存失控的影响，但调用方仍应保持依赖更新并进行安全审计。

## 24. 性能和复用

单实例复用会省去重复创建 V8 和 Worker 的开销，但保留 Window/DOM/Cookie 等状态。一次性模式使用预热进程：任务开始前在同一 PID 中加载新的 profile 和 V8 isolate，只执行一份 JavaScript，完成后立即关闭该进程并补充新的空白 Worker。

当前机器上的参考测试曾得到：

- Python 创建单 Worker：约几十毫秒。
- 小型脚本执行和请求读取：通常小于数毫秒。
- 2026-08-06 Release DLL 复测 10 份 `ips.js` 黑盒任务：10/10 成功，10 个 `/tl` 均导出，总墙钟 3.753 秒。

这些数字只用于说明架构能够并行，不是跨机器性能承诺。脚本复杂度、CPU 核数、内存压力、Debug/Release 和指纹配置都会改变结果。

## 25. 验证和回归测试

Rust 回归：

```powershell
cargo fmt --all
cargo check --all-targets
cargo test --lib -- --test-threads=1
cargo test --test options_ffi --test process_isolation --test profile_ffi
cargo test --test network_capture_ffi --test ips_blackbox
cargo clippy --all-targets
```

Python Pool 回归：

```powershell
py -3 -m unittest tests/python_pool_smoke.py tests/ips_pool_blackbox.py -v
py -3 -m unittest tests/python_mac_profile_smoke.py -v
```

本地 FastAPI 黑盒计时入口为 `demo/ips_api_blackbox.py`。它不解析 `ips.js`，每个任务分别读取一次文件、创建唯一 `scriptId` 和独立 profile，并在 API 外部汇总 HTTP wall/min/mean/median/P95/max。

`tests/ips_pool_blackbox.py` 将 `demo/ips.js` 作为不透明输入，以 10 个线程和 10 个隔离 Worker 并发执行；每个 task 响应后立即提取 `/tl` 请求并关闭对应 Worker，最后断言 Worker 数为 0。

## 26. 示例索引

| 文件 | 内容 |
|---|---|
| `examples/run_sandbox.py` | DLL 单实例 Python 绑定 |
| `examples/edge_profile.py` | 完整强类型指纹 dataclass |
| `examples/edge_runtime_options.py` | 页面、回放、确定性和限制配置 |
| `examples/run_typed_page.py` | HTML、DOM 和离线 fetch 示例 |
| `examples/edge_sandbox_pool.py` | 多 Worker 并发池 |
| `demo/sandbox_worker_api.py` | 预热一次性 Worker 的通用本地 FastAPI 服务 |
| `demo/wizzair.py` | 内部异步执行业务和沙箱 Worker 的业务 FastAPI 服务 |
| `demo/wizzair_api_client.py` | 并发本地 API 客户端 |
| `demo/ips_api_blackbox.py` | `ips.js` 不透明输入、PID 与分阶段计时报告 |
| `examples/mac_edge_profile.py` | Apple Silicon Mac Edge 150 指纹 preset |
| `demo/mac_call_edge_sandbox.py` | 使用 Mac preset 调用沙箱 |
| `tests/python_pool_smoke.py` | 不同指纹、超时和释放回归 |
| `tests/python_mac_profile_smoke.py` | Mac profile 的 Window/iframe/Worker 回归 |
| `tests/ips_pool_blackbox.py` | 10 Worker 请求提取与立即释放黑盒回归 |

## 27. 功能边界总结

使用前应明确以下边界：

- 沙箱模拟 Edge HTTPS Window，但不是可见浏览器窗口。
- 不启动浏览器、不使用 CDP、不真实访问互联网。
- HTML/DOM、事件、Worker 和大量 Web API 有原生状态实现，但真实硬件、GPU 和桌面渲染由确定性模型替代。
- NetworkReplay 提供响应；Network Capture 导出请求，两者方向不同。
- 请求 Capture 当前结构化覆盖 XHR/fetch，不是通用抓包器。
- Native Trace 用于 API 行为审计，不应用于普通请求导出。
- 普通复用模式的 Worker 指纹固定；一次性预热模式会在执行前重建 isolate 并加载该任务的独立 profile。
- 多 Worker 适合互不相关脚本；共享同一 Window/DOM 的逻辑必须串行。
- Worker 进程关闭后 V8/DOM 内存释放，但已导出的 Python 请求副本需要调用 `clear_network_requests()` 或释放 Pool 对象才能回收。

按以上边界使用时，推荐的生产调用路径是：强类型配置 → 进程隔离创建 → 有限超时执行 → 结构化请求导出 → 清理请求 → 关闭或按相同指纹复用 Worker。
