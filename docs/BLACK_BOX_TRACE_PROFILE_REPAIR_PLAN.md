# Edge 沙箱黑盒 Trace、Profile 与 Rust 修复计划

## 目标

以 `demo/ips.js`、`demo/ips1.js` 和 `demo/ips2.js` 作为三份不透明的黑盒一致性样本，在不读取或解释其源码的前提下，使用沙箱原生 trace 与 typed stdout 找出：

1. 已暴露但缺少实现的 Edge API；
2. 返回值、异常、原型或调用语义偏离 Edge 的 API；
3. 当前被固定在 Rust 或默认 profile 中、但真实设备上应随设备变化的浏览器字段；
4. `TextEncoder.prototype.encode` 输入中能够与浏览器 API trace 返回值直接进行字面匹配的字段；
5. profile 已提供配置、但 Rust 运行时没有正确消费或传播的字段。

确认问题后修复 Rust 源码和必要的 Python profile 映射，完成本地构建、回归测试、提交、GitHub 推送和多平台 CI 检查。

## 强制边界

- 不读取、搜索、反格式化、反混淆或解释 `demo/ips.js` 源码。
- 不分析业务协议、请求签名、请求体结构、算法、站点安全逻辑或检测规则。
- 不访问外部业务目标，不对真实目标发送请求。
- 不解码大型 `TextEncoder` 聚合载荷，只允许“trace 字面值是否原样出现在输入中”的直接关联。
- 不把 Edge/Chromium 固定不变量误改成随机值。
- 不为消除报错而加入占位符；新增 profile 值必须有本地 Edge 证据、公开浏览器语义证据或真实设备采集依据。
- `ips.js`、trace、stdout、失败 profile、业务脚本及本地采集结果只留在本机，不提交 GitHub、不打包、不发布。

## 阶段一：生成可复现的黑盒证据

1. 固定运行时随机种子、页面地址、时钟策略、超时和 trace 排除项。
2. 三份样本分别使用相同的 Windows profile seed 集合，执行独立 Worker，形成对照组。
3. 每组执行结束后立即销毁 Worker。
4. 每组保存：
   - 原生 trace；
   - typed stdout；
   - profile catalog ID；
   - CPU、内存、屏幕、WebGL 等选择结果；
   - `/tl` 是否出现、执行异常和请求计数。
5. 完整数据写入 `build/ips-trace-audit-20260807/`，终端不展开大 trace。

完成条件：三份不透明输入在相同的多个 profile 下都有可复现、可比较、Worker 正常释放的本地样本。

## 阶段二：流式汇总与固定字段识别

1. 每份 trace 只流式扫描一次，生成按 API path 聚合的 TSV。
2. 从 typed stdout 中只提取 `TextEncoder.prototype.encode` 的原始字符串参数。
3. 生成 trace 返回值/参数与 TextEncoder 输入的直接字面关联表。
4. 先在同一文件内跨 seed 比较，再在三份对照文件之间比较同一 API path 的结果；相同 encoder 片段如果没有 trace API 来源，不作为沙箱硬编码证据。
5. 跨 seed 比较同一 API path 的结果：
   - 始终相同且属于浏览器规范不变量：保留固定；
   - 始终相同但真实设备应变化：列为 profile/Rust 候选；
   - profile 已变化但 trace 不变化：列为 Rust 映射缺陷；
   - trace 变化且组合关系合理：不修改。

完成条件：每个修复候选都具备 API path、trace 序号、返回值、涉及的 profile 字段以及是否直接出现在 TextEncoder 输入中的证据。

## 阶段三：Edge 语义核对

候选项按以下优先级核对：

1. 仓库内已有 Edge 实机采集和回归测试；
2. Chromium/Edge 的标准行为或官方公开资料；
3. 用户提供的真实设备采集。

重点核对：

- `MediaRecorder.isTypeSupported` 与媒体能力集合；
- 键盘布局和 locale 相关映射；
- DOM/CSS 布局值与输入控件矩形；
- Plugin/MimeType 构造器、原型和异常语义；
- WebGL、screen、DPR、CPU、内存与 profile 的传播；
- storage、speech、timezone、language 等已配置字段是否真正生效。

完成条件：没有 Edge 证据的猜测不进入实现。

## 阶段四：Rust 与 Profile 修复

1. 优先修复 Rust API 函数体、对象关系和配置消费路径。
2. 只有真实设备可变字段才扩展 profile；浏览器固定语义留在 Rust。
3. 每个 API 独立实现，禁止名称循环生成 API，禁止 JSON 字符串形式的 ABI 配置，禁止 JavaScript `Proxy`。
4. trace 继续走透明 V8/Rust 拦截路径，关闭时不记录并避免额外热路径开销。
5. 保证 Windows、macOS、Android 分支互不硬编码覆盖。

完成条件：修复项在 profile 配置改变时能够正确传播，同时对象形态、原型链、函数行为和异常语义保持 Edge 对齐。

## 阶段五：本地验证

1. 运行受影响 Rust 单元测试和 Python profile 测试。
2. 构建 release DLL。
3. 用最小 API 探针验证每个修复点。
4. 再次执行同一黑盒矩阵，比较修复前后 trace 摘要。
5. 验证：
   - `/tl` 捕获链路仍工作；
   - Worker 在每次执行后销毁；
   - trace 关闭时无记录开销；
   - stdout、network request 与内存释放接口正常；
   - 没有把不变量随机化。

完成条件：针对性测试和黑盒回归均通过，且没有新增崩溃、超时或 Worker 泄漏。

## 阶段六：GitHub 与 CI

1. 查看远端、当前分支、工作树和 diff。
2. 仅暂存本次沙箱源码、profile、测试、工具和本文档。
3. 明确排除：
   - `demo/ips.js`；
   - `demo/wizzair*.py` 及其他业务测试文件；
   - `build/` 下 trace/stdout/失败样本/二进制；
   - 用户采集的原始设备文件；
   - token、代理、URL、header、请求体和本地配置。
4. 提交并推送 GitHub。
5. 启动仓库已有的多平台 wheel CI，检查 Windows、Linux、macOS 各任务状态和日志。
6. CI 失败时只根据构建日志修复跨平台源码或工作流，不擅自发布 PyPI。

完成条件：授权范围内的提交已推送，CI 已启动并给出各平台明确状态；PyPI 发布需用户另行明确授权和提供已构建制品。

## 交付物

- 本计划文档；
- 本地黑盒样本清单和紧凑 TSV 汇总；
- 修复候选与证据对应表；
- Rust/profile 修复和回归测试；
- Git 提交哈希与 GitHub CI 运行链接/状态。
