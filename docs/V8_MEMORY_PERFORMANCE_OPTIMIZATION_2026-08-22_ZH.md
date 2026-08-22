# V8 内存与执行性能优化（2026-08-22）

## 回滚点

优化前完整快照：

`D:/sandbox/edge-sandbox-backups/pre-v8-optimization-20260822-144342`

优化前 DLL SHA256：

`61BE76A5FD8F8972938E76962C97B0D27EC6C78F730518C4173E3EC2232E75A7`

## 优化边界

- 不删除任何 DOM/BOM/Web API；
- 不改变 Window/Navigator/iframe 表及顺序；
- 不跳过真实定时器或任务循环；
- 不提高 768 MiB Worker RSS 限制；
- V8 总 heap limit 保持 256 MiB；
- 只在 W6 executor 默认启用紧凑 young generation；通用 EdgeSandbox 保持 V8
  默认值，除非调用者显式配置。

## 新增真实 V8 诊断

Python `EdgeSandbox.v8_memory_statistics()` 返回 typed 原生字段，不读取伪装给
JavaScript 的 `performance.memory`：

- total/used/physical/available heap；
- executable heap；
- global handles；
- malloced/external/peak malloced memory；
- native/detached contexts；
- total allocated bytes；
- JIT code and metadata；
- bytecode and metadata；
- external script source size。

`EdgeSandbox.low_memory_notification()` 仅作为显式诊断接口提供。W6 一次性 Worker
默认不调用它，因为 Worker 随后销毁，强制 GC 只会增加延迟并可能改变
WeakRef/finalization 时机。

## Typed V8 ResourceConstraints

`SandboxLimits` 新增：

- `max_young_generation_bytes`
- `max_code_range_bytes`

两项均通过 typed C ABI 和二进制 Worker IPC 传输，不使用 JSON 字符串。runtime
options schema 从 3 更新为 4。

当显式设置 young generation 时，Rust 同时调整 old generation，并扣除 V8 的
2 MiB 显式 generation 固定开销，使最终 `heap_size_limit` 精确保持调用者配置的
总上限。

## A/B 结果

同一 Android WebView 136 profile、当前约 650 KB 黑盒脚本：

| 配置 | 执行后 RSS | V8 heap physical | used heap | 墙钟 |
|---|---:|---:|---:|---:|
| V8 默认 | 约 164.0 MB | 约 62.46 MB | 约 41.36 MB | 约 2.971 s |
| young 4 MiB | 约 147.9 MB | 约 48.87 MB | 约 42.82 MB | 约 2.980 s |
| young 8 MiB | 约 153.5 MB | 约 53.96 MB | 约 40.79 MB | 约 2.979 s |
| young 16 MiB | 约 166.4 MB | 约 62.85 MB | 约 41.31 MB | 约 2.962 s |
| young 32 MiB | 约 184.0 MB | 约 80.85 MB | 约 54.64 MB | 约 2.967 s |

code range 32/64/128 MiB 对 RSS 没有稳定收益，未作为默认优化。JIT code 与 metadata
仅约 2.94 MB，瓶颈在 young-generation physical pages。

最终版本额外补偿 old generation 后：

- `heap_size_limit = 268435456`，精确 256 MiB；
- 单 Worker RSS：约 150.3 MB；
- V8 heap physical：约 49.4 MB；
- used heap：约 43.7 MB；
- JIT code/metadata：约 2.96 MB；
- 墙钟：约 2.978 秒；
- CPU：约 0.641 秒。

## 20 并发

8 个一次性 Worker、20 个 HTTP 并发：

| V8 配置 | 成功 | 总耗时 | 子进程 RSS 峰值合计 |
|---|---:|---:|---:|
| V8 默认 | 20/20 | 9.828 s | 1,327,960,064 bytes |
| young 4 MiB 实验 | 20/20 | 9.942 s | 1,209,016,320 bytes |
| 最终对齐版本 | 20/20 | 9.946 s | 1,206,034,432 bytes |

最终版本相对默认降低约 122 MB 峰值 RSS，吞吐差异约 1.2%，20 个响应均包含
`x-kpsdk-dv`/`x-kpsdk-dt`，且未传 `logFlg` 时均无 console 捕获。

## 数据与二进制

- 完整 A/B：`build/v8-memory-ab/runs.json`
- 汇总：`build/v8-memory-ab/summary.tsv`
- 基准工具：`tools/benchmark_v8_memory_profiles.py`
- 最终 DLL：`target/release/edge_sandbox.dll`
- DLL 大小：78,421,504 字节
- DLL SHA256：
  `67C5C8E0CDB18F6323EA0632553A5A418E5BF9B18C7E1334D418339460842D2E`
