# 按请求启用 Console 捕获（2026-08-22）

## 目标

`demo/w6_sandbox_executor_api.py` 只有在请求体中出现 `logFlg` 字段时才
获取沙箱 console 输出。未出现该字段的生产请求不能承担 console 参数深度
序列化与 Python ESSO 解码开销。

## 行为

- 未传 `logFlg`：
  - Worker 原生 console capture 保持关闭；
  - `console.log/info/debug/warn/error/dir/dirxml/table/trace` 保留标准函数形态
    和返回语义，但不遍历、复制、保留其参数；
  - Python 不调用 `stdout()`；
  - HTTP 响应不包含 `console` 字段。
- 请求字段中出现 `logFlg`：
  - 无论字段值是 `true`、`false` 或 `null`，都在执行前开启本次 Worker 的
    console capture；
  - 响应新增 `console`，包含 sequence、level、frame_url、text 和逐项 typed
    arguments；
  - TypedArray/ArrayBuffer 参数使用 Base64，嵌套 sequence/object 保留结构；
  - Worker 执行完成后销毁，Python 中的任务日志记录同步清除。

`token` 仍只兼容接收，不参与 profile、JavaScript 或 console 开关。

## W6 executor 的 DV/DT 返回

`demo/w6_sandbox_executor_api.py` 的成功响应会在 `headers` 中写入：

- `x-kpsdk-dv`：每次请求只从 `x_kpsdk_dvs` 选择一次；同一个值通过 preload
  赋给 `window.m42`，并原样写入响应 headers，不进行第二次随机选择。
- `x-kpsdk-dt`：由 `demo/encrypt_dt.js` 的 `get_dt()` 在独立 ExecJS 上下文中生成；
  不使用沙箱 `evaluate()` 返回值，也不从捕获到的 `/tl` 请求字段推导。

DT 生成器通过锁串行调用，并由 `asyncio.to_thread()` 移出 FastAPI 事件循环；
不产生外部网络请求。

## max_resident_bytes 限制器修复

W6 executor 的 `max_resident_bytes` 保持 768 MiB，没有通过扩大容量规避错误。

审计证据：

- 空 W6 Worker RSS 约 50 MB；
- 当前约 650 KB 的黑盒脚本执行峰值约 168 MB；
- 同一 Worker 连续替换 100 套 Android profile 后约 55 MB，没有旧 V8 isolate
  的线性残留；
- 40 套 profile 在 768 MiB 下全部完成；扩大至 200 套高并发矩阵时没有 RSS
  超限，只有 CPU 饱和引发的正常超时。

实际限制器存在两个问题：

1. Windows 把 RSS 配置同时传给 Job Object 的 `PROCESS_MEMORY_LIMIT`。该字段
   限制的是 committed virtual memory，并不是 Working Set/RSS；V8 地址空间提交
   可能因此被错误终止。现在 Job Object 只保留单进程与父进程关闭时终止策略，
   RSS 统一由 `GetProcessMemoryInfo().WorkingSetSize` 监控。
2. 原实现单次 5 ms RSS 采样越线就立即杀死 Worker。现在要求连续 3 次采样均
   超过 768 MiB，短暂映射/GC 峰值不会误杀；持续超限仍会在约 15 ms 内终止。

Linux 使用 `/proc/<pid>/status` 的 `VmRSS`，macOS 使用
`proc_taskinfo.pti_resident_size`，不存在 Windows Job Object 指标混用；连续采样
确认逻辑则统一应用于三个平台。新版错误会同时包含实际观测 RSS、配置上限和
连续越线次数，便于后续审计。


## 原生实现

- `src/console_capture.rs` 增加 capture enabled 状态。关闭状态在进入对象、
  数组、Arguments、Error、TypedArray 深度观察前直接返回。
- `src/runtime.rs`、`src/isolated_runtime.rs` 和 `src/ffi.rs` 增加独立开关路径。
- `examples/run_sandbox.py` 暴露 `set_stdout_capture_enabled(bool)`。
- `examples/edge_sandbox_pool.py` 默认关闭池内 Worker 的 console capture；
  `submit(capture_stdout=True)` 才为单个任务开启并保存结果。
- 直接创建 `EdgeSandbox` 的历史行为保持默认 capture-on，不影响既有
  `sandbox.stdout()` 调用。

## 验证

- Rust 原生开关：1/1 通过。
- 既有 TextEncoder/stdout 回归：1/1 通过。
- W6 sandbox executor API：4/4 通过。
- 既有 W6 Worker API：4/4 通过。

本地 release DLL：

- 路径：`target/release/edge_sandbox.dll`
- 大小：78,421,504 字节
- SHA256：`67C5C8E0CDB18F6323EA0632553A5A418E5BF9B18C7E1334D418339460842D2E`
