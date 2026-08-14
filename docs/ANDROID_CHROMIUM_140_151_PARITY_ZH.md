# Chromium 140–151 PC / Android HTTPS 差异与实现基线

> 本文由 `tools/generate_android_chromium_parity_report.py` 从保存的 HTTPS 证据自动生成。

本文只比较浏览器环境，不分析任何业务 JavaScript。结构证据与可变值证据分开保存；Pixel 4 的单机值只作为一条设备组合，不作为全部 Android 的固定默认。

## 证据边界

- Android 实机：Pixel 4，ADB 序列号 `9C181021C0D7D6`。
- 页面来源：`https://example.com/`，所有正式样本均要求 `isSecureContext === true`。
- Android 版本来自 Chromium 官方 Android Arm64 snapshot；每个 APK 的 revision、URL、压缩包元数据和 SHA-256 在机器报告中保存。
- PC 对照来自 Chrome for Testing 官方构建。Chrome 与同版本 Edge 共用 Chromium/Blink/V8 的基础 Web API 表；Edge 自有扩展仍以 Edge 证据单独处理。
- ChromeDriver 注入的 `cdc_*` 和 `ret_nodes` 共 8 个传输属性在统计前剔除，剔除规则和具体名称保存在每份 Android 证据中。
- HTTP、空白页和不安全上下文样本不进入正式基线，因为权限、媒体、设备和若干接口会被安全上下文改变。

## 顶层表规模

| 版本 | PC Window | Android Window | PC Navigator | Android Navigator | PC Worker | Android Worker | 原型存在差异的接口数 |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 140 | 1196 | 1191 | 83 | 81 | 328 | 321 | 45 |
| 141 | 1200 | 1195 | 83 | 81 | 332 | 325 | 45 |
| 142 | 1202 | 1196 | 83 | 81 | 332 | 325 | 47 |
| 143 | 1204 | 1199 | 83 | 81 | 332 | 325 | 48 |
| 144 | 1208 | 1204 | 83 | 81 | 333 | 326 | 45 |
| 145 | 1213 | 1207 | 83 | 81 | 334 | 329 | 47 |
| 146 | 1219 | 1213 | 83 | 81 | 334 | 329 | 46 |
| 147 | 1230 | 1226 | 83 | 81 | 334 | 329 | 49 |
| 148 | 1231 | 1226 | 83 | 81 | 334 | 329 | 45 |
| 149 | 1232 | 1232 | 83 | 82 | 334 | 329 | 46 |
| 150 | 1232 | 1230 | 83 | 82 | 334 | 327 | 52 |
| 151 | 1236 | 1234 | 83 | 82 | 335 | 328 | 50 |

## 每个版本的顶层平台差异

### 140

- Android 独有：`ContentIndex`、`onorientationchange`、`orientation`、`ontouchcancel`、`ontouchend`、`ontouchmove`、`ontouchstart`、`BarcodeDetector`、`ContactAddress`、`ContactsManager`、`CookieDeprecationLabel`、`NDEFMessage`、`NDEFReader`、`NDEFReadingEvent`、`NDEFRecord`、`getDigitalGoodsService`
- PC 独有：`WindowControlsOverlayGeometryChangeEvent`、`WindowControlsOverlay`、`documentPictureInPicture`、`CaptureController`、`DocumentPictureInPicture`、`EyeDropper`、`FileSystemObserver`、`FontData`、`HID`、`HIDConnectionEvent`、`HIDDevice`、`HIDInputReportEvent`、`PressureObserver`、`PressureRecord`、`SharedWorker`、`Summarizer`、`Translator`、`queryLocalFonts`、`CropTarget`、`DocumentPictureInPictureEvent`、`RestrictionTarget`
- 两端共有成员的相对顺序一致：`false`。

### 141

- Android 独有：`ContentIndex`、`onorientationchange`、`orientation`、`ontouchcancel`、`ontouchend`、`ontouchmove`、`ontouchstart`、`BarcodeDetector`、`ContactAddress`、`ContactsManager`、`CookieDeprecationLabel`、`NDEFMessage`、`NDEFReader`、`NDEFReadingEvent`、`NDEFRecord`、`getDigitalGoodsService`
- PC 独有：`WindowControlsOverlayGeometryChangeEvent`、`WindowControlsOverlay`、`documentPictureInPicture`、`CaptureController`、`DocumentPictureInPicture`、`EyeDropper`、`FileSystemObserver`、`FontData`、`HID`、`HIDConnectionEvent`、`HIDDevice`、`HIDInputReportEvent`、`PressureObserver`、`PressureRecord`、`SharedWorker`、`Summarizer`、`Translator`、`queryLocalFonts`、`CropTarget`、`DocumentPictureInPictureEvent`、`RestrictionTarget`
- 两端共有成员的相对顺序一致：`false`。

### 142

- Android 独有：`ContentIndex`、`onorientationchange`、`orientation`、`ontouchcancel`、`ontouchend`、`ontouchmove`、`ontouchstart`、`BarcodeDetector`、`ContactAddress`、`ContactsManager`、`CookieDeprecationLabel`、`NDEFMessage`、`NDEFReader`、`NDEFReadingEvent`、`NDEFRecord`、`getDigitalGoodsService`
- PC 独有：`documentPictureInPicture`、`CaptureController`、`DocumentPictureInPicture`、`EyeDropper`、`FileSystemObserver`、`FontData`、`HID`、`HIDConnectionEvent`、`HIDDevice`、`HIDInputReportEvent`、`PressureObserver`、`PressureRecord`、`SharedWorker`、`SpeechRecognitionPhrase`、`Summarizer`、`Translator`、`queryLocalFonts`、`CropTarget`、`DocumentPictureInPictureEvent`、`RestrictionTarget`、`WindowControlsOverlay`、`WindowControlsOverlayGeometryChangeEvent`
- 两端共有成员的相对顺序一致：`false`。

### 143

- Android 独有：`ContentIndex`、`AnimationTrigger`、`onorientationchange`、`orientation`、`ontouchcancel`、`ontouchend`、`ontouchmove`、`ontouchstart`、`BarcodeDetector`、`ContactAddress`、`ContactsManager`、`CookieDeprecationLabel`、`NDEFMessage`、`NDEFReader`、`NDEFReadingEvent`、`NDEFRecord`、`getDigitalGoodsService`
- PC 独有：`documentPictureInPicture`、`CaptureController`、`DocumentPictureInPicture`、`EyeDropper`、`FileSystemObserver`、`FontData`、`HID`、`HIDConnectionEvent`、`HIDDevice`、`HIDInputReportEvent`、`PressureObserver`、`PressureRecord`、`SharedWorker`、`SpeechRecognitionPhrase`、`Summarizer`、`Translator`、`queryLocalFonts`、`CropTarget`、`DocumentPictureInPictureEvent`、`RestrictionTarget`、`WindowControlsOverlay`、`WindowControlsOverlayGeometryChangeEvent`
- 两端共有成员的相对顺序一致：`false`。

### 144

- Android 独有：`ContentIndex`、`AnimationTrigger`、`onorientationchange`、`orientation`、`ontouchcancel`、`ontouchend`、`ontouchmove`、`ontouchstart`、`BarcodeDetector`、`ContactAddress`、`ContactsManager`、`NDEFMessage`、`NDEFReader`、`NDEFReadingEvent`、`NDEFRecord`、`getDigitalGoodsService`
- PC 独有：`documentPictureInPicture`、`CaptureController`、`DocumentPictureInPicture`、`EyeDropper`、`FileSystemObserver`、`FontData`、`HID`、`HIDConnectionEvent`、`HIDDevice`、`HIDInputReportEvent`、`PressureObserver`、`PressureRecord`、`SharedWorker`、`SpeechRecognitionPhrase`、`Summarizer`、`Translator`、`queryLocalFonts`、`CropTarget`、`DocumentPictureInPictureEvent`、`RestrictionTarget`
- 两端共有成员的相对顺序一致：`false`。

### 145

- Android 独有：`Sanitizer`、`ContentIndex`、`onorientationchange`、`orientation`、`ontouchcancel`、`ontouchend`、`ontouchmove`、`ontouchstart`、`BarcodeDetector`、`ContactAddress`、`ContactsManager`、`NDEFMessage`、`NDEFReader`、`NDEFReadingEvent`、`NDEFRecord`、`getDigitalGoodsService`
- PC 独有：`crashReport`、`documentPictureInPicture`、`CaptureController`、`CrashReportContext`、`DocumentPictureInPicture`、`EyeDropper`、`FileSystemObserver`、`FontData`、`HID`、`HIDConnectionEvent`、`HIDDevice`、`HIDInputReportEvent`、`LanguageDetector`、`PressureObserver`、`PressureRecord`、`SharedWorker`、`SpeechRecognitionPhrase`、`Summarizer`、`Translator`、`queryLocalFonts`、`DocumentPictureInPictureEvent`、`XSLTProcessor`
- 两端共有成员的相对顺序一致：`false`。

### 146

- Android 独有：`ContentIndex`、`onorientationchange`、`orientation`、`ontouchcancel`、`ontouchend`、`ontouchmove`、`ontouchstart`、`BarcodeDetector`、`ContactAddress`、`ContactsManager`、`NDEFMessage`、`NDEFReader`、`NDEFReadingEvent`、`NDEFRecord`、`getDigitalGoodsService`
- PC 独有：`documentPictureInPicture`、`AudioPlaybackStats`、`CaptureController`、`DocumentPictureInPicture`、`EyeDropper`、`FileSystemObserver`、`FontData`、`HID`、`HIDConnectionEvent`、`HIDDevice`、`HIDInputReportEvent`、`LanguageDetector`、`PressureObserver`、`PressureRecord`、`SharedWorker`、`SpeechRecognitionPhrase`、`Summarizer`、`Translator`、`queryLocalFonts`、`DocumentPictureInPictureEvent`、`XSLTProcessor`
- 两端共有成员的相对顺序一致：`false`。

### 147

- Android 独有：`ContentIndex`、`CSSPseudoElement`、`onorientationchange`、`orientation`、`ontouchcancel`、`ontouchend`、`ontouchmove`、`ontouchstart`、`BarcodeDetector`、`ContactAddress`、`ContactsManager`、`NDEFMessage`、`NDEFReader`、`NDEFReadingEvent`、`NDEFRecord`、`getDigitalGoodsService`
- PC 独有：`documentPictureInPicture`、`CaptureController`、`DocumentPictureInPicture`、`EyeDropper`、`FileSystemObserver`、`FontData`、`HID`、`HIDConnectionEvent`、`HIDDevice`、`HIDInputReportEvent`、`LanguageDetector`、`PressureObserver`、`PressureRecord`、`SharedWorker`、`SpeechRecognitionPhrase`、`Summarizer`、`Translator`、`queryLocalFonts`、`DocumentPictureInPictureEvent`、`XSLTProcessor`
- 两端共有成员的相对顺序一致：`false`。

### 148

- Android 独有：`ContentIndex`、`onorientationchange`、`orientation`、`ontouchcancel`、`ontouchend`、`ontouchmove`、`ontouchstart`、`BarcodeDetector`、`ContactAddress`、`ContactsManager`、`NDEFMessage`、`NDEFReader`、`NDEFReadingEvent`、`NDEFRecord`、`getDigitalGoodsService`
- PC 独有：`documentPictureInPicture`、`CaptureController`、`DocumentPictureInPicture`、`EyeDropper`、`FileSystemObserver`、`FontData`、`HID`、`HIDConnectionEvent`、`HIDDevice`、`HIDInputReportEvent`、`LanguageDetector`、`LanguageModel`、`PressureObserver`、`PressureRecord`、`SpeechRecognitionPhrase`、`Summarizer`、`Translator`、`queryLocalFonts`、`DocumentPictureInPictureEvent`、`XSLTProcessor`
- 两端共有成员的相对顺序一致：`false`。

### 149

- Android 独有：`ContentIndex`、`onorientationchange`、`orientation`、`ontouchcancel`、`ontouchend`、`ontouchmove`、`ontouchstart`、`BarcodeDetector`、`ContactAddress`、`ContactsManager`、`ModelContext`、`NDEFMessage`、`NDEFReader`、`NDEFReadingEvent`、`NDEFRecord`、`WebMCPEvent`、`getDigitalGoodsService`
- PC 独有：`CaptureController`、`EyeDropper`、`FileSystemObserver`、`FontData`、`HID`、`HIDConnectionEvent`、`HIDDevice`、`HIDInputReportEvent`、`LanguageDetector`、`LanguageModel`、`PressureObserver`、`PressureRecord`、`SpeechRecognitionPhrase`、`Summarizer`、`Translator`、`queryLocalFonts`、`XSLTProcessor`
- 两端共有成员的相对顺序一致：`false`。

### 150

- Android 独有：`ContentIndex`、`onorientationchange`、`orientation`、`ontouchcancel`、`ontouchend`、`ontouchmove`、`ontouchstart`、`BarcodeDetector`、`ContactAddress`、`ContactsManager`、`ModelContext`、`NDEFMessage`、`NDEFReader`、`NDEFReadingEvent`、`NDEFRecord`、`WebMCPEvent`、`getDigitalGoodsService`
- PC 独有：`CaptureController`、`EyeDropper`、`FileSystemObserver`、`FontData`、`HID`、`HIDConnectionEvent`、`HIDDevice`、`HIDInputReportEvent`、`LanguageDetector`、`LanguageModel`、`PressureObserver`、`PressureRecord`、`SpeechRecognitionPhrase`、`Summarizer`、`Translator`、`queryLocalFonts`、`CropTarget`、`RestrictionTarget`、`XSLTProcessor`
- 两端共有成员的相对顺序一致：`false`。

### 151

- Android 独有：`ContentIndex`、`onorientationchange`、`orientation`、`ontouchcancel`、`ontouchend`、`ontouchmove`、`ontouchstart`、`BarcodeDetector`、`ContactAddress`、`ContactsManager`、`ModelContext`、`NDEFMessage`、`NDEFReader`、`NDEFReadingEvent`、`NDEFRecord`、`WebMCPEvent`、`getDigitalGoodsService`
- PC 独有：`CaptureController`、`EyeDropper`、`FileSystemObserver`、`FontData`、`HID`、`HIDConnectionEvent`、`HIDDevice`、`HIDInputReportEvent`、`LanguageDetector`、`LanguageModel`、`PressureObserver`、`PressureRecord`、`SpeechRecognitionPhrase`、`Summarizer`、`Translator`、`queryLocalFonts`、`CropTarget`、`RestrictionTarget`、`XSLTProcessor`
- 两端共有成员的相对顺序一致：`false`。

## 需要单独实现的差异类别

- 移动输入：触控事件处理器、粗指针/无 hover 媒体查询、`maxTouchPoints`、软键盘与方向变化。
- Android 接口：联系人、Web NFC、条码、Content Index，以及只在部分版本出现的 Cookie/ModelContext/WebMCP 接口。
- 桌面专用接口隐藏：HID、local fonts、EyeDropper、画中画和部分 AI 接口不能泄漏到 Android Window/Navigator。
- 媒体：编解码支持、MediaSource/MediaRecorder、摄像头约束、设备枚举和 AudioContext 参数必须按移动平台选择。
- Performance：原型成员、Observer entry type 和各 Entry 的 `toJSON()` 字段都存在版本增量，不能只改顶层键。
- Realm：顶层 Window、iframe Window、Dedicated Worker 与 WorkerNavigator 分别应用同一平台/版本基线，用户属性不能跨 Realm 泄漏。
- 安全上下文：联系人、NFC、媒体设备等行为以 HTTPS 证据为准；跨源 iframe 仍需执行同源访问限制。

## 同名 API 的 PC / Android 行为差异

以下值来自 Chromium 151 的 PC HTTPS 对照与 Pixel 4 Android 11 HTTPS 快照。它们说明同一个属性存在于两端，并不代表返回值也应相同。

| 差异面 | PC 证据示例 | Pixel 4 证据 | 实现归属 |
|---|---|---|---|
| `navigator.platform` | `Win32` | `Linux armv81` | 平台默认；显式 Navigator profile 可覆盖 |
| UA-CH | desktop，`architecture=x86`、`bitness=64` | mobile，`architecture/bitness=""`、model=`Pixel 4`、formFactors=`Mobile` | UA + 设备 profile 联合生成 |
| PDF 插件 | `pdfViewerEnabled=true`，内置 PDF plugin/MIME | `false`，plugin/MIME 列表为空 | 平台默认/Plugin profile |
| 输入方式 | fine pointer、hover、通常无触摸 | coarse pointer、no hover、5 点触摸 | Navigator + MediaPreferences profile |
| Screen/viewport | 示例 `1280×720`、DPR 1 | Screen `393×830`、inner `392×727`、VisualViewport `392.727…×726.909…`、DPR 2.75 | Screen、Window、VisualViewport 三组独立字段；隐藏 iframe 可为另一生命周期状态 |
| WebGL 2 | RTX 示例：texture 16384、viewport 32767、S3TC/BPTC | Adreno 640：texture 4096、viewport 16384、ASTC/ETC | GPU-linked WebGL profile，禁止跨平台混用扩展与能力值 |
| WebGPU | 由桌面 GPU/驱动/系统决定 | Android 12+、驱动与 blocklist 共同决定；Android 11 Pixel 4 不标记可用 | 设备目录的可用条件 + WebGPU profile |
| WebAudio | 48 kHz、baseLatency 0.01 | 48 kHz、baseLatency 0.0026666666666666666 | WebAudio profile |
| 媒体编解码 | PC 样本支持 AAC/H.264/HEVC | 当前 Chromium Android snapshot 对这些返回不支持，但支持 Opus/VP8/VP9/AV1 的对应组合 | Media profile，不能只按容器推断 codec |
| NetworkInformation | PC prototype 仅有 `onchange/effectiveType/rtt/downlink/saveData` | 额外有 `type/downlinkMax/ontypechange`；当前为 wifi，`downlinkMax=Infinity` | 平台表决定 legacy member 存在性；Network profile 决定已存在属性的值 |
| Permissions | 桌面权限/功能开关按平台分支 | 未触发权限时：camera/microphone/geolocation/notifications 为 prompt；sensor permission granted；speaker-selection unsupported；top-level-storage-access invalid-origin | Permissions profile；权限变更属于页面生命周期 |
| 设备与传感器 | HID/local fonts/SharedWorker 等桌面接口可能存在 | NFC、Contacts、Barcode、方向/触摸接口；通用 motion/orientation sensor 可用 | 平台+版本表决定存在性，Sensors/Permissions 决定值和拒绝分支 |
| Virtual Keyboard / posture | 通常没有移动键盘状态 | `boundingRect=0,0,0,0`、`overlaysContent=false`；普通手机 posture=`continuous` | 结构已按版本表；显示键盘/折叠状态属于运行事件，不能从 UA 固定 |
| Speech voices | OS 与安装语音包相关 | 本次干净 Chromium snapshot 返回空数组 | Speech profile；国家语言只筛选候选，不把一次空数组推广为所有 Android |
| Performance | entry 结构随版本变化；memory/now 为运行状态 | 151 entry type 列表同版本化；本次 heap limit 1530000000，连续 now 读取因精度量化相同 | 版本表 + Timing/Memory/Performance profile，禁止从屏幕反推 |

### 三层配置规则

1. 平台/版本表决定属性是否存在、顺序、描述符、原型、静态对象和 Symbol 键。顶层 Window、iframe 与 Dedicated Worker 分 Realm 应用。
2. 设备 profile 决定相互关联的硬件值：Screen/DPR/viewport 候选、CPU、内存、触摸点、GPU renderer、WebGL/WebGPU 能力、媒体编解码、字体及语音候选。
3. 页面生命周期决定可变状态：焦点/可见性、user activation、权限、媒体授权后的 label/deviceId、软键盘矩形、姿态/方向事件、网络质量、Performance timeline 与时钟。

任何一层都不能通过字符串替换另一层。例如把 UA 改成 Android 不能继续保留 Windows PDF plugins、D3D renderer 或 fine pointer；反过来也不能把 Pixel 4 的 `392×727` 视口硬编码到所有 Android。

## 当前修复状态

- PC 与 Android 140–151：Window 自有键、完整描述符、iframe 隔离、Navigator 顺序、所有可见构造器 prototype/static/global object 名称以及 `Reflect.ownKeys` 哈希已严格回归。
- PC 与 Android 140–151 Dedicated Worker：全局键、WorkerNavigator、构造器/对象/Symbol 哈希已严格回归。
- Android 移动 API：触摸/方向 handlers、input capture、Contacts、Web NFC、Barcode、Content Index、ModelContext/WebMCP 的版本门槛和 WebIDL 形态已实现。
- Android 默认值：UA-CH、PDF plugins、coarse/no-hover、AudioContext、网络 legacy 字段、媒体支持、权限默认和 Pixel 4 WebGL 能力基线已进入 typed profile/运行时。
- 取证注意：直接读取 PC 上不存在的 `navigator.connection.type` 得到 `undefined`，某些 JSON/WebDriver 传输会把它归一成 `null`。是否存在必须用 prototype descriptor/`in`/`Reflect.ownKeys` 判断，不能从传输后的 `null` 推断属性存在。
- 仍需更多设备证据：Samsung/MediaTek/PowerVR 的完整 WebGL 数值；各厂商 Android WebGPU 实际 adapter；Google Chrome/Edge 正式包的 voice inventory；软键盘显示过程；折叠屏 posture/viewport 变化；权限由 prompt 变更后的事件序列。这些均保持可配置，不使用占位结果冒充证据。

## Chromium 151 示例：Navigator / Worker 差异

- Navigator Android 独有：`modelContext`、`contacts`
- Navigator PC 独有：`hid`、`registerProtocolHandler`、`unregisterProtocolHandler`
- WorkerNavigator Android 独有：—
- WorkerNavigator PC 独有：`hid`

## Performance 版本节点

- 140–143：`Performance.prototype` 尚无 `interactionCount`。
- 144 起：加入 `interactionCount`。
- 143 起：资源/导航条目的 JSON 开始出现 `contentEncoding`。
- 145 起：条目增加 `confidence`；paint/long-animation-frame 增加 paint/presentation 时间字段。
- 148 起：资源/导航条目的 JSON 增加 `contentType`。
- 151：Entry 增加 `navigationId`，PerformanceObserver 新增 `interaction-contentful-paint` 与 `soft-navigation`。

## 输出与维护规则

- 完整机器可读差异：`build/chromium-android-version-surfaces/pc-android-parity-140-151.json`。
- Pixel 4 Chromium 151 行为证据：`build/chromium-android-version-surfaces/android-151-https-api-behavior.json`。
- Rust 运行时表由 `tools/generate_browser_surface_data.py` 分别生成 PC 与 Android 模块；Android 模块包含平台内不变但与 PC 不同的原型，避免桌面成员泄漏。
- 后续每次修复必须同时更新证据、生成表、回归测试和 `docs/SANDBOX_REPAIR_AUDIT_ZH.md`。
- 当前文档是结构基线，不把 Pixel 4 单机可配置值误当成全体 Android 固定值；屏幕、DPR、WebGL、CPU、内存、语言、网络和媒体设备值仍由移动 profile 提供。
