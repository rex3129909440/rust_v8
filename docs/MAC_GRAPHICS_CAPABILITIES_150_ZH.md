# Chromium 150 macOS 图形能力资料

本地能力目录位于 `demo/fp/mac_graphics_capability_catalog.py`。目录只保存能够由指定版本源码或 Apple 官方能力表确认的值；没有用占位值，也没有从业务脚本推断任何内容。

## 版本锁定

- Chromium branch-head：`7879`，提交 `6b856e0afe5890e231137725eac0449907f4fdb2`
- ANGLE `chromium/7879`：提交 `e53ecb3f8dbd797748dd21eea0a5606b54d82802`
- Dawn `chromium/7879`：提交 `23cf554e645f61acabcd10aac24bfe6d6b0eeeec`

## 本地数据的来源

- [ANGLE Metal DisplayMtl.mm](https://chromium.googlesource.com/angle/angle/+/e53ecb3f8dbd797748dd21eea0a5606b54d82802/src/libANGLE/renderer/metal/DisplayMtl.mm)：WebGL 上限、Metal renderer description、扩展初始化。
- [ANGLE Metal mtl_common.h](https://chromium.googlesource.com/angle/angle/+/e53ecb3f8dbd797748dd21eea0a5606b54d82802/src/libANGLE/renderer/metal/mtl_common.h)：采样器、UBO、默认 uniform 等 Metal 后端常量。
- [ANGLE Constants.h](https://chromium.googlesource.com/angle/angle/+/e53ecb3f8dbd797748dd21eea0a5606b54d82802/src/libANGLE/Constants.h)：transform feedback 与 draw buffer 常量。
- [Dawn Metal PhysicalDeviceMTL.mm](https://dawn.googlesource.com/dawn/+/23cf554e645f61acabcd10aac24bfe6d6b0eeeec/src/dawn/native/metal/PhysicalDeviceMTL.mm)：Mac2 物理能力、WebGPU feature、subgroup 与 architecture 语义。
- [Dawn Limits.cpp](https://dawn.googlesource.com/dawn/+/23cf554e645f61acabcd10aac24bfe6d6b0eeeec/src/dawn/native/Limits.cpp)：WebGPU adapter limit tiers。
- [Chromium WebGPU decoder](https://chromium.googlesource.com/chromium/src/+/6b856e0afe5890e231137725eac0449907f4fdb2/gpu/command_buffer/service/webgpu_decoder_impl.cc)：默认启用 Dawn tiered adapter limits。
- [Apple Metal Feature Set Tables](https://developer.apple.com/metal/capabilities/)：M1/M2/M3/M4/M5 到 Apple7/8/9/9/10 的映射、纹理格式与 MSAA 能力。

## 已确认的差异

| 系列 | Metal family | WebGL `MAX_SAMPLES` | WebGPU architecture | 压缩纹理 |
|---|---:|---:|---|---|
| M1 | Apple7 | 4 | `metal-3` | BC、ETC2/EAC、ASTC |
| M2 | Apple8 | 4 | `metal-3` | BC、ETC2/EAC、ASTC |
| M3 | Apple9 | 4 | `metal-3` | BC、ETC2/EAC、ASTC |
| M4 | Apple9 | 4 | `metal-3` | BC、ETC2/EAC、ASTC |
| M5 | Apple10 | 8 | `metal-3` | BC、ETC2/EAC、ASTC |

ANGLE 在 Chromium 150 的 macOS Metal 路径把 WebGL 2D/cube/renderbuffer/viewport 上限限制为 16384。因此即使 Apple10 的原始 Metal 2D texture 上限是 32768，WebGL 观察值仍使用 ANGLE 的 16384。Dawn 在 macOS 上先选择 Mac2 表，再由 Chromium 应用 tier，所以这里的 WebGPU texture dimension 也仍为 16384。

## 不猜填的边界

Intel/AMD Mac 的设备名与机器组合仍保留在硬件目录中，但默认随机池不再选择它们。原因是 ANGLE 的最大采样数来自 `MTLDevice.supportsTextureSampleCount`，Dawn 的 buffer 上限来自 `MTLDevice.maxBufferLength`，公开产品规格没有给出目录中每块旧 GPU 的完整运行时结果。只有取得相同 Chromium 版本在真实机器上的本地、非业务能力探针结果后，才应将对应行标记为可用。
