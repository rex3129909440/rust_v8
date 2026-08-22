# 沙箱性能与内存第一轮优化（2026-08-22）

## 不变约束

- 每个 Worker 仍是独立 OS 进程；
- 每个 Worker 仍只执行一份 JavaScript，完成后销毁并创建替补；
- `max_resident_bytes` 仍为 768 MiB；
- V8 heap 仍为 256 MiB；
- 不缩短浏览器定时器，不跳过任务循环，不改变 Edge/WebView API 语义；
- 未传 `logFlg` 时原生 console capture 关闭。

## 分阶段基线

Windows release DLL、W6 Android WebView 136 profile、当前约 650 KB 黑盒脚本：

| 阶段 | 基线 |
|---|---:|
| 随机 profile 生成（平均） | 1.197 ms |
| Worker 冷启动（平均） | 48.176 ms |
| profile 重载（平均） | 33.399 ms |
| DV preload | 0.330 ms |
| JavaScript evaluate | 2956.496 ms |
| 捕获请求导出 | 0.404 ms |
| 一次性 Worker 总流程 | 3045.482 ms |
| 空 Worker RSS | 约 50 MB |
| 脚本完成后 Worker RSS | 约 163 MB |

脚本执行期间墙钟约 2.956 秒，Worker CPU 约 0.641 秒，CPU 比例约 21.7%；
约 78% 时间属于真实浏览器定时器等待。该部分不能通过虚假推进时钟优化。

## 4C4G 自适应 Worker

HTTP 并发数与同时运行的 V8 进程数分离。服务可以接收 20 个并发请求，但不会
在 4 GB 主机上创建 20 个同时活跃的完整 V8 进程。

自动 Worker 数同时受以下条件约束：

- CPU：逻辑 CPU 数乘 2，适配当前定时器等待占比；
- 内存：预留 768 MiB 给 OS/Python，每个活跃 Worker 按 384 MiB 安全预算；
- 自动上限：8；
- Linux 同时识别 cgroup v1/v2 的 CPU quota 和 memory limit。

因此 4 核 4 GB 默认选择 8 个活跃 Worker。20 个 HTTP 请求会进入同一个有界池，
超出的请求排队；V8 的 30 秒执行超时从任务真正进入 Worker 后开始。

20 并发实测：

| 活跃 Worker | 成功数 | 总耗时 | 子进程 RSS 峰值合计 |
|---:|---:|---:|---:|
| 4 | 20/20 | 15.977 s | 667,578,368 bytes |
| 8 | 20/20 | 10.047 s | 1,327,673,344 bytes |

两组测试均未传 `logFlg`，全部响应都没有 `console` 字段。

## DT 生成优化

原实现每个请求同步等待一次 ExecJS/Node 启动，平均约 24.5 ms。不能复用同一个
持久 JS 上下文，因为 `encrypt_dt.js` 内部状态会改变 DT 的随机分布。

现实现使用容量 32 的后台有界 reservoir：

- 每个 DT 仍通过原来的 fresh-per-call `encrypt_dt.js/get_dt()` 生成；
- 后台始终最多启动一个 Node 生成任务，不产生并发 Node 内存峰值；
- 请求只从队列取一个已经独立生成的 DT；
- 队列预热完成后，单次获取平均约 0.002 ms，最大约 0.018 ms；
- 实测输出长度仍覆盖 42、43、44、45，与 fresh ExecJS 分布一致；
- 服务关闭时后台线程停止。

## 主要剩余成本

- 约 2.3 秒以上是脚本真实等待，不应破坏浏览器时钟语义；
- Worker 冷启动与 profile 重载合计约 81 ms；
- release DLL 映射在活跃 Worker 中约 78 MB，页面大部分来自版本化浏览器表；
  DLL 页面可由 OS 跨进程共享，不能直接把各 Worker RSS 相加当成物理内存；
- 后续若压缩版本表，需要避免把共享只读页变成每个 Worker 私有解压堆。

## 回归

- W6 sandbox executor API：5/5；
- 既有 W6 Worker API：4/4；
- 20 并发实际脚本：20/20；
- 普通与 `logFlg` 路径均成功；
- 持续 RSS 超限仍会终止 Worker 并自动恢复。
