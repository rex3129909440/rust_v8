# Edge 沙箱修复审计日志

本文档是沙箱缺陷修复的永久记录。自 2026-08-08 起，任何影响 Web API 行为、对象形态、原型关系、页面解析、布局、时钟、媒体、指纹配置、Worker、trace 或 Python 包装层的修复，都必须在本文档中追加一条记录。

## 强制记录规则

每次修复必须先记录证据，再记录实现。至少包含以下内容：

1. 日期、源码基线、目标平台、浏览器证据版本和测试二进制。
2. 复现输入、运行参数、profile 标识、页面 URL、HTML 初始化方式、超时和随机种子。含业务信息的值只保存在本地，文档使用脱敏描述。
3. 期望结果、修复前结果、差异路径以及原生 trace 的 API path/record ID。
4. 缺陷分类：API 未实现、返回值错误、异常语义错误、Web IDL 转换错误、对象关系错误、布局错误、时钟错误、profile 未传播、trace 错误或打包错误。
5. 根因、修改文件、修改方式和为什么符合 Edge 证据。
6. 新增的最小回归测试、黑盒回归结果、Worker 释放结果和构建结果。
7. 状态必须使用“已修复并验证”“已定位未修复”“证据不足”“非缺陷”之一。没有证据不得标记完成。
8. 如果生成了 wheel，记录 wheel 路径、平台标签、包版本、原生库 SHA-256、安装命令和安装后加载路径。

禁止仅在对话中说明修复结果。代码发生变化但本文档没有对应条目时，该修复视为没有完成交付记录。

## 黑盒证据边界

- `ips.js` 等输入只作为不透明脚本执行，不读取、不搜索、不格式化、不解释其源码。
- 不分析业务协议、请求签名、请求体算法、站点规则或安全检测逻辑。
- 只观察沙箱公开行为：typed stdout、原生 trace、API 最小探针、DOM 结果、异常、网络回放结果和 Worker 生命周期。
- `success.json`、`fail.json`、原始 trace、业务脚本和用户设备原始采集文件只保存在本机，不提交到 GitHub，不打包进 wheel。
- 浏览器样本与本地样本版本不一致时，差异只能作为候选证据；必须再用明确的 Edge 行为证据或最小探针确认，不能直接把所有差异写成常量。

## `success.json`、`fail.json` 和 `null` 的含义

### 样本如何生成

本轮比较对象均为长度为 436 的聚合数组：

- `user-test/success.json`：用户提供的真实 Chromium/Edge 148 成功样本。
- `user-test/fail.json`：当前沙箱执行同一份不透明输入后，从 `TextEncoder.prototype.encode` 的 typed stdout 中提取的本地样本。
- `user-test/native-trace.log`：生成本地样本的同一次运行导出的原生 API trace。

本地生成流程如下：

1. `build/capture_parser_page_blackbox.py` 通过页面 HTML 和 `network_replay` 加载不透明脚本，不读取脚本内容。
2. 初始化 hook 使用沙箱暴露的 V8 保护接口包装 `TextEncoder.prototype.encode`，把调用参数和返回的 `Uint8Array` 写入 typed stdout。
3. 开启原生 trace，运行结束后一次性导出到 `user-test/native-trace.log`，避免在终端展开大量日志。
4. 捕获器只对 stdout 中的 JSON 字符串做结构解析，收集长度为 436 的数组；如果存在多个候选，则按元素类型与参考样本的匹配数量选择最接近者。
5. 选中的数组写入 `user-test/fail.json`，随后按相同数组索引与 `success.json` 比较。

### 为什么会看到 `null`

`fail.json` 或 `success.json` 中的 `null` 不能单独证明某个 API 直接返回了 JavaScript `null`。数组经过 `JSON.stringify` 后，下列值都会丢失为 `null`：

- JavaScript `null`；
- 数组元素中的 `undefined`；
- 数组空位；
- `NaN`、`Infinity` 和 `-Infinity`。

因此必须结合原生 trace 或最小 API 探针判断来源。原生 trace 中明确记录为 `result=null` 才表示该次 V8 API 返回的是实际 null；`result=undefined`、`threw=true` 和缺少 trace 记录具有不同含义。

### 本轮 null 差异清单

| 数组路径 | 浏览器样本 | 修复前本地 | 当前本地 | 结论 |
| --- | ---: | ---: | ---: | --- |
| `[111][0]` | `0` | `null` | `0` | 已修复并验证。trace 将该值定位到 `Element.animate()` 创建的 Animation 初始时间/进度链。沙箱以前让运行中的动画保持 `currentTime=None`，导致 `overallProgress` 和 computed timing 序列化为 null。 |
| `[401][0]` | `2699` | `null` | `null` | 已精确定位到 `performance.getEntriesByType("resource")` 返回项的 `PerformanceEntry.name`。本地页面/回放把脚本 URL 脱敏为 `...//xxx`，该输入的派生值非有限，JSON 显示为 null；真实路径和查询参数形态的单属性哨兵使该槽同步变为有限数值。沙箱已按 replay URL 传播，不硬编码 `2699`；调用方应传入与 `<script src>` 一致的完整 URL。 |
| `[217][0]` | `null` | `0` | `0` | Edge 150 重新采集结果为 `0`，当前沙箱与目标 Edge 一致，判定为非缺陷；旧样本不能覆盖 Edge 150 证据。 |

当前 trace 中可确认的实际 null 包括：

- 若干 `Object.getPrototypeOf(...)` 到达原型链末端；这是正常原型语义。
- 尚未连接到文档的 iframe 的 `contentWindow`；当前探针场景返回 null。
- `attachShadow({mode: "closed"})` 后读取宿主的 `shadowRoot`；应返回 null。

这些 trace null 与 `[401][0]` 尚无唯一关联，不能混为同一个缺陷。

## 2026-08-08：436 项黑盒差异与 Edge 行为修复

### 基线与证据

- Git 基线：`864f7f4`，分支 `main`，修复尚在工作树中。
- 本地原生库：`target/debug/edge_sandbox.dll`，2026-08-08 17:09:14 构建。
- 本地样本：`user-test/fail.json`，2026-08-08 17:09:27 生成，共 436 项。
- 原生 trace：`user-test/native-trace.log`，同一轮执行产生，共 5139 条记录。
- 浏览器参考：`user-test/success.json`，用户提供的 148 成功样本，共 436 项。
- 验证：Rust library 测试 221 项通过、0 项失败；下表列出的每项均增加了定向回归测试。索引 401 仍为“证据不足”，不包含在完成项中。

### 已修复并验证

| 范围 | 修复前缺陷及可观察结果 | 根因 | 修复实现 | 回归测试 |
| --- | --- | --- | --- | --- |
| Animation 初始状态 | 聚合索引 `[111][0]` 为 null；新建并自动播放的 Animation 的 `currentTime`、`overallProgress` 和 computed timing 没有形成 Edge 的初始 `0` 链。 | `play()` 只修改 `playState`，未在具有 effect 和 timeline 时初始化 current time；computed timing 也没有读取所属 Animation 的 local time。 | `src/web/animation.rs` 在播放初始化时设置 `currentTime=0`，按 effect timing 计算 `overallProgress`；`src/web/animation_effect.rs` 计算 `localTime/progress/currentIteration`；默认 timeline 使用当前 document 的同一对象。 | `p0_tests::element_animate_starts_at_zero_overall_progress` |
| DocumentTimeline | `document.timeline.currentTime` 没有随 realm 时钟推进，`originTime` 和动画默认 timeline 的关系不完整。 | DocumentTimeline 被当成静态 optional number 保存。 | `src/web/animation_timeline.rs`、`document_timeline.rs`、`document_timeline_property.rs` 将 document origin 与当前 realm 的 performance clock 关联，并复用 document.timeline。 | `p0_tests::document_timeline_tracks_the_realm_clock_and_origin_time` |
| Web IDL sequence | InputEvent/Blob 会读取 `length` 和数字索引，缺少首先访问 `Symbol.iterator` 的行为，非可迭代对象错误语义也不正确。 | 把 `sequence<T>` 错当成 array-like。 | `src/webidl.rs` 增加迭代协议转换；`src/web/input_event.rs` 与 `src/web/blob.rs` 使用 `@@iterator`、`next()`、`done/value` 路径，并保留异常。 | `p0_tests::input_event_uses_webidl_dictionary_and_sequence_conversion_order`；`p0_tests::blob_uses_webidl_sequence_conversion_before_blob_property_bag` |
| RequestInit / fetch | 可观察 getter 顺序缺少 `adAuctionHeaders`、`attributionReporting` 及 presence check；无效 URL 会在字典转换前过早失败；getter 可能被重复读取。 | RequestInit 没有按 Web IDL 字典成员字典序做一次性快照。 | `src/web/request.rs` 对 20 个成员按固定 Web IDL 顺序单次读取，并对 `attributionReporting` 做可观察 has；`src/web/fetch_global.rs` 在 URL 拒绝前完成相同转换。 | `p0_tests::request_init_is_snapshotted_before_url_validation_in_webidl_order`；`p0_tests::fetch_init_is_snapshotted_before_url_validation_in_webidl_order` |
| HTML 外部脚本和 replay | parser-inserted `<script src>` 没有稳定执行 replay 内容，`document.currentScript`、完整 body 节点数和异常堆栈 source URL 不完整；URL fragment/默认端口/host 大小写会导致 replay 匹配失败。 | replay 使用原始字符串精确比较；classic script 编译未设置 ScriptOrigin；页面完成阶段缺少统一收尾。 | `src/network_replay.rs` 使用 HTTP(S) 规范化且去 fragment 的网络 key；`src/web/html_script_element.rs` 以实际脚本 URL 编译并设置 source origin；parser 执行后保留 currentScript 和完整 DOM。 | `page_init_tests::parser_inserted_external_script_uses_network_replay_and_sees_complete_body`；network replay 单元测试 |
| BODY/隐藏 iframe 布局 | 零尺寸 iframe 的 `innerWidth/innerHeight` 为 0 时，body 中的 inline iframe 不产生父行盒，`document.body.clientHeight` 错误为 0。 | scroll/client metrics 只累计子元素 box，没有计算文本或 replaced inline element 所形成的 line box。 | `src/web/element_layout.rs` 识别文本和 inline/replaced 元素，按继承 font-size 计算 normal line box；`display:none` 仍为 0。 | `fingerprint_full_tests::zero_sized_inline_iframe_still_creates_the_parent_line_box`，期望 `18|0|1|0` |
| matchMedia viewport | 隐藏窗口 `inner*=0` 时，`width` 搜索错误回退到 `screen.width`，出现约 `1920`；0 轴 aspect-ratio/orientation 语义错误。 | viewport feature 和 device feature 共用了 screen fallback；ratio 拒绝 0 分母/0 轴。 | `src/web/media_query_list.rs` 使 width/height/aspect-ratio 只读 viewport，device-* 独立读 screen，并实现 0/1、1/0 和 orientation 语义。 | `fingerprint_full_tests::match_media_width_search_cannot_fall_through_to_screen_width`，期望 `0.0|1920.0`；zero-axis ratio 单元测试 |
| matchMedia overflow | `(overflow-inline: none)` 错误为 true，Edge 桌面滚动 viewport 应为 `scroll`。 | 离散媒体特征常量错误。 | `src/web/media_query_list.rs` 将 `overflow-inline` 与 `overflow-block` 对齐为 `scroll`。 | `fingerprint_full_tests::match_media_overflow_axes_match_edge_scroll_viewport` |
| Performance timeline | `performance.getEntries()` 缺少完整 navigation/visibility/resource/paint 顺序；navigation load 字段未在页面结束时完成，`duration` 不等于 `loadEventEnd`；`confidence` 形态错误。 | 页面解析结束没有 timeline finalize；NavigationTiming 使用响应时长作为 entry duration；无浏览器进程数据时错误构造了 confidence 对象。 | `src/web/performance.rs`、`performance_entry.rs`、`performance_navigation_timing.rs`、`performance_resource_timing.rs` 在页面及 iframe load 后完成 navigation，添加初始 visibility 和有内容时的两项 paint；无证据的 confidence 返回 null。 | `p0_tests::page_load_populates_navigation_visibility_resource_and_paint_entries_in_edge_order` |
| Navigator 属性顺序 | `Navigator.prototype` 中 `windowControlsOverlay` 相对 `hardwareConcurrency` 的顺序偏离 Edge，进而影响 own-property name 聚合。 | 安装顺序和 Edge 150 证据不一致。 | 恢复 Blink 顺序：`webkitPersistentStorage, windowControlsOverlay, hardwareConcurrency, ...`。 | `navigator_fingerprint_tests::navigator_prototype_keeps_window_controls_overlay_in_blink_order` |
| Web Audio | OfflineAudio 的 10 kHz triangle + DynamicsCompressor 聚合值约为 `1.3005...`，浏览器样本为 `124.04344611517445`。 | oscillator 使用理想数学波形，compressor 是逐样本简化模型，缺少 Blink 的 band-limited 4096 表、look-ahead、32-frame envelope 和 adaptive release。 | `src/web/oscillator_node.rs` 实现 band-limited square/saw/triangle 表插值；`src/web/audio_render.rs` 实现有状态 compressor kernel。 | `p0_tests::offline_audio_triangle_compressor_matches_edge_rendering_kernel`；`p0_tests::oscillator_triangle_uses_the_edge_band_limited_waveform` |
| media `canPlayType` | 仅凭 container 推断未知 codec 可播放，`video/mp4; codecs=bogus` 等返回值错误。 | 使用宽松 media type container 匹配。 | `src/fingerprint_environment.rs` 明确 probably/maybe 能力集；`src/web/html_media_element_can_play_type.rs` 使用完整 capability 匹配。 | `p0_tests::media_can_play_type_does_not_infer_unknown_codecs_from_a_container` |
| `Document.createEvent` | 桌面环境创建不支持的 `TouchEvent` 时异常 message 与 Edge 不一致。 | 通用 NotSupportedError 文本缺少 API 和输入类型。 | `src/web/document_create_event.rs` 返回 Edge 形式的 `NotSupportedError` 名称、code 和 message；不影响 `new TouchEvent()`。 | `p0_tests::desktop_document_create_event_rejects_touch_event` |
| WebGL 参数 35377 | `MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS` 使用 `212992`，Edge 证据为 `212988`。 | 默认 WebGL2 静态限制值偏差 4。 | `src/fingerprint_surface.rs` 修正为 `212988`；仍通过 profile 分支保持平台能力数据的独立传播。 | `fingerprint_full_tests::webgl_static_limits_match_the_captured_edge_150_parameter_evidence` |
| iframe 页面结束 | iframe `srcdoc`/navigation 执行脚本后没有触发与主页面一致的 performance finalize。 | iframe navigation 只运行 parser 脚本和 microtask checkpoint。 | `src/web/html_i_frame_element.rs` 在 iframe 页面脚本和微任务结束后调用 performance finalize，再派发 load。 | Performance timeline 与 iframe 回归覆盖 |

### 复核后判定为非缺陷

| 项目 | 当前证据 | 后续要求 |
| --- | --- | --- |
| 聚合索引 `[401][0]` | 对象身份差分确认最终 `[401]` 等于内部 `11[246]`；trace `3783`–`3790` 将其限定到资源性能项的 `PerformanceEntry.name`。本地脚本 URL 为 `...//xxx` 时结果为 null；仅把 `name` 换成带真实路径和查询参数形态的 URL 后，两处同时变为 `[2938175]`。全程没有读取黑盒源码。 | 来源已定位。Rust 保持按 `NetworkReplayEntry.url` 生成 name；调用方修正 `<script src>` 与 replay URL，禁止硬编码 `2699` 或诊断值 `2938175`。 |
| 聚合索引 `[217][0]` | 旧成功样本为 null，本地为 0；重新采集的 Edge 150 为 0。 | 保留当前 0，按 Edge 150 证据关闭。 |
| 跨版本差异 | `success.json` 是 148 样本，运行目标为 Edge 150。部分属性顺序和新 API 会随版本变化。 | Edge 150 实机证据优先；148 样本只用于发现候选，不能覆盖明确的 150 证据。 |

复核产物为本地证据，不进入 wheel：

- `build/edge-150-blackbox-textencoder-rerun.json`：本机 Microsoft Edge `150.0.4078.65`，索引 401 为 null，索引 217 为 0。
- `build/edge-150-blackbox-textencoder.json`：此前独立 Edge 150 采集，索引 401 为 null。
- `build/chromium-148-blackbox-textencoder.json`：Chrome `148.0.7778.217` 采集，索引 401 为 null。
- `build/diagnostic-freeze-identity.trace.log`：只通过运行时对象身份追踪最终槽位的内部来源，不读取或解释黑盒源码。

### 非缺陷和被排除差异

- screen、GPU、CPU、内存、DPR、语言、时区等已可配置字段，在不同 profile 下出现不同值不计为 API 缺陷；只检查配置是否被正确消费和组合。
- 时间、随机数、UUID、资源时序和运行耗时不要求逐字等于单次浏览器样本，只检查类型、范围、顺序约束和同一时钟域关系。
- Edge 148 与 Edge 150 的新增 Window 属性差异不通过删除 Edge 150 属性来对齐旧样本。
- trace 中原型链终点、closed shadow root、未连接 iframe 等符合语义的 null 不修成数值或占位对象。

### 构建与安装状态

- release 构建：`cargo build --release --lib`，2026-08-08 完成；优化构建耗时 6 分 32 秒。
- 本地 wheel：`build/local-wheel-repair-audit-20260808-1749/rexiaohe_sandbox-2.1.1-py3-none-win_amd64.whl`。
- wheel SHA-256：`D9198EFDD8D734E126D28EC38DF6379BDC0C45A4F651BFE9AC831DC971FC9DC4`。
- release DLL、打包暂存 DLL、wheel 内 DLL 和安装后 DLL 的 SHA-256 均为 `71C22BD0703A1D6BB14C97BC011E805D1CA7EC01C06590084BC3EE662C22287E`。
- 安装命令：`py -3.14 -m pip install --force-reinstall --no-deps <本地 wheel>`；同版本 `2.1.1` 已被本地新版覆盖。
- 安装包路径：`C:/Users/EDY/AppData/Local/Programs/Python/Python314/Lib/site-packages/edge_sandbox/__init__.py`。
- 安装 DLL 路径：`C:/Users/EDY/AppData/Local/Programs/Python/Python314/Lib/site-packages/edge_sandbox/_native/edge_sandbox.dll`。
- 安装后最小探针：matchMedia overflow 为 `false|true|false|true`；Animation 的 `currentTime|overallProgress|localTime|progress|currentIteration` 为 `0|0|0|0|0`；RequestInit 可观察前缀为 `g:adAuctionHeaders,g:attributionReporting,h:attributionReporting,g:body`。

## 后续条目模板

复制以下模板追加到文件末尾：

```markdown
## YYYY-MM-DD：修复标题

### 证据元数据

- Git 基线：
- 平台与浏览器版本：
- profile/seed：
- 测试原生库及 SHA-256：
- 本地证据文件：

### 差异与根因

| 证据路径/trace ID | 期望 | 修复前 | 根因 | 状态 |
| --- | --- | --- | --- | --- |

### 实现

- 修改文件：
- 行为变化：
- profile 传播变化：
- 对象形态/异常/性能影响：

### 验证

- 最小回归测试：
- 黑盒回归：
- Worker/内存：
- 构建与 wheel：
- 未解决项：
```

## 2026-08-08：聚合索引 `[401][0]` 的精确 trace 来源

### 结论

状态：**已定位；不是 Rust API 直接返回 `null`，也不是需要硬编码的指纹值。**

`[401][0]` 来自 `performance.getEntriesByType("resource")` 返回的脚本资源记录，具体输入是该记录继承自 `PerformanceEntry` 的 `name`。本地黑盒运行把外部脚本配置成了脱敏占位地址 `...//xxx`；消费该资源地址后得到非有限数值，聚合数组经 `JSON.stringify` 后显示为 `null`。

沙箱的 `PerformanceResourceTiming.name` 已按 `network_replay` 中实际配置的 `url` 生成。这里不能把旧样本的 `2699` 写死到 Rust，也不能把 `null` 修成任意占位值；正确做法是让页面 `<script src>` 与 `NetworkReplayEntry.url` 使用同一条真实、完整的脚本 URL（包括路径和查询参数）。

### 仅由 trace 得到的证据链

1. 不读取、不搜索、不解释黑盒 JavaScript。通过运行时对象身份比较，最终 436 项数组的 `[401]` 与第 12 次内部冻结数组 `11[246]` 是同一个结果对象。
2. 比较第 11 次与第 12 次冻结数组，只新增了 22 个结果对象。目标 `11[246]` 位于资源性能项结果之后、iframe `navigator.hardwareConcurrency` 结果之前。
3. 基线原生 trace 的精确调用窗为：
   - `3783`：测试开始的 `performance.now()`；
   - `3784`–`3787`：取得 `performance` 并调用 `getEntriesByType("resource")`；
   - `3788`–`3789`：读取 `PerformanceEntry.name`，本地结果为脱敏占位资源 URL `...//xxx`；
   - `3790`：测试结束的 `performance.now()`；
   - `3791`–`3797`：下一项 iframe `navigator.hardwareConcurrency`，所以目标来源边界没有歧义。
4. 排除两个容易混淆的原生 `null`：诊断 getter 已实际命中并替换断开 iframe 的 `contentWindow` 和 closed shadow host 的 `shadowRoot`，但 `11[246]` 与最终 `[401]` 均保持 `[null]`。
5. 排除窗口几何：分别替换 `screenX/screenY/screenLeft/screenTop/inner*/outer*/devicePixelRatio` 后，目标仍为 `[null]`。
6. 单属性验证：只替换 `PerformanceEntry.prototype.name`，并返回带真实脚本路径和查询参数形态的 URL 后，内部 `11[246]` 和最终 `[401]` 同时由 `[null]` 变为 `[2938175]`。这证明 `name` 是该槽位的输入来源；`2938175` 仅是诊断哨兵运行的派生结果，不进入产品实现。

### 本地证据文件

- `build/diagnostic-freeze-checkpoint.trace.log`：基线 trace 与记录号。
- `build/diagnostic-freeze-identity.trace.log`：最终槽位到内部结果对象的身份映射。
- `build/diagnostic-batch-delta.trace.log`、`build/diagnostic-batch-delta.stdout.log`：第 11/12 批次的 22 项增量映射。
- `build/diagnostic-native-null-elimination.trace.log`：`contentWindow`/`shadowRoot` 排除实验。
- `build/diagnostic-window-geometry-source.trace.log`：窗口几何排除实验。
- `build/diagnostic-performance-name-shaped-source.trace.log`、`build/diagnostic-performance-name-shaped-source.stdout.log`：`PerformanceEntry.name` 单属性确认实验。

这些文件是本地诊断产物，不进入 wheel，也不上传业务样本。

## 2026-08-08：随机 profile 资源地址与 Performance timeline 回填

### 状态

**已实现并验证。** 本次实现承接 `[401][0]` 的精确来源，不硬编码旧样本值，也不在 Rust 中生成业务占位字符串。

### 根因

- 随机 profile 以前只包含 Navigator、screen、GPU、CPU、内存、字体等指纹面，没有与每次执行绑定的脚本资源上下文。
- `evaluate(source_url=...)` 以前只把 URL 写入 V8 `ScriptOrigin`，供错误堆栈使用；它没有在脚本执行前创建 `PerformanceResourceTiming`。
- 一次性 Worker 的启动页因此仍只暴露固定 `...//xxx` replay 资源，即使任务已经传入独立 `source_url`。

### 实现

- `demo/get_random_fp.py`
  - 新增 `ResourceLoadProfile` 并挂到 `RandomFingerprint.resource_load`。
  - 使用同一个 profile RNG 生成 86/43 字符的 URL-safe 不透明标识；同 seed 可复现，不同 seed 可变化。
  - `script_url(page_url, x_kpsdk_v)` 强制绝对 HTTPS 页面，继承 origin 和页面目录，并关联同一版本参数。
- `demo/wizzair.py`
  - 每个业务任务从自己的 `fingerprint.resource_load` 生成沙箱脚本 URL，并作为该次 `pool.submit(..., source_url=...)` 的输入。
  - 不改变固定 Worker 池、一次性 Worker、任务结束销毁和预热替补结构。
- `examples/run_complete_iframe_hook.py`
  - `parser_script_body` 为空时不再插入 `<script src="xxx">`，也不再创建空的占位 replay；确实传入 parser body 时保持原有 HTML/replay 加载路径。
  - 因此业务 Worker 在任务脚本执行期间只有本次 profile 关联的资源项，不会同时残留 `xxx`。
- `src/runtime.rs`、`src/web/performance_resource_timing.rs`
  - `evaluate_with_source_url` 在编译/执行脚本前创建脚本 resource entry。
  - 资源项使用传入 URL、源代码字节数、`initiatorType="script"`、HTTP 200 和 `text/javascript`；随后仍用同一 URL 创建 V8 `ScriptOrigin`。
- `demo/__init__.py`
  - 将 `ResourceLoadProfile` 纳入 wheel 的延迟导出集合。

### 验证

- Rust：`cargo test --lib`，222 项通过，0 项失败。
- Python profile/catalog：31 项通过，0 项失败；其中空 parser body/有 parser body 两条路径均有断言。
- 最小原生 DLL 探针：seed `803431` 生成的完整脚本 URL在脚本执行期间由 `performance.getEntriesByType("resource").at(-1).name` 原样返回；其余字段为 `resource|script|200|text/javascript`。
- 两 Worker 一次性并发探针：两个不同 seed 生成两个不同 URL，两个任务都在各自 Worker 中返回自己的 URL，结果为 `2 True`；任务 profile 和资源上下文没有串线。
- 使用业务默认 runtime options 的单 Worker 探针：脚本执行期间 resource 数量为 `1`，唯一 `name` 是该 seed 生成的完整 URL，不再包含 `xxx`。
- 本次仅生成本地 Debug DLL 进行验证，未构建 wheel、未安装、未上传或发布。
## 2026-08-08：User Timing 与 Performance Timeline 语义修复

### 边界与证据

- 本次只检查公开 Web API 行为，不读取或分析任何业务 JavaScript。
- 规范依据：W3C User Timing（`https://www.w3.org/TR/user-timing/`）与 W3C Performance Timeline（`https://www.w3.org/TR/performance-timeline/`）。
- 浏览器证据：本机 Microsoft Edge `150.0.4078.65`，以独立 HTML 探针验证异常名称/DOMException code、dictionary getter 顺序、结构化克隆和时间线顺序。
- 本地探针：`build/edge-user-timing-probe.html`、`build/run-user-timing-dll-probe.py`。二者仅为本地诊断文件，不进入 wheel。

### 修复前缺陷与根因

| 项目 | 修复前行为 | Edge / 规范行为 | 根因 |
| --- | --- | --- | --- |
| 不存在的 mark | `measure()` 回退到 0 或当前时间 | 抛 `SyntaxError` DOMException，code 12 | `measure_point()` 使用 fallback 吞掉失败 |
| options 合法性 | 接受 duration-only、start+duration+end | 均抛 `TypeError` | 未实现 PerformanceMeasureOptions 组合约束 |
| 负数与非有限数 | 接受负 timestamp 与 `NaN` | 抛 `TypeError` | 数字转换失败/非有限值被 fallback 吞掉 |
| PerformanceTiming 保留名 | `navigationStart` 可被用作 mark | Window 中抛 `SyntaxError` | 未区分 Window/Worker realm，缺少旧 Timing 名称算法 |
| `detail` | 函数被浅拷贝为 `{}`；循环图可能递归溢出 | 函数抛 `DataCloneError`；循环引用被保留 | 使用自定义递归浅克隆而非结构化克隆 |
| dictionary getter 顺序 | mark 为 `startTime,detail`；measure 为 `start,end,duration,detail` | mark 为 `detail,startTime`；measure 为 `detail,duration,end,start` | 未按 Web IDL 字典成员名顺序转换 |
| PerformanceMark 默认时间 | 构造器默认 `startTime=0` | 默认使用当前 realm 的 `performance.now()` | 构造器使用固定值 |
| 时间线结果顺序 | `getEntries*()` 按插入顺序 | 按 `startTime` 稳定排序 | 缺少 Performance Timeline filter 的 chronological sort |
| 旧式第三参数 | 空 options 加 `endMark` 时忽略 end mark | 使用该 mark 计算 end time | options 分支遗漏旧式第三参数 |

### 实现

- `src/web/performance.rs`
  - Performance 记录增加 Window/Worker realm 类型，不把 Window 的旧 `PerformanceTiming` 规则错误套给 Worker。
  - 实现 21 个旧 `PerformanceTiming` 保留名称、最近 mark 查找、缺失 mark 的 `SyntaxError`、零值旧 timing 的 `InvalidAccessError`。
  - 完成 `PerformanceMeasureOptions` 校验、有限数与非负 timestamp 校验、允许规范明确允许的负最终 duration。
  - 按 Web IDL 顺序一次性读取 options getter，并在计算完成后结构化克隆 detail。
  - `getEntries()`、`getEntriesByName()`、`getEntriesByType()` 改为按 startTime 稳定排序。
- `src/web/performance_mark.rs`
  - `PerformanceMark` 构造器使用当前 realm 时钟；Window 中拒绝保留 timing 名称。
  - detail 改用既有 structured-clone 引擎，保留对象图与循环引用，拒绝函数等不可克隆值。
- `src/web/performance_timing.rs`
  - 增加内部名称到旧 timing 数值的完整映射，供 User Timing 转换算法复用。
- `src/web/performance_observer_entry_list.rs`
  - 三个 `getEntries*()` 入口同步应用 chronological sort。
- `src/web/performance_global.rs`、`src/web/worker_global_scope.rs`
  - 创建 Performance 时明确传递 Window/Worker realm 类型。

### 验证

- 新增回归：`p0_tests::user_timing_matches_edge_errors_clone_semantics_and_chronological_order`。
- 既有回归：`p0_tests::performance_timeline_observer_queue_and_measure_options_are_functional`，Observer 列表期望顺序同步修正为 chronological order。
- Debug DLL 独立 Python 探针输出与 Edge 证据一致：异常种类/code、循环引用、getter 顺序及 `sort-early,sort-late` 均通过。
- 完整 Rust 测试：`cargo test --lib`，`223 passed; 0 failed`。
- 本次仅构建本地 Debug DLL；未生成 wheel、未安装、未上传 GitHub、未发布 PyPI。

## 2026-08-08：URLSearchParams Web IDL 转换与活迭代器修复

### 边界与证据

- 本次只检查公开的 URL / Web IDL API 行为，不读取或分析业务 JavaScript。
- 规范依据：WHATWG URL Standard（`https://url.spec.whatwg.org/#interface-urlsearchparams`）与 Web IDL Standard（`https://webidl.spec.whatwg.org/#idl-sequence`、`#idl-record`、`#idl-iterable`）。
- 浏览器证据：本机 Microsoft Edge `150.0.4078.65`，独立探针验证 union 分派、sequence/record 转换、异常、prototype descriptor、live iterator 与 `forEach()` 变更可见性。
- 本地证据文件：`build/edge-url-search-params-probe.html`、`build/run-url-search-params-dll-probe.py`；仅用于诊断，不进入 wheel。

### 修复前缺陷与根因

| 项目 | 修复前行为 | Edge / 规范行为 | 根因 |
| --- | --- | --- | --- |
| union 分派 | 只有 Array 被当作 sequence；Map、生成器和自定义 iterable 被当作 record | 对象存在 callable `@@iterator` 时必须优先转换为 sequence | 使用 `is_array()` 代替 Web IDL GetMethod/iterator protocol |
| primitive init | `null`、数字等因不能转 Object 而抛错 | union 的字符串分支，分别得到 `null=`、`12=` 等 | 缺少 union 的 primitive-to-USVString fallback |
| inner sequence | 只读取索引 0/1；一项或三项 pair 没有完整的 iterator/长度校验 | inner sequence 大小必须恰好为 2，否则 `TypeError` | 把 pair 当 array-like，而不是 `sequence<USVString>` |
| record 转换 | 默认 own-property 枚举没有严格执行 enumerable key/符号键转换 | 只读取 enumerable own keys；enumerable Symbol 转 USVString 时抛 `TypeError` | 未执行 Web IDL record conversion |
| iterator 数据 | `entries/keys/values` 返回 Array Iterator 快照 | 返回专用、实时的 URLSearchParams Iterator | 先复制 pairs 到 Array 再调用 `Array.prototype.values()` |
| iterator 形态 | 显示为 `[object Array Iterator]`，prototype/brand 错误 | `[object URLSearchParams Iterator]`，专用 prototype、native `next()` 与正确父链 | 没有实现 Web IDL pair iterator object |
| mutation 可见性 | iterator 和 `forEach()` 不观察 append/delete | append 可在后续迭代读到，删除未访问项会改变后续结果 | 遍历克隆的 `ParamsRecord` 快照 |
| prototype tag | `URLSearchParams.prototype` 缺少 `Symbol.toStringTag` | own-key 顺序为 `... constructor, Symbol.toStringTag, Symbol.iterator` | prototype 安装阶段漏项 |

### 实现

- `src/web/url_search_params.rs`
  - union 分派先保留 string/string-object 分支；其他对象读取 `@@iterator`，callable 时复用通用 Web IDL `sequence_values()`，否则按 record 转换；primitive 回退为 USVString。
  - outer/inner sequence 均使用 iterator protocol，inner 长度必须等于 2。
  - record 使用 OwnOnly + ONLY_ENUMERABLE + ConvertToString，覆盖 symbol key 的转换异常。
  - 新增 realm-local `URLSearchParams Iterator` prototype、`next()`、`Symbol.toStringTag`、正确 `%IteratorPrototype%` 父链和 iterator brand state。
  - iterator 只保存源 params identity 与当前 index，每次 `next()` 都读取当前 live list。
  - `forEach()` 按 live index 逐项读取，不再预先复制整个列表。
  - `URLSearchParams.prototype` 增加只读、不可枚举、可配置的 `Symbol.toStringTag`。

### Edge 对照结果

- Map：`a=2`；嵌套生成器：`x=7`。
- 一项/三项/non-iterable pair：均为 `TypeError`。
- non-enumerable record property 被忽略；enumerable Symbol key 抛 `TypeError`。
- iterator own keys：`next, Symbol(Symbol.toStringTag)`；`next` descriptor 为 `true,true,true,next,0`；tag descriptor 为 `false,false,true`。
- iterator 对 append 的后续结果为 `b:2,c:3`；删除尚未访问的 `b` 后下一次结果为 `{done:true}`。
- `forEach()` 中 append 得到 `a,b,c`，删除尚未访问的 `b` 得到 `a,c`。

### 验证

- 新增回归：`p0_tests::url_search_params_uses_webidl_union_conversion_and_live_pair_iterators`。
- native trace 开启后，`URLSearchParams.entries()` 和 iterator `next()` 均有 call 记录；`next` 保持 `function next() { [native code] }`。
- Debug DLL 独立 Python 探针输出与 Edge 证据一致。
- 完整 Rust 测试：`cargo test --lib`，`224 passed; 0 failed`。
- 本次仅重新构建本地 Debug DLL；未生成 wheel、未安装、未上传 GitHub、未发布 PyPI。

## 2026-08-08：Headers ByteString、校验与活迭代器修复

### 边界与证据

- 本次只检查公开 Fetch `Headers` API，不读取或分析业务 JavaScript。
- 规范依据：WHATWG Fetch Standard（`https://fetch.spec.whatwg.org/#headers-class`、header name/value、sort and combine）与 Web IDL ByteString/sequence/record/iterable 转换规则。
- 浏览器证据：本机 Microsoft Edge `150.0.4078.65`，通过独立页面确认构造 union、ByteString、name/value 校验、Set-Cookie、prototype/iterator descriptor 和迭代期间变更。
- 本地证据：`build/edge-headers-probe.html`、`build/run-headers-dll-probe.py`，仅用于诊断，不进入 wheel。

### 修复前缺陷与根因

| 项目 | 修复前行为 | Edge / 规范行为 | 根因 |
| --- | --- | --- | --- |
| HeadersInit sequence | 只把 Array 当 sequence，Map/生成器被错误处理 | callable `@@iterator` 优先进入 nested sequence 转换 | 使用 `is_array()` 和数值索引代替 Web IDL iterator protocol |
| pair 校验 | 一项/三项/non-iterable pair 被跳过或截成前两项 | 必须抛 `TypeError` | 未将 inner value 转成 `sequence<ByteString>`，也未检查 size=2 |
| record 转换 | 非法属性可能被静默跳过 | 只读取 enumerable own keys；Symbol key 转 ByteString 抛错 | 未完整执行 record conversion |
| primitive init | `null`、数字会生成空 Headers | 二者都不是 HeadersInit 对象，抛 `TypeError` | Object 转换失败被静默忽略 |
| ByteString | emoji 等非 ISO-8859-1 字符可进入 header | name/value 转换阶段抛 `TypeError` | 使用通用 DOMString 转换 |
| name 处理 | 自动 trim header name，把 `" x "` 接受为 `x` | header name 不做 trim，空格使其成为无效 token | 错把 value normalization 套给 name |
| value 校验 | 只 trim SP/HTAB，仍接受 NUL/CR/LF | trim 后必须拒绝 NUL 和 HTTP newline | 缺少 Fetch header value validation |
| 必填参数 | `append()`/`set("x")` 会把缺失值转成 `undefined` | 根据 IDL arity 抛 `TypeError` | 没有 required-argument 前置检查 |
| iterator | 返回 `[object Array Iterator]` 快照 | 专用 `[object Headers Iterator]`，读取 sort-and-combine live list | 把 combined 结果复制进 Array |
| forEach | 遍历调用前快照，回调 append/delete 不可见 | 每一步基于当前 sort-and-combine 结果 | 预先 clone 整个 header list |
| prototype tag | 缺少 `Headers.prototype[Symbol.toStringTag]` | own-key 末尾为 constructor、toStringTag、iterator | 安装阶段漏项 |

### 实现

- `src/web/headers.rs`
  - HeadersInit 对象读取 `@@iterator` 并支持 Map、生成器、自定义 iterable；非 iterable 对象按 enumerable own record 转换。
  - inner pair 通过通用 Web IDL sequence converter 读取并严格要求两项。
  - 增加 ByteString 转换，拒绝 Unicode code point 大于 U+00FF 的输入。
  - header name 不再 trim；value 只 trim SP/HTAB并拒绝 NUL/CR/LF。
  - append/delete/get/has/set/forEach 增加 IDL required-argument 检查。
  - 保留 Set-Cookie 独立值，并在迭代时按 Fetch sort-and-combine 排序、合并其他重复 header。
  - 新增 realm-local Headers Iterator prototype、native `next()`、正确 `%IteratorPrototype%` 父链和 live index 状态。
  - `Headers.prototype` 与 iterator prototype 均补齐只读、不可枚举、可配置的 `Symbol.toStringTag`。

### Edge 对照结果

- Map 输出 `[["a","1"],["b","2"]]`；嵌套生成器输出 `[["x-one","value"]]`。
- spaced name、NUL/LF value、emoji name/value、错误 pair 与缺失参数均为 `TypeError`。
- 重复普通 header 读取为 `2, 3`；`getSetCookie()` 保留 `a=1`、`b=2` 两项。
- iterator object tag、prototype keys、descriptor、父链与 Edge 一致；append `b` 后未完成 iterator 依次返回 `b:2`、`c:3`。
- `forEach()` 在回调中 append 后访问顺序为 `a,b,c`。

### 验证

- 新增回归：`p0_tests::headers_validate_bytestrings_and_expose_live_sorted_pair_iterators`。
- native trace 开启后 `Headers.entries()` 和 iterator `next()` 都有 call 记录；`next` 保持 `[native code]`。
- Debug DLL 独立 Python 探针与 Edge 证据一致。
- 完整 Rust 测试：`cargo test --lib`，`225 passed; 0 failed`。
- 本轮尚未改造 Request/Response/fetch 产生的 Headers guard；guard 是下一次独立审计项，避免把核心 Headers 行为与上下文权限规则混在一次变更中。
- 本次仅重新构建本地 Debug DLL；未生成 wheel、未安装、未上传 GitHub、未发布 PyPI。
## 2026-08-08：随机硬件目录的版本上限与增量扩展审计

### 结论

- `navigator.hardwareConcurrency` 与 `navigator.deviceMemory` 是两个独立字段。前者没有“浏览器最多 8”的规则，当前目录覆盖 1–192 个逻辑处理器，显式配置经 `u32` FFI 原样传入 Window、iframe 和 Worker。
- 目标为 Edge/Chromium 150 时，`deviceMemory` 才使用桌面 2/4/8/16/32 GiB 桶；Android 使用 1/2/4/8 GiB 桶。该版本规则不得反向限制 CPU 逻辑处理器数。
- 所有新增 GPU 和屏幕记录均追加在旧目录之后；未替换旧 ID、旧值或旧权重。

### 公开证据与目录变化

- CPU API 语义：WHATWG HTML `hardwareConcurrency` 与 MDN 均定义为用户代理可用的逻辑处理器数，允许浏览器报告较低值，但没有 8 的标准上限。
- 内存桶：Chrome 147 release notes 明确将桌面集合更新为 2/4/8/16/32，Android 为 1/2/4/8。
- NVIDIA 增量库存：当前 NVIDIA Open GPU Modules README 提供 225 个产品、336 个精确产品/PCI Device ID 对；使用固定的 Edge 150 Dawn `gpu_info.json` 解析架构。155 个产品、256 个完整产品/ID 对可进入普通显示适配器候选池，产生 512 个“显示 Device ID/不显示 Device ID”ANGLE 形态；计算卡、DRIVE、CMP、DGX 等只保留在库存，不进入随机池。
- 屏幕增量：旧 104 条 PC 屏幕记录保持原顺序，追加 20 条带厂商来源的 Surface、6K、Dual-UHD 和 8K 显示状态，共 124 条基础记录。6K/8K 不再与低端、旧款、普通办公或便携硬件任意交叉。
- Windows 工作区：基础屏幕目录不变；组合阶段按 UA-CH `platformVersion=10` 生成 40 CSS px 的 Windows 10 工作区，按 `platformVersion=15` 生成 48 CSS px 的 Windows 11 工作区。

### 代码位置

- `demo/fp/pc_navigator_hardware_catalog.py`：CPU/RAM 行、GPU/硬件兼容规则及 Lovelace/Ada 映射。
- `tools/generate_nvidia_open_gpu_extension_catalog.py`：可重复生成的当前 NVIDIA + 固定 Edge 150 Dawn 证据快照。
- `demo/fp/nvidia_open_gpu_extension_catalog.py`：生成后的独立增量库存。
- `demo/fp/windows_webgl_gpu_catalog.py`：旧候选保持为前缀，追加且去重完整增量候选。
- `demo/fp/pc_screen_extension_catalog.py`、`demo/fp/screen_profile_catalog.py`：独立屏幕增量和 Windows 版本化工作区。
- `demo/get_random_fp.py`：版本化 `deviceMemory` 桶、工作区物化和组合一致性审计。

### 验证

- Python profile/catalog 回归：23 项通过，0 项失败。
- Rust library 回归：227 项通过，0 项失败；包括用户自定义硬件值不受旧桶限制、Window/iframe/Worker 指纹传播和 WebGPU 可用性。
- 10,000 个固定种子 Windows 随机样本：0 个一致性错误；CPU 实际抽样覆盖 2–192；`deviceMemory` 仅出现 2/4/8/16/32；Windows 10 工作区差始终为 40，Windows 11 始终为 48；抽到 1,459 个 GPU profile ID 与 166 个版本化屏幕状态。

### 未夸大的边界

当前目录是大规模、来源可追溯的测试集合，不声称穷尽世界上每个 OEM subsystem ID、驱动字符串或未来型号。NVIDIA 当前公开表已经作为独立增量快照接入；AMD/Intel 继续保留现有 PCI ID Repository + Dawn 目录，后续扩展也必须使用同样的“独立增量、完整记录、目标 Edge 架构可映射”规则，不能猜测 Device ID 或覆盖已有数据。

## 2026-08-08：独立 evaluate 的 Document、NetworkInformation 与用户偏好 profile

### 边界与依据

- 本次只修复公开 Web API 与强类型 profile 的传播，不读取或分析任何业务 JavaScript，也没有执行黑盒业务脚本。
- Network Information 依据 WICG Network Information 的 effective connection type 阈值，以及 Chromium `NetworkStateNotifier` 的可观测量化实现。生成器将用户要求的 RTT 范围限制为 0–600ms，并使用 Chromium 的 50ms 桶；downlink 使用 50Kbit/s（0.05Mbps）桶并限制在 0.05–10.00Mbps。
- 媒体偏好依据 W3C Media Queries Level 5：`prefers-reduced-motion`、`prefers-reduced-transparency`、`prefers-contrast`、`forced-colors`、`prefers-reduced-data`、pointer/hover、color gamut 与 dynamic range 作为 profile 状态输入，而不是运行时常量。
- 设备姿态依据 W3C Device Posture：值域仅为 `continuous`/`folded`；非折叠设备保持 `continuous`，目录中带 `foldable` 标签的 Android 设备才允许抽到 `folded`。
- `UserActivation` 保持状态约束：`isActive=true` 时 `hasBeenActive` 必须为 true；生成器只会产生 `(false,false)`、`(true,false)`、`(true,true)` 三种状态。
- BODY 的 `5` 和 `23` 不作为全局硬编码。它们是调用方可选的 typed profile 输入；未传入时继续使用正常 DOM/布局计算。

### 修复前缺陷与根因

| 项目 | 修复前行为 | 根因 |
| --- | --- | --- |
| NetworkInformation | 随机 profile 只有三组固定观察值，RTT/downlink/saveData 变化量不足 | `_network_profile()` 从三元常量表取值，且没有从 RTT/downlink 推导 effectiveType |
| reduced-data | `saveData` 与媒体查询偏好没有同源约束 | Navigator network 与 MediaPreferences 分开构造 |
| 显示/无障碍偏好 | 多个字段继承 preset 固定值；`prefers-reduced-transparency` 即使配置也始终按 no-preference 判断 | profile 缺字段或 `matchMedia()` 写死 |
| Device posture | Navigator 可读 profile 中的 `folded`，但 `matchMedia('(device-posture: folded)')` 永远为 false | `media_query_list.rs` 固定比较 `continuous` |
| UserActivation | `navigator.userActivation` 构造时固定为 `(false,false)` | Navigator 创建路径没有读取 fingerprint |
| 独立 evaluate 的 BODY | 不加载 page HTML 时 BODY 为空，无法通过 profile 指定 childElementCount/clientHeight | EdgeFingerprint、C ABI 与文档初始化阶段均没有 Document 配置面 |

### 实现

- `src/fingerprint.rs`、`src/fingerprint_environment.rs`：增加 DocumentFingerprint、UserActivation 字段、媒体偏好字段和状态验证；旧序列化 profile 通过 `serde(default)` 保持兼容。
- `src/ffi.rs`、`examples/edge_profile.py`、`examples/run_sandbox.py`：增加独立 typed field ID；profile schema 从 7 升为 8。配置仍走 C ABI 的 string/u32/f64/bool setter，不使用 JSON 字符串。
- `src/web/document_global.rs`：默认 Document 创建并完成可选 HTML 解析后，按目标数量补足真实 `HTMLDivElement`。独立 `evaluate()` 在用户代码第一行运行前即可通过 `children`、选择器、遍历和 `document.all` 观察这些节点。已有 HTML 节点多于目标值时不删除。
- `src/web/element_client_height.rs`：只有 BODY 且存在显式 `body_client_height` 时使用 profile 数值；其他元素与未配置 BODY 继续进入布局计算。
- `src/web/navigator.rs`：创建 `UserActivation` 时读取 profile。
- `src/web/media_query_list.rs`：消费 reduced transparency、video dynamic range 与真实 device posture profile。
- `demo/get_random_fp.py`：RTT/downlink 先生成再推导 effectiveType；saveData 同步到 prefers-reduced-data；显示/无障碍偏好采用独立 seed 流；pointer/hover 按桌面/Android 形态关联；只有可折叠 Android 允许 folded。
- `examples/country_profile.py`：公开 API 增加 `body_child_element_count` 和 `body_client_height` 可选参数并原样传入 typed profile。

### 验证

- `cargo test --lib`：加入全部新回归后 229 项通过、0 项失败。
- `fingerprint_full_tests::standalone_evaluate_materializes_profiled_body_state_before_script_execution`：不提供 page HTML，脚本首行观察到 `5|true|23`，5 个子节点均为真实 DIV。
- `fingerprint_environment_tests::environment_fingerprint_drives_each_exposed_api`：reduced transparency、video dynamic range、folded media query 和 UserActivation 均由 profile 驱动。
- `ffi::tests::new_document_media_and_activation_fields_cross_the_typed_ffi`：新增字段通过独立 u32/f64/bool/string C ABI setter 写入并通过 profile 校验。
- `py -3.11 -m unittest tests.python_random_profile_consistency`：5 项通过、0 项失败。固定 500 个 seed 的 Windows 样本实际覆盖 RTT 0 和 600、186 个 downlink 桶、saveData true/false 与 isActive true/false；500 个 Android 样本同时覆盖 continuous/folded，审计未出现交叉字段冲突。
- 新构建 `target/debug/edge_sandbox.dll` 后通过 Python `EdgeSandbox` 的 self-hosted typed C ABI 调用，独立 evaluate 输出 `5|true|23|500|7.5|3g|false|false|true|false`，确认 BODY、NetworkInformation、reduced-data 与 UserActivation 在实际 DLL 路径一致。
- `git diff --check`：本次修改无空白错误。
- 本次未构建 wheel、未安装、未上传 GitHub、未发布 PyPI。

## 2026-08-09：国家随机 profile 固定默认 BODY 状态

### 要求与实现

- 业务调用链 `wizzair.py -> create_country_profile_details(country_code, user_agent)` 不额外传 Document 参数，因此此前仍会得到未配置的 BODY 状态。
- `demo/get_random_fp.py` 的 `get_random_fp_details()`、`get_random_fp()` 与公开包装 `examples/country_profile.py` 现在默认使用 `body_child_element_count=5`、`body_client_height=23.0`。
- 固定值位于 profile 组装层，底层 DOM API 仍读取 typed C ABI 传入的 DocumentFingerprint；没有把 `childElementCount` 或 `clientHeight` getter 硬编码为常量。
- 调用方显式传入其他数值时仍可覆盖；显式传入 `None` 时不下发对应 Document 字段，恢复正常 DOM/布局计算。

### 验证

- `py -3.11 -m unittest tests.python_random_profile_consistency`：5项通过、0项失败；同时验证默认 `5/23` 与显式覆盖 `3/18`。
- 使用新构建的 `target/debug/edge_sandbox.dll`，调用 `create_country_profile_details("US", seed=803431)` 时不传 Document 参数，独立 `evaluate()` 返回 `5|23`。
- 本次仅修改 Python profile 默认配置、测试和文档；Rust运行时与profile schema没有再次变化，未生成wheel、未安装、未上传或发布。

## 2026-08-09：V8 15.0 heap limit 与完整 memory 快照目录

### 证据与结论

- Blink `MemoryInfo::GetHeapSize` 直接映射 V8 `used_heap_size()`、`total_physical_size()`、`heap_size_limit()`；不是从 `navigator.deviceMemory` 复制，也不是三个独立的硬件常量。
- V8 `Isolate::GetHeapStatistics` 先读取 live object size，再读取 committed physical memory；官方实现要求 `used <= total`。`heap_size_limit` 来自 `Heap::MaxReserved()`。
- V8 15.0.245.2 桌面 64 位 old generation 按物理内存 1:2 增长并在 4 GiB 封顶，young generation 由 semi-space 公式加入；8 GiB 及以上精确上限为 `4395630592`。项目此前对 Windows ≥8 GiB 使用旧值 `4294705152`，与嵌入的 V8 150 不一致。
- Android 采用独立的 1:4 old-generation 比例，8 GiB 以下还使用 8 MiB semi-space 上限；项目此前错误复用桌面映射，导致 2/3/4/6/8/12 GiB Android 的 heap limit 偏大。
- Blink 非精确模式有完整 100 桶，首尾为 `10000000` 和 `3760000000`，更新间隔 20 分钟；精确模式更新窗口为 50ms。站点锁定的 HTTPS renderer 选择精确模式。
- `totalJSHeapSize` 与 `usedJSHeapSize` 随分配、GC 和页面负载变化。不存在可诚实穷举的有限硬件值表，因此不能继续使用修复前的 `12..48 MiB` 随机总量和 `0.58..0.91` 随机比例。

### 实现

- 新增 `demo/fp/v8_memory_profile_catalog.py`：逐项实现 V8 15.0 desktop/Android heap 公式、Blink 100 桶算法、完整实测 snapshot pair 目录和带权选择；没有占位值或独立字段乱序组合。
- `demo/get_random_fp.py`：Windows、macOS、Android 均由同一个证据模块计算 `jsHeapSizeLimit`；`total/used` 只从完整快照行取值；`RandomFingerprint.memory_snapshot_profile_id` 保留选择来源；审计拒绝未知或被拆散的快照。
- 桌面公式覆盖 1/2/3/4/6/≥8 GiB 的 6 个精确上限；当前 Win64 Edge 150 随机池排除不满足系统内存要求的 1 GiB obsolete 行，实际覆盖其余 5 个，显式测试仍可调用完整公式。Android 目录覆盖 2/3/4/6/8/12/≥16 GiB 的 7 个独立精确上限；相应非精确结果由完整 Blink 桶表计算，不把 `3760000000` 错写成所有环境的固定精确上限。
- `src/fingerprint_environment.rs` 的固定 Edge 150 默认值改为内置 V8 150 初始化快照；Windows/Mac Python preset 和完整 iframe demo 改用已有真实浏览器采集对，不再保留零值或无来源比例随机值。
- profile schema 与 C ABI 没有变化：六个 memory 字段仍分别通过 u64 setter 传入，继续允许用户显式覆盖。

### 实测快照来源

| ID | 平台 | `totalJSHeapSize` | `usedJSHeapSize` | 来源 |
| --- | --- | ---: | ---: | --- |
| `edge148_windows_loaded_page_user_sample` | Windows | 98833423 | 62981207 | `user-test/success.json` 用户浏览器样本 |
| `edge150_macos_m5_loaded_page_capture` | macOS | 189287527 | 180511835 | `demo/full-edge-profile-2026-08-07T04-06-07.238Z.json` |
| `v8_15_retained_0_objects` | 跨平台 V8 语义 | 8388608 | 7002608 | 嵌入 V8 15.0.245.2、0 个保留对象 |
| `v8_15_retained_1000_objects` | 跨平台 V8 语义 | 9310208 | 6582720 | 嵌入 V8 15.0.245.2、1000 个保留对象 |
| `v8_15_retained_5000_objects` | 跨平台 V8 语义 | 9834496 | 7149000 | 嵌入 V8 15.0.245.2、5000 个保留对象 |
| `v8_15_retained_10000_objects` | 跨平台 V8 语义 | 11833344 | 7721512 | 嵌入 V8 15.0.245.2、10000 个保留对象 |
| `v8_15_retained_25000_objects` | 跨平台 V8 语义 | 12533760 | 9576920 | 嵌入 V8 15.0.245.2、25000 个保留对象 |
| `v8_15_retained_50000_objects` | 跨平台 V8 语义 | 17592320 | 11866480 | 嵌入 V8 15.0.245.2、50000 个保留对象 |
| `v8_15_retained_100000_objects` | 跨平台 V8 语义 | 23248896 | 17799720 | 嵌入 V8 15.0.245.2、100000 个保留对象 |
| `v8_15_retained_250000_objects` | 跨平台 V8 语义 | 43364352 | 32828400 | 嵌入 V8 15.0.245.2、250000 个保留对象 |
| `v8_15_retained_500000_objects` | 跨平台 V8 语义 | 74006528 | 56718360 | 嵌入 V8 15.0.245.2、500000 个保留对象 |

### 验证

- `runtime::tests::embedded_v8_150_heap_limits_match_the_desktop_profile_catalog` 直接建立 1/2/3/4/6/8 GiB 资源约束 isolate，V8 返回值逐项等于 Python 目录。
- `tests.python_v8_memory_profile_catalog` 校验 Chromium 官方量化示例、完整 100 桶首尾、桌面 6 阈值、Android 7 阈值，以及 900 个跨平台随机 profile 的 heap/snapshot/console 一致性。
- `cargo test --lib`：230 项通过、0 项失败。
- memory 相关 Python 回归连同随机 profile、Windows 和 Android 目录测试：20 项通过、0 项失败。
- 新构建 `target/debug/edge_sandbox.dll` 后经 typed Python C ABI 分别传入 Windows 32 GiB、Mac 16 GiB、Android 8 GiB profile，实际 JavaScript 读数逐字段等于 profile；Android 返回独立上限 `2248146944`，桌面返回 `4395630592`。不传 profile 的固定 Edge 150 默认读数为 `4395630592|8388608|7002608`，`console.memory` 同组一致。
- 本次未生成 wheel、未安装、未上传 GitHub、未发布 PyPI。

## 2026-08-09：Linux 3.2.1 wheel 打包与 PyPI 发布

- 仅使用 `build/edge-sandbox-linux-x64.zip` 中的 CI Linux x86_64 wheel；ZIP SHA-256 为 `E4CB29A58B0456E60F58A84AD50114B44A40977C31053A459808F56E572F157D`。Windows 和两个 macOS ZIP 内仍为旧版 `0.1.2`，本次没有使用或发布。
- 将当前 Python 调用层、国家随机 profile 与 `demo/fp` 下 25 个 profile 模块装入 Linux wheel；未包含测试、demo 业务文件、`ips.js`、`__pycache__` 或 `.pyc`。
- CI `libedge_sandbox.so` SHA-256：`77805E3E5B63194B1341E2AD83EA31EFF9E7567CC9C090D6BE1F09340A619934`。
- wheel：`build/release-3.2.1-linux-20260809/dist/rexiaohe_sandbox-3.2.1-py3-none-manylinux_2_28_x86_64.whl`；大小 `24576586` 字节；SHA-256 `4E6F13F0819E9B45D8954C2D2E77E40E41CE4052686DAB55961A76B7408BD09A`。
- `twine check`、35 个当前源码成员逐文件哈希核对和 ZIP 直接导入随机 profile 冒烟均通过。
- 已发布到 PyPI `rexiaohe-sandbox==3.2.1`；PyPI JSON 返回的文件名、大小和 SHA-256 与本地产物一致。本版本仅提供 `py3-none-manylinux_2_28_x86_64`。

## 2026-08-09：多轮黑盒 trace / TextEncoder 固定值深度审计

### 边界与方法

- `user-test/ips.js` 只作为不透明字节执行；本轮没有打开、搜索或解释其源码。
- 黑盒入口改为真实页面解析与外部脚本 replay 路径：hook 与不透明字节作为 replay 脚本内容加载，随后只执行 `void 0` 排空页面任务。这样 `Document` 子节点、脚本资源 timing 和 frame provenance 与生产 HTML 加载路径一致。
- 每轮只把 native trace、结构化 stdout 和最后一次 JSON-array TextEncoder 输入写入独立文件；分析工具不重复读取大 trace，也不处理业务协议。
- screen、DPR、WebGL/GPU 已经是 typed profile 字段，按要求不计入“仍然硬编码”的缺陷候选。

### 黑盒矩阵

- 第一阶段：US/DE/JP/BR/CN，seed `101,202,303,606,707,808,909`，共 7 轮。
- 离散边界补充：seed `6,10,15,1,16,23,35,150`，覆盖 44.1/48 kHz、两种 V8 heap limit、UserActivation true/false、saveData true/false、light/dark、forced colors、reduced motion/transparency、contrast 和 inverted colors，共 8 轮。
- 合计 15 轮均成功：每轮 15 次 TextEncoder、1 个捕获请求、1 个 `/tl`、无执行异常；单轮原生 trace 约 5,030–5,080 条。
- 15 轮共有 752 个相同“操作 + API 路径 + 参数”调用键；其中 157 个 primitive 返回值在全部运行中出现，124 个 primitive 同时存在于每轮最终 JSON。剔除 screen/GPU/Canvas 后，只有 24 行属于 profile 相关表面。
- 接入随机资源 URL 后又以当前最终代码重跑 10 轮（US 6 轮，DE/JP/BR/CN 各 1 轮）：10/10 均为 15 次 TextEncoder、1 个请求、1 个 `/tl`、0 错误；共有 756 个公共调用键、159 个固定 primitive、125 个每轮最终 JSON 可见 primitive，剔除 screen/GPU/Canvas 后为 25 行。新增的一行只是 Windows `maxTouchPoints=0`，不是漏接字段。

### 已证明随 typed profile 变化的最终 JSON 字段

`tools/correlate_profile_manifest_json.py` 对 8 个带完整 manifest 的样本做逐叶等值映射，得到 16 条直接因果映射：

| profile 输入 | 最终 JSON 路径 |
| --- | --- |
| `hardware_concurrency` | `[117][0]`、`[123][0]` |
| `device_memory_gb` / 对应物理内存行 | `[151][0]`、`[310][0]` |
| `language` | `[75][0]`、`[158][0]`、`[180][0]`、`[388][0]` |
| `network_rtt` | `[103][0]`、`[427][0]` |
| `user_activation_has_been_active` | `[411][0]` |
| `user_activation_is_active` | `[220][0]` |
| `color_scheme` | `[113][0]` |
| `contrast` | `[337][0]` |
| `audio.sample_rate` | `[199][0]` |
| `performance.memory.jsHeapSizeLimit` | `[361][0]` |
| `performance.memory.totalJSHeapSize` | `[282][0]` |
| `performance.memory.usedJSHeapSize` | `[317][0]` |

`effectiveType/downlink/saveData` 没有直接叶映射，是因为这份不透明负载只读取了 `connection.rtt`；trace 没有出现另外三个 getter。媒体偏好不是缺口：逐查询 trace 已验证 reduced-motion、reduced-transparency、forced-colors 和 inverted-colors 的互补 `matches` 会随相应 profile 翻转。

### 真正固定但不属于缺陷的项目

- Windows `navigator.platform="Win32"`、`webdriver=false`、PDF plugin/mime 清单、`visibilityState="visible"` 是所选桌面分支的标准形态，不应按 seed 随机。
- Windows 非触摸指针表面的 `navigator.maxTouchPoints=0` 与当前 PC 分支一致；触摸 Windows/Android 仍由硬件 profile 独立配置，不能对同一非触摸分支随机填非零值。
- `Animation.playState="running"`、初始 `OfflineAudioContext.currentTime=0` 和媒体 codec 的 `maybe/probably` 是 API/能力语义，不是设备指纹常量。
- CSS system colors 与本机 Edge 150 的独立取证表一致；深浅偏好本身已由 `matchMedia` 驱动，不能把每种 system color 随机拆散。
- AudioContext 的剩余固定候选已用本机 Edge `150.0.4078.65` 独立标准页验证：`sampleRate=48000`、`baseLatency=0.01`、`outputLatency=0`、destination `channelCount/maxChannelCount=2`、`channelCountMode="explicit"`、`channelInterpretation="speakers"`。它们与当前 Windows catalog 一致；Mac 继续使用真实 M5 采集的 `baseLatency=0.005333333333333333`，没有混用 Windows 值。
- 本轮审计时 standalone BODY 使用的 `5/18` 来自 typed profile 默认；页面路径中的 parser/script 节点会改变实际 child count。Rust getter 没有把它按 screen 推导，也没有全局硬编码：显式 `None` 仍进入正常 DOM/CSS 布局。该默认值已在后续 2026-08-09 条目中调整为 `2/18`。

### 本轮确认并修复的缺陷

1. **Windows input CSS 与 DPR/locale 脱节**
   - 旧随机 profile 对所有 Windows 屏幕和地区复用单一输入控件几何。
   - 使用本机 Edge 150 独立标准页采集 8 个 DPR（1–3）× 5 个 UI locale（en/de/ja/pt/zh），新增 `demo/fp/windows_css_profile_catalog.py`。
   - `demo/get_random_fp.py` 现在按 Windows screen DPR、首选 locale 和 Chromium major 选择完整 CSS 行；Mac/Android 分支保持独立。
   - 受控 profile 哨兵把 TextEncoder 中的 range/color/button/checkbox/text/file/search/time/submit 路径逐一映射到对应 CSS 字段；修复后跨国家矩阵的固定成功样本差异由 115 项降至 93 项。

2. **Chromium 148/150 `Navigator.prototype` 属性顺序混用**
   - 本机 Edge 150 证据为 `webkitPersistentStorage,windowControlsOverlay,hardwareConcurrency,...`；用户提供的 Chromium 148 成功证据为 `webkitPersistentStorage,hardwareConcurrency,...,vibrate,windowControlsOverlay,constructor`。
   - `src/web/navigator.rs` 现在从 typed UA 读取 major：仅 major 148 使用已取证的 148 顺序，其余版本保留 Edge 150 顺序，不对无证据版本外推。
   - 重新编译 DLL 后，Chromium 148 两轮黑盒中 `[55]` 与 `[202]` 的整段 Navigator 顺序差异完全消失；固定差异由 Edge-150 跨版本比较的 93 项降至 59 项。

3. **黑盒矩阵入口错误**
   - `tools/run_ips_profile_matrix.py` 现在使用页面 + external replay 路径，并新增 `--user-agent`、timeout/task-turn 参数、最终 JSON 单独导出和完整 profile manifest。
   - Chromium 148 的 UA 现在确实选择 148 CSS/原型分支，不再始终以默认 150 运行后拿来与 148 样本比较。

### `[401][0]` 的复核

- 扩展受控扰动逐一排除了 `Object.getPrototypeOf`、closed `shadowRoot`、断开 iframe `contentWindow`、XHR header、WebGL1/2 prototype `getParameter`、Navigator prototype `hardwareConcurrency`、`Document.createEvent` 和缺失 property descriptor；9 轮中目标均保持 `[null]`。
- 更早的对象身份与单属性哨兵证据已把来源精确定位为资源时间项 `PerformanceEntry.name` 的 URL 派生值。重新采集的本机 Edge 150 和 Chromium 148 对当前脱敏资源 URL 都是 null；旧成功样本的 `2699` 使用不同完整资源 URL。
- 因此不把 `2699` 写成 profile，也不把合法 null 改成占位数值。调用路径已经按 `NetworkReplayEntry.url` 传播真实资源 name。
- 审计矩阵随后也接入每个 seed 自己的 `ResourceLoadProfile`，同步改写页面 `<script src>` 和 replay URL。seed 101/202 均成功执行，`[401]` 从占位 `/xxx` 场景的 `[null]` 变为有限数值 `[2940585]`；两轮数值相同是因为两个 URL 使用相同路径/字段/长度形态，token 内容不同。该值只用于再次证明输入来源，不进入 Rust 或 profile 默认值。

### 审计产物与验证

- 聚合器：`tools/analyze_fixed_trace_results.py`。
- typed profile 到最终 JSON 映射：`tools/correlate_profile_manifest_json.py`。
- CSS 源映射：`tools/probe_css_profile_mapping.py`。
- null/undefined 排除矩阵：`tools/probe_null_source_mapping.py`。
- 15 轮固定值结果：`build/ips-fixed-profile-audit-20260809/fixed-trace-expanded-preferences/`、`analysis-expanded-preferences/`。
- 当前随机资源代码的最终 10 轮结果：`build/ips-fixed-profile-audit-20260809/fixed-trace-final-current/`、`analysis-final-current/`、`profile-json-final-current/` 以及 `final-current-{us,de,jp,br,cn}/`。
- Chromium 148 回归：`build/ips-fixed-profile-audit-20260809/chromium148-cn-seed101/`、`chromium148-cn-seed202/` 和 `analysis-chromium148-cn/`。
- 随机资源 URL 回归：`build/ips-fixed-profile-audit-20260809/dynamic-resource-us/`。
- 本轮只重新构建本地 Debug DLL；未生成 wheel、未安装、未上传 GitHub、未发布 PyPI。

## 2026-08-09：国家随机 profile 的 BODY 默认高度调整为 18

- `demo/get_random_fp.py` 的 `get_random_fp_details()`、`get_random_fp()` 以及公开包装 `examples/country_profile.py` 的默认 `body_client_height` 从 `23.0` 调整为 `18.0`；该次修复时 `body_child_element_count` 仍为 5。

### 2026-08-09：默认 BODY 子元素目标数调整为 2

- 黑盒页面初始化时已包含两个真实 `SCRIPT` 元素；此前默认目标数为 5，会额外补入三个占位 `DIV`。
- 国家随机 profile 与公开包装层的 `body_child_element_count` 默认值统一调整为 `2`，不再为已有两个脚本节点补占位元素。
- `body_client_height` 默认值仍为 `18.0`，显式调用参数仍可覆盖这两个字段。

### 2026-08-09：默认 BODY clientHeight 调整为 0

- 隐藏且零尺寸的 iframe 中，真实浏览器的 `document.body.clientWidth` 与 `document.body.clientHeight` 均为 `0`。
- 国家随机 profile 与公开包装层的 `body_client_height` 默认值由 `18.0` 调整为 `0.0`；`body_child_element_count` 默认值保持 `2`。
- 这只是 profile 默认值变更；调用方仍可显式传入其他高度，或传入 `None` 使用正常 HTML/CSS 布局计算。
- 该变化只发生在 Python profile 组装层。Rust `HTMLBodyElement.clientHeight` getter 仍读取 typed `DocumentFingerprint`，没有把 18 写死，也没有改成按 screen 推导。
- 调用方显式传入其他数值仍会覆盖默认；传入 `None` 仍恢复 DOM/CSS 布局计算。
- 使用文档和默认/覆盖回归测试同步更新。本次不涉及 profile schema、C ABI 或 DLL 二进制修改。
## 2026-08-09：Performance Timeline 压缩体大小与 typed profile 修复

### 缺陷与用户证据

- 真实 Edge 页面样本中，navigation 为 `transferSize=887`、`encodedBodySize=587`、`decodedBodySize=847`、`contentEncoding="zstd"`；脚本 resource 为 `291481 / 291181 / 609863 / "zstd"`。
- 两条记录都满足 Resource Timing 规范的普通网络响应关系 `transferSize = encodedBodySize + 300`，而 decoded 大小是 Content-Encoding 解码后的 HTML/JavaScript 总字节数。
- 旧实现只拿本地明文 `body.len()`，同时写入 encoded 和 decoded，因而所有 gzip/deflate/br/zstd 资源都会丢失压缩前后差异；`evaluate(source_url=...)` 也无法表达真实服务器响应的 encoded 大小。

### 规范和 Chromium 证据

- W3C Resource Timing：`encodedBodySize` 返回 resource info 的 encoded size，`decodedBodySize` 返回 decoded size；普通网络传输的 `transferSize` 为 encoded size 加 300，local cache 为 0，validated cache 为 300。
- Chromium `HttpResponseInfo::encoded_body_size` 保存线上接收、Content-Encoding 解码前的原始 body 大小，供磁盘缓存再次命中时仍能返回正确的 Resource Timing 数值。
- Chromium 网络栈使用 source stream 解码 gzip/deflate、brotli 和 zstd。浏览器是解码方，不存在可从 evaluate 明文唯一反推外部服务器 encoded 大小的“Chrome 固定压缩算法”。

### 实现

- 新增 `src/fingerprint_performance.rs`：`PerformanceFingerprint` 和 `PerformanceEntryFingerprint` typed 数据结构，覆盖 navigation、resource、visibility-state、paint 及其完整可观察字段。
- `EdgeFingerprint.performance.entries=None` 保留自动 timeline；显式 ordered list（包括空 list）覆盖根 realm 初始 timeline，并抑制自动 navigation/visibility/resource/paint 重复项。用户创建的 mark/measure 继续正常追加。
- `content_encoding` 只接受空字符串、gzip、deflate、br、zstd。压缩条目要求真实 encoded/decoded 大小；`transfer_size` 省略时由 Rust 计算 `encoded + 300`，显式值优先。
- `src/web/performance_resource_timing.rs` 和 `performance_navigation_timing.rs` 从 profile 构造正确的 Web IDL 实例及继承链，不返回普通对象空壳。
- 新增独立 C ABI `EdgeSandboxPerformanceEntryProfile`、clear/append 函数；Python `PerformanceEntryProfile` / `PerformanceProfile` 逐字段写入二进制结构，禁止 JSON 字符串配置。profile schema 从 8 升为 9。

### 验证

- `tools/verify_performance_profile.py` 使用 navigation → visible → resource 顺序构造真实形态记录；实际 DLL 返回构造器依次为 `PerformanceNavigationTiming`、`VisibilityStateEntry`、`PerformanceResourceTiming`。
- Rust 自动从 `587` 得到 transfer `887`，从 `291181` 得到 `291481`；decoded 分别保持 `847` 与 `609863`。
- gzip、deflate、br、zstd 四种 contentEncoding 均通过 typed profile 实际 DLL 验证。
- 配置和调用层全程没有 JSON 字符串传参。

## 2026-08-09：`performance.now()` Chromium 单调时钟与 TimeClamper 修复

### 状态

**已修复并验证。** 本条只修复浏览器公开时钟语义，不根据任何业务脚本反推常量，也不把单次采样耗时写入 profile。

### 证据与修复前缺陷

- W3C High Resolution Time Level 3 要求 `performance.now()` 返回从当前 global 的时间原点到调用时刻的单调时长；同一时间原点的连续结果不得因墙钟调整而倒退。非 cross-origin-isolated 环境的时间精度不得高于 100 微秒。
- Chromium `Performance::now()` 读取 monotonic tick clock，并通过 `MonotonicTimeToDOMHighResTimeStamp` 转换。
- Chromium `TimeClamper` 对每个 100 微秒区间生成稳定的伪随机转换阈值；返回区间起点或下一起点。转换相对时间时，会分别收敛绝对单调时刻和 realm 时间原点，然后相减。
- 修复前 `src/determinism.rs` 直接对已经相减的相对毫秒数执行向下取整。虽然输出也落在 0.1ms 网格，但缺少 Chromium 的稳定阈值抖动，且没有遵循“两个绝对时刻分别收敛”的路径。

### 实现

- `src/determinism.rs` 按 Chromium 算法移植 100µs TimeClamper：10 位低微秒拆分、MurmurHash3、每 sandbox 随机 secret、区间内稳定阈值和负值对称处理。
- 普通模式继续使用 `std::time::Instant`，不依赖 `SystemTime` 计算经过时长；墙钟只用于 `Date`/epoch 对齐。
- `performance.now()` 改为 `clamp(monotonicNow) - clamp(realmOrigin)`，负结果按 Web API 路径归零。
- iframe 和 Worker 保留独立时间原点；根 Window、iframe、Worker 仍共享 sandbox 的单调时钟和 TimeClamper secret。
- `Event.timeStamp`、requestAnimationFrame callback、Worker RAF、Gamepad 和 Sensor 的 fallback 时间戳统一复用相同收敛路径，避免相关 API 之间出现不同网格。
- 固定时钟模式从 `clock_epoch_ms` 与 `random_seed` 派生稳定 secret，保证相同确定性配置可复现；普通模式使用进程随机哈希状态生成 secret。
- 函数对象仍通过原生绑定安装；`performance.now.name === "now"`、`length === 0`、原型描述符和 `[native code]` 形态不变。

### 验证

- `determinism::tests::chromium_style_time_clamper_is_monotonic_jittered_and_quantized`：验证单调、100µs 网格以及区间内确实存在稳定的提前转换，而非固定 floor。
- `determinism::tests::chromium_style_time_clamper_preserves_negative_symmetry`：验证 Chromium 负时间处理。
- `p0_tests::performance_now_uses_a_realm_relative_monotonic_100_microsecond_grid`：10 万次调用不倒退、不离开 0.1ms 网格，并验证 iframe 的 `timeOrigin + now` 与父 realm 共用同一时间轴。
- 原有 `edge_clock_semantics_link_performance_date_events_timers_and_animation_frames`、确定性时钟和 Worker 时钟回归继续通过。
- 完整 Rust library 回归：237 项通过、0 项失败；Python profile 回归：10 项通过、0 项失败。
- 已重建 `target/debug/edge_sandbox.dll`，大小 134742016 bytes，SHA-256 `EEA3DE46EF9DEF18A69496DB4E972CB5A92146109FA02CDF786F73F0CC035E2E`。
- `python -m tools.verify_performance_profile` 使用该 DLL 复验 navigation/visibility/resource 对象形态与 gzip、deflate、br、zstd 四种编码，全部通过。
- 本轮未生成 wheel、未安装到 site-packages、未上传 GitHub、未发布 PyPI。

## 2026-08-09：自动脚本资源的真实 gzip/deflate/br/zstd 大小

### 缺陷

- `evaluate(source, source_url=...)` 旧路径只有 `source.len()` 一个数字，并把它同时写入 `encodedBodySize` 和 `decodedBodySize`；`transferSize` 再直接使用该数字加 300。只要资源带 Content-Encoding，这三个字段的关系就是错误的。
- 只给出 `decodedBodySize` 数字无法唯一得到压缩大小：结果还取决于实际源码字节、编码格式、压缩级别、字典和编码器版本。本路径能够修复，是因为 Rust 在 `evaluate` 边界拥有完整 UTF-8 源码，而不是因为存在固定压缩率公式。
- `network_replay` 已提供解码后的 body 与响应头，但旧实现即使读到 `Content-Encoding` 也没有执行编码；image replay 同样复用了错误的单一大小。

### 实现

- 新增 `src/content_encoding.rs`，直接使用维护中的 Rust 库生成真实编码字节：gzip 和 HTTP zlib-framed deflate 使用 `flate2`，Brotli 使用 `brotli`，Zstandard 使用 `zstd`。
- `record_evaluated_script` 改为接收完整源码字节。`decodedBodySize=source.as_bytes().len()`；`encodedBodySize` 为对应编码输出的实际长度；普通 200 网络记录使用 `transferSize=encodedBodySize+300`。
- `PerformanceFingerprint.evaluated_script_content_encoding` 是 typed 字段，未传入时默认 `zstd`，支持显式覆盖为 `gzip`、`deflate`、`br`、`zstd` 和空字符串。Python `PerformanceProfile` 经字段 ID 95 写入 C ABI，不使用 JSON；profile schema 从 9 升为 10。
- replay 的 `Content-Encoding` 响应头是该资源的编码依据；replay body 继续作为浏览器解码后内容供脚本、XHR、fetch 或图片消费，压缩结果只用于构造 Resource Timing 大小。
- 未配置编码的 identity 路径只读取字节长度，不分配压缩缓冲区；只有创建资源记录且编码非空时才发生压缩。
- 该模型生成合法、确定的编码大小，但不能宣称与任意远端服务器逐字节相同。精确远端值还受服务端级别、字典、分块及版本影响；`PerformanceEntryProfile` 仍保留显式 `encoded_body_size/decoded_body_size/transfer_size` 作为网络证据回填入口。

### 验证

- `content_encoding::tests::all_http_encodings_round_trip_real_bytes`：四种算法的压缩输出均可由对应解码器完整还原源码，且测试文本的 encoded 大小小于 decoded 大小。
- `content_encoding::tests::identity_is_a_zero_work_size_path`：空编码和 identity 直接返回源码长度。
- `runtime::tests::evaluated_script_resource_sizes_use_real_http_compression`：逐一创建 gzip、deflate、br、zstd profile，并在被执行脚本内部读取自动 resource；四轮均验证 `contentEncoding`、实际 encoded、源码 decoded 及 `encoded+300` transfer。
- `runtime::tests::replay_resource_sizes_follow_the_content_encoding_header`：HTML 外部脚本的四种 replay 响应均执行解码后源码，同时按响应头生成不同的 encoded/decoded 大小。
- 完整 Rust library 回归：241 项通过、0 项失败。Python 随机 profile 回归 5 项通过，`examples` 全量 compileall 通过。
- 已重建 `target/debug/edge_sandbox.dll`，大小 137337856 bytes。typed Python C ABI 实测 300 字节源码分别得到 gzip `203/300/503`、deflate `191/300/491`、br `175/300/475`、zstd `198/300/498`（顺序为 encoded/decoded/transfer）。
- 本轮未生成 wheel、未安装到 site-packages、未上传 GitHub、未发布 PyPI。

### 默认编码确认

- 按最终要求，自动 `evaluate(..., source_url=...)` 资源在未传入编码时使用 `zstd`；显式的 gzip/deflate/br/zstd/identity 配置和 replay 响应头优先级不变。
- Rust 默认 fingerprint 与 Python `PerformanceProfile` 默认值保持同步，未改变 profile schema。

## 2026-08-09：多轮黑盒固定值与动态来源深度审计

### 边界与方法

- 本轮没有打开、解析或解释 `ips.js`、`ips1.js`、`ips2.js`；脚本只通过 `Path.read_bytes()` 交给沙箱执行。
- `success.json` 只保留为历史结构证据，本轮没有把其中任何具体数值当成修改目标。
- 18 轮 Windows 运行覆盖 US/DE/JP/HK/CN/BR、不同 profile seed，固定 runtime RNG；每轮分别保存完整 typed profile、native trace、结构化 stdout、TextEncoder 参数、最终数组和 DOM 快照。
- 路径级比较以 JSON 路径为键，不再使用“某个相同标量曾在任意位置出现”作为因果证据。

### 运行矩阵结果

- 18 轮共有 960 个共同最终叶路径，其中 762 个固定、198 个变化，另有 273 个条件出现路径。
- typed profile 有 1083 个共同叶字段，其中 630 个在这 18 轮固定、453 个变化；确认 57 条 profile 字段到最终 JSON 路径的完全相同变化向量。
- native trace 有 759 个共同调用签名，其中 701 个结果序列固定、58 个变化。
- 154 条“每轮都出现且结果为同一 primitive”的 trace 记录已全部分类，人工未分类数为 0：82 条已有 typed profile，33 条由页面、网络、DOM、布局、媒体源或运行时状态派生，8 条属于 API 形态语义，13 条属于当前缺少的关联输入。
- 原始与分类结果位于 `build/deep-profile-audit-schema10-20260809/analysis-windows`、`analysis-windows-legacy-candidates` 和 `analysis-windows-dynamicity`。

### 大样本 profile 组合器复核

18 次未抽中低概率状态不能证明写死，因此新增 `tools/analyze_profile_composer_variability.py`，不执行任何 JavaScript，单独检查组合器可达到的 typed profile 状态：

| 平台 | 样本 | 固定路径 | 变化路径 | 条件路径 |
| --- | ---: | ---: | ---: | ---: |
| Windows | 2000 | 617 | 447 | 197 |
| macOS | 1000 | 4008 | 335 | 563 |
| Android | 1000 | 534 | 62 | 29 |

- Windows 的 `forced_colors`、`inverted_colors`、`reduced_motion`、`reduced_transparency` 在 2000 组中都实际出现两种状态；18 轮执行矩阵没有抽中其中部分低概率状态，不代表组合器固定。
- Windows WebGL 在同一 D3D11/ANGLE 能力分支中有 704 种 renderer、3 种 unmasked vendor；112 个其余叶字段共享同一版本锁定的能力表。WebGPU 有 6 个变化字段、41 个固定字段。固定的后端 limit 不能独立随机化；只有增加整套有来源的 backend/feature-level capability bundle 才能安全扩展。
- Mac 默认池只包含具备完整公开 Chromium 150 Metal 能力记录的 Apple-silicon 候选；renderer 有 18 种，`webgl2_max_samples` 有 4/8 两种。Intel/AMD 行仍在 inventory，但默认池明确排除，不用猜测的 limits 补位。
- Canvas 的 11 个字段在三平台组合器中都固定。Rust 已有 typed `CanvasProfile`，但随机组合器缺少多套真实采集的完整渲染行；这属于“可配置但 catalog 证据不足”，不能用随机 salt 假装设备差异。
- Permissions 的 18 个字段已有 typed 配置，但国家随机 profile 使用固定初始站点权限状态。它们是站点/用户授权状态而不是硬件指纹，不应为了增加组合量而互相独立随机。

### TextEncoder 参数固定性

`tools/classify_textencoder_dynamicity.py` 对 15 个稳定 call index 生成 `textencoder-dynamicity.tsv`：

- call 1–3 随时区、墙钟和 locale 变化；call 11 随键盘布局变化；call 12 随 speech voice catalog 变化；call 13 与运行时随机状态变化；最终 aggregate call 14 也变化。
- call 4 是固定 MIME 查询字符串列表，是调用方输入而不是 API 返回值，不能由沙箱随机修改。
- call 5 是固定 CSS 数学表达式的规范化 px 结果；应随 CSS/viewport 输入变化，不应添加 profile 噪声。
- call 0 和 call 6–10 是固定的测量/像素向量候选。它们只有在字体、Canvas、颜色和渲染后端以一整套真实采集行切换时才应变化；当前没有足够证据把某个单独数值映射到任意 profile 字段。

### Resource Timing 为什么在多轮中固定

- 同一 parser 资源每轮执行的源码字节完全相同，因此 `decodedBodySize=587817`、zstd `encodedBodySize=300733`、`transferSize=301033` 固定；资源 URL 中的随机身份仍会变化。
- 同一运行中的 `void 0`（6 字节）自动资源为 `15/6/315`，69 字节审计脚本为 `78/69/378`，已经直接证明 size 按实际源码和编码派生，不是 profile 硬编码。
- 因此资源大小应由源码、replay body、Content-Encoding 和响应元数据派生；完整 `PerformanceProfile.entries` 继续提供真实网络证据回填。不能为了让多轮数值不同而随机 transfer/encoded/decoded。

### 确认的关联缺陷：forced-colors 与 CSS system colors 脱节

- 组合器能产生 `matchMedia('(forced-colors: active)').matches === true`，但旧 Rust `SYSTEM_COLORS` 始终返回普通 Windows 浅色 palette，两个相关观测面互相矛盾。
- CSS Color Adjustment Level 1 为自动化测试定义了整套 light/dark forced-colors palette；本次按 `MediaPreferencesProfile.forced_colors + color_scheme` 整体选择 palette，不对单个颜色做随机处理。
- `getComputedStyle()` 现在在 forced-colors dark 下把 `CanvasText` 解析为 `rgb(255, 255, 255)`、`Highlight` 解析为 `rgb(26, 235, 255)`；普通 profile 仍保持原 Edge 150 palette。
- `forced_colors_use_one_coherent_profile_selected_palette` 与 `forced_color_media_state_and_computed_system_palette_stay_coherent` 均通过。本修复复用已有字段，不改变 ABI、profile schema 或 options schema。
- 规范证据：`https://drafts.csswg.org/css-color-adjust-1/#forced-colors-mode`。

### 仍适合增加 typed 状态、但本轮未擅自改变默认行为的项目

1. `document.hasFocus()` 当前 Rust 固定 true；应由页面系统焦点和 active-document 链派生，可增加初始 page focus 状态。
2. `document.visibilityState/hidden` 当前固定 `visible/false`；两者必须来自同一个 visibility state，并与初始 `VisibilityStateEntry` 联动。
3. 六个 `BarProp.visible` 当前全部用 true 构造；规范要求它们始终相同并返回 `!is_popup`，应增加一个 page/window `is_popup` 状态，而不是六个独立布尔值。
4. 普通/forced CSS system colors 若要覆盖真实 OS 自定义高对比主题，需要增加一整套有来源的 palette；不能独立随机每个 keyword。

HTML 规范证据：

- Document visibility/hidden/hasFocus：`https://html.spec.whatwg.org/multipage/interaction.html`
- BarProp 与 popup 关系：`https://html.spec.whatwg.org/multipage/nav-history-apis.html#the-barprop-interface`

### BODY 默认状态回归

- 国家随机 profile 的默认 `body_child_element_count/body_client_height` 已调整为 `2/0`。
- 新默认黑盒运行中，页面初始化为两个真实 SCRIPT；负载执行期间 trace 读取到 `document.body.childElementCount=4`、`clientWidth=0`、`clientHeight=0`。
- 临时节点清理后的最终 DOM 为两个 SCRIPT 加一个 IFRAME，共 3 个元素；运行期间读数与最终快照不同属于真实 DOM 生命周期，不应把最终数量硬覆盖到 getter。

### Document 初始页面状态 typed 配置（schema 11）

- 深度审计中确认的三个关联状态已经实现，不再保留为固定 getter：`DocumentProfile.has_focus`、`visibility_state` 与 `is_popup`。
- `visibility_state` 只接受 `visible/hidden`，并同时驱动 `document.hidden`、`document.visibilityState`、`document.webkitVisibilityState` 和自动创建的首条 `VisibilityStateEntry.name`，避免同一页面状态互相矛盾。
- `is_popup` 作为单一页面状态驱动 `locationbar/menubar/personalbar/scrollbars/statusbar/toolbar.visible === !is_popup`，没有拆成六个可产生矛盾组合的字段。
- 新字段通过独立 string/bool C ABI setter 写入，不使用 JSON 字符串。Rust 与 Python profile schema 同步由 10 升为 11；旧 DLL/新 Python 或新 DLL/旧 Python 会被显式拒绝，不能静默错配。
- 国家 profile 的 `has_focus` 默认通过独立 seed 流以 50/50 生成 `True/False`，同一 seed 可复现且不扰动硬件、语言等原随机序列；调用方显式传入布尔值时优先。`visibility_state="visible"`、`is_popup=False` 仍是普通非弹窗页面默认值。
- 1000 个连续 seed 的组合器回归得到 `True=509`、`False=491`；实际 DLL 对 seed 0/2 分别返回 `document.hasFocus() === false/true`。重复 seed 803431 两次均为 false，证明可复现。
- Rust 行为测试 `document_initial_state_is_coherent_across_document_performance_and_bar_props`、非法状态测试、typed FFI 单元测试和 Python profile 测试均通过。
- 新构建 `target/debug/edge_sandbox.dll` 经 Python 实际调用返回 `2|0|0|false|true|hidden|hidden|true`，依次证明 BODY 子元素数、clientWidth/clientHeight、焦点、隐藏状态、Document/Performance 联动及六个 BarProp 的配置均生效。
- schema 11 默认黑盒回归仍为 `trace=5120`、`TextEncoder=15`、`requests=1`、`/tl=1`、无异常；trace 中主文档读取到 `childElementCount=4`、`clientWidth=0`、`clientHeight=0`，iframe BODY 也为 `clientWidth=0/clientHeight=0`。产物位于 `build/profile-body-zero-schema11-check/`。
- 本次只重建本地 Debug DLL，未生成或安装 wheel，未上传 GitHub，也未发布 PyPI。

## 2026-08-10：全局时钟、精度与调度关系第二轮修复

### Edge/Chromium 证据

- Chromium 当前 `TimeClamper` 明确定义普通上下文 100 微秒、cross-origin isolated 上下文 5 微秒，并用每个时间区间内稳定的伪随机阈值选择相邻网格点：`https://chromium.googlesource.com/chromium/src/+/HEAD/third_party/blink/renderer/core/timing/time_clamper.h`、`time_clamper.cc`。
- Chromium timing README 要求所有公开的高精度时间戳经 Performance/TimeClamper 路径转换：`https://chromium.googlesource.com/chromium/src/+/HEAD/third_party/blink/renderer/core/timing/README.md`。
- High Resolution Time Level 3 定义每个 global 自己的 time origin 和单调时间：`https://www.w3.org/TR/hr-time-3/`。
- HTML timer 算法要求计时器编号表属于当前 global，并规定嵌套超过 5 层后的 4 ms 最小延迟：`https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html`。
- requestIdleCallback 定义 idle deadline 与 50 ms 上限：`https://w3c.github.io/requestidlecallback/`；DOM 规范定义 `AbortSignal.timeout()` 的异步 active-time 超时：`https://dom.spec.whatwg.org/#dom-abortsignal-timeout`。
- 使用本机 Edge 150 HTTPS/headless 证据页 `tools/edge_clock_probe.html` 复核：同一帧的多个 RAF 回调时间戳完全相等，`document.timeline.currentTime` 等于该帧时间戳；新 iframe 的 Performance time origin 独立；父/子 realm 的 timer、RAF 与 idle ID 都从自己的编号表开始；Event timestamp 使用当前 Performance 时间轴；普通 idle deadline 接近 50 ms；原生 `Temporal.Now` 六个方法的名称、顺序、长度及 native 形态得到记录。

### 修复内容

- `src/determinism.rs` 增加可复用的单次单调时间快照和统一 epoch 纳秒接口。`Date`、Performance 与确定性 `Temporal.Now` 不再从彼此无关的系统时钟重复取样。
- `src/web/performance.rs` 可从同一个单调快照派生任意 realm 的 DOMHighResTimeStamp，供一批关联回调共享。
- Window/iframe 的 timeout、interval、RAF 与 idle callback 状态全部改为 realm-local 编号表、回调表和取消表；子窗口不能再误取消父窗口相同数字的任务。跨 realm 且截止时间相同的 timer 另用全局注册序号保持 FIFO，不依赖 HashMap 或 realm 编号。
- RAF 在一次渲染批次只采样一次单调时刻，同批回调得到完全相同的参数，并同步采样当前 realm 的 `DocumentTimeline`。Worker RAF 和 XRSession RAF 同样改用关联 Performance 时间轴；XR 回调不再同步获得固定 0。
- `DocumentTimeline.currentTime` 改为渲染/任务边界采样值，而不是每次 getter 临时读取一个更晚的 `performance.now()`。
- idle callback 支持 `options.timeout`、`didTimeout` 和统一时钟 deadline；只有没有更高优先级任务的轮次才进入 idle callback。
- `AbortSignal.timeout()` 从“创建时立即 aborted”修复为到期后异步派发 `TimeoutError`；`scheduler.postTask({delay})` 与 `scheduler.yield()` 从立即完成修复为任务队列调度。
- 普通模式继续保留 V8 150 原生 Temporal 行为；仅确定性时钟模式原位替换 `Temporal.Now` 六个方法，使 Instant、PlainDate/Time/DateTime、ZonedDateTime 与配置 epoch/时区一致，同时保持原属性顺序、函数名称、length 和 `[native code]`。
- JS Self-Profiling 的经过时间改用所属 realm 的 Performance 时钟，移除第二个独立 `std::time::Instant`。
- 本次没有增加 profile 字段，没有修改 profile schema、C ABI、Python binding 或 trace 形态。

### 精度边界

- 当前沙箱的 `crossOriginIsolated=false`，因此全部公开高精度时间仍走 100 微秒网格。Chromium 的 5 微秒常量已经有证据，但在真正支持 COOP/COEP 隔离前不启用，避免产生隔离状态与精度互相矛盾的环境。
- rAF 参数是渲染机会采样时间；回调体内随后读取的 `performance.now()` 可以略晚，但不得早于帧时间。`document.timeline.currentTime` 在同一批次必须严格等于 rAF 参数。
- `Date.now()` 仍是整数毫秒；Temporal Instant 在普通模式保留 V8 的亚毫秒能力，在确定性模式精确派生自 `clock_epoch_ms` 与任务推进值。

### 回归验证

- 新增 realm-local ID、同截止时间跨 realm timer 顺序、RAF 批次一致性、DocumentTimeline、idle timeout、AbortSignal、Scheduler 与 Temporal 的六组回归测试。
- `cargo test --all-targets -- --nocapture`：255 项通过，0 项失败。
- 本轮没有构建 release DLL/SO、wheel，没有安装包，没有上传 GitHub，也没有发布 PyPI。
