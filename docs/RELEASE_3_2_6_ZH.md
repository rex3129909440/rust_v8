# rexiaohe-sandbox 3.2.6 修复与发布记录

## 范围

本版本仅发布 Linux x86-64 wheel。黑盒 JavaScript 只作为不可见输入执行，未读取或分析其源码。

## 修复项

1. Windows 随机 profile 现在会按种子组合 `navigator.maxTouchPoints`，并以成对值配置 `screen.colorDepth` 与 `screen.pixelDepth`。Windows 桌面候选覆盖 `0/5/10` 触点及 `24/24`、`32/32` 色深组合，Mac 与 Android 保持各自平台规则。
2. 隔离 Worker 初始化与重新初始化不再被固定 30 秒窗口错误截断。配置的执行超时大于 30 秒时，初始化等待窗口同步延长并保留 100 ms 收尾宽限；短超时仍保留 30 秒最低启动窗口。
3. Rust 集成测试从已移除的独立 Worker EXE 迁移到 DLL/SO 自托管进程隔离路径。Linux/macOS 直接覆盖自托管集成测试，Windows 的动态库发布形态由 Python/native wheel 烟测覆盖。
4. Linux wheel CI 在打包前执行 release 模式的 248 项 native 库回归；构建 wheel 后，再对实际安装的 SO 执行进程隔离、profile 重初始化、一次性 Worker 池和 XHR 网络捕获烟测。避免仅构建成功但发布物无法工作，同时避免为多个 V8 集成测试目标重复执行 release LTO 链接。
5. `document.all` 的 legacy callable/undetectable V8 模板绑定此前只声明了 MSVC C++ ABI 符号，Linux/macOS 会退化为普通对象。现在补充 Itanium C++ ABI 绑定，使 `typeof document.all === "undefined"`、布尔假值、`== null` 与函数调用语义在 DLL/SO 上一致。

## 本地验证

- `cargo fmt --all -- --check`
- `cargo test --all-targets`：248 项通过，0 项失败；Windows 上仅适用 Linux/macOS 自托管加载方式的集成测试按平台跳过。
- `pytest` profile/catalog/native 回归：39 项通过。
- profile 专项回归：22 项通过，覆盖随机触点、色深、平台一致性与 native profile schema。

## 发布约束

- wheel 版本：`3.2.6`
- 平台标签：`py3-none-manylinux_2_28_x86_64`
- 包名：`rexiaohe-sandbox`
- 发布物不得包含 `demo/ips*.js`、用户业务文件、黑盒输出或 `token.txt`。
