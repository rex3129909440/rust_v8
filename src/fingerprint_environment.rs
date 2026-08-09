#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct FontFingerprint {
    pub families: Vec<String>,
    pub allow_unknown_families: bool,
    pub local_fonts: Vec<LocalFontFingerprint>,
    pub metrics: Vec<FontMetricFingerprint>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct FontMetricFingerprint {
    pub family: String,
    pub width_scale: f64,
    pub monospace: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct CssFingerprint {
    pub body: String,
    pub input_common: String,
    pub input_hidden: String,
    pub input_search: String,
    pub input_checkbox_radio: String,
    pub input_range: String,
    pub input_color: String,
    pub input_date: String,
    pub input_time: String,
    pub input_datetime_local: String,
    pub input_month: String,
    pub input_week: String,
    pub input_image: String,
    pub input_button: String,
    pub input_submit_reset: String,
    pub input_file: String,
    pub input_text: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct DocumentFingerprint {
    pub body_child_element_count: Option<u32>,
    pub body_client_height: Option<f64>,
    pub has_focus: Option<bool>,
    pub visibility_state: Option<String>,
    pub is_popup: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct LocalFontFingerprint {
    pub postscript_name: String,
    pub full_name: String,
    pub family: String,
    pub style: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct MediaDeviceFingerprint {
    pub device_id: String,
    pub kind: String,
    pub label: String,
    pub group_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct RtcCodecFingerprint {
    pub mime_type: String,
    pub clock_rate: u32,
    pub channels: Option<u16>,
    pub sdp_fmtp_line: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct RtcHeaderExtensionFingerprint {
    pub kind: String,
    pub uri: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct MediaFingerprint {
    pub devices: Vec<MediaDeviceFingerprint>,
    pub supported_constraints: Vec<String>,
    pub can_play_probably_types: Vec<String>,
    pub can_play_maybe_types: Vec<String>,
    pub media_source_types: Vec<String>,
    pub media_recorder_types: Vec<String>,
    pub decoding_supported_types: Vec<String>,
    pub decoding_smooth_types: Vec<String>,
    pub decoding_power_efficient_types: Vec<String>,
    pub encoding_supported_types: Vec<String>,
    pub encoding_smooth_types: Vec<String>,
    pub encoding_power_efficient_types: Vec<String>,
    pub image_decoder_types: Vec<String>,
    #[serde(default = "default_audio_decoder_codecs")]
    pub audio_decoder_codecs: Vec<String>,
    #[serde(default = "default_audio_encoder_codecs")]
    pub audio_encoder_codecs: Vec<String>,
    #[serde(default = "default_video_decoder_codecs")]
    pub video_decoder_codecs: Vec<String>,
    #[serde(default = "default_video_encoder_codecs")]
    pub video_encoder_codecs: Vec<String>,
    pub rtc_audio_codecs: Vec<RtcCodecFingerprint>,
    pub rtc_video_codecs: Vec<RtcCodecFingerprint>,
    pub rtc_header_extensions: Vec<RtcHeaderExtensionFingerprint>,
    pub rtc_offer_sdp: String,
    pub rtc_answer_sdp: String,
}

fn default_audio_decoder_codecs() -> Vec<String> {
    vec!["opus".to_owned()]
}

fn default_audio_encoder_codecs() -> Vec<String> {
    vec!["opus".to_owned()]
}

fn default_video_decoder_codecs() -> Vec<String> {
    vec!["vp8".to_owned()]
}

fn default_video_encoder_codecs() -> Vec<String> {
    vec!["vp8".to_owned()]
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct PermissionsFingerprint {
    pub accelerometer: String,
    pub background_sync: String,
    pub camera: String,
    pub clipboard_read: String,
    pub clipboard_write: String,
    pub geolocation: String,
    pub gyroscope: String,
    pub local_fonts: String,
    pub magnetometer: String,
    pub microphone: String,
    pub midi: String,
    pub notifications: String,
    pub payment_handler: String,
    pub persistent_storage: String,
    pub speaker_selection: String,
    pub storage_access: String,
    pub top_level_storage_access: String,
    pub window_management: String,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct BatteryFingerprint {
    pub charging: bool,
    pub charging_time: f64,
    pub discharging_time: f64,
    pub level: f64,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct GeolocationFingerprint {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub accuracy: f64,
    pub altitude_accuracy: Option<f64>,
    pub heading: Option<f64>,
    pub speed: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MediaPreferencesFingerprint {
    pub color_scheme: String,
    pub contrast: String,
    pub reduced_motion: bool,
    pub reduced_transparency: bool,
    pub reduced_data: bool,
    pub forced_colors: bool,
    pub inverted_colors: bool,
    pub monochrome_bits: u32,
    pub color_gamut: String,
    pub pointer: String,
    pub any_pointer: String,
    pub hover: String,
    pub any_hover: String,
    pub display_mode: String,
    pub dynamic_range: String,
    pub video_dynamic_range: String,
    pub scripting: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct MimeTypeFingerprint {
    pub mime_type: String,
    pub suffixes: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct PluginFingerprint {
    pub name: String,
    pub filename: String,
    pub description: String,
    pub mime_types: Vec<MimeTypeFingerprint>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct PluginListFingerprint {
    pub plugins: Vec<PluginFingerprint>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct GamepadFingerprint {
    pub id: String,
    pub index: u32,
    pub connected: bool,
    pub mapping: String,
    pub axes: Vec<f64>,
    pub buttons: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct UsbDeviceFingerprint {
    pub usb_version_major: u8,
    pub usb_version_minor: u8,
    pub usb_version_subminor: u8,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_version_major: u8,
    pub device_version_minor: u8,
    pub device_version_subminor: u8,
    pub manufacturer_name: Option<String>,
    pub product_name: Option<String>,
    pub serial_number: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct HidDeviceFingerprint {
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct SerialPortFingerprint {
    pub usb_vendor_id: u16,
    pub usb_product_id: u16,
    pub connected: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct BluetoothDeviceFingerprint {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct KeyboardLayoutEntryFingerprint {
    pub code: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct MidiPortFingerprint {
    pub id: String,
    pub manufacturer: String,
    pub name: String,
    pub version: String,
    pub state: String,
    pub connection: String,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct HardwareDevicesFingerprint {
    pub gamepads: Vec<GamepadFingerprint>,
    pub usb_devices: Vec<UsbDeviceFingerprint>,
    pub hid_devices: Vec<HidDeviceFingerprint>,
    pub serial_ports: Vec<SerialPortFingerprint>,
    pub bluetooth_available: bool,
    pub bluetooth_devices: Vec<BluetoothDeviceFingerprint>,
    pub keyboard_layout: Vec<KeyboardLayoutEntryFingerprint>,
    pub device_posture: String,
    pub midi_inputs: Vec<MidiPortFingerprint>,
    pub midi_outputs: Vec<MidiPortFingerprint>,
    pub midi_sysex_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SensorsFingerprint {
    pub available: bool,
    pub accelerometer: [f64; 3],
    pub gravity: [f64; 3],
    pub linear_acceleration: [f64; 3],
    pub gyroscope: [f64; 3],
    pub absolute_orientation_quaternion: [f64; 4],
    pub relative_orientation_quaternion: [f64; 4],
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TimingFingerprint {
    pub clock_epoch_ms: Option<i64>,
    pub clock_step_ms: u64,
    pub random_seed: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct XrFingerprint {
    pub supported_session_modes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct MemoryFingerprint {
    pub performance_js_heap_size_limit: u64,
    pub performance_total_js_heap_size: u64,
    pub performance_used_js_heap_size: u64,
    pub console_js_heap_size_limit: u64,
    pub console_total_js_heap_size: u64,
    pub console_used_js_heap_size: u64,
}

impl Default for FontFingerprint {
    fn default() -> Self {
        Self {
            families: vec![
                "Arial".to_owned(),
                "Calibri".to_owned(),
                "Cambria".to_owned(),
                "Consolas".to_owned(),
                "Courier New".to_owned(),
                "Microsoft YaHei".to_owned(),
                "Segoe UI".to_owned(),
                "Tahoma".to_owned(),
                "Times New Roman".to_owned(),
                "Verdana".to_owned(),
            ],
            // Preserve the former permissive behavior unless a caller opts
            // into strict installed-font emulation.
            allow_unknown_families: true,
            local_fonts: vec![LocalFontFingerprint {
                postscript_name: "EdgeSandboxSans-Regular".to_owned(),
                full_name: "Edge Sandbox Sans Regular".to_owned(),
                family: "Edge Sandbox Sans".to_owned(),
                style: "Regular".to_owned(),
            }],
            metrics: Vec::new(),
        }
    }
}

impl Default for CssFingerprint {
    fn default() -> Self {
        Self {
            body: "display:block;margin:8px".to_owned(),
            input_common: String::new(),
            input_hidden: "display:none;width:auto;height:auto;padding:0;border-width:0".to_owned(),
            input_search: "display:inline-block;box-sizing:border-box;width:177px;height:21px;padding:1px 2px;border-width:2px".to_owned(),
            input_checkbox_radio: "display:inline-block;box-sizing:border-box;width:13px;height:13px;padding:0;border-width:0".to_owned(),
            input_range: "display:inline-block;box-sizing:content-box;width:129px;height:16px;padding:0;border-width:0".to_owned(),
            input_color: "display:inline-block;box-sizing:border-box;width:50px;height:27px;padding:1px 2px;border-width:1px".to_owned(),
            input_date: "display:inline-block;box-sizing:content-box;width:113.328125px;height:19px;padding:0 0 0 1px;border-width:2px".to_owned(),
            input_time: "display:inline-block;box-sizing:content-box;width:70px;height:20px;padding:0 0 0 1px;border-width:2px".to_owned(),
            input_datetime_local: "display:inline-block;box-sizing:content-box;width:159.328125px;height:19px;padding:0 0 0 1px;border-width:2px".to_owned(),
            input_month: "display:inline-block;box-sizing:content-box;width:107.328125px;height:19px;padding:0 0 0 1px;border-width:2px".to_owned(),
            input_week: "display:inline-block;box-sizing:content-box;width:135.328125px;height:19px;padding:0 0 0 1px;border-width:2px".to_owned(),
            input_image: "display:inline-block;box-sizing:content-box;width:0;height:0;padding:0;border-width:0".to_owned(),
            input_button: "display:inline-block;box-sizing:border-box;width:16px;height:21px;padding:1px 6px;border-width:2px".to_owned(),
            input_submit_reset: "display:inline-block;box-sizing:border-box;width:42.671875px;height:23px;padding:1px 6px;border-width:2px".to_owned(),
            input_file: "display:inline-block;box-sizing:content-box;width:253px;height:23px;padding:0;border-width:0".to_owned(),
            input_text: "display:inline-block;box-sizing:content-box;width:169px;height:15px;padding:1px 2px;border-width:2px".to_owned(),
        }
    }
}

impl Default for MediaFingerprint {
    fn default() -> Self {
        Self {
            devices: vec![
                MediaDeviceFingerprint {
                    device_id: "default-audio".to_owned(),
                    kind: "audioinput".to_owned(),
                    label: "Default Audio Input".to_owned(),
                    group_id: "default".to_owned(),
                },
                MediaDeviceFingerprint {
                    device_id: "default-video".to_owned(),
                    kind: "videoinput".to_owned(),
                    label: "Default Video Input".to_owned(),
                    group_id: "default".to_owned(),
                },
            ],
            supported_constraints: vec!["width".to_owned(), "height".to_owned()],
            can_play_probably_types: vec![
                "audio/aac".to_owned(),
                "audio/mp4;codecs=mp4a.40.2".to_owned(),
                "audio/mpeg".to_owned(),
                "audio/ogg;codecs=vorbis".to_owned(),
                "audio/wav;codecs=1".to_owned(),
                "video/mp4;codecs=avc1.42c00d".to_owned(),
                "video/mp4;codecs=avc1.42e01e".to_owned(),
                "video/mp4;codecs=avc1.64001e,mp4a.40.2".to_owned(),
                "video/mp4;codecs=mp4a.40.2".to_owned(),
                "video/ogg;codecs=opus".to_owned(),
                "video/webm;codecs=vorbis,vp9".to_owned(),
                "video/webm;codecs=vp8,vorbis".to_owned(),
                "video/webm;codecs=vorbis".to_owned(),
            ],
            can_play_maybe_types: vec![
                "audio/wav".to_owned(),
                "audio/webm".to_owned(),
                "audio/x-m4a".to_owned(),
                "audio/x-mpegurl".to_owned(),
                "video/mp4".to_owned(),
            ],
            media_source_types: vec![
                "audio/mpeg".to_owned(),
                "audio/webm;*".to_owned(),
                "video/webm;*".to_owned(),
            ],
            media_recorder_types: vec![
                "".to_owned(),
                "video/webm".to_owned(),
                "audio/webm".to_owned(),
                "video/mp4".to_owned(),
                "audio/mp4".to_owned(),
            ],
            decoding_supported_types: vec!["*".to_owned()],
            decoding_smooth_types: vec!["*".to_owned()],
            decoding_power_efficient_types: Vec::new(),
            encoding_supported_types: Vec::new(),
            encoding_smooth_types: Vec::new(),
            encoding_power_efficient_types: Vec::new(),
            image_decoder_types: vec!["image/*".to_owned()],
            audio_decoder_codecs: default_audio_decoder_codecs(),
            audio_encoder_codecs: default_audio_encoder_codecs(),
            video_decoder_codecs: default_video_decoder_codecs(),
            video_encoder_codecs: default_video_encoder_codecs(),
            rtc_audio_codecs: vec![
                RtcCodecFingerprint {
                    mime_type: "audio/opus".to_owned(),
                    clock_rate: 48_000,
                    channels: None,
                    sdp_fmtp_line: None,
                },
                RtcCodecFingerprint {
                    mime_type: "audio/PCMU".to_owned(),
                    clock_rate: 8_000,
                    channels: None,
                    sdp_fmtp_line: None,
                },
            ],
            rtc_video_codecs: vec![
                RtcCodecFingerprint {
                    mime_type: "video/VP8".to_owned(),
                    clock_rate: 90_000,
                    channels: None,
                    sdp_fmtp_line: None,
                },
                RtcCodecFingerprint {
                    mime_type: "video/H264".to_owned(),
                    clock_rate: 90_000,
                    channels: None,
                    sdp_fmtp_line: None,
                },
            ],
            rtc_header_extensions: Vec::new(),
            rtc_offer_sdp: String::new(),
            rtc_answer_sdp: String::new(),
        }
    }
}

impl Default for PermissionsFingerprint {
    fn default() -> Self {
        Self {
            accelerometer: "granted".to_owned(),
            background_sync: "granted".to_owned(),
            camera: "denied".to_owned(),
            clipboard_read: "prompt".to_owned(),
            clipboard_write: "granted".to_owned(),
            geolocation: "denied".to_owned(),
            gyroscope: "granted".to_owned(),
            local_fonts: "prompt".to_owned(),
            magnetometer: "granted".to_owned(),
            microphone: "denied".to_owned(),
            midi: "prompt".to_owned(),
            notifications: "denied".to_owned(),
            payment_handler: "granted".to_owned(),
            persistent_storage: "granted".to_owned(),
            speaker_selection: "prompt".to_owned(),
            storage_access: "granted".to_owned(),
            top_level_storage_access: "granted".to_owned(),
            window_management: "granted".to_owned(),
        }
    }
}

impl PermissionsFingerprint {
    pub(crate) fn state(&self, name: &str) -> Option<&str> {
        Some(match name {
            "accelerometer" => &self.accelerometer,
            "background-sync" => &self.background_sync,
            "camera" => &self.camera,
            "clipboard-read" => &self.clipboard_read,
            "clipboard-write" => &self.clipboard_write,
            "geolocation" => &self.geolocation,
            "gyroscope" => &self.gyroscope,
            "local-fonts" => &self.local_fonts,
            "magnetometer" => &self.magnetometer,
            "microphone" => &self.microphone,
            "midi" => &self.midi,
            "notifications" => &self.notifications,
            "payment-handler" => &self.payment_handler,
            "persistent-storage" => &self.persistent_storage,
            "speaker-selection" => &self.speaker_selection,
            "storage-access" => &self.storage_access,
            "top-level-storage-access" => &self.top_level_storage_access,
            "window-management" => &self.window_management,
            _ => return None,
        })
    }
}

impl Default for BatteryFingerprint {
    fn default() -> Self {
        Self {
            charging: true,
            charging_time: 0.0,
            discharging_time: f64::INFINITY,
            level: 1.0,
        }
    }
}

impl Default for GeolocationFingerprint {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            altitude: None,
            accuracy: 0.0,
            altitude_accuracy: None,
            heading: None,
            speed: None,
        }
    }
}

impl Default for MediaPreferencesFingerprint {
    fn default() -> Self {
        Self {
            color_scheme: "light".to_owned(),
            contrast: "no-preference".to_owned(),
            reduced_motion: false,
            reduced_transparency: false,
            reduced_data: false,
            forced_colors: false,
            inverted_colors: false,
            monochrome_bits: 0,
            color_gamut: "srgb".to_owned(),
            pointer: "fine".to_owned(),
            any_pointer: "fine".to_owned(),
            hover: "hover".to_owned(),
            any_hover: "hover".to_owned(),
            display_mode: "browser".to_owned(),
            dynamic_range: "standard".to_owned(),
            video_dynamic_range: "standard".to_owned(),
            scripting: "enabled".to_owned(),
        }
    }
}

impl Default for PluginListFingerprint {
    fn default() -> Self {
        let pdf_types = vec![
            MimeTypeFingerprint {
                mime_type: "application/pdf".to_owned(),
                suffixes: "pdf".to_owned(),
                description: "Portable Document Format".to_owned(),
            },
            MimeTypeFingerprint {
                mime_type: "text/pdf".to_owned(),
                suffixes: "pdf".to_owned(),
                description: "Portable Document Format".to_owned(),
            },
        ];
        let plugin = |name: &str| PluginFingerprint {
            name: name.to_owned(),
            filename: "internal-pdf-viewer".to_owned(),
            description: "Portable Document Format".to_owned(),
            mime_types: pdf_types.clone(),
        };
        Self {
            plugins: vec![
                plugin("PDF Viewer"),
                plugin("Chrome PDF Viewer"),
                plugin("Chromium PDF Viewer"),
                plugin("Microsoft Edge PDF Viewer"),
                plugin("WebKit built-in PDF"),
            ],
        }
    }
}

impl Default for HardwareDevicesFingerprint {
    fn default() -> Self {
        Self {
            gamepads: Vec::new(),
            usb_devices: Vec::new(),
            hid_devices: Vec::new(),
            serial_ports: Vec::new(),
            bluetooth_available: true,
            bluetooth_devices: vec![BluetoothDeviceFingerprint {
                id: "edge-sandbox-device".to_owned(),
                name: Some("Edge Sandbox Device".to_owned()),
            }],
            keyboard_layout: vec![
                KeyboardLayoutEntryFingerprint {
                    code: "KeyA".to_owned(),
                    value: "a".to_owned(),
                },
                KeyboardLayoutEntryFingerprint {
                    code: "KeyB".to_owned(),
                    value: "b".to_owned(),
                },
                KeyboardLayoutEntryFingerprint {
                    code: "Digit1".to_owned(),
                    value: "1".to_owned(),
                },
            ],
            device_posture: "continuous".to_owned(),
            midi_inputs: vec![MidiPortFingerprint {
                id: "input-0".to_owned(),
                manufacturer: "Edge Sandbox".to_owned(),
                name: "Virtual MIDI input".to_owned(),
                version: "1.0".to_owned(),
                state: "connected".to_owned(),
                connection: "closed".to_owned(),
            }],
            midi_outputs: vec![MidiPortFingerprint {
                id: "output-0".to_owned(),
                manufacturer: "Edge Sandbox".to_owned(),
                name: "Virtual MIDI output".to_owned(),
                version: "1.0".to_owned(),
                state: "connected".to_owned(),
                connection: "closed".to_owned(),
            }],
            midi_sysex_enabled: false,
        }
    }
}

impl Default for SensorsFingerprint {
    fn default() -> Self {
        Self {
            available: true,
            accelerometer: [0.0, 0.0, 0.0],
            gravity: [0.0, 0.0, 0.0],
            linear_acceleration: [0.0, 0.0, 0.0],
            gyroscope: [0.0, 0.0, 0.0],
            absolute_orientation_quaternion: [0.0, 0.0, 0.0, 1.0],
            relative_orientation_quaternion: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

impl Default for TimingFingerprint {
    fn default() -> Self {
        Self {
            clock_epoch_ms: None,
            clock_step_ms: 1,
            random_seed: None,
        }
    }
}

impl Default for XrFingerprint {
    fn default() -> Self {
        Self {
            supported_session_modes: vec![
                "inline".to_owned(),
                "immersive-vr".to_owned(),
                "immersive-ar".to_owned(),
            ],
        }
    }
}

impl Default for MemoryFingerprint {
    fn default() -> Self {
        Self {
            performance_js_heap_size_limit: 4_395_630_592,
            performance_total_js_heap_size: 8_388_608,
            performance_used_js_heap_size: 7_002_608,
            console_js_heap_size_limit: 4_395_630_592,
            console_total_js_heap_size: 8_388_608,
            console_used_js_heap_size: 7_002_608,
        }
    }
}

impl Default for UsbDeviceFingerprint {
    fn default() -> Self {
        Self {
            usb_version_major: 1,
            usb_version_minor: 0,
            usb_version_subminor: 0,
            device_class: 0,
            device_subclass: 0,
            device_protocol: 0,
            vendor_id: 0x045e,
            product_id: 1,
            device_version_major: 1,
            device_version_minor: 0,
            device_version_subminor: 0,
            manufacturer_name: Some("Microsoft".to_owned()),
            product_name: Some("Edge Sandbox USB".to_owned()),
            serial_number: Some("EDGE0001".to_owned()),
        }
    }
}

impl Default for HidDeviceFingerprint {
    fn default() -> Self {
        Self {
            vendor_id: 0x045e,
            product_id: 1,
            product_name: "Edge Sandbox HID".to_owned(),
        }
    }
}

impl Default for SerialPortFingerprint {
    fn default() -> Self {
        Self {
            usb_vendor_id: 0x045e,
            usb_product_id: 1,
            connected: true,
        }
    }
}

impl FontFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        validate_string_list("font families", &self.families, 4096, 256)?;
        if self.local_fonts.len() > 4096
            || self.local_fonts.iter().any(|font| {
                [
                    &font.postscript_name,
                    &font.full_name,
                    &font.family,
                    &font.style,
                ]
                .iter()
                .any(|value| value.is_empty() || value.len() > 4096 || value.contains('\0'))
            })
        {
            return Err("local font fingerprint is invalid".to_owned());
        }
        if self.metrics.len() > 4096
            || self.metrics.iter().any(|metric| {
                metric.family.is_empty()
                    || metric.family.len() > 4096
                    || metric.family.contains('\0')
                    || !metric.width_scale.is_finite()
                    || !(0.1..=10.0).contains(&metric.width_scale)
            })
        {
            return Err("font metric fingerprint is invalid".to_owned());
        }
        Ok(())
    }
}

impl CssFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        let declarations = [
            &self.body,
            &self.input_common,
            &self.input_hidden,
            &self.input_search,
            &self.input_checkbox_radio,
            &self.input_range,
            &self.input_color,
            &self.input_date,
            &self.input_time,
            &self.input_datetime_local,
            &self.input_month,
            &self.input_week,
            &self.input_image,
            &self.input_button,
            &self.input_submit_reset,
            &self.input_file,
            &self.input_text,
        ];
        if declarations
            .iter()
            .any(|value| value.len() > 16_384 || value.contains('\0'))
        {
            return Err("CSS fingerprint is invalid".to_owned());
        }
        Ok(())
    }
}

impl DocumentFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self
            .body_child_element_count
            .is_some_and(|count| count > 10_000)
            || self.body_client_height.is_some_and(|height| {
                !height.is_finite() || !(0.0..=10_000_000.0).contains(&height)
            })
        {
            return Err("document fingerprint is invalid".to_owned());
        }
        if self
            .visibility_state
            .as_deref()
            .is_some_and(|state| !matches!(state, "visible" | "hidden"))
        {
            return Err("document visibility_state must be visible or hidden".to_owned());
        }
        Ok(())
    }
}

impl MediaFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.devices.len() > 256
            || self.devices.iter().any(|device| {
                !matches!(
                    device.kind.as_str(),
                    "audioinput" | "audiooutput" | "videoinput"
                ) || [&device.device_id, &device.label, &device.group_id]
                    .iter()
                    .any(|value| value.len() > 4096)
            })
        {
            return Err("media device fingerprint is invalid".to_owned());
        }
        validate_string_list(
            "supported media constraints",
            &self.supported_constraints,
            256,
            128,
        )?;
        for (name, values) in [
            ("canPlayType probably values", &self.can_play_probably_types),
            ("canPlayType maybe values", &self.can_play_maybe_types),
            ("MediaSource types", &self.media_source_types),
            ("MediaRecorder types", &self.media_recorder_types),
            ("decoding supported types", &self.decoding_supported_types),
            ("decoding smooth types", &self.decoding_smooth_types),
            (
                "decoding power-efficient types",
                &self.decoding_power_efficient_types,
            ),
            ("encoding supported types", &self.encoding_supported_types),
            ("encoding smooth types", &self.encoding_smooth_types),
            (
                "encoding power-efficient types",
                &self.encoding_power_efficient_types,
            ),
            ("ImageDecoder types", &self.image_decoder_types),
            ("AudioDecoder codecs", &self.audio_decoder_codecs),
            ("AudioEncoder codecs", &self.audio_encoder_codecs),
            ("VideoDecoder codecs", &self.video_decoder_codecs),
            ("VideoEncoder codecs", &self.video_encoder_codecs),
        ] {
            validate_string_list(name, values, 1024, 1024)?;
        }
        for codec in self
            .rtc_audio_codecs
            .iter()
            .chain(self.rtc_video_codecs.iter())
        {
            if codec.mime_type.is_empty()
                || codec.mime_type.len() > 256
                || codec.clock_rate == 0
                || codec
                    .sdp_fmtp_line
                    .as_ref()
                    .is_some_and(|line| line.len() > 4096)
            {
                return Err("RTC codec fingerprint is invalid".to_owned());
            }
        }
        if self.rtc_audio_codecs.len() > 256
            || self.rtc_video_codecs.len() > 256
            || self.rtc_header_extensions.len() > 256
            || self.rtc_header_extensions.iter().any(|extension| {
                !matches!(extension.kind.as_str(), "audio" | "video")
                    || extension.uri.is_empty()
                    || extension.uri.len() > 4096
            })
        {
            return Err("RTC capability fingerprint is invalid".to_owned());
        }
        if self.rtc_offer_sdp.len() > 1_048_576
            || self.rtc_answer_sdp.len() > 1_048_576
            || self.rtc_offer_sdp.contains('\0')
            || self.rtc_answer_sdp.contains('\0')
        {
            return Err("RTC session-description fingerprint is invalid".to_owned());
        }
        Ok(())
    }
}

impl PermissionsFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        let regular_values = [
            &self.accelerometer,
            &self.background_sync,
            &self.camera,
            &self.clipboard_read,
            &self.clipboard_write,
            &self.geolocation,
            &self.gyroscope,
            &self.local_fonts,
            &self.magnetometer,
            &self.microphone,
            &self.midi,
            &self.notifications,
            &self.payment_handler,
            &self.persistent_storage,
            &self.storage_access,
            &self.window_management,
        ];
        if regular_values
            .iter()
            .any(|state| !matches!(state.as_str(), "granted" | "denied" | "prompt"))
            || !matches!(
                self.speaker_selection.as_str(),
                "granted" | "denied" | "prompt" | "unsupported"
            )
            || !matches!(
                self.top_level_storage_access.as_str(),
                "granted" | "denied" | "prompt" | "invalid-origin"
            )
        {
            return Err("permission state or configured browser rejection is invalid".to_owned());
        }
        Ok(())
    }
}

impl BatteryFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if !self.charging_time.is_finite() && !self.charging_time.is_infinite()
            || self.charging_time < 0.0
            || !self.discharging_time.is_finite() && !self.discharging_time.is_infinite()
            || self.discharging_time < 0.0
            || !self.level.is_finite()
            || !(0.0..=1.0).contains(&self.level)
        {
            return Err("battery fingerprint is outside supported bounds".to_owned());
        }
        Ok(())
    }
}

impl GeolocationFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if !self.latitude.is_finite()
            || !(-90.0..=90.0).contains(&self.latitude)
            || !self.longitude.is_finite()
            || !(-180.0..=180.0).contains(&self.longitude)
            || !self.accuracy.is_finite()
            || self.accuracy < 0.0
            || optional_f64_invalid(self.altitude)
            || optional_nonnegative_f64_invalid(self.altitude_accuracy)
            || optional_f64_invalid(self.heading)
            || optional_nonnegative_f64_invalid(self.speed)
        {
            return Err("geolocation fingerprint is outside supported bounds".to_owned());
        }
        Ok(())
    }
}

impl MediaPreferencesFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if !matches!(self.color_scheme.as_str(), "light" | "dark")
            || !matches!(
                self.contrast.as_str(),
                "no-preference" | "more" | "less" | "custom"
            )
            || !matches!(self.color_gamut.as_str(), "srgb" | "p3" | "rec2020")
            || !matches!(self.pointer.as_str(), "none" | "coarse" | "fine")
            || !matches!(self.any_pointer.as_str(), "none" | "coarse" | "fine")
            || !matches!(self.hover.as_str(), "none" | "hover")
            || !matches!(self.any_hover.as_str(), "none" | "hover")
            || !matches!(
                self.display_mode.as_str(),
                "browser" | "fullscreen" | "standalone" | "minimal-ui" | "window-controls-overlay"
            )
            || !matches!(self.dynamic_range.as_str(), "standard" | "high")
            || !matches!(self.video_dynamic_range.as_str(), "standard" | "high")
            || !matches!(self.scripting.as_str(), "none" | "initial-only" | "enabled")
            || self.monochrome_bits > 64
        {
            return Err("media preference fingerprint is invalid".to_owned());
        }
        Ok(())
    }
}

impl PluginListFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.plugins.len() > 256
            || self.plugins.iter().any(|plugin| {
                plugin.name.is_empty()
                    || plugin.name.len() > 1024
                    || plugin.filename.len() > 4096
                    || plugin.description.len() > 4096
                    || plugin.mime_types.len() > 256
                    || plugin.mime_types.iter().any(|mime| {
                        mime.mime_type.is_empty()
                            || mime.mime_type.len() > 1024
                            || mime.suffixes.len() > 4096
                            || mime.description.len() > 4096
                    })
            })
        {
            return Err("plugin fingerprint is invalid".to_owned());
        }
        Ok(())
    }
}

impl HardwareDevicesFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.gamepads.len() > 64
            || self.gamepads.iter().any(|gamepad| {
                gamepad.id.len() > 4096
                    || gamepad.mapping.len() > 128
                    || gamepad.axes.len() > 64
                    || gamepad.buttons.len() > 128
                    || gamepad
                        .axes
                        .iter()
                        .any(|value| !value.is_finite() || !(-1.0..=1.0).contains(value))
                    || gamepad
                        .buttons
                        .iter()
                        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
            })
            || self.usb_devices.len() > 256
            || self.hid_devices.len() > 256
            || self.serial_ports.len() > 256
            || self.bluetooth_devices.len() > 256
            || self.keyboard_layout.len() > 512
            || self.midi_inputs.len() > 256
            || self.midi_outputs.len() > 256
        {
            return Err("hardware device fingerprint is invalid".to_owned());
        }
        if self.usb_devices.iter().any(|device| {
            [
                device.manufacturer_name.as_ref(),
                device.product_name.as_ref(),
                device.serial_number.as_ref(),
            ]
            .into_iter()
            .flatten()
            .any(|value| value.len() > 4096)
        }) || self
            .hid_devices
            .iter()
            .any(|device| device.product_name.len() > 4096)
            || self.bluetooth_devices.iter().any(|device| {
                device.id.is_empty()
                    || device.id.len() > 4096
                    || device.name.as_ref().is_some_and(|name| name.len() > 4096)
            })
            || self.keyboard_layout.iter().any(|entry| {
                entry.code.is_empty()
                    || entry.code.len() > 256
                    || entry.value.len() > 256
                    || entry.code.contains('\0')
                    || entry.value.contains('\0')
            })
            || !matches!(self.device_posture.as_str(), "continuous" | "folded")
            || self
                .midi_inputs
                .iter()
                .chain(self.midi_outputs.iter())
                .any(|port| {
                    [&port.id, &port.manufacturer, &port.name, &port.version]
                        .iter()
                        .any(|value| value.is_empty() || value.len() > 4096 || value.contains('\0'))
                        || !matches!(port.state.as_str(), "connected" | "disconnected")
                        || !matches!(port.connection.as_str(), "open" | "closed" | "pending")
                })
        {
            return Err("hardware device string fingerprint is invalid".to_owned());
        }
        Ok(())
    }
}

impl SensorsFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self
            .accelerometer
            .iter()
            .chain(self.gravity.iter())
            .chain(self.linear_acceleration.iter())
            .chain(self.gyroscope.iter())
            .chain(self.absolute_orientation_quaternion.iter())
            .chain(self.relative_orientation_quaternion.iter())
            .any(|value| !value.is_finite())
        {
            return Err("sensor fingerprint contains a non-finite reading".to_owned());
        }
        Ok(())
    }
}

impl TimingFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.clock_step_ms > 86_400_000 {
            return Err("fingerprint clock step must not exceed one day".to_owned());
        }
        Ok(())
    }

    pub(crate) fn apply(&self, deterministic: &mut crate::DeterministicExecution) {
        if let Some(epoch) = self.clock_epoch_ms {
            deterministic.clock_epoch_ms = Some(epoch);
            deterministic.clock_step_ms = self.clock_step_ms;
        }
        if let Some(seed) = self.random_seed {
            deterministic.random_seed = Some(seed);
        }
    }
}

impl XrFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        validate_string_list(
            "XR supported session modes",
            &self.supported_session_modes,
            3,
            32,
        )?;
        if self
            .supported_session_modes
            .iter()
            .any(|mode| !matches!(mode.as_str(), "inline" | "immersive-vr" | "immersive-ar"))
        {
            return Err("XR session mode fingerprint is invalid".to_owned());
        }
        Ok(())
    }
}

impl MemoryFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
        if self.performance_js_heap_size_limit == 0
            || self.performance_js_heap_size_limit > MAX_SAFE_INTEGER
            || self.performance_total_js_heap_size > self.performance_js_heap_size_limit
            || self.performance_used_js_heap_size > self.performance_total_js_heap_size
            || self.console_js_heap_size_limit == 0
            || self.console_js_heap_size_limit > MAX_SAFE_INTEGER
            || self.console_total_js_heap_size > self.console_js_heap_size_limit
            || self.console_used_js_heap_size > self.console_total_js_heap_size
        {
            return Err("memory fingerprint is outside supported bounds".to_owned());
        }
        Ok(())
    }
}

pub(crate) fn media_type_matches(patterns: &[String], media_type: &str) -> bool {
    let candidate = media_type.trim().to_ascii_lowercase();
    patterns.iter().any(|pattern| {
        let pattern = pattern.trim().to_ascii_lowercase();
        if pattern == "*" || pattern == candidate {
            return true;
        }
        if let Some(prefix) = pattern.strip_suffix('*') {
            return candidate.starts_with(prefix);
        }
        !pattern.is_empty()
            && !pattern.contains(';')
            && candidate
                .strip_prefix(&pattern)
                .is_some_and(|tail| tail.trim_start().starts_with(';'))
    })
}

fn canonical_media_type(value: &str) -> String {
    let mut parts = value.split(';');
    let base = parts.next().unwrap_or_default().trim().to_ascii_lowercase();
    let mut normalized = base;
    for parameter in parts {
        let parameter = parameter.trim().to_ascii_lowercase();
        if parameter.is_empty() {
            continue;
        }
        normalized.push(';');
        if let Some((name, value)) = parameter.split_once('=') {
            normalized.push_str(name.trim());
            normalized.push('=');
            let value = value.trim().trim_matches('"');
            if name.trim() == "codecs" {
                normalized.push_str(
                    &value
                        .split(',')
                        .map(str::trim)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            } else {
                normalized.push_str(value);
            }
        } else {
            normalized.push_str(&parameter);
        }
    }
    normalized
}

/// Match capability records without treating a bare MIME type as support for
/// every codec parameter. Chromium reports the base container and individual
/// codec combinations independently for MediaSource and MediaRecorder.
pub(crate) fn media_capability_matches(patterns: &[String], media_type: &str) -> bool {
    let candidate = canonical_media_type(media_type);
    patterns.iter().any(|pattern| {
        let pattern = canonical_media_type(pattern);
        if pattern == "*" || pattern == candidate {
            return true;
        }
        pattern
            .strip_suffix('*')
            .is_some_and(|prefix| candidate.starts_with(prefix))
    })
}

fn validate_string_list(
    name: &str,
    values: &[String],
    maximum_items: usize,
    maximum_length: usize,
) -> Result<(), String> {
    if values.len() > maximum_items
        || values
            .iter()
            .any(|value| value.len() > maximum_length || value.contains('\0'))
    {
        return Err(format!("{name} fingerprint is invalid"));
    }
    Ok(())
}

fn optional_f64_invalid(value: Option<f64>) -> bool {
    value.is_some_and(|value| !value.is_finite())
}

fn optional_nonnegative_f64_invalid(value: Option<f64>) -> bool {
    value.is_some_and(|value| !value.is_finite() || value < 0.0)
}
