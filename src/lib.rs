mod console_capture;
mod content_encoding;
#[cfg(test)]
mod crypto_full_tests;
mod determinism;
#[cfg(test)]
mod dom_full_tests;
#[cfg(test)]
mod edge_evidence_tests;
#[cfg(test)]
mod event_full_tests;
mod execution_config;
pub mod ffi;
mod fingerprint;
mod fingerprint_environment;
#[cfg(test)]
mod fingerprint_environment_tests;
#[cfg(test)]
mod fingerprint_full_tests;
mod fingerprint_performance;
mod fingerprint_surface;
mod iframe_hook;
#[cfg(test)]
mod iframe_hook_tests;
#[cfg(test)]
mod iframe_window_proxy_tests;
mod intrinsics;
mod isolated_runtime;
#[cfg(test)]
mod iterator_full_tests;
mod locale_runtime;
#[cfg(test)]
mod media_element_tests;
#[cfg(test)]
mod message_clone_tests;
#[cfg(test)]
mod navigator_fingerprint_tests;
mod network_capture;
mod network_replay;
#[cfg(test)]
mod p0_tests;
mod page_init;
#[cfg(test)]
mod page_init_tests;
#[cfg(test)]
mod proxy_trace_full_tests;
#[cfg(test)]
mod realm_matrix_tests;
pub mod runtime;
mod runtime_options;
mod trace;
#[allow(
    clippy::all,
    dead_code,
    unused_assignments,
    unused_imports,
    unused_mut,
    unused_variables
)]
mod web;
mod webidl;
#[cfg(test)]
mod worker_full_tests;
#[cfg(test)]
mod worker_lifecycle_tests;

pub use console_capture::{CapturedConsoleOutput, ConsoleLevel, ConsoleValue};
pub use execution_config::{DeterministicExecution, SandboxLimits};
pub use fingerprint::{
    EdgeFingerprint, NavigatorFingerprint, NetworkFingerprint, SpeechFingerprint,
    SpeechVoiceFingerprint, UserAgentBrandFingerprint, UserAgentDataFingerprint,
};
pub use fingerprint_environment::{
    BatteryFingerprint, BluetoothDeviceFingerprint, CssFingerprint, DocumentFingerprint,
    FontFingerprint, FontMetricFingerprint, GamepadFingerprint, GeolocationFingerprint,
    HardwareDevicesFingerprint, HidDeviceFingerprint, KeyboardLayoutEntryFingerprint,
    LocalFontFingerprint, MediaDeviceFingerprint, MediaFingerprint, MediaPreferencesFingerprint,
    MemoryFingerprint, MidiPortFingerprint, MimeTypeFingerprint, PermissionsFingerprint,
    PluginFingerprint, PluginListFingerprint, RtcCodecFingerprint, RtcHeaderExtensionFingerprint,
    SensorsFingerprint, SerialPortFingerprint, TimingFingerprint, UsbDeviceFingerprint,
    XrFingerprint,
};
pub use fingerprint_performance::{PerformanceEntryFingerprint, PerformanceFingerprint};
pub use fingerprint_surface::{
    AudioFingerprint, CanvasFingerprint, LocaleFingerprint, RenderingFingerprint,
    ScreenFingerprint, StorageFingerprint, WebGlFingerprint, WebGpuFingerprint,
};
pub use iframe_hook::IframeHook;
pub use isolated_runtime::{IsolatedEdgeRuntime, run_isolated_worker};
pub use network_capture::{CapturedNetworkRequest, NetworkRequestSource};
pub use network_replay::NetworkReplayEntry;
pub use page_init::PageInit;
pub use runtime::{EdgeRuntime, Evaluation};
pub use runtime_options::EdgeRuntimeOptions;
pub use trace::TraceEntry;
