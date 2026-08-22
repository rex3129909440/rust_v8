# Android speechSynthesis 与 Media profile 修复记录（2026-08-22）

## 修复范围

本轮只修改 Android Chromium / Android WebView profile 与对应的 WebView
全局属性暴露边界，不改变 Windows、macOS 和普通桌面 Chromium 分支。

## speechSynthesis

新增 `demo/fp/android_speech_synthesis_voice_catalog.py`：

- `get_random_fp_details(country_code, android_ua, seed=...)` 根据国家已选择的
  `navigator.languages` 生成 Android TTS locale 子集。
- 同一显式 seed 得到相同 voice 列表；未指定 seed 时每次重新选择。
- voice 形态为 `voiceURI/name/lang/localService/default`。
- 名称遵循 Chromium Android 实现：`Locale.getDisplayLanguage()` 加空格和
  `Locale.getDisplayCountry()`，不再错误混用 Microsoft/Apple 桌面 voice。
- 最多五个 locale，首个国家主语言为默认 voice；次要语言和 `en-US` 后备按
  独立随机选择进入已安装语言子集。

边界：Pixel 4 上官方 Android WebView 140、149、150 的 HTTPS 实测都没有
`window.speechSynthesis`，完整 Window 表也不包含 SpeechSynthesis 构造器。
因此 Rust 安装路径现在按版本/平台 Window 表控制这些属性：Android Chrome
可以使用 country voice profile；官方 WebView 直接访问得到 `undefined`，而不是
仅从 `Object.getOwnPropertyNames` 隐藏后仍可被直接访问。

## Media

新增 `demo/fp/android_media_capability_catalog.py`，不再把下列 API 当成同一份
能力表：

1. `HTMLMediaElement.canPlayType()`：浏览器识别并可播放的 MIME/codec。
2. `MediaSource.isTypeSupported()`：可追加到 MSE SourceBuffer 的流格式。
3. `MediaRecorder.isTypeSupported()`：可作为录制输出的格式。
4. `navigator.mediaCapabilities.decodingInfo()`：supported、smooth 与
   powerEfficient 三层结果。
5. `navigator.mediaCapabilities.encodingInfo()`：与 MediaRecorder 独立。
6. WebCodecs Audio/Video decoder/encoder codec 列表。

Android 设备仍来自现有成组设备 catalog。能力按 Android 版本与设备
`mediaTier` 派生：

- Android 5+：Opus、HEVC、VP9 路径。
- Android 10+：AV1 软件解码路径可用。
- `av1-hardware` 设备：AV1 同时进入 power-efficient 列表。
- 其他设备：AV1 可以 supported/smooth，但不报告 power-efficient。
- `restrictOwnAudio` 延续 Chromium 141+ 的版本门控。

Pixel 4 / Android 11 / WebView 150 的实测特殊项已经作为 WebView 基线：

- canPlayType：AAC、Opus、Vorbis、FLAC、WAV、H.264、HEVC、VP8、VP9、AV1
  返回 `probably`。
- MediaSource：AAC/Opus/H.264/HEVC/VP8/VP9/AV1 为 true；Vorbis/FLAC/WAV
  为 false。
- MediaRecorder：AAC/Opus/H.264/HEVC/VP8/AV1 为 true，VP9 为 false。
- MediaCapabilities decodingInfo：上述测试解码项 supported/smooth 为 true；
  Pixel 4 的 AV1 powerEfficient 为 false。
- MediaCapabilities encodingInfo：实测的 `type: "webrtc"` 项均为 false。
  这不会覆盖 MediaRecorder 的 MIME 支持结果。

## 证据来源

- Android Voice：<https://developer.android.com/reference/android/speech/tts/Voice>
- Android TextToSpeechService：
  <https://developer.android.com/reference/android/speech/tts/TextToSpeechService>
- Chromium Android TTS 实现：
  <https://chromium.googlesource.com/chromium/src/+/705855f311cc054f3b1517808f41b51e5a8ffd4e/content/public/android/java/src/org/chromium/content/browser/TtsPlatformImpl.java>
- Android 支持的媒体格式：
  <https://developer.android.com/media/platform/supported-formats>
- Chromium Android MediaCodecUtil：
  <https://chromium.googlesource.com/chromium/src/+/master/media/base/android/java/src/org/chromium/media/MediaCodecUtil.java>
- Chromium MediaCapabilities：
  <https://chromium.googlesource.com/chromium/src/+/4506bddb89f547500c9b6e38754d7fa20d6449cf/third_party/blink/renderer/modules/media_capabilities/media_capabilities.cc>
- 本地实机证据：
  `build/android-webview-speech-media/pixel4-webview150-speech-media.json`

## 回归测试

- `tests/python_android_speech_media_profile.py`
  - US/JP/DE/HK 国家语音选择；
  - seed 可复现；
  - Pixel 4 WebView Media 四层能力；
  - 软件 AV1 与硬件 AV1 的 powerEfficient 分离；
  - Chromium 140/141 `restrictOwnAudio` 门控。
- `src/iframe_window_proxy_tests.rs`
  - WebView 140-150 不再直接泄漏 SpeechSynthesis 全局对象。
