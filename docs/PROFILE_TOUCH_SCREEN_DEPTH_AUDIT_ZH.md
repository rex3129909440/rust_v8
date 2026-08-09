# 随机 Profile：触控点与屏幕色深修复

日期：2026-08-09

## 修复原因

用户提供的真实 Windows 浏览器观测为：

- `navigator.maxTouchPoints = 10`
- `screen.colorDepth = 32`
- `screen.pixelDepth = 32`

修改前，Windows PC 硬件目录虽然已经包含 `maxTouchPoints` 为 `0/5/10` 的真实硬件行，但触控行在大目录中的权重过低。1000 个 seed 的实际结果是 `0:971、5:16、10:13`，因此普通测试很容易只看到 0。

Windows 屏幕构造器则确实始终写入 `24/24`，无法生成用户实测的 `32/32`。

## 实现

- Windows PC 继续使用现有硬件行生成 `0/5/10`，没有把触控点与 CPU、内存、form factor 拆开随机；
- 对真实触控硬件行增加目录内抽样权重，使 5 点和 10 点设备在普通随机测试中可见；
- Windows 屏幕增加成对的 `24/24` 与 `32/32` 选择；
- `colorDepth` 和 `pixelDepth` 作为原子 pair 选择，禁止产生 `24/32`、`32/24`；
- 选择结果写入 screen profile ID，例如 `_depth32`，相同 seed 可以稳定复现；
- macOS 保持 `maxTouchPoints=0`，屏幕深度保持现有 `24/24` 或 `30/30`；
- Android 保持 `maxTouchPoints=5`、`24/24`，不混入 Windows 实测值；
- `audit_random_fp()` 增加平台级允许值检查；
- `verify_random_fp()` 现在会从真实沙箱读取并核对 `colorDepth/pixelDepth`；

## 5000 个 Windows seed 的结果

| maxTouchPoints | colorDepth | pixelDepth | 数量 |
| ---: | ---: | ---: | ---: |
| 0 | 24 | 24 | 3803 |
| 0 | 32 | 32 | 948 |
| 5 | 24 | 24 | 122 |
| 5 | 32 | 32 | 36 |
| 10 | 24 | 24 | 70 |
| 10 | 32 | 32 | 21 |

所有六种组合均可达，且没有出现 color/pixel 深度不一致的组合。

## 沙箱运行时验证

seed 203 选择：

- 硬件行：`pc_10c_8g_touch10_surface`
- 屏幕行：`pc_1280x720_1x_lowend__win10_taskbar40_depth32`
- Python typed profile：`(10, 32, 32)`
- Rust/V8 实际读取：`(10, 32, 32)`

验证证明三个值不只是在 Python 对象中变化，已经通过 typed FFI 进入沙箱并由对应浏览器 API 暴露。
