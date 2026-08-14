# Android 随机设备族 Profile 修复记录

日期：2026-08-14

## 修复目标

Pixel 4 的 HTTPS 实机数据只是一条设备证据，不能作为全部 Android
设备的默认值。本次把 Android 随机 profile 改成“先选择一台完整设备，
再从这台设备派生全部相关表面”的模型，禁止型号、系统、GPU、内存、屏幕、
字体和媒体能力分别自由拼接。

本次只修改沙箱 profile 组合与回归测试，没有读取或分析业务 JavaScript，
没有上传仓库、构建 wheel 或发布 PyPI。

## 已修复的组合关系

1. Chromium 的冻结 Android UA `Android 10; K` 只作为低熵 UA 文本保留，
   不再被解释为真实 Android 10 设备或型号 `K`。`Sec-CH-UA-Model` 和
   `Sec-CH-UA-Platform-Version` 从选中的真实设备记录派生。
2. Chrome/Edge 140–151 的 Android 完整版本使用移动发布版本表，不再复用
   桌面补丁版本。
3. 每条设备记录绑定以下不可拆分字段：
   - 可运行 Android 版本范围；
   - OEM、型号、CPU 逻辑核数和物理内存选项；
   - CSS 屏幕宽高、DPR、触控点数；
   - GPU vendor/model、ANGLE renderer、GPU 家族能力表；
   - WebGPU 可用性与 Android 版本门槛；
   - OEM/Android 版本字体表；
   - 媒体硬件能力档位。
4. Pixel 4 的 6 GiB 物理内存、4 GiB `deviceMemory` 桶、
   `1530000000` heap limit 和实测 WebGL2 扩展只绑定 Pixel 4，不再扩散到
   其他设备。
5. 其他 GPU 使用独立家族能力档：Adreno 7xx、Adreno 6xx、Adreno 5xx、
   Adreno 4xx、Mali Valhall、Mali Bifrost、Samsung Xclipse/RDNA2 和
   PowerVR Rogue。不同家族会生成不同的 texture/renderbuffer/viewport、
   samples、uniform limits、压缩格式和扩展集合。
6. Android clean-profile 不再继承 Windows 的 5 个 PDF plugin，也不再注入
   Microsoft 桌面语音。`plugins`、`mimeTypes` 和初始化阶段 voice list 均为
   Android 实机证据对应的空状态。
7. Android 不再继承 Windows input 控件 CSS。新增 Android Chromium 控件
   主题，Pixel 4 只用于校准 Android Blink 主题：text/search 192×21.2727、
   checkbox/radio 16×16、date/time/month/week 138.784×21.0909、
   button 15.6364×21.2727、file 254.909×21.2727；边框按设备 DPR 对齐到
   物理像素。
8. 字体按 Android 版本和 OEM 选择。Android 10/11 不再出现 Roboto Flex；
   Android 14+ 可包含 Roboto Flex；Samsung 设备单独加入 SamsungOne 与
   Samsung Sans，其他 OEM 不会混入。
9. Chromium 141+ 的媒体约束加入 `restrictOwnAudio`。AV1 是否属于
   power-efficient 解码集合由设备媒体档位决定，不再对全部手机固定。
10. foldable 设备在没有同时切换铰链和 active-screen geometry 时只返回
    `continuous`，避免“姿态 folded、屏幕仍是普通展开尺寸”的矛盾组合。

## 当前设备覆盖

目录中共有 20 条 Android 设备记录。Chrome/Edge 140–151 要求 Android 10+
后，当前随机池实际可选 17 个型号：

- Google：Pixel 3、4、5、6、7、8、Pixel 9 Pro Fold；
- Samsung：Galaxy Z Fold 5/6、S24 Ultra、A55、S20 Qualcomm/Exynos、
  A71、A51；
- Microsoft：Surface Duo；
- Motorola：moto g power (2022)。

Galaxy S8+ 两个 SoC 版本和 Moto G4 仍保留在历史设备目录，但其最高 Android
版本低于 Chromium 140–151 的最低系统要求，因此不会进入当前随机池。

## 证据来源

- Chrome DevTools 官方设备目录（型号、CSS screen、DPR、UA-CH model）：
  <https://github.com/ChromeDevTools/devtools-frontend/blob/main/front_end/models/emulation/EmulatedDevices.ts>
- Microsoft Edge Mobile Stable 发布记录：
  <https://learn.microsoft.com/en-us/deployedge/microsoft-edge-relnote-mobile-stable-channel>
- Chrome for Testing / Android 140–151 归档与本项目 HTTPS 取证：
  `build/chromium-android-version-surfaces/`
- Pixel 4 实机：项目保存的 Chromium 140–151 HTTPS 证据矩阵；本轮另用已连接
  Pixel 4、Chrome 149 在 `https://example.com/` 复核完整 input 几何。
- Android WebGL 数值档位来自 Web3D Survey 的 Android 实际分布；设备与 GPU
  的关联来自对应厂商规格和 Dawn GPU inventory：
  <https://web3dsurvey.com/webgl/parameters/MAX_TEXTURE_SIZE>
- AOSP 字体目录：
  <https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/data/fonts/fonts.xml>
- SamsungOne 官方设计说明：
  <https://design.samsung.com/global/contents/samsung-one/>

## 验证结果

- Android 专项及相关 profile 测试：19/19 通过。
- Python 全量 `python_*.py` 测试：72 项中 71 项通过；唯一失败是既有 Mac
  测试的“未绑定字段白名单”没有包含此前已经存在的
  `NetworkInformation.connectionType/downlinkMax`，与本次 Android 修改无关。
- 5000 组组合覆盖 10 个国家、Chrome/Edge 各 2500 组、Chromium 140–151
  均匀取样：17 个可用型号全部命中，形成 748 个完整组合，内部一致性问题 0。
- 每个 Android 10+ 设备型号均使用显式 UA 单独物化测试，验证 model、OS、
  GPU、screen、DPR、OEM font、plugins、speech 和 WebGL 后端没有串线。

## 证据边界

Pixel 4 是当前唯一拥有本项目完整 HTTPS WebGL 参数实机捕获的 Android 型号。
其他设备的型号/屏幕/SoC/RAM 关系来自公开设备资料，WebGL limits 使用对应 GPU
家族的真实观测档位，而不是声称已经逐台实机逐参数采集。后续取得其他设备的
完整 HTTPS 捕获后，应增加新的 device-capture 行并替换对应家族档位；不得为
增加随机数量而独立随机单个 WebGL limit。
