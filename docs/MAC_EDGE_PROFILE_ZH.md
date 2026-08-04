# Mac Edge 150 指纹预设

项目提供 `examples.mac_edge_profile.mac_edge_150_profile()`，用于给沙箱实例安装一套可回归的 Apple Silicon macOS Edge 150 测试指纹。

当前默认硬件基线是一台可实际成立的 10 核、32GB Apple M2 Pro 配置。它只是项目的默认测试 preset，不代表所有 Mac 都具有相同数值：

- `navigator.platform === "MacIntel"`，这是 Chromium 在 Apple Silicon Mac 上保留的兼容值。
- UA-CH 为 `platform === "macOS"`、`architecture === "arm"`。
- `navigator.hardwareConcurrency === 10`。
- `navigator.deviceMemory === 32`。
- WebGL renderer 为 Apple M2 Pro 的 ANGLE Metal 形态。
- WebGL/WebGL2 的 extensions、压缩纹理格式、全部数值 limits、shader precision 和 context attributes 均显式配置，不再回退到 Windows 默认 profile。
- WebGPU 配置使用 `apple / apple8`、非 fallback adapter、32/32 subgroup 以及 ASTC/ETC2 features；按当前 Edge profile，脚本可见的 adapter `device` 和 `description` 保持空字符串。
- 字体、语音、媒体能力、权限、插件、传感器、硬件设备、WebAudio、屏幕和 Window 尺寸均由该 preset 显式覆盖。
- 默认 Window 为 `1440×820` CSS px；显式传入 `0` 时仍保留 `0`，不会被 Screen 派生值覆盖。
- 未传 `time_zone` 时，每次调用 `mac_edge_150_profile()` 都读取 Python 宿主本机时区，并向 ICU 传递 IANA 时区 ID。
- 主 Window、同源 iframe、Worker 以及开启 native trace 后均继承同一份配置。

Apple 官方 M2 Pro 同时存在 10 核/12 核 CPU 和 16GB/32GB 统一内存配置，因此不能只根据 `Apple M2 Pro` renderer 推断唯一的 CPU 或内存值。这里选用 10 核/32GB 组合，并归入 Apple8 GPU family；目标机器的 Edge 实测证据始终优先于 preset。

## 调用

```python
from pathlib import Path

from examples.mac_edge_profile import mac_edge_150_profile
from examples.run_sandbox import EdgeSandbox

library = Path("dist/windows-x64/edge_sandbox.dll")
profile = mac_edge_150_profile()

with EdgeSandbox(library=library, profile=profile) as sandbox:
    value = sandbox.evaluate(
        "[navigator.userAgent, navigator.platform, "
        "navigator.userAgentData.platform, navigator.hardwareConcurrency, "
        "navigator.deviceMemory, "
        "Intl.DateTimeFormat().resolvedOptions().timeZone, "
        "new Date().getTimezoneOffset(), screen.width, "
        "innerWidth, innerHeight, devicePixelRatio].join('|')"
    )
    print(value)
```

## 自定义非 GPU 参数

```python
profile = mac_edge_150_profile(
    macos_platform_version="15.5.0",
    locale="zh-CN",
    time_zone="Asia/Shanghai",
    hardware_concurrency=12,
    device_memory_gb=32.0,
    screen_width=1728,
    screen_height=1117,
    avail_height=1079,
    inner_width=1600,
    inner_height=950,
    device_pixel_ratio=2.0,
)
```

`time_zone=None` 是默认行为，表示使用调用方本机时区。显式传入 IANA 时区时，如果没有传 `time_zone_offset_minutes`，profile 会按该时区的当前规则自动计算 offset；Date/Intl 的历史与夏令时转换仍由 ICU 时区规则决定。只有需要覆盖当前 offset 字段时才显式填写 `time_zone_offset_minutes`。

`hardware_concurrency` 和 `device_memory_gb` 完全由用户配置，不存在浏览器档位白名单或项目人为上限。前者通过 C ABI `u32` 传递，后者通过 `f64` 传递。例如下面的非标准测试值也会原样进入 Window、iframe 和 Worker：

```python
profile = mac_edge_150_profile(
    hardware_concurrency=37,
    device_memory_gb=31.5,
)
```

指纹在 `EdgeSandbox` 创建时固定。不同执行任务需要不同指纹时，应给每个任务传入独立 profile，由沙箱池选择或重建 Worker。

替换 `dist/windows-x64/edge_sandbox.dll` 后，必须完全重启已经加载过旧 DLL 的 Python/宿主进程。如果仍看到 `device_memory_gb must use a Chromium-exposed bucket`，说明当前进程仍在使用带旧档位校验的 DLL。

## 用户传入字体

Mac preset 默认使用 SF Pro、Helvetica、Menlo、Monaco 和 Apple Color Emoji 等 Mac 字体。调用方可以完整替换字体 family 列表和 `queryLocalFonts()` 返回的本地字体记录：

```python
from examples.edge_profile import LocalFontProfile
from examples.mac_edge_profile import mac_edge_150_profile

profile = mac_edge_150_profile(
    font_families=(
        "User Mac Sans",
        "Helvetica Neue",
        "Menlo",
    ),
    local_fonts=(
        LocalFontProfile(
            postscript_name="UserMacSans-Regular",
            full_name="User Mac Sans Regular",
            family="User Mac Sans",
            style="Regular",
        ),
    ),
    allow_unknown_font_families=False,
)
```

- `font_families=None`：使用 preset 内置的 Mac 字体集合。
- `font_families=()`：显式清空已安装字体 family；CSS generic family 仍按标准可用。
- `local_fonts=None`：使用 preset 内置的 Mac 本地字体记录。
- `local_fonts=()`：令 `queryLocalFonts()` 返回空集合。
- `allow_unknown_font_families=False`：`document.fonts.check()` 只接受已配置 family 和 CSS generic family；设为 `True` 才允许未列出的字体。

这些值按沙箱实例隔离，并传播到主 Window、iframe 和 Worker。它们不会修改 DLL/Rust 的 Windows 默认字体；不传 `mac_edge_150_profile()` 的实例仍使用底层原有默认 profile。

## Intel Mac 边界

本 preset 不提供 Intel Mac 分支。项目当前没有一份能够同时闭环 UA-CH、ANGLE renderer、WebGL limits、WebGPU adapter/limits、字体和媒体能力的 Intel Mac Edge 实测证据，因此不能把猜测值标记为可回归结果。获得目标 Intel Mac 的采集证据后，应新增独立 preset 和独立测试，而不是复用 Apple M2 Pro limits。

## 有意不绑定的用户态字段

`geolocation` 和 `timing` 不属于 Apple M2 Pro/操作系统固有指纹，保持由调用方或沙箱运行策略决定；`navigator.doNotTrack` 保持未设置语义。这三个字段不会导致 Windows GPU、字体、语音或设备默认值混入 Mac preset。
