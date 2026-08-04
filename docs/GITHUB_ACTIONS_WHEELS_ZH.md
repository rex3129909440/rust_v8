# GitHub Actions 三平台 Python Wheel

## 1. 产物形式

Python 绑定通过 `ctypes` 调用稳定的 C ABI，不是与某一个 CPython 小版本绑定的
扩展模块。因此项目生成的是 `py3-none-<platform>` Wheel，每个 Wheel 包含：

- `edge_sandbox` Python 包；
- 当前平台唯一的原生库；
- 不包含项目 Worker EXE，也不依赖外部 Worker EXE。

当前工作流生成四个独立产物：

| Actions Artifact | Wheel 平台 | 内置原生库 |
| --- | --- | --- |
| `edge-sandbox-windows-x64` | `win_amd64` | `edge_sandbox.dll` |
| `edge-sandbox-linux-x64` | `manylinux_2_28_x86_64` | `libedge_sandbox.so` |
| `edge-sandbox-macos-arm64` | `macosx_12_0_arm64` | `libedge_sandbox.dylib` |
| `edge-sandbox-macos-x64` | `macosx_12_0_x86_64` | `libedge_sandbox.dylib` |

Python 最低版本为 3.11。`py3-none` 表示原生库不依赖 CPython 的私有 ABI；它不表示
Wheel 可以跨操作系统使用，最终的平台标签仍会限制 `pip` 只能安装匹配的 OS 和 CPU
版本。

## 2. 工作流文件

工作流位于 `.github/workflows/python-wheels.yml`。它只在以下场景运行：

1. GitHub 仓库的 Actions 页面手动选择 `build Python wheels`，点击
   `Run workflow`；
2. 推送形如 `v0.1.0` 的 Git tag。

没有配置为每次普通 push 都构建，因为 V8 的四平台 Release 构建耗时和 CI 用量都比较
高。

各平台都执行完整链路：

1. 在对应的原生 GitHub runner 上安装 Rust 1.93；
2. 执行 `cargo build --release --lib --locked`；
3. 将 DLL、SO 或 DYLIB 放入 Wheel 的 `edge_sandbox/_native`；
4. 构建平台 Wheel；
5. 用 `pip` 安装刚生成的 Wheel；
6. 创建真实隔离 Worker，执行带 `source_url` 的 `1 + 1` 冒烟测试；
7. 只有加载和执行都成功才上传 Actions Artifact。

Linux 不直接在 Ubuntu 24.04 上伪装成 manylinux。它在 PyPA
`manylinux_2_28_x86_64` 构建容器内编译，再由 `auditwheel repair` 检查 ELF
依赖并修复 Wheel。这样可避免发布一个依赖过新 glibc、却带有错误兼容标签的 SO。

## 3. 下载与安装

工作流完成后，在该次 Actions run 的 `Artifacts` 区域下载与目标机器匹配的产物。
解压 Artifact 后安装其中的 Wheel：

```python
import subprocess
import sys

subprocess.run(
    [
        sys.executable,
        "-m",
        "pip",
        "install",
        r"D:\downloads\edge_sandbox-0.1.0-py3-none-win_amd64.whl",
    ],
    check=True,
)
```

安装后不需要传入 DLL 路径：

```python
from edge_sandbox import EdgeSandbox

with EdgeSandbox() as sandbox:
    result = sandbox.evaluate(
        "({ value: 40 + 2 }).value",
        source_url="app://production/example.js",
    )

print(result)
```

`find_native_artifacts()` 会优先读取已安装包内的 `_native` 目录；在源码目录直接运行时，
仍兼容原来的 `target/release` 和 `target/debug` 查找路径。

## 4. 发布版本

创建正式版本时，先确保 `Cargo.toml` 中的 `package.version` 正确，再推送同版本 tag：

```text
v0.1.0
```

Wheel 版本自动读取 `Cargo.toml`，不维护第二份版本号。本工作流默认只上传 Actions
Artifact，不会自动发布 PyPI，也不需要 PyPI token。若以后需要发布 PyPI，建议另建一个
只在 GitHub Release 审批后运行的发布 job，并使用 PyPI Trusted Publishing。

## 5. 二进制大小

Windows Release DLL 当前约 71.9 MB，其中主要是静态链接的 V8、ICU 和机器码/只读
数据。PDB 是独立调试文件，不会放入 Wheel。实测 Windows Wheel 约 25.5 MB，因为
Wheel 会压缩 DLL。

当前 Release 已启用 `strip = true`、thin LTO 和单 codegen unit。继续改为
`opt-level = "z"` 只会优化 Rust/Web API 胶水层，对已经预编译的 V8 静态库作用有限。
若禁用 ICU/Intl、移除 V8 功能或把 V8 拆成额外动态库，才可能显著缩小单个 DLL，但会
破坏当前 Edge API/时区/Intl 兼容性，或者把体积转移到第二个文件，因此不作为默认构建。

不建议对生产 DLL 使用 UPX：Wheel 已经提供传输压缩，而 UPX 会增加加载开销，并可能
影响代码签名、CFG/安全软件兼容性。Windows PDB 可以作为单独调试 Artifact 保存，但不应
放入面向生产环境的 Wheel。
