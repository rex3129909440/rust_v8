# W6 Sandbox Executor API 完整使用文档

本文对应当前文件：

`demo/w6_sandbox_executor_api.py`

该服务只负责在隔离的 Android WebView 沙箱中执行调用方已经取得的 JavaScript，
并返回沙箱捕获到的最后一条 `/tl` 请求。服务自身不会下载 `ips`、不会发送
`/tl`、不会访问业务接口，也不使用代理。

## 1. 运行结构

```text
本地HTTP请求
    │
    ├─ ips       → 一次性隔离V8 Worker
    ├─ js_url    → V8 source URL
    ├─ ua        → 随机Android WebView profile
    ├─ token     → 兼容接收但忽略
    └─ logFlg?   → 仅字段出现时开启console capture
                         │
                         └─ 捕获最后一条 /tl
                                ├─ headers
                                └─ binary body
```

每个 Worker 只执行一份 JavaScript，执行后立即销毁并创建替补。不同请求的 V8
isolate、DOM、全局变量、profile、请求记录和 console 记录互不共享。

## 2. 启动服务

不使用命令行参数，直接从 Python 启动：

```python
from demo.w6_sandbox_executor_api import serve_local

serve_local(
    host="127.0.0.1",
    port=8765,
    default_timeout_ms=30_000,
)
```

默认只绑定本机地址。不要在没有额外身份认证和访问控制的情况下绑定公网地址，
因为该接口允许调用方提交 JavaScript。

也可以嵌入已有 ASGI 服务：

```python
from demo.w6_sandbox_executor_api import create_app

app = create_app(
    maximum_workers=8,
    default_timeout_ms=30_000,
)
```

## 3. Worker 数量

不显式传 `maximum_workers` 时，服务根据 CPU 和内存自动选择：

- CPU 预算：逻辑 CPU 数乘 2；
- 内存预算：为宿主保留 768 MiB，每个活跃 Worker 按 384 MiB；
- 自动上限：8；
- Linux 同时读取 cgroup v1/v2 的 CPU quota 和 memory limit。

4 核 4 GB 主机默认选择 8 个活跃 Worker。20 个 HTTP 请求可以同时进入服务，
但只会有 8 个完整 V8 进程同时执行，其余请求在池内等待。执行超时从任务真正
进入 V8 后开始，不包含池内排队时间。

如果业务 JavaScript 是纯 CPU 密集型，可显式降低为 4；如果宿主资源更多，也可
显式设置，但不建议在 4 GB 主机上创建 20 个同时活跃的 V8 Worker。

## 4. 执行接口

```text
POST /v1/execute
Content-Type: application/json
```

请求示例：

```python
payload = {
    "ips": response.text,
    "js_url": src,
    "token": "620570abfaa210ca674767fcb137da47",
    "ua": "wizz-air/8.1.9 (com.wizzair.WizzAirApp; build:2207; android 9)",
}
```

### 4.1 ips

- 必填字符串；
- 作为 JavaScript 源码执行一次；
- 最大 32 MiB；
- 服务不下载、不修改、不重复执行该源码。

沙箱表达式最终返回值不会进入 HTTP 响应。HTTP 响应只取 `/tl` 请求数据。

### 4.2 js_url

- 必填字符串；
- 只作为 V8 `ScriptOrigin/source URL`；
- 用于异常堆栈、脚本来源和 PerformanceResourceTiming；
- 不会导航页面，也不会从该地址发起网络请求。

### 4.3 ua

- 必填 Android/App WebView UA；
- 用于构建每次请求独立的随机 Android WebView profile；
- UA 中有 Chrome/Chromium/Edge 版本时使用该主版本；
- 示例中的自定义 App UA 没有 Chromium token，因此使用 W6 WebView 136 表。

当前请求结构没有国家字段，因此 profile 国家基线为 `US`。

### 4.4 token

兼容字段，可以为空。服务不读取、不校验、不注入 JavaScript，也不写入 profile
或返回值。

### 4.5 logFlg

可选字段。开关依据是“字段是否出现”，不是字段的布尔值：

```python
# 不收集console
{"ips": "...", "js_url": "...", "token": "...", "ua": "..."}

# 以下三种都会收集console
{"logFlg": True,  ...}
{"logFlg": False, ...}
{"logFlg": None,  ...}
```

未出现 `logFlg` 时：

- Rust 原生 console capture 关闭；
- 不遍历数组、Arguments、对象、Error、TypedArray；
- Python 不调用 `stdout()`；
- 响应不包含 `console`。

出现 `logFlg` 时才启用本次 Worker 的 typed console capture。

## 5. DV 与 M42

每个请求从 `demo/dv.py` 的 `x_kpsdk_dvs` 只选择一次：

```text
selected_dv
    ├─ preload赋值给 window.m42
    └─ 返回为 headers["x-kpsdk-dv"]
```

因此：

```javascript
window.m42 === 返回结果.headers["x-kpsdk-dv"]
```

不会二次随机，也不会从捕获请求中反推 DV。

## 6. DT

`headers["x-kpsdk-dt"]` 来自 `demo/encrypt_dt.js` 的 `get_dt()`，不是沙箱
`evaluate()` 返回值，也不是从 `/tl` 请求推导。

服务使用容量为 32 的后台有界预生成池：

- 每个 DT 仍由 fresh-per-call ExecJS/Node JavaScript 上下文生成；
- 不复用会改变内部随机分布的持久 JS 上下文；
- 后台同时最多运行一个 Node 生成任务；
- 一个 DT 只从队列取出一次；
- 队列为空时最多等待一秒，之后现场调用 JavaScript 生成；
- 预热后获取约 0.002 ms，避免请求路径等待约 24.5 ms 的 Node 启动。

## 7. 成功响应

未传 `logFlg`：

```json
{
  "headers": {
    "content-type": "application/octet-stream",
    "x-kpsdk-dv": "与window.m42相同的值",
    "x-kpsdk-dt": "encrypt_dt.js生成的值"
  },
  "body": "AAEC/w==",
  "body_encoding": "base64"
}
```

`body` 是 `tl_request.body` 的 Base64，避免任意二进制被 UTF-8 解码破坏：

```python
import base64

body_bytes = base64.b64decode(result["body"])
```

返回 `headers` 以捕获到的最后一条 `/tl` 请求 headers 为基础，并确保覆盖：

```text
x-kpsdk-dv = 本次window.m42
x-kpsdk-dt = 本次encrypt_dt.js生成值
```

传入 `logFlg` 后额外出现：

```json
{
  "console": [
    {
      "sequence": 1,
      "level": "log",
      "frame_url": "https://页面地址/",
      "text": "debug-value [1, two]",
      "arguments": [
        {
          "kind": "string",
          "value": "debug-value",
          "truncated": false
        },
        {
          "kind": "bytes",
          "type_name": "Uint8Array",
          "value": "AH//",
          "encoding": "base64",
          "truncated": false
        }
      ]
    }
  ]
}
```

## 8. /tl 选择规则

执行完成后读取本次 Worker 的请求记录，按逆序选择最后一条满足以下条件的记录：

```python
request.url.rstrip("/").endswith("/tl")
```

没有捕获到 `/tl` 时返回 HTTP 500：

```text
sandbox did not capture a /tl request
```

捕获记录复制到 Python 后立即从 Worker 池清除。Worker 随后销毁，不会污染下一个
请求。

## 9. 健康接口

```text
GET /health
```

响应示例：

```json
{
  "status": "ok",
  "maximum_workers": 8,
  "live_workers": 8,
  "worker_process_ids": [1001, 1002, 1003, 1004]
}
```

## 10. 运行限制

W6 executor 当前固定限制：

```python
SandboxLimits(
    timeout_ms=30_000,
    max_heap_bytes=256 * 1024 * 1024,
    max_young_generation_bytes=4 * 1024 * 1024,
    max_code_range_bytes=None,
    max_resident_bytes=768 * 1024 * 1024,
    max_source_bytes=32 * 1024 * 1024,
    max_output_bytes=8 * 1024 * 1024,
)
```

V8 young generation 使用 4 MiB，同时补偿 old generation，实际
`heap_size_limit` 仍精确为 256 MiB。没有提高 RSS 或 heap 上限。

## 11. V8 诊断接口

这些接口返回真实 V8 数据，不是 profile 中展示给 JavaScript 的
`performance.memory`：

```python
with EdgeSandbox(...) as sandbox:
    statistics = sandbox.v8_memory_statistics()
    print(statistics.total_heap_size)
    print(statistics.total_physical_size)
    print(statistics.used_heap_size)
    print(statistics.code_and_metadata_size)
    print(statistics.bytecode_and_metadata_size)
    print(statistics.external_memory)
```

显式低内存通知：

```python
sandbox.low_memory_notification()
```

W6 一次性 Worker 默认不调用强制 GC，因为 Worker 随后销毁，调用只会增加延迟并
可能改变 WeakRef/finalization 时机。

## 12. 常见错误

### JavaScript执行超时

```text
JavaScript execution exceeded the configured timeout
```

表示真正进入 V8 后超过 `timeout_ms`，不包含池内排队时间。

### Worker RSS持续超限

```text
isolated Edge worker exceeded max_resident_bytes:
observed ... bytes, limit 805306368 bytes, across 3 consecutive samples
```

三平台均要求连续三次 RSS 采样超限才终止，短暂映射/GC 峰值不会被单点误杀。
Windows 使用 WorkingSetSize，Linux 使用 VmRSS，macOS 使用 resident size。

### runtime-options schema不一致

```text
native runtime-options schema does not match the Python binding
```

当前 runtime-options schema 为 4。Python 文件和 DLL 必须来自同一次构建，更新
DLL 后必须重启长期运行的 FastAPI/Python 进程。

### DLL无法覆盖

Windows 进程加载 DLL 后会锁定文件。先停止对应服务，或把实验 DLL复制到其它
路径测试；不要强制删除使用中的 DLL。

## 13. 4C4G与20并发实测

8 个一次性 Worker、20 个并发请求：

```text
成功：20/20
总耗时：约9.946秒
子进程RSS峰值合计：1,206,034,432 bytes
console：20个响应全部省略
x-kpsdk-dv/dt：20个响应全部存在
```

V8 优化前同一矩阵峰值约 1,327,960,064 bytes，最终版本降低约 122 MB，吞吐
差异约 1.2%。

## 14. 版本与文件

```text
profile schema         = 15
runtime-options schema = 4
```

关键文件：

```text
demo/w6_sandbox_executor_api.py
demo/android_call_edge_sandbox.py
demo/dv.py
demo/encrypt_dt.js
examples/edge_sandbox_pool.py
examples/run_sandbox.py
examples/edge_runtime_options.py
target/release/edge_sandbox.dll
```

当前 Windows DLL：

```text
size   = 78,421,504 bytes
SHA256 = 67C5C8E0CDB18F6323EA0632553A5A418E5BF9B18C7E1334D418339460842D2E
```

## 15. 回滚

V8 优化前完整备份：

`D:/sandbox/edge-sandbox-backups/pre-v8-optimization-20260822-144342`

备份 DLL SHA256：

`61BE76A5FD8F8972938E76962C97B0D27EC6C78F730518C4173E3EC2232E75A7`

回滚时必须同时恢复 Python绑定和 DLL，不能只替换其中一个，否则 schema 校验会
拒绝启动。

## 16. 相关审计

- `docs/V8_MEMORY_PERFORMANCE_OPTIMIZATION_2026-08-22_ZH.md`
- `docs/SANDBOX_PERFORMANCE_MEMORY_OPTIMIZATION_2026-08-22_ZH.md`
- `docs/OPTIONAL_CONSOLE_CAPTURE_2026-08-22_ZH.md`
- `docs/ANDROID_WEBVIEW_RANDOM_PROFILE_GAP_AUDIT_2026-08-22_ZH.md`
