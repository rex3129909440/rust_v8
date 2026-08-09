use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::ptr;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EdgeSandboxBuffer {
    pub data: *mut u8,
    pub len: usize,
}

/// A borrowed UTF-8 string for typed C ABI list inputs.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EdgeSandboxStringView {
    pub data: *const u8,
    pub len: usize,
}

impl Default for EdgeSandboxBuffer {
    fn default() -> Self {
        Self {
            data: ptr::null_mut(),
            len: 0,
        }
    }
}

/// Strongly typed WebAudio fingerprint settings for native embedders.
///
/// The layout is stable C ABI data and deliberately avoids JSON or another
/// string-based configuration envelope.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeSandboxAudioProfile {
    pub sample_rate: f64,
    pub base_latency: f64,
    pub output_latency: f64,
    pub noise_seed: u64,
    pub max_channel_count: u32,
    pub channel_noise_amplitude: f32,
    pub frequency_noise_amplitude: f32,
    pub time_domain_noise_amplitude: f32,
}

impl Default for EdgeSandboxAudioProfile {
    fn default() -> Self {
        let profile = crate::AudioFingerprint::default();
        Self {
            sample_rate: profile.sample_rate,
            base_latency: profile.base_latency,
            output_latency: profile.output_latency,
            noise_seed: profile.noise_seed,
            max_channel_count: profile.max_channel_count,
            channel_noise_amplitude: profile.channel_noise_amplitude,
            frequency_noise_amplitude: profile.frequency_noise_amplitude,
            time_domain_noise_amplitude: profile.time_domain_noise_amplitude,
        }
    }
}

impl From<EdgeSandboxAudioProfile> for crate::AudioFingerprint {
    fn from(profile: EdgeSandboxAudioProfile) -> Self {
        Self {
            sample_rate: profile.sample_rate,
            max_channel_count: profile.max_channel_count,
            base_latency: profile.base_latency,
            output_latency: profile.output_latency,
            noise_seed: profile.noise_seed,
            channel_noise_amplitude: profile.channel_noise_amplitude,
            frequency_noise_amplitude: profile.frequency_noise_amplitude,
            time_domain_noise_amplitude: profile.time_domain_noise_amplitude,
        }
    }
}

/// Opaque, mutable builder for a complete [`crate::EdgeFingerprint`].
///
/// Native callers populate it through the typed setter functions below and
/// pass it to [`edge_sandbox_create_self_hosted_with_profile`]. The worker receives a
/// cloned, validated profile through the existing binary IPC protocol.
pub struct EdgeSandboxProfile {
    fingerprint: crate::EdgeFingerprint,
}

/// Opaque, mutable builder for a complete [`crate::EdgeRuntimeOptions`].
///
/// This builder carries page initialization, offline network replay, resource
/// limits and deterministic execution settings without a JSON envelope. A
/// fingerprint builder may be copied into it with
/// [`edge_sandbox_options_set_profile`].
pub struct EdgeSandboxOptions {
    options: crate::EdgeRuntimeOptions,
}

/// Deterministic execution settings for the typed options builder.
///
/// `has_clock_epoch_ms` and `has_random_seed` distinguish an explicit value
/// from the runtime defaults. Reserved bytes must be zero.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeSandboxDeterministicOptions {
    pub clock_epoch_ms: i64,
    pub clock_step_ms: u64,
    pub random_seed: u64,
    pub max_task_turns: u32,
    pub has_clock_epoch_ms: u8,
    pub has_random_seed: u8,
    pub reserved: [u8; 6],
}

impl Default for EdgeSandboxDeterministicOptions {
    fn default() -> Self {
        let value = crate::DeterministicExecution::default();
        Self {
            clock_epoch_ms: value.clock_epoch_ms.unwrap_or_default(),
            clock_step_ms: value.clock_step_ms,
            random_seed: value.random_seed.unwrap_or_default(),
            max_task_turns: value.max_task_turns as u32,
            has_clock_epoch_ms: value.clock_epoch_ms.is_some() as u8,
            has_random_seed: value.random_seed.is_some() as u8,
            reserved: [0; 6],
        }
    }
}

/// Resource limits for the typed options builder.
///
/// A zero field means "use the isolated runtime default". Every accepted
/// non-zero value is validated before a worker process is created.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EdgeSandboxLimits {
    pub timeout_ms: u64,
    pub max_heap_bytes: u64,
    pub max_resident_bytes: u64,
    pub max_source_bytes: u64,
    pub max_output_bytes: u64,
}

/// One typed Performance Timeline profile record.
///
/// String fields are borrowed for the synchronous append call. Optional
/// response-size fields are selected by their corresponding `has_*` flag.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EdgeSandboxPerformanceEntryProfile {
    pub name: EdgeSandboxStringView,
    pub entry_type: EdgeSandboxStringView,
    pub initiator_type: EdgeSandboxStringView,
    pub delivery_type: EdgeSandboxStringView,
    pub next_hop_protocol: EdgeSandboxStringView,
    pub render_blocking_status: EdgeSandboxStringView,
    pub content_type: EdgeSandboxStringView,
    pub content_encoding: EdgeSandboxStringView,
    pub worker_matched_source_type: EdgeSandboxStringView,
    pub worker_final_source_type: EdgeSandboxStringView,
    pub navigation_type: EdgeSandboxStringView,
    pub start_time: f64,
    pub duration: f64,
    pub worker_start: f64,
    pub worker_router_evaluation_start: f64,
    pub worker_cache_lookup_start: f64,
    pub redirect_start: f64,
    pub redirect_end: f64,
    pub fetch_start: f64,
    pub domain_lookup_start: f64,
    pub domain_lookup_end: f64,
    pub connect_start: f64,
    pub secure_connection_start: f64,
    pub connect_end: f64,
    pub request_start: f64,
    pub response_start: f64,
    pub first_interim_response_start: f64,
    pub final_response_headers_start: f64,
    pub response_end: f64,
    pub unload_event_start: f64,
    pub unload_event_end: f64,
    pub dom_interactive: f64,
    pub dom_content_loaded_event_start: f64,
    pub dom_content_loaded_event_end: f64,
    pub dom_complete: f64,
    pub load_event_start: f64,
    pub load_event_end: f64,
    pub critical_ch_restart: f64,
    pub activation_start: f64,
    pub paint_time: f64,
    pub presentation_time: f64,
    pub transfer_size: u64,
    pub encoded_body_size: u64,
    pub decoded_body_size: u64,
    pub redirect_count: u32,
    pub response_status: u16,
    pub has_transfer_size: u8,
    pub has_encoded_body_size: u8,
    pub has_decoded_body_size: u8,
    pub has_response_status: u8,
    pub reserved: [u8; 8],
}

pub mod profile_field {
    pub const ID: u32 = 1;
    pub const LOCALE: u32 = 2;
    pub const TIME_ZONE: u32 = 3;

    pub const NAVIGATOR_USER_AGENT: u32 = 10;
    pub const NAVIGATOR_APP_VERSION: u32 = 11;
    pub const NAVIGATOR_APP_CODE_NAME: u32 = 12;
    pub const NAVIGATOR_APP_NAME: u32 = 13;
    pub const NAVIGATOR_PLATFORM: u32 = 14;
    pub const NAVIGATOR_PRODUCT: u32 = 15;
    pub const NAVIGATOR_PRODUCT_SUB: u32 = 16;
    pub const NAVIGATOR_VENDOR: u32 = 17;
    pub const NAVIGATOR_VENDOR_SUB: u32 = 18;
    pub const NAVIGATOR_LANGUAGE: u32 = 19;
    pub const NAVIGATOR_DO_NOT_TRACK: u32 = 20;

    pub const UA_PLATFORM: u32 = 30;
    pub const UA_ARCHITECTURE: u32 = 31;
    pub const UA_BITNESS: u32 = 32;
    pub const UA_MODEL: u32 = 33;
    pub const UA_PLATFORM_VERSION: u32 = 34;
    pub const UA_FULL_VERSION: u32 = 35;
    pub const NETWORK_EFFECTIVE_TYPE: u32 = 40;

    pub const CANVAS_DATA_URL_SALT: u32 = 50;
    pub const WEBGL_VENDOR: u32 = 60;
    pub const WEBGL_RENDERER: u32 = 61;
    pub const WEBGL_UNMASKED_VENDOR: u32 = 62;
    pub const WEBGL_UNMASKED_RENDERER: u32 = 63;
    pub const WEBGL1_VERSION: u32 = 64;
    pub const WEBGL1_SHADING_LANGUAGE_VERSION: u32 = 65;
    pub const WEBGL2_VERSION: u32 = 66;
    pub const WEBGL2_SHADING_LANGUAGE_VERSION: u32 = 67;
    pub const WEBGL_CONTEXT_POWER_PREFERENCE: u32 = 68;
    pub const WEBGPU_VENDOR: u32 = 70;
    pub const WEBGPU_ARCHITECTURE: u32 = 71;
    pub const WEBGPU_DEVICE: u32 = 72;
    pub const WEBGPU_DESCRIPTION: u32 = 73;
    pub const RTC_OFFER_SDP: u32 = 74;
    pub const RTC_ANSWER_SDP: u32 = 75;
    pub const SCREEN_ORIENTATION_TYPE: u32 = 76;
    pub const DEVICE_POSTURE: u32 = 77;
    pub const CSS_BODY: u32 = 78;
    pub const CSS_INPUT_COMMON: u32 = 79;
    pub const CSS_INPUT_HIDDEN: u32 = 80;
    pub const CSS_INPUT_SEARCH: u32 = 81;
    pub const CSS_INPUT_CHECKBOX_RADIO: u32 = 82;
    pub const CSS_INPUT_RANGE: u32 = 83;
    pub const CSS_INPUT_COLOR: u32 = 84;
    pub const CSS_INPUT_DATE: u32 = 85;
    pub const CSS_INPUT_TIME: u32 = 86;
    pub const CSS_INPUT_DATETIME_LOCAL: u32 = 87;
    pub const CSS_INPUT_MONTH: u32 = 88;
    pub const CSS_INPUT_WEEK: u32 = 89;
    pub const CSS_INPUT_IMAGE: u32 = 90;
    pub const CSS_INPUT_BUTTON: u32 = 91;
    pub const CSS_INPUT_SUBMIT_RESET: u32 = 92;
    pub const CSS_INPUT_FILE: u32 = 93;
    pub const CSS_INPUT_TEXT: u32 = 94;
    pub const PERFORMANCE_EVALUATED_SCRIPT_CONTENT_ENCODING: u32 = 95;

    pub const PERMISSION_ACCELEROMETER: u32 = 800;
    pub const PERMISSION_BACKGROUND_SYNC: u32 = 801;
    pub const PERMISSION_CAMERA: u32 = 802;
    pub const PERMISSION_CLIPBOARD_READ: u32 = 803;
    pub const PERMISSION_CLIPBOARD_WRITE: u32 = 804;
    pub const PERMISSION_GEOLOCATION: u32 = 805;
    pub const PERMISSION_GYROSCOPE: u32 = 806;
    pub const PERMISSION_LOCAL_FONTS: u32 = 807;
    pub const PERMISSION_MAGNETOMETER: u32 = 808;
    pub const PERMISSION_MICROPHONE: u32 = 809;
    pub const PERMISSION_MIDI: u32 = 810;
    pub const PERMISSION_NOTIFICATIONS: u32 = 811;
    pub const PERMISSION_PAYMENT_HANDLER: u32 = 812;
    pub const PERMISSION_PERSISTENT_STORAGE: u32 = 813;
    pub const PERMISSION_SPEAKER_SELECTION: u32 = 814;
    pub const PERMISSION_STORAGE_ACCESS: u32 = 815;
    pub const PERMISSION_TOP_LEVEL_STORAGE_ACCESS: u32 = 816;
    pub const PERMISSION_WINDOW_MANAGEMENT: u32 = 817;

    pub const MEDIA_PREFERENCE_COLOR_SCHEME: u32 = 820;
    pub const MEDIA_PREFERENCE_CONTRAST: u32 = 821;
    pub const MEDIA_PREFERENCE_COLOR_GAMUT: u32 = 822;
    pub const MEDIA_PREFERENCE_POINTER: u32 = 823;
    pub const MEDIA_PREFERENCE_ANY_POINTER: u32 = 824;
    pub const MEDIA_PREFERENCE_HOVER: u32 = 825;
    pub const MEDIA_PREFERENCE_ANY_HOVER: u32 = 826;
    pub const MEDIA_PREFERENCE_DISPLAY_MODE: u32 = 827;
    pub const MEDIA_PREFERENCE_DYNAMIC_RANGE: u32 = 828;
    pub const MEDIA_PREFERENCE_SCRIPTING: u32 = 829;
    pub const MEDIA_PREFERENCE_VIDEO_DYNAMIC_RANGE: u32 = 830;

    pub const NAVIGATOR_LANGUAGES: u32 = 100;
    pub const UA_FORM_FACTORS: u32 = 101;
    pub const WEBGL1_EXTENSIONS: u32 = 102;
    pub const WEBGL2_EXTENSIONS: u32 = 103;
    pub const WEBGPU_FEATURES: u32 = 104;
    pub const FONT_FAMILIES: u32 = 105;
    pub const MEDIA_SUPPORTED_CONSTRAINTS: u32 = 106;
    pub const MEDIA_CAN_PLAY_PROBABLY_TYPES: u32 = 107;
    pub const MEDIA_CAN_PLAY_MAYBE_TYPES: u32 = 108;
    pub const MEDIA_SOURCE_TYPES: u32 = 109;
    pub const MEDIA_RECORDER_TYPES: u32 = 110;
    pub const MEDIA_DECODING_SUPPORTED_TYPES: u32 = 111;
    pub const MEDIA_DECODING_SMOOTH_TYPES: u32 = 112;
    pub const MEDIA_DECODING_POWER_EFFICIENT_TYPES: u32 = 113;
    pub const MEDIA_ENCODING_SUPPORTED_TYPES: u32 = 114;
    pub const MEDIA_ENCODING_SMOOTH_TYPES: u32 = 115;
    pub const MEDIA_ENCODING_POWER_EFFICIENT_TYPES: u32 = 116;
    pub const IMAGE_DECODER_TYPES: u32 = 117;
    pub const XR_SUPPORTED_SESSION_MODES: u32 = 118;
    pub const AUDIO_DECODER_CODECS: u32 = 119;
    pub const AUDIO_ENCODER_CODECS: u32 = 120;
    pub const VIDEO_DECODER_CODECS: u32 = 121;
    pub const VIDEO_ENCODER_CODECS: u32 = 122;

    pub const HARDWARE_CONCURRENCY: u32 = 200;
    pub const MAX_TOUCH_POINTS: u32 = 201;
    pub const NETWORK_RTT: u32 = 202;
    pub const WEBGPU_MAX_TEXTURE_DIMENSION_2D: u32 = 203;
    pub const AUDIO_MAX_CHANNEL_COUNT: u32 = 204;
    pub const MEDIA_PREFERENCE_MONOCHROME_BITS: u32 = 205;
    pub const WEBGPU_MAX_TEXTURE_DIMENSION_1D: u32 = 206;
    pub const WEBGPU_MAX_TEXTURE_DIMENSION_3D: u32 = 207;
    pub const WEBGPU_MAX_TEXTURE_ARRAY_LAYERS: u32 = 208;
    pub const WEBGPU_MAX_BIND_GROUPS: u32 = 209;
    pub const WEBGPU_MAX_BIND_GROUPS_PLUS_VERTEX_BUFFERS: u32 = 210;
    pub const WEBGPU_MAX_BINDINGS_PER_BIND_GROUP: u32 = 211;
    pub const WEBGPU_MAX_DYNAMIC_UNIFORM_BUFFERS_PER_PIPELINE_LAYOUT: u32 = 212;
    pub const WEBGPU_MAX_DYNAMIC_STORAGE_BUFFERS_PER_PIPELINE_LAYOUT: u32 = 213;
    pub const WEBGPU_MAX_SAMPLED_TEXTURES_PER_SHADER_STAGE: u32 = 214;
    pub const WEBGPU_MAX_SAMPLERS_PER_SHADER_STAGE: u32 = 215;
    pub const WEBGPU_MAX_STORAGE_BUFFERS_PER_SHADER_STAGE: u32 = 216;
    pub const WEBGPU_MAX_STORAGE_TEXTURES_PER_SHADER_STAGE: u32 = 217;
    pub const WEBGPU_MAX_UNIFORM_BUFFERS_PER_SHADER_STAGE: u32 = 218;
    pub const WEBGPU_MIN_UNIFORM_BUFFER_OFFSET_ALIGNMENT: u32 = 219;
    pub const WEBGPU_MIN_STORAGE_BUFFER_OFFSET_ALIGNMENT: u32 = 220;
    pub const WEBGPU_MAX_VERTEX_BUFFERS: u32 = 221;
    pub const WEBGPU_MAX_VERTEX_ATTRIBUTES: u32 = 222;
    pub const WEBGPU_MAX_VERTEX_BUFFER_ARRAY_STRIDE: u32 = 223;
    pub const WEBGPU_MAX_INTER_STAGE_SHADER_VARIABLES: u32 = 224;
    pub const WEBGPU_MAX_COLOR_ATTACHMENTS: u32 = 225;
    pub const WEBGPU_MAX_COLOR_ATTACHMENT_BYTES_PER_SAMPLE: u32 = 226;
    pub const WEBGPU_MAX_COMPUTE_WORKGROUP_STORAGE_SIZE: u32 = 227;
    pub const WEBGPU_MAX_COMPUTE_INVOCATIONS_PER_WORKGROUP: u32 = 228;
    pub const WEBGPU_MAX_COMPUTE_WORKGROUP_SIZE_X: u32 = 229;
    pub const WEBGPU_MAX_COMPUTE_WORKGROUP_SIZE_Y: u32 = 230;
    pub const WEBGPU_MAX_COMPUTE_WORKGROUP_SIZE_Z: u32 = 231;
    pub const WEBGPU_MAX_COMPUTE_WORKGROUPS_PER_DIMENSION: u32 = 232;
    pub const WEBGPU_MAX_IMMEDIATE_SIZE: u32 = 233;
    pub const WEBGPU_MAX_STORAGE_BUFFERS_IN_FRAGMENT_STAGE: u32 = 234;
    pub const WEBGPU_MAX_STORAGE_TEXTURES_IN_FRAGMENT_STAGE: u32 = 235;
    pub const WEBGPU_MAX_STORAGE_BUFFERS_IN_VERTEX_STAGE: u32 = 236;
    pub const WEBGPU_MAX_STORAGE_TEXTURES_IN_VERTEX_STAGE: u32 = 237;
    pub const SCREEN_ORIENTATION_ANGLE: u32 = 238;
    pub const WEBGPU_SUBGROUP_MIN_SIZE: u32 = 239;
    pub const WEBGPU_SUBGROUP_MAX_SIZE: u32 = 240;
    pub const DOCUMENT_BODY_CHILD_ELEMENT_COUNT: u32 = 241;

    pub const TIME_ZONE_OFFSET_MINUTES: u32 = 300;
    pub const SCREEN_WIDTH: u32 = 301;
    pub const SCREEN_HEIGHT: u32 = 302;
    pub const SCREEN_AVAIL_WIDTH: u32 = 303;
    pub const SCREEN_AVAIL_HEIGHT: u32 = 304;
    pub const SCREEN_AVAIL_LEFT: u32 = 305;
    pub const SCREEN_AVAIL_TOP: u32 = 306;
    pub const SCREEN_COLOR_DEPTH: u32 = 307;
    pub const SCREEN_PIXEL_DEPTH: u32 = 308;
    pub const WEBGL_MAX_TEXTURE_SIZE: u32 = 309;
    pub const WEBGL_MAX_CUBE_MAP_TEXTURE_SIZE: u32 = 310;
    pub const WEBGL_MAX_RENDERBUFFER_SIZE: u32 = 311;
    pub const WEBGL_MAX_VIEWPORT_WIDTH: u32 = 312;
    pub const WEBGL_MAX_VIEWPORT_HEIGHT: u32 = 313;
    pub const WEBGL_MAX_VERTEX_ATTRIBS: u32 = 314;
    pub const WEBGL_MAX_VERTEX_TEXTURE_IMAGE_UNITS: u32 = 315;
    pub const WEBGL_MAX_TEXTURE_IMAGE_UNITS: u32 = 316;
    pub const WEBGL_MAX_COMBINED_TEXTURE_IMAGE_UNITS: u32 = 317;
    pub const WEBGL2_MAX_DRAW_BUFFERS: u32 = 318;
    pub const WEBGL2_MAX_COLOR_ATTACHMENTS: u32 = 319;
    pub const WEBGL2_MAX_SAMPLES: u32 = 320;
    pub const WEBGL_SHADER_PRECISION_RANGE_MIN: u32 = 321;
    pub const WEBGL_SHADER_PRECISION_RANGE_MAX: u32 = 322;
    pub const WEBGL_SHADER_PRECISION_BITS: u32 = 323;
    pub const WEBGL_MAX_VERTEX_UNIFORM_VECTORS: u32 = 324;
    pub const WEBGL_MAX_VARYING_VECTORS: u32 = 325;
    pub const WEBGL_MAX_FRAGMENT_UNIFORM_VECTORS: u32 = 326;
    pub const WEBGL_SUBPIXEL_BITS: u32 = 327;
    pub const WEBGL2_MAX_3D_TEXTURE_SIZE: u32 = 328;
    pub const WEBGL2_MAX_ARRAY_TEXTURE_LAYERS: u32 = 329;
    pub const WEBGL2_MAX_VERTEX_UNIFORM_COMPONENTS: u32 = 330;
    pub const WEBGL2_MAX_FRAGMENT_UNIFORM_COMPONENTS: u32 = 331;
    pub const WEBGL2_MAX_VARYING_COMPONENTS: u32 = 332;
    pub const WEBGL2_MAX_VERTEX_OUTPUT_COMPONENTS: u32 = 333;
    pub const WEBGL2_MAX_FRAGMENT_INPUT_COMPONENTS: u32 = 334;
    pub const WEBGL2_MAX_VERTEX_UNIFORM_BLOCKS: u32 = 335;
    pub const WEBGL2_MAX_FRAGMENT_UNIFORM_BLOCKS: u32 = 336;
    pub const WEBGL2_MAX_COMBINED_UNIFORM_BLOCKS: u32 = 337;
    pub const WEBGL2_MAX_UNIFORM_BUFFER_BINDINGS: u32 = 338;
    pub const WEBGL2_MAX_UNIFORM_BLOCK_SIZE: u32 = 339;
    pub const WEBGL2_MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS: u32 = 340;
    pub const WEBGL2_MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS: u32 = 341;
    pub const WEBGL2_MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS: u32 = 342;
    pub const WEBGL2_MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS: u32 = 343;
    pub const WEBGL2_MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS: u32 = 344;
    pub const WEBGL2_MAX_PROGRAM_TEXEL_OFFSET: u32 = 345;
    pub const WEBGL2_MAX_ELEMENTS_VERTICES: u32 = 346;
    pub const WEBGL2_MAX_ELEMENTS_INDICES: u32 = 347;
    pub const WEBGL2_MAX_ELEMENT_INDEX: u32 = 348;

    pub const STORAGE_QUOTA_BYTES: u32 = 400;
    pub const STORAGE_USAGE_BYTES: u32 = 401;
    pub const AUDIO_NOISE_SEED: u32 = 402;
    pub const WEBGPU_MAX_UNIFORM_BUFFER_BINDING_SIZE: u32 = 403;
    pub const WEBGPU_MAX_STORAGE_BUFFER_BINDING_SIZE: u32 = 404;
    pub const WEBGPU_MAX_BUFFER_SIZE: u32 = 405;
    pub const TIMING_CLOCK_STEP_MS: u32 = 406;
    pub const TIMING_RANDOM_SEED: u32 = 407;
    pub const PERFORMANCE_JS_HEAP_SIZE_LIMIT: u32 = 408;
    pub const PERFORMANCE_TOTAL_JS_HEAP_SIZE: u32 = 409;
    pub const PERFORMANCE_USED_JS_HEAP_SIZE: u32 = 410;
    pub const CONSOLE_JS_HEAP_SIZE_LIMIT: u32 = 411;
    pub const CONSOLE_TOTAL_JS_HEAP_SIZE: u32 = 412;
    pub const CONSOLE_USED_JS_HEAP_SIZE: u32 = 413;

    pub const TIMING_CLOCK_EPOCH_MS: u32 = 900;

    pub const DEVICE_MEMORY_GB: u32 = 500;
    pub const NETWORK_DOWNLINK: u32 = 501;
    pub const SCREEN_VIEWPORT_WIDTH: u32 = 502;
    pub const SCREEN_VIEWPORT_HEIGHT: u32 = 503;
    pub const SCREEN_OUTER_WIDTH: u32 = 504;
    pub const SCREEN_OUTER_HEIGHT: u32 = 505;
    pub const WINDOW_INNER_WIDTH: u32 = SCREEN_VIEWPORT_WIDTH;
    pub const WINDOW_INNER_HEIGHT: u32 = SCREEN_VIEWPORT_HEIGHT;
    pub const WINDOW_OUTER_WIDTH: u32 = SCREEN_OUTER_WIDTH;
    pub const WINDOW_OUTER_HEIGHT: u32 = SCREEN_OUTER_HEIGHT;
    pub const SCREEN_X: u32 = 506;
    pub const SCREEN_Y: u32 = 507;
    pub const SCREEN_DEVICE_PIXEL_RATIO: u32 = 508;
    pub const CANVAS_TEXT_WIDTH_SCALE: u32 = 509;
    pub const WEBGL_MAX_ANISOTROPY: u32 = 510;
    pub const AUDIO_SAMPLE_RATE: u32 = 511;
    pub const AUDIO_BASE_LATENCY: u32 = 512;
    pub const AUDIO_OUTPUT_LATENCY: u32 = 513;
    pub const BATTERY_CHARGING_TIME: u32 = 514;
    pub const BATTERY_DISCHARGING_TIME: u32 = 515;
    pub const BATTERY_LEVEL: u32 = 516;
    pub const GEOLOCATION_LATITUDE: u32 = 517;
    pub const GEOLOCATION_LONGITUDE: u32 = 518;
    pub const GEOLOCATION_ALTITUDE: u32 = 519;
    pub const GEOLOCATION_ACCURACY: u32 = 520;
    pub const GEOLOCATION_ALTITUDE_ACCURACY: u32 = 521;
    pub const GEOLOCATION_HEADING: u32 = 522;
    pub const GEOLOCATION_SPEED: u32 = 523;
    pub const WEBGL_ALIASED_POINT_SIZE_MIN: u32 = 524;
    pub const WEBGL_ALIASED_POINT_SIZE_MAX: u32 = 525;
    pub const WEBGL_ALIASED_LINE_WIDTH_MIN: u32 = 526;
    pub const WEBGL_ALIASED_LINE_WIDTH_MAX: u32 = 527;
    pub const SENSOR_ACCELEROMETER_X: u32 = 528;
    pub const SENSOR_ACCELEROMETER_Y: u32 = 529;
    pub const SENSOR_ACCELEROMETER_Z: u32 = 530;
    pub const SENSOR_GRAVITY_X: u32 = 531;
    pub const SENSOR_GRAVITY_Y: u32 = 532;
    pub const SENSOR_GRAVITY_Z: u32 = 533;
    pub const SENSOR_LINEAR_ACCELERATION_X: u32 = 534;
    pub const SENSOR_LINEAR_ACCELERATION_Y: u32 = 535;
    pub const SENSOR_LINEAR_ACCELERATION_Z: u32 = 536;
    pub const SENSOR_GYROSCOPE_X: u32 = 537;
    pub const SENSOR_GYROSCOPE_Y: u32 = 538;
    pub const SENSOR_GYROSCOPE_Z: u32 = 539;
    pub const SENSOR_ABSOLUTE_ORIENTATION_X: u32 = 540;
    pub const SENSOR_ABSOLUTE_ORIENTATION_Y: u32 = 541;
    pub const SENSOR_ABSOLUTE_ORIENTATION_Z: u32 = 542;
    pub const SENSOR_ABSOLUTE_ORIENTATION_W: u32 = 543;
    pub const SENSOR_RELATIVE_ORIENTATION_X: u32 = 544;
    pub const SENSOR_RELATIVE_ORIENTATION_Y: u32 = 545;
    pub const SENSOR_RELATIVE_ORIENTATION_Z: u32 = 546;
    pub const SENSOR_RELATIVE_ORIENTATION_W: u32 = 547;
    pub const CANVAS_ACTUAL_BOUNDING_BOX_LEFT: u32 = 548;
    pub const CANVAS_ACTUAL_BOUNDING_BOX_RIGHT_SCALE: u32 = 549;
    pub const CANVAS_FONT_BOUNDING_BOX_ASCENT: u32 = 550;
    pub const CANVAS_FONT_BOUNDING_BOX_DESCENT: u32 = 551;
    pub const CANVAS_ACTUAL_BOUNDING_BOX_ASCENT: u32 = 552;
    pub const CANVAS_ACTUAL_BOUNDING_BOX_DESCENT: u32 = 553;
    pub const CANVAS_HANGING_BASELINE: u32 = 554;
    pub const CANVAS_ALPHABETIC_BASELINE: u32 = 555;
    pub const CANVAS_IDEOGRAPHIC_BASELINE: u32 = 556;
    pub const VISUAL_VIEWPORT_OFFSET_LEFT: u32 = 557;
    pub const VISUAL_VIEWPORT_OFFSET_TOP: u32 = 558;
    pub const VISUAL_VIEWPORT_PAGE_LEFT: u32 = 559;
    pub const VISUAL_VIEWPORT_PAGE_TOP: u32 = 560;
    pub const VISUAL_VIEWPORT_SCALE: u32 = 561;
    pub const WEBGL2_MAX_TEXTURE_LOD_BIAS: u32 = 562;
    pub const DOCUMENT_BODY_CLIENT_HEIGHT: u32 = 563;

    pub const AUDIO_CHANNEL_NOISE_AMPLITUDE: u32 = 600;
    pub const AUDIO_FREQUENCY_NOISE_AMPLITUDE: u32 = 601;
    pub const AUDIO_TIME_DOMAIN_NOISE_AMPLITUDE: u32 = 602;

    pub const NAVIGATOR_COOKIE_ENABLED: u32 = 700;
    pub const NAVIGATOR_ON_LINE: u32 = 701;
    pub const NAVIGATOR_WEBDRIVER: u32 = 702;
    pub const NAVIGATOR_PDF_VIEWER_ENABLED: u32 = 703;
    pub const UA_MOBILE: u32 = 704;
    pub const UA_WOW64: u32 = 705;
    pub const NETWORK_SAVE_DATA: u32 = 706;
    pub const STORAGE_PERSISTED: u32 = 707;
    pub const FONT_ALLOW_UNKNOWN_FAMILIES: u32 = 708;
    pub const BATTERY_CHARGING: u32 = 709;
    pub const MEDIA_PREFERENCE_REDUCED_MOTION: u32 = 710;
    pub const MEDIA_PREFERENCE_REDUCED_DATA: u32 = 711;
    pub const MEDIA_PREFERENCE_FORCED_COLORS: u32 = 712;
    pub const MEDIA_PREFERENCE_INVERTED_COLORS: u32 = 713;
    pub const WEBGL_CONTEXT_ALPHA: u32 = 714;
    pub const WEBGL_CONTEXT_ANTIALIAS: u32 = 715;
    pub const WEBGL_CONTEXT_DEPTH: u32 = 716;
    pub const WEBGL_CONTEXT_DESYNCHRONIZED: u32 = 717;
    pub const WEBGL_CONTEXT_FAIL_IF_MAJOR_PERFORMANCE_CAVEAT: u32 = 718;
    pub const WEBGL_CONTEXT_PREMULTIPLIED_ALPHA: u32 = 719;
    pub const WEBGL_CONTEXT_PRESERVE_DRAWING_BUFFER: u32 = 720;
    pub const WEBGL_CONTEXT_STENCIL: u32 = 721;
    pub const WEBGL_CONTEXT_XR_COMPATIBLE: u32 = 722;
    pub const BLUETOOTH_AVAILABLE: u32 = 723;
    pub const MIDI_SYSEX_ENABLED: u32 = 724;
    pub const WEBGPU_DEVELOPER_FEATURES: u32 = 725;
    pub const WEBGPU_IS_FALLBACK_ADAPTER: u32 = 726;
    pub const SENSORS_AVAILABLE: u32 = 727;
    pub const WEBGPU_AVAILABLE: u32 = 728;
    pub const MEDIA_PREFERENCE_REDUCED_TRANSPARENCY: u32 = 729;
    pub const NAVIGATOR_USER_ACTIVATION_HAS_BEEN_ACTIVE: u32 = 730;
    pub const NAVIGATOR_USER_ACTIVATION_IS_ACTIVE: u32 = 731;
}

pub struct EdgeSandboxHandle {
    runtime: crate::IsolatedEdgeRuntime,
}

fn input_bytes<'a>(data: *const u8, len: usize) -> Result<&'a [u8], String> {
    if len == 0 {
        return Ok(&[]);
    }
    if data.is_null() {
        return Err("input pointer is null".to_owned());
    }
    // SAFETY: the FFI caller promises that `data` addresses `len` readable
    // bytes and keeps them alive for the duration of the call.
    Ok(unsafe { std::slice::from_raw_parts(data, len) })
}

fn input_string(data: *const u8, len: usize, name: &str) -> Result<String, String> {
    String::from_utf8(input_bytes(data, len)?.to_vec())
        .map_err(|_| format!("{name} is not valid UTF-8"))
}

unsafe fn input_string_views(
    values: *const EdgeSandboxStringView,
    len: usize,
    name: &str,
) -> Result<Vec<String>, String> {
    if len == 0 {
        return Ok(Vec::new());
    }
    if values.is_null() {
        return Err(format!("{name} pointer is null"));
    }
    // SAFETY: the caller guarantees `values` addresses `len` readable views.
    let views = unsafe { std::slice::from_raw_parts(values, len) };
    views
        .iter()
        .enumerate()
        .map(|(index, value)| input_string(value.data, value.len, &format!("{name}[{index}]")))
        .collect()
}

unsafe fn input_f64_values(data: *const f64, len: usize, name: &str) -> Result<Vec<f64>, String> {
    if len == 0 {
        return Ok(Vec::new());
    }
    if data.is_null() {
        return Err(format!("{name} pointer is null"));
    }
    // SAFETY: the enclosing FFI caller promises `len` readable f64 values.
    Ok(unsafe { std::slice::from_raw_parts(data, len) }.to_vec())
}

fn reset_buffer(output: *mut EdgeSandboxBuffer) {
    if output.is_null() {
        return;
    }
    // SAFETY: the caller supplied a writable output structure.
    unsafe {
        *output = EdgeSandboxBuffer::default();
    }
}

fn write_buffer(output: *mut EdgeSandboxBuffer, value: String) -> Result<(), String> {
    write_byte_buffer(output, value.into_bytes())
}

fn write_byte_buffer(output: *mut EdgeSandboxBuffer, value: Vec<u8>) -> Result<(), String> {
    if output.is_null() {
        return Err("output buffer pointer is null".to_owned());
    }
    if value.is_empty() {
        reset_buffer(output);
        return Ok(());
    }
    let bytes = value.into_boxed_slice();
    let len = bytes.len();
    let data = Box::into_raw(bytes).cast::<u8>();
    // SAFETY: the caller supplied a writable output structure and ownership
    // of the boxed byte slice is transferred to it.
    unsafe {
        *output = EdgeSandboxBuffer { data, len };
    }
    Ok(())
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return format!("edge-sandbox native API panicked: {message}");
    }
    if let Some(message) = payload.downcast_ref::<&str>() {
        return format!("edge-sandbox native API panicked: {message}");
    }
    "edge-sandbox native API panicked".to_owned()
}

fn report_error(output: *mut EdgeSandboxBuffer, message: String) {
    let _ = write_buffer(output, message);
}

fn profile_operation(
    error_out: *mut EdgeSandboxBuffer,
    operation: impl FnOnce() -> Result<(), String>,
) -> bool {
    reset_buffer(error_out);
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(())) => true,
        Ok(Err(message)) => {
            report_error(error_out, message);
            false
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            false
        }
    }
}

unsafe fn profile_mut<'a>(
    profile: *mut EdgeSandboxProfile,
) -> Result<&'a mut EdgeSandboxProfile, String> {
    // SAFETY: the caller of the enclosing FFI function guarantees that the
    // pointer is live, uniquely borrowed, and was returned by this library.
    unsafe {
        profile
            .as_mut()
            .ok_or_else(|| "Edge profile handle is null".to_owned())
    }
}

unsafe fn profile_ref<'a>(
    profile: *const EdgeSandboxProfile,
) -> Result<&'a EdgeSandboxProfile, String> {
    // SAFETY: the caller of the enclosing FFI function guarantees that the
    // pointer is live and was returned by this library.
    unsafe {
        profile
            .as_ref()
            .ok_or_else(|| "Edge profile handle is null".to_owned())
    }
}

unsafe fn options_mut<'a>(
    options: *mut EdgeSandboxOptions,
) -> Result<&'a mut EdgeSandboxOptions, String> {
    // SAFETY: the caller of the enclosing FFI function guarantees that the
    // pointer is live, uniquely borrowed, and was returned by this library.
    unsafe {
        options
            .as_mut()
            .ok_or_else(|| "Edge options handle is null".to_owned())
    }
}

unsafe fn options_ref<'a>(
    options: *const EdgeSandboxOptions,
) -> Result<&'a EdgeSandboxOptions, String> {
    // SAFETY: the caller of the enclosing FFI function guarantees that the
    // pointer is live and was returned by this library.
    unsafe {
        options
            .as_ref()
            .ok_or_else(|| "Edge options handle is null".to_owned())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn edge_sandbox_profile_schema_version() -> u32 {
    10
}

fn performance_profile_string(value: EdgeSandboxStringView, name: &str) -> Result<String, String> {
    input_string(value.data, value.len, name)
}

/// Enables an exact ordered Performance Timeline override and removes any
/// previously appended records. An empty override makes `getEntries()` empty.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_performance_entries(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .performance
            .entries = Some(Vec::new());
        Ok(())
    })
}

/// Appends one typed root-realm Performance Timeline record.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_performance_entry(
    profile: *mut EdgeSandboxProfile,
    value: *const EdgeSandboxPerformanceEntryProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        let value = unsafe {
            value
                .as_ref()
                .ok_or_else(|| "performance entry profile pointer is null".to_owned())?
        };
        if value.reserved != [0; 8]
            || value.has_transfer_size > 1
            || value.has_encoded_body_size > 1
            || value.has_decoded_body_size > 1
            || value.has_response_status > 1
        {
            return Err("performance entry profile flags are invalid".to_owned());
        }
        let entry = crate::PerformanceEntryFingerprint {
            name: performance_profile_string(value.name, "performance entry name")?,
            entry_type: performance_profile_string(value.entry_type, "performance entry type")?,
            start_time: value.start_time,
            duration: value.duration,
            initiator_type: performance_profile_string(
                value.initiator_type,
                "performance initiator type",
            )?,
            delivery_type: performance_profile_string(
                value.delivery_type,
                "performance delivery type",
            )?,
            next_hop_protocol: performance_profile_string(
                value.next_hop_protocol,
                "performance next hop protocol",
            )?,
            render_blocking_status: performance_profile_string(
                value.render_blocking_status,
                "performance render blocking status",
            )?,
            content_type: performance_profile_string(
                value.content_type,
                "performance content type",
            )?,
            content_encoding: performance_profile_string(
                value.content_encoding,
                "performance content encoding",
            )?,
            worker_start: value.worker_start,
            worker_router_evaluation_start: value.worker_router_evaluation_start,
            worker_cache_lookup_start: value.worker_cache_lookup_start,
            worker_matched_source_type: performance_profile_string(
                value.worker_matched_source_type,
                "performance worker matched source type",
            )?,
            worker_final_source_type: performance_profile_string(
                value.worker_final_source_type,
                "performance worker final source type",
            )?,
            redirect_start: value.redirect_start,
            redirect_end: value.redirect_end,
            fetch_start: value.fetch_start,
            domain_lookup_start: value.domain_lookup_start,
            domain_lookup_end: value.domain_lookup_end,
            connect_start: value.connect_start,
            secure_connection_start: value.secure_connection_start,
            connect_end: value.connect_end,
            request_start: value.request_start,
            response_start: value.response_start,
            first_interim_response_start: value.first_interim_response_start,
            final_response_headers_start: value.final_response_headers_start,
            response_end: value.response_end,
            transfer_size: (value.has_transfer_size != 0).then_some(value.transfer_size),
            encoded_body_size: (value.has_encoded_body_size != 0)
                .then_some(value.encoded_body_size),
            decoded_body_size: (value.has_decoded_body_size != 0)
                .then_some(value.decoded_body_size),
            response_status: (value.has_response_status != 0).then_some(value.response_status),
            unload_event_start: value.unload_event_start,
            unload_event_end: value.unload_event_end,
            dom_interactive: value.dom_interactive,
            dom_content_loaded_event_start: value.dom_content_loaded_event_start,
            dom_content_loaded_event_end: value.dom_content_loaded_event_end,
            dom_complete: value.dom_complete,
            load_event_start: value.load_event_start,
            load_event_end: value.load_event_end,
            navigation_type: performance_profile_string(
                value.navigation_type,
                "performance navigation type",
            )?,
            redirect_count: value.redirect_count,
            critical_ch_restart: value.critical_ch_restart,
            activation_start: value.activation_start,
            paint_time: value.paint_time,
            presentation_time: value.presentation_time,
        };
        let entries = unsafe { profile_mut(profile)? }
            .fingerprint
            .performance
            .entries
            .get_or_insert_with(Vec::new);
        entries.push(entry);
        Ok(())
    })
}

/// Allocates a profile builder initialized with the fixed Chrome 150 defaults.
#[unsafe(no_mangle)]
pub extern "C" fn edge_sandbox_profile_create(
    error_out: *mut EdgeSandboxBuffer,
) -> *mut EdgeSandboxProfile {
    reset_buffer(error_out);
    match catch_unwind(AssertUnwindSafe(|| {
        Box::into_raw(Box::new(EdgeSandboxProfile {
            fingerprint: crate::EdgeFingerprint::default(),
        }))
    })) {
        Ok(profile) => profile,
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            ptr::null_mut()
        }
    }
}

/// Releases a profile builder returned by [`edge_sandbox_profile_create`].
///
/// # Safety
///
/// `profile` must be null or a live pointer returned by this library, and it
/// must be destroyed at most once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_destroy(profile: *mut EdgeSandboxProfile) {
    if !profile.is_null() {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe {
            drop(Box::from_raw(profile));
        }
    }
}

/// Sets a UTF-8 scalar field selected by the stable profile field ID.
///
/// # Safety
///
/// `profile` must be live, `data` must address `len` readable bytes, and
/// `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_set_string(
    profile: *mut EdgeSandboxProfile,
    field: u32,
    data: *const u8,
    len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let value = input_string(data, len, "profile string")?;
        // SAFETY: guaranteed by this function's FFI contract.
        let fingerprint = &mut unsafe { profile_mut(profile)? }.fingerprint;
        match field {
            profile_field::ID => fingerprint.id = value,
            profile_field::LOCALE => fingerprint.locale.locale = value,
            profile_field::TIME_ZONE => fingerprint.locale.time_zone = value,
            profile_field::NAVIGATOR_USER_AGENT => {
                fingerprint.navigator.app_version =
                    value.strip_prefix("Mozilla/").unwrap_or(&value).to_owned();
                fingerprint.navigator.user_agent = value;
            }
            profile_field::NAVIGATOR_APP_VERSION => {
                fingerprint.navigator.app_version = value;
            }
            profile_field::NAVIGATOR_APP_CODE_NAME => {
                fingerprint.navigator.app_code_name = value;
            }
            profile_field::NAVIGATOR_APP_NAME => fingerprint.navigator.app_name = value,
            profile_field::NAVIGATOR_PLATFORM => fingerprint.navigator.platform = value,
            profile_field::NAVIGATOR_PRODUCT => fingerprint.navigator.product = value,
            profile_field::NAVIGATOR_PRODUCT_SUB => fingerprint.navigator.product_sub = value,
            profile_field::NAVIGATOR_VENDOR => fingerprint.navigator.vendor = value,
            profile_field::NAVIGATOR_VENDOR_SUB => fingerprint.navigator.vendor_sub = value,
            profile_field::NAVIGATOR_LANGUAGE => {
                fingerprint.navigator.language = value.clone();
                if let Some(first) = fingerprint.navigator.languages.first_mut() {
                    *first = value;
                } else {
                    fingerprint.navigator.languages.push(value);
                }
            }
            profile_field::NAVIGATOR_DO_NOT_TRACK => {
                fingerprint.navigator.do_not_track = Some(value);
            }
            profile_field::UA_PLATFORM => {
                fingerprint.navigator.user_agent_data.platform = value;
            }
            profile_field::UA_ARCHITECTURE => {
                fingerprint.navigator.user_agent_data.architecture = value;
            }
            profile_field::UA_BITNESS => {
                fingerprint.navigator.user_agent_data.bitness = value;
            }
            profile_field::UA_MODEL => fingerprint.navigator.user_agent_data.model = value,
            profile_field::UA_PLATFORM_VERSION => {
                fingerprint.navigator.user_agent_data.platform_version = value;
            }
            profile_field::UA_FULL_VERSION => {
                fingerprint.navigator.user_agent_data.ua_full_version = value;
            }
            profile_field::NETWORK_EFFECTIVE_TYPE => {
                fingerprint.navigator.network.effective_type = value;
            }
            profile_field::CANVAS_DATA_URL_SALT => {
                fingerprint.rendering.canvas.data_url_salt = value;
            }
            profile_field::WEBGL_VENDOR => fingerprint.rendering.webgl.vendor = value,
            profile_field::WEBGL_RENDERER => fingerprint.rendering.webgl.renderer = value,
            profile_field::WEBGL_UNMASKED_VENDOR => {
                fingerprint.rendering.webgl.unmasked_vendor = value;
            }
            profile_field::WEBGL_UNMASKED_RENDERER => {
                fingerprint.rendering.webgl.unmasked_renderer = value;
            }
            profile_field::WEBGL1_VERSION => {
                fingerprint.rendering.webgl.webgl1_version = value;
            }
            profile_field::WEBGL1_SHADING_LANGUAGE_VERSION => {
                fingerprint.rendering.webgl.webgl1_shading_language_version = value;
            }
            profile_field::WEBGL2_VERSION => {
                fingerprint.rendering.webgl.webgl2_version = value;
            }
            profile_field::WEBGL2_SHADING_LANGUAGE_VERSION => {
                fingerprint.rendering.webgl.webgl2_shading_language_version = value;
            }
            profile_field::WEBGL_CONTEXT_POWER_PREFERENCE => {
                fingerprint.rendering.webgl.context_power_preference = value;
            }
            profile_field::WEBGPU_VENDOR => fingerprint.rendering.webgpu.vendor = value,
            profile_field::WEBGPU_ARCHITECTURE => {
                fingerprint.rendering.webgpu.architecture = value;
            }
            profile_field::WEBGPU_DEVICE => fingerprint.rendering.webgpu.device = value,
            profile_field::WEBGPU_DESCRIPTION => {
                fingerprint.rendering.webgpu.description = value;
            }
            profile_field::RTC_OFFER_SDP => fingerprint.media.rtc_offer_sdp = value,
            profile_field::RTC_ANSWER_SDP => fingerprint.media.rtc_answer_sdp = value,
            profile_field::SCREEN_ORIENTATION_TYPE => fingerprint.screen.orientation_type = value,
            profile_field::DEVICE_POSTURE => {
                fingerprint.hardware_devices.device_posture = value;
            }
            profile_field::CSS_BODY => fingerprint.css.body = value,
            profile_field::CSS_INPUT_COMMON => fingerprint.css.input_common = value,
            profile_field::CSS_INPUT_HIDDEN => fingerprint.css.input_hidden = value,
            profile_field::CSS_INPUT_SEARCH => fingerprint.css.input_search = value,
            profile_field::CSS_INPUT_CHECKBOX_RADIO => {
                fingerprint.css.input_checkbox_radio = value;
            }
            profile_field::CSS_INPUT_RANGE => fingerprint.css.input_range = value,
            profile_field::CSS_INPUT_COLOR => fingerprint.css.input_color = value,
            profile_field::CSS_INPUT_DATE => fingerprint.css.input_date = value,
            profile_field::CSS_INPUT_TIME => fingerprint.css.input_time = value,
            profile_field::CSS_INPUT_DATETIME_LOCAL => {
                fingerprint.css.input_datetime_local = value;
            }
            profile_field::CSS_INPUT_MONTH => fingerprint.css.input_month = value,
            profile_field::CSS_INPUT_WEEK => fingerprint.css.input_week = value,
            profile_field::CSS_INPUT_IMAGE => fingerprint.css.input_image = value,
            profile_field::CSS_INPUT_BUTTON => fingerprint.css.input_button = value,
            profile_field::CSS_INPUT_SUBMIT_RESET => {
                fingerprint.css.input_submit_reset = value;
            }
            profile_field::CSS_INPUT_FILE => fingerprint.css.input_file = value,
            profile_field::CSS_INPUT_TEXT => fingerprint.css.input_text = value,
            profile_field::PERFORMANCE_EVALUATED_SCRIPT_CONTENT_ENCODING => {
                fingerprint.performance.evaluated_script_content_encoding = value;
            }
            profile_field::PERMISSION_ACCELEROMETER => {
                fingerprint.permissions.accelerometer = value;
            }
            profile_field::PERMISSION_BACKGROUND_SYNC => {
                fingerprint.permissions.background_sync = value;
            }
            profile_field::PERMISSION_CAMERA => fingerprint.permissions.camera = value,
            profile_field::PERMISSION_CLIPBOARD_READ => {
                fingerprint.permissions.clipboard_read = value;
            }
            profile_field::PERMISSION_CLIPBOARD_WRITE => {
                fingerprint.permissions.clipboard_write = value;
            }
            profile_field::PERMISSION_GEOLOCATION => {
                fingerprint.permissions.geolocation = value;
            }
            profile_field::PERMISSION_GYROSCOPE => fingerprint.permissions.gyroscope = value,
            profile_field::PERMISSION_LOCAL_FONTS => {
                fingerprint.permissions.local_fonts = value;
            }
            profile_field::PERMISSION_MAGNETOMETER => {
                fingerprint.permissions.magnetometer = value;
            }
            profile_field::PERMISSION_MICROPHONE => {
                fingerprint.permissions.microphone = value;
            }
            profile_field::PERMISSION_MIDI => fingerprint.permissions.midi = value,
            profile_field::PERMISSION_NOTIFICATIONS => {
                fingerprint.permissions.notifications = value;
            }
            profile_field::PERMISSION_PAYMENT_HANDLER => {
                fingerprint.permissions.payment_handler = value;
            }
            profile_field::PERMISSION_PERSISTENT_STORAGE => {
                fingerprint.permissions.persistent_storage = value;
            }
            profile_field::PERMISSION_SPEAKER_SELECTION => {
                fingerprint.permissions.speaker_selection = value;
            }
            profile_field::PERMISSION_STORAGE_ACCESS => {
                fingerprint.permissions.storage_access = value;
            }
            profile_field::PERMISSION_TOP_LEVEL_STORAGE_ACCESS => {
                fingerprint.permissions.top_level_storage_access = value;
            }
            profile_field::PERMISSION_WINDOW_MANAGEMENT => {
                fingerprint.permissions.window_management = value;
            }
            profile_field::MEDIA_PREFERENCE_COLOR_SCHEME => {
                fingerprint.media_preferences.color_scheme = value;
            }
            profile_field::MEDIA_PREFERENCE_CONTRAST => {
                fingerprint.media_preferences.contrast = value;
            }
            profile_field::MEDIA_PREFERENCE_COLOR_GAMUT => {
                fingerprint.media_preferences.color_gamut = value;
            }
            profile_field::MEDIA_PREFERENCE_POINTER => {
                fingerprint.media_preferences.pointer = value;
            }
            profile_field::MEDIA_PREFERENCE_ANY_POINTER => {
                fingerprint.media_preferences.any_pointer = value;
            }
            profile_field::MEDIA_PREFERENCE_HOVER => {
                fingerprint.media_preferences.hover = value;
            }
            profile_field::MEDIA_PREFERENCE_ANY_HOVER => {
                fingerprint.media_preferences.any_hover = value;
            }
            profile_field::MEDIA_PREFERENCE_DISPLAY_MODE => {
                fingerprint.media_preferences.display_mode = value;
            }
            profile_field::MEDIA_PREFERENCE_DYNAMIC_RANGE => {
                fingerprint.media_preferences.dynamic_range = value;
            }
            profile_field::MEDIA_PREFERENCE_SCRIPTING => {
                fingerprint.media_preferences.scripting = value;
            }
            profile_field::MEDIA_PREFERENCE_VIDEO_DYNAMIC_RANGE => {
                fingerprint.media_preferences.video_dynamic_range = value;
            }
            _ => return Err(format!("unknown string profile field {field}")),
        }
        Ok(())
    })
}

/// Clears an optional profile string.
///
/// # Safety
///
/// `profile` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_optional_string(
    profile: *mut EdgeSandboxProfile,
    field: u32,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        let fingerprint = &mut unsafe { profile_mut(profile)? }.fingerprint;
        match field {
            profile_field::NAVIGATOR_DO_NOT_TRACK => {
                fingerprint.navigator.do_not_track = None;
            }
            _ => return Err(format!("unknown optional-string profile field {field}")),
        }
        Ok(())
    })
}

fn string_list_mut(
    fingerprint: &mut crate::EdgeFingerprint,
    field: u32,
) -> Result<&mut Vec<String>, String> {
    match field {
        profile_field::NAVIGATOR_LANGUAGES => Ok(&mut fingerprint.navigator.languages),
        profile_field::UA_FORM_FACTORS => {
            Ok(&mut fingerprint.navigator.user_agent_data.form_factors)
        }
        profile_field::WEBGL1_EXTENSIONS => Ok(&mut fingerprint.rendering.webgl.webgl1_extensions),
        profile_field::WEBGL2_EXTENSIONS => Ok(&mut fingerprint.rendering.webgl.webgl2_extensions),
        profile_field::WEBGPU_FEATURES => Ok(&mut fingerprint.rendering.webgpu.features),
        profile_field::FONT_FAMILIES => Ok(&mut fingerprint.fonts.families),
        profile_field::MEDIA_SUPPORTED_CONSTRAINTS => {
            Ok(&mut fingerprint.media.supported_constraints)
        }
        profile_field::MEDIA_CAN_PLAY_PROBABLY_TYPES => {
            Ok(&mut fingerprint.media.can_play_probably_types)
        }
        profile_field::MEDIA_CAN_PLAY_MAYBE_TYPES => {
            Ok(&mut fingerprint.media.can_play_maybe_types)
        }
        profile_field::MEDIA_SOURCE_TYPES => Ok(&mut fingerprint.media.media_source_types),
        profile_field::MEDIA_RECORDER_TYPES => Ok(&mut fingerprint.media.media_recorder_types),
        profile_field::MEDIA_DECODING_SUPPORTED_TYPES => {
            Ok(&mut fingerprint.media.decoding_supported_types)
        }
        profile_field::MEDIA_DECODING_SMOOTH_TYPES => {
            Ok(&mut fingerprint.media.decoding_smooth_types)
        }
        profile_field::MEDIA_DECODING_POWER_EFFICIENT_TYPES => {
            Ok(&mut fingerprint.media.decoding_power_efficient_types)
        }
        profile_field::MEDIA_ENCODING_SUPPORTED_TYPES => {
            Ok(&mut fingerprint.media.encoding_supported_types)
        }
        profile_field::MEDIA_ENCODING_SMOOTH_TYPES => {
            Ok(&mut fingerprint.media.encoding_smooth_types)
        }
        profile_field::MEDIA_ENCODING_POWER_EFFICIENT_TYPES => {
            Ok(&mut fingerprint.media.encoding_power_efficient_types)
        }
        profile_field::IMAGE_DECODER_TYPES => Ok(&mut fingerprint.media.image_decoder_types),
        profile_field::XR_SUPPORTED_SESSION_MODES => {
            Ok(&mut fingerprint.xr.supported_session_modes)
        }
        profile_field::AUDIO_DECODER_CODECS => Ok(&mut fingerprint.media.audio_decoder_codecs),
        profile_field::AUDIO_ENCODER_CODECS => Ok(&mut fingerprint.media.audio_encoder_codecs),
        profile_field::VIDEO_DECODER_CODECS => Ok(&mut fingerprint.media.video_decoder_codecs),
        profile_field::VIDEO_ENCODER_CODECS => Ok(&mut fingerprint.media.video_encoder_codecs),
        _ => Err(format!("unknown string-list profile field {field}")),
    }
}

/// Clears a typed string-list field before appending replacement values.
///
/// # Safety
///
/// `profile` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_string_list(
    profile: *mut EdgeSandboxProfile,
    field: u32,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        let fingerprint = &mut unsafe { profile_mut(profile)? }.fingerprint;
        string_list_mut(fingerprint, field)?.clear();
        Ok(())
    })
}

/// Appends one UTF-8 value to a typed string-list field.
///
/// # Safety
///
/// `profile` must be live, `data` must address `len` readable bytes, and
/// `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_string(
    profile: *mut EdgeSandboxProfile,
    field: u32,
    data: *const u8,
    len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let value = input_string(data, len, "profile list string")?;
        // SAFETY: guaranteed by this function's FFI contract.
        let fingerprint = &mut unsafe { profile_mut(profile)? }.fingerprint;
        string_list_mut(fingerprint, field)?.push(value);
        Ok(())
    })
}

/// Sets an unsigned 32-bit profile field.
///
/// # Safety
///
/// `profile` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_set_u32(
    profile: *mut EdgeSandboxProfile,
    field: u32,
    value: u32,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        let fingerprint = &mut unsafe { profile_mut(profile)? }.fingerprint;
        match field {
            profile_field::HARDWARE_CONCURRENCY => {
                fingerprint.navigator.hardware_concurrency = value;
            }
            profile_field::MAX_TOUCH_POINTS => fingerprint.navigator.max_touch_points = value,
            profile_field::NETWORK_RTT => fingerprint.navigator.network.rtt = value,
            profile_field::WEBGPU_MAX_TEXTURE_DIMENSION_2D => {
                fingerprint.rendering.webgpu.max_texture_dimension_2d = value;
            }
            profile_field::AUDIO_MAX_CHANNEL_COUNT => {
                fingerprint.rendering.audio.max_channel_count = value;
            }
            profile_field::MEDIA_PREFERENCE_MONOCHROME_BITS => {
                fingerprint.media_preferences.monochrome_bits = value;
            }
            profile_field::DOCUMENT_BODY_CHILD_ELEMENT_COUNT => {
                fingerprint.document.body_child_element_count = Some(value);
            }
            profile_field::WEBGPU_MAX_TEXTURE_DIMENSION_1D => {
                fingerprint.rendering.webgpu.max_texture_dimension_1d = value;
            }
            profile_field::WEBGPU_MAX_TEXTURE_DIMENSION_3D => {
                fingerprint.rendering.webgpu.max_texture_dimension_3d = value;
            }
            profile_field::WEBGPU_MAX_TEXTURE_ARRAY_LAYERS => {
                fingerprint.rendering.webgpu.max_texture_array_layers = value;
            }
            profile_field::WEBGPU_MAX_BIND_GROUPS => {
                fingerprint.rendering.webgpu.max_bind_groups = value;
            }
            profile_field::WEBGPU_MAX_BIND_GROUPS_PLUS_VERTEX_BUFFERS => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_bind_groups_plus_vertex_buffers = value;
            }
            profile_field::WEBGPU_MAX_BINDINGS_PER_BIND_GROUP => {
                fingerprint.rendering.webgpu.max_bindings_per_bind_group = value;
            }
            profile_field::WEBGPU_MAX_DYNAMIC_UNIFORM_BUFFERS_PER_PIPELINE_LAYOUT => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_dynamic_uniform_buffers_per_pipeline_layout = value;
            }
            profile_field::WEBGPU_MAX_DYNAMIC_STORAGE_BUFFERS_PER_PIPELINE_LAYOUT => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_dynamic_storage_buffers_per_pipeline_layout = value;
            }
            profile_field::WEBGPU_MAX_SAMPLED_TEXTURES_PER_SHADER_STAGE => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_sampled_textures_per_shader_stage = value;
            }
            profile_field::WEBGPU_MAX_SAMPLERS_PER_SHADER_STAGE => {
                fingerprint.rendering.webgpu.max_samplers_per_shader_stage = value;
            }
            profile_field::WEBGPU_MAX_STORAGE_BUFFERS_PER_SHADER_STAGE => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_storage_buffers_per_shader_stage = value;
            }
            profile_field::WEBGPU_MAX_STORAGE_TEXTURES_PER_SHADER_STAGE => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_storage_textures_per_shader_stage = value;
            }
            profile_field::WEBGPU_MAX_UNIFORM_BUFFERS_PER_SHADER_STAGE => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_uniform_buffers_per_shader_stage = value;
            }
            profile_field::WEBGPU_MIN_UNIFORM_BUFFER_OFFSET_ALIGNMENT => {
                fingerprint
                    .rendering
                    .webgpu
                    .min_uniform_buffer_offset_alignment = value;
            }
            profile_field::WEBGPU_MIN_STORAGE_BUFFER_OFFSET_ALIGNMENT => {
                fingerprint
                    .rendering
                    .webgpu
                    .min_storage_buffer_offset_alignment = value;
            }
            profile_field::WEBGPU_MAX_VERTEX_BUFFERS => {
                fingerprint.rendering.webgpu.max_vertex_buffers = value;
            }
            profile_field::WEBGPU_MAX_VERTEX_ATTRIBUTES => {
                fingerprint.rendering.webgpu.max_vertex_attributes = value;
            }
            profile_field::WEBGPU_MAX_VERTEX_BUFFER_ARRAY_STRIDE => {
                fingerprint.rendering.webgpu.max_vertex_buffer_array_stride = value;
            }
            profile_field::WEBGPU_MAX_INTER_STAGE_SHADER_VARIABLES => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_inter_stage_shader_variables = value;
            }
            profile_field::WEBGPU_MAX_COLOR_ATTACHMENTS => {
                fingerprint.rendering.webgpu.max_color_attachments = value;
            }
            profile_field::WEBGPU_MAX_COLOR_ATTACHMENT_BYTES_PER_SAMPLE => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_color_attachment_bytes_per_sample = value;
            }
            profile_field::WEBGPU_MAX_COMPUTE_WORKGROUP_STORAGE_SIZE => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_compute_workgroup_storage_size = value;
            }
            profile_field::WEBGPU_MAX_COMPUTE_INVOCATIONS_PER_WORKGROUP => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_compute_invocations_per_workgroup = value;
            }
            profile_field::WEBGPU_MAX_COMPUTE_WORKGROUP_SIZE_X => {
                fingerprint.rendering.webgpu.max_compute_workgroup_size_x = value;
            }
            profile_field::WEBGPU_MAX_COMPUTE_WORKGROUP_SIZE_Y => {
                fingerprint.rendering.webgpu.max_compute_workgroup_size_y = value;
            }
            profile_field::WEBGPU_MAX_COMPUTE_WORKGROUP_SIZE_Z => {
                fingerprint.rendering.webgpu.max_compute_workgroup_size_z = value;
            }
            profile_field::WEBGPU_MAX_COMPUTE_WORKGROUPS_PER_DIMENSION => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_compute_workgroups_per_dimension = value;
            }
            profile_field::WEBGPU_MAX_IMMEDIATE_SIZE => {
                fingerprint.rendering.webgpu.max_immediate_size = value;
            }
            profile_field::WEBGPU_MAX_STORAGE_BUFFERS_IN_FRAGMENT_STAGE => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_storage_buffers_in_fragment_stage = value;
            }
            profile_field::WEBGPU_MAX_STORAGE_TEXTURES_IN_FRAGMENT_STAGE => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_storage_textures_in_fragment_stage = value;
            }
            profile_field::WEBGPU_MAX_STORAGE_BUFFERS_IN_VERTEX_STAGE => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_storage_buffers_in_vertex_stage = value;
            }
            profile_field::WEBGPU_MAX_STORAGE_TEXTURES_IN_VERTEX_STAGE => {
                fingerprint
                    .rendering
                    .webgpu
                    .max_storage_textures_in_vertex_stage = value;
            }
            profile_field::SCREEN_ORIENTATION_ANGLE => {
                fingerprint.screen.orientation_angle = u16::try_from(value)
                    .map_err(|_| "screen orientation angle does not fit u16".to_owned())?;
            }
            profile_field::WEBGPU_SUBGROUP_MIN_SIZE => {
                fingerprint.rendering.webgpu.subgroup_min_size = value;
            }
            profile_field::WEBGPU_SUBGROUP_MAX_SIZE => {
                fingerprint.rendering.webgpu.subgroup_max_size = value;
            }
            profile_field::WEBGL2_MAX_ELEMENT_INDEX => {
                fingerprint.rendering.webgl.webgl2_max_element_index = value;
            }
            _ => return Err(format!("unknown u32 profile field {field}")),
        }
        Ok(())
    })
}

/// Sets a signed 32-bit profile field.
///
/// # Safety
///
/// `profile` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_set_i32(
    profile: *mut EdgeSandboxProfile,
    field: u32,
    value: i32,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        let fingerprint = &mut unsafe { profile_mut(profile)? }.fingerprint;
        match field {
            profile_field::TIME_ZONE_OFFSET_MINUTES => {
                fingerprint.locale.time_zone_offset_minutes = value;
            }
            profile_field::SCREEN_WIDTH => {
                let previous = f64::from(fingerprint.screen.width);
                let inner_follows_screen = fingerprint.screen.viewport_width == previous;
                let outer_follows_screen = fingerprint.screen.outer_width == previous;
                let available_follows_screen =
                    fingerprint.screen.avail_width == fingerprint.screen.width;
                fingerprint.screen.width = value;
                if available_follows_screen {
                    fingerprint.screen.avail_width = value;
                }
                if inner_follows_screen {
                    fingerprint.screen.viewport_width = f64::from(value);
                }
                if outer_follows_screen {
                    fingerprint.screen.outer_width = f64::from(value);
                }
            }
            profile_field::SCREEN_HEIGHT => {
                let previous = f64::from(fingerprint.screen.height);
                let inner_follows_screen = fingerprint.screen.viewport_height == previous;
                let outer_follows_screen = fingerprint.screen.outer_height == previous;
                let available_follows_screen =
                    fingerprint.screen.avail_height == fingerprint.screen.height;
                fingerprint.screen.height = value;
                if available_follows_screen {
                    fingerprint.screen.avail_height = value;
                }
                if inner_follows_screen {
                    fingerprint.screen.viewport_height = f64::from(value);
                }
                if outer_follows_screen {
                    fingerprint.screen.outer_height = f64::from(value);
                }
            }
            profile_field::SCREEN_AVAIL_WIDTH => fingerprint.screen.avail_width = value,
            profile_field::SCREEN_AVAIL_HEIGHT => fingerprint.screen.avail_height = value,
            profile_field::SCREEN_AVAIL_LEFT => fingerprint.screen.avail_left = value,
            profile_field::SCREEN_AVAIL_TOP => fingerprint.screen.avail_top = value,
            profile_field::SCREEN_COLOR_DEPTH => fingerprint.screen.color_depth = value,
            profile_field::SCREEN_PIXEL_DEPTH => fingerprint.screen.pixel_depth = value,
            profile_field::WEBGL_MAX_TEXTURE_SIZE => {
                fingerprint.rendering.webgl.max_texture_size = value;
            }
            profile_field::WEBGL_MAX_CUBE_MAP_TEXTURE_SIZE => {
                fingerprint.rendering.webgl.max_cube_map_texture_size = value;
            }
            profile_field::WEBGL_MAX_RENDERBUFFER_SIZE => {
                fingerprint.rendering.webgl.max_renderbuffer_size = value;
            }
            profile_field::WEBGL_MAX_VIEWPORT_WIDTH => {
                fingerprint.rendering.webgl.max_viewport_width = value;
            }
            profile_field::WEBGL_MAX_VIEWPORT_HEIGHT => {
                fingerprint.rendering.webgl.max_viewport_height = value;
            }
            profile_field::WEBGL_MAX_VERTEX_ATTRIBS => {
                fingerprint.rendering.webgl.max_vertex_attribs = value;
            }
            profile_field::WEBGL_MAX_VERTEX_UNIFORM_VECTORS => {
                fingerprint.rendering.webgl.max_vertex_uniform_vectors = value;
            }
            profile_field::WEBGL_MAX_VARYING_VECTORS => {
                fingerprint.rendering.webgl.max_varying_vectors = value;
            }
            profile_field::WEBGL_MAX_FRAGMENT_UNIFORM_VECTORS => {
                fingerprint.rendering.webgl.max_fragment_uniform_vectors = value;
            }
            profile_field::WEBGL_MAX_VERTEX_TEXTURE_IMAGE_UNITS => {
                fingerprint.rendering.webgl.max_vertex_texture_image_units = value;
            }
            profile_field::WEBGL_MAX_TEXTURE_IMAGE_UNITS => {
                fingerprint.rendering.webgl.max_texture_image_units = value;
            }
            profile_field::WEBGL_MAX_COMBINED_TEXTURE_IMAGE_UNITS => {
                fingerprint.rendering.webgl.max_combined_texture_image_units = value;
            }
            profile_field::WEBGL_SUBPIXEL_BITS => {
                fingerprint.rendering.webgl.subpixel_bits = value;
            }
            profile_field::WEBGL2_MAX_3D_TEXTURE_SIZE => {
                fingerprint.rendering.webgl.webgl2_max_3d_texture_size = value;
            }
            profile_field::WEBGL2_MAX_ARRAY_TEXTURE_LAYERS => {
                fingerprint.rendering.webgl.webgl2_max_array_texture_layers = value;
            }
            profile_field::WEBGL2_MAX_DRAW_BUFFERS => {
                fingerprint.rendering.webgl.webgl2_max_draw_buffers = value;
            }
            profile_field::WEBGL2_MAX_COLOR_ATTACHMENTS => {
                fingerprint.rendering.webgl.webgl2_max_color_attachments = value;
            }
            profile_field::WEBGL2_MAX_SAMPLES => {
                fingerprint.rendering.webgl.webgl2_max_samples = value;
            }
            profile_field::WEBGL2_MAX_VERTEX_UNIFORM_COMPONENTS => {
                fingerprint
                    .rendering
                    .webgl
                    .webgl2_max_vertex_uniform_components = value;
            }
            profile_field::WEBGL2_MAX_FRAGMENT_UNIFORM_COMPONENTS => {
                fingerprint
                    .rendering
                    .webgl
                    .webgl2_max_fragment_uniform_components = value;
            }
            profile_field::WEBGL2_MAX_VARYING_COMPONENTS => {
                fingerprint.rendering.webgl.webgl2_max_varying_components = value;
            }
            profile_field::WEBGL2_MAX_VERTEX_OUTPUT_COMPONENTS => {
                fingerprint
                    .rendering
                    .webgl
                    .webgl2_max_vertex_output_components = value;
            }
            profile_field::WEBGL2_MAX_FRAGMENT_INPUT_COMPONENTS => {
                fingerprint
                    .rendering
                    .webgl
                    .webgl2_max_fragment_input_components = value;
            }
            profile_field::WEBGL2_MAX_VERTEX_UNIFORM_BLOCKS => {
                fingerprint.rendering.webgl.webgl2_max_vertex_uniform_blocks = value;
            }
            profile_field::WEBGL2_MAX_FRAGMENT_UNIFORM_BLOCKS => {
                fingerprint
                    .rendering
                    .webgl
                    .webgl2_max_fragment_uniform_blocks = value;
            }
            profile_field::WEBGL2_MAX_COMBINED_UNIFORM_BLOCKS => {
                fingerprint
                    .rendering
                    .webgl
                    .webgl2_max_combined_uniform_blocks = value;
            }
            profile_field::WEBGL2_MAX_UNIFORM_BUFFER_BINDINGS => {
                fingerprint
                    .rendering
                    .webgl
                    .webgl2_max_uniform_buffer_bindings = value;
            }
            profile_field::WEBGL2_MAX_UNIFORM_BLOCK_SIZE => {
                fingerprint.rendering.webgl.webgl2_max_uniform_block_size = value;
            }
            profile_field::WEBGL2_MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS => {
                fingerprint
                    .rendering
                    .webgl
                    .webgl2_max_combined_vertex_uniform_components = value;
            }
            profile_field::WEBGL2_MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS => {
                fingerprint
                    .rendering
                    .webgl
                    .webgl2_max_combined_fragment_uniform_components = value;
            }
            profile_field::WEBGL2_MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS => {
                fingerprint
                    .rendering
                    .webgl
                    .webgl2_max_transform_feedback_separate_attribs = value;
            }
            profile_field::WEBGL2_MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS => {
                fingerprint
                    .rendering
                    .webgl
                    .webgl2_max_transform_feedback_interleaved_components = value;
            }
            profile_field::WEBGL2_MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS => {
                fingerprint
                    .rendering
                    .webgl
                    .webgl2_max_transform_feedback_separate_components = value;
            }
            profile_field::WEBGL2_MAX_PROGRAM_TEXEL_OFFSET => {
                fingerprint.rendering.webgl.webgl2_max_program_texel_offset = value;
            }
            profile_field::WEBGL2_MAX_ELEMENTS_VERTICES => {
                fingerprint.rendering.webgl.webgl2_max_elements_vertices = value;
            }
            profile_field::WEBGL2_MAX_ELEMENTS_INDICES => {
                fingerprint.rendering.webgl.webgl2_max_elements_indices = value;
            }
            profile_field::WEBGL_SHADER_PRECISION_RANGE_MIN => {
                fingerprint.rendering.webgl.shader_precision_range_min = value;
            }
            profile_field::WEBGL_SHADER_PRECISION_RANGE_MAX => {
                fingerprint.rendering.webgl.shader_precision_range_max = value;
            }
            profile_field::WEBGL_SHADER_PRECISION_BITS => {
                fingerprint.rendering.webgl.shader_precision_bits = value;
            }
            _ => return Err(format!("unknown i32 profile field {field}")),
        }
        Ok(())
    })
}

/// Sets an unsigned 64-bit profile field.
///
/// # Safety
///
/// `profile` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_set_u64(
    profile: *mut EdgeSandboxProfile,
    field: u32,
    value: u64,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        let fingerprint = &mut unsafe { profile_mut(profile)? }.fingerprint;
        match field {
            profile_field::STORAGE_QUOTA_BYTES => fingerprint.storage.quota_bytes = value,
            profile_field::STORAGE_USAGE_BYTES => fingerprint.storage.usage_bytes = value,
            profile_field::AUDIO_NOISE_SEED => fingerprint.rendering.audio.noise_seed = value,
            profile_field::WEBGPU_MAX_UNIFORM_BUFFER_BINDING_SIZE => {
                fingerprint.rendering.webgpu.max_uniform_buffer_binding_size = value;
            }
            profile_field::WEBGPU_MAX_STORAGE_BUFFER_BINDING_SIZE => {
                fingerprint.rendering.webgpu.max_storage_buffer_binding_size = value;
            }
            profile_field::WEBGPU_MAX_BUFFER_SIZE => {
                fingerprint.rendering.webgpu.max_buffer_size = value;
            }
            profile_field::TIMING_CLOCK_STEP_MS => fingerprint.timing.clock_step_ms = value,
            profile_field::TIMING_RANDOM_SEED => fingerprint.timing.random_seed = Some(value),
            profile_field::PERFORMANCE_JS_HEAP_SIZE_LIMIT => {
                fingerprint.memory.performance_js_heap_size_limit = value;
            }
            profile_field::PERFORMANCE_TOTAL_JS_HEAP_SIZE => {
                fingerprint.memory.performance_total_js_heap_size = value;
            }
            profile_field::PERFORMANCE_USED_JS_HEAP_SIZE => {
                fingerprint.memory.performance_used_js_heap_size = value;
            }
            profile_field::CONSOLE_JS_HEAP_SIZE_LIMIT => {
                fingerprint.memory.console_js_heap_size_limit = value;
            }
            profile_field::CONSOLE_TOTAL_JS_HEAP_SIZE => {
                fingerprint.memory.console_total_js_heap_size = value;
            }
            profile_field::CONSOLE_USED_JS_HEAP_SIZE => {
                fingerprint.memory.console_used_js_heap_size = value;
            }
            _ => return Err(format!("unknown u64 profile field {field}")),
        }
        Ok(())
    })
}

/// Sets a signed 64-bit profile field.
///
/// # Safety
///
/// `profile` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_set_i64(
    profile: *mut EdgeSandboxProfile,
    field: u32,
    value: i64,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        let fingerprint = &mut unsafe { profile_mut(profile)? }.fingerprint;
        match field {
            profile_field::TIMING_CLOCK_EPOCH_MS => {
                fingerprint.timing.clock_epoch_ms = Some(value);
            }
            _ => return Err(format!("unknown i64 profile field {field}")),
        }
        Ok(())
    })
}

/// Sets a 64-bit floating-point profile field.
///
/// # Safety
///
/// `profile` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_set_f64(
    profile: *mut EdgeSandboxProfile,
    field: u32,
    value: f64,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        let fingerprint = &mut unsafe { profile_mut(profile)? }.fingerprint;
        match field {
            profile_field::DEVICE_MEMORY_GB => fingerprint.navigator.device_memory_gb = value,
            profile_field::NETWORK_DOWNLINK => fingerprint.navigator.network.downlink = value,
            profile_field::SCREEN_VIEWPORT_WIDTH => fingerprint.screen.viewport_width = value,
            profile_field::SCREEN_VIEWPORT_HEIGHT => fingerprint.screen.viewport_height = value,
            profile_field::SCREEN_OUTER_WIDTH => fingerprint.screen.outer_width = value,
            profile_field::SCREEN_OUTER_HEIGHT => fingerprint.screen.outer_height = value,
            profile_field::SCREEN_X => fingerprint.screen.screen_x = value,
            profile_field::SCREEN_Y => fingerprint.screen.screen_y = value,
            profile_field::SCREEN_DEVICE_PIXEL_RATIO => {
                fingerprint.screen.device_pixel_ratio = value;
            }
            profile_field::CANVAS_TEXT_WIDTH_SCALE => {
                fingerprint.rendering.canvas.text_width_scale = value;
            }
            profile_field::WEBGL_MAX_ANISOTROPY => {
                fingerprint.rendering.webgl.max_anisotropy = value;
            }
            profile_field::WEBGL2_MAX_TEXTURE_LOD_BIAS => {
                fingerprint.rendering.webgl.webgl2_max_texture_lod_bias = value;
            }
            profile_field::DOCUMENT_BODY_CLIENT_HEIGHT => {
                fingerprint.document.body_client_height = Some(value);
            }
            profile_field::AUDIO_SAMPLE_RATE => {
                fingerprint.rendering.audio.sample_rate = value;
            }
            profile_field::AUDIO_BASE_LATENCY => {
                fingerprint.rendering.audio.base_latency = value;
            }
            profile_field::AUDIO_OUTPUT_LATENCY => {
                fingerprint.rendering.audio.output_latency = value;
            }
            profile_field::BATTERY_CHARGING_TIME => fingerprint.battery.charging_time = value,
            profile_field::BATTERY_DISCHARGING_TIME => {
                fingerprint.battery.discharging_time = value;
            }
            profile_field::BATTERY_LEVEL => fingerprint.battery.level = value,
            profile_field::GEOLOCATION_LATITUDE => fingerprint.geolocation.latitude = value,
            profile_field::GEOLOCATION_LONGITUDE => fingerprint.geolocation.longitude = value,
            profile_field::GEOLOCATION_ALTITUDE => fingerprint.geolocation.altitude = Some(value),
            profile_field::GEOLOCATION_ACCURACY => fingerprint.geolocation.accuracy = value,
            profile_field::GEOLOCATION_ALTITUDE_ACCURACY => {
                fingerprint.geolocation.altitude_accuracy = Some(value);
            }
            profile_field::GEOLOCATION_HEADING => fingerprint.geolocation.heading = Some(value),
            profile_field::GEOLOCATION_SPEED => fingerprint.geolocation.speed = Some(value),
            profile_field::WEBGL_ALIASED_POINT_SIZE_MIN => {
                fingerprint.rendering.webgl.aliased_point_size_min = value;
            }
            profile_field::WEBGL_ALIASED_POINT_SIZE_MAX => {
                fingerprint.rendering.webgl.aliased_point_size_max = value;
            }
            profile_field::WEBGL_ALIASED_LINE_WIDTH_MIN => {
                fingerprint.rendering.webgl.aliased_line_width_min = value;
            }
            profile_field::WEBGL_ALIASED_LINE_WIDTH_MAX => {
                fingerprint.rendering.webgl.aliased_line_width_max = value;
            }
            profile_field::SENSOR_ACCELEROMETER_X => {
                fingerprint.sensors.accelerometer[0] = value;
            }
            profile_field::SENSOR_ACCELEROMETER_Y => {
                fingerprint.sensors.accelerometer[1] = value;
            }
            profile_field::SENSOR_ACCELEROMETER_Z => {
                fingerprint.sensors.accelerometer[2] = value;
            }
            profile_field::SENSOR_GRAVITY_X => fingerprint.sensors.gravity[0] = value,
            profile_field::SENSOR_GRAVITY_Y => fingerprint.sensors.gravity[1] = value,
            profile_field::SENSOR_GRAVITY_Z => fingerprint.sensors.gravity[2] = value,
            profile_field::SENSOR_LINEAR_ACCELERATION_X => {
                fingerprint.sensors.linear_acceleration[0] = value;
            }
            profile_field::SENSOR_LINEAR_ACCELERATION_Y => {
                fingerprint.sensors.linear_acceleration[1] = value;
            }
            profile_field::SENSOR_LINEAR_ACCELERATION_Z => {
                fingerprint.sensors.linear_acceleration[2] = value;
            }
            profile_field::SENSOR_GYROSCOPE_X => fingerprint.sensors.gyroscope[0] = value,
            profile_field::SENSOR_GYROSCOPE_Y => fingerprint.sensors.gyroscope[1] = value,
            profile_field::SENSOR_GYROSCOPE_Z => fingerprint.sensors.gyroscope[2] = value,
            profile_field::SENSOR_ABSOLUTE_ORIENTATION_X => {
                fingerprint.sensors.absolute_orientation_quaternion[0] = value;
            }
            profile_field::SENSOR_ABSOLUTE_ORIENTATION_Y => {
                fingerprint.sensors.absolute_orientation_quaternion[1] = value;
            }
            profile_field::SENSOR_ABSOLUTE_ORIENTATION_Z => {
                fingerprint.sensors.absolute_orientation_quaternion[2] = value;
            }
            profile_field::SENSOR_ABSOLUTE_ORIENTATION_W => {
                fingerprint.sensors.absolute_orientation_quaternion[3] = value;
            }
            profile_field::SENSOR_RELATIVE_ORIENTATION_X => {
                fingerprint.sensors.relative_orientation_quaternion[0] = value;
            }
            profile_field::SENSOR_RELATIVE_ORIENTATION_Y => {
                fingerprint.sensors.relative_orientation_quaternion[1] = value;
            }
            profile_field::SENSOR_RELATIVE_ORIENTATION_Z => {
                fingerprint.sensors.relative_orientation_quaternion[2] = value;
            }
            profile_field::SENSOR_RELATIVE_ORIENTATION_W => {
                fingerprint.sensors.relative_orientation_quaternion[3] = value;
            }
            profile_field::CANVAS_ACTUAL_BOUNDING_BOX_LEFT => {
                fingerprint.rendering.canvas.actual_bounding_box_left = value;
            }
            profile_field::CANVAS_ACTUAL_BOUNDING_BOX_RIGHT_SCALE => {
                fingerprint.rendering.canvas.actual_bounding_box_right_scale = value;
            }
            profile_field::CANVAS_FONT_BOUNDING_BOX_ASCENT => {
                fingerprint.rendering.canvas.font_bounding_box_ascent = value;
            }
            profile_field::CANVAS_FONT_BOUNDING_BOX_DESCENT => {
                fingerprint.rendering.canvas.font_bounding_box_descent = value;
            }
            profile_field::CANVAS_ACTUAL_BOUNDING_BOX_ASCENT => {
                fingerprint.rendering.canvas.actual_bounding_box_ascent = value;
            }
            profile_field::CANVAS_ACTUAL_BOUNDING_BOX_DESCENT => {
                fingerprint.rendering.canvas.actual_bounding_box_descent = value;
            }
            profile_field::CANVAS_HANGING_BASELINE => {
                fingerprint.rendering.canvas.hanging_baseline = value;
            }
            profile_field::CANVAS_ALPHABETIC_BASELINE => {
                fingerprint.rendering.canvas.alphabetic_baseline = value;
            }
            profile_field::CANVAS_IDEOGRAPHIC_BASELINE => {
                fingerprint.rendering.canvas.ideographic_baseline = value;
            }
            profile_field::VISUAL_VIEWPORT_OFFSET_LEFT => {
                fingerprint.screen.visual_viewport_offset_left = value;
            }
            profile_field::VISUAL_VIEWPORT_OFFSET_TOP => {
                fingerprint.screen.visual_viewport_offset_top = value;
            }
            profile_field::VISUAL_VIEWPORT_PAGE_LEFT => {
                fingerprint.screen.visual_viewport_page_left = value;
            }
            profile_field::VISUAL_VIEWPORT_PAGE_TOP => {
                fingerprint.screen.visual_viewport_page_top = value;
            }
            profile_field::VISUAL_VIEWPORT_SCALE => {
                fingerprint.screen.visual_viewport_scale = value;
            }
            _ => return Err(format!("unknown f64 profile field {field}")),
        }
        Ok(())
    })
}

/// Sets a 32-bit floating-point profile field.
///
/// # Safety
///
/// `profile` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_set_f32(
    profile: *mut EdgeSandboxProfile,
    field: u32,
    value: f32,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        let fingerprint = &mut unsafe { profile_mut(profile)? }.fingerprint;
        match field {
            profile_field::AUDIO_CHANNEL_NOISE_AMPLITUDE => {
                fingerprint.rendering.audio.channel_noise_amplitude = value;
            }
            profile_field::AUDIO_FREQUENCY_NOISE_AMPLITUDE => {
                fingerprint.rendering.audio.frequency_noise_amplitude = value;
            }
            profile_field::AUDIO_TIME_DOMAIN_NOISE_AMPLITUDE => {
                fingerprint.rendering.audio.time_domain_noise_amplitude = value;
            }
            _ => return Err(format!("unknown f32 profile field {field}")),
        }
        Ok(())
    })
}

/// Sets a boolean profile field.
///
/// # Safety
///
/// `profile` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_set_bool(
    profile: *mut EdgeSandboxProfile,
    field: u32,
    value: bool,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        let fingerprint = &mut unsafe { profile_mut(profile)? }.fingerprint;
        match field {
            profile_field::NAVIGATOR_COOKIE_ENABLED => {
                fingerprint.navigator.cookie_enabled = value;
            }
            profile_field::NAVIGATOR_ON_LINE => fingerprint.navigator.on_line = value,
            profile_field::NAVIGATOR_WEBDRIVER => fingerprint.navigator.webdriver = value,
            profile_field::NAVIGATOR_PDF_VIEWER_ENABLED => {
                fingerprint.navigator.pdf_viewer_enabled = value;
            }
            profile_field::UA_MOBILE => fingerprint.navigator.user_agent_data.mobile = value,
            profile_field::UA_WOW64 => fingerprint.navigator.user_agent_data.wow64 = value,
            profile_field::NETWORK_SAVE_DATA => fingerprint.navigator.network.save_data = value,
            profile_field::STORAGE_PERSISTED => fingerprint.storage.persisted = value,
            profile_field::FONT_ALLOW_UNKNOWN_FAMILIES => {
                fingerprint.fonts.allow_unknown_families = value;
            }
            profile_field::BATTERY_CHARGING => fingerprint.battery.charging = value,
            profile_field::MEDIA_PREFERENCE_REDUCED_MOTION => {
                fingerprint.media_preferences.reduced_motion = value;
            }
            profile_field::MEDIA_PREFERENCE_REDUCED_DATA => {
                fingerprint.media_preferences.reduced_data = value;
            }
            profile_field::MEDIA_PREFERENCE_FORCED_COLORS => {
                fingerprint.media_preferences.forced_colors = value;
            }
            profile_field::MEDIA_PREFERENCE_INVERTED_COLORS => {
                fingerprint.media_preferences.inverted_colors = value;
            }
            profile_field::MEDIA_PREFERENCE_REDUCED_TRANSPARENCY => {
                fingerprint.media_preferences.reduced_transparency = value;
            }
            profile_field::NAVIGATOR_USER_ACTIVATION_HAS_BEEN_ACTIVE => {
                fingerprint.navigator.user_activation_has_been_active = value;
            }
            profile_field::NAVIGATOR_USER_ACTIVATION_IS_ACTIVE => {
                fingerprint.navigator.user_activation_is_active = value;
            }
            profile_field::WEBGL_CONTEXT_ALPHA => {
                fingerprint.rendering.webgl.context_alpha = value;
            }
            profile_field::WEBGL_CONTEXT_ANTIALIAS => {
                fingerprint.rendering.webgl.context_antialias = value;
            }
            profile_field::WEBGL_CONTEXT_DEPTH => {
                fingerprint.rendering.webgl.context_depth = value;
            }
            profile_field::WEBGL_CONTEXT_DESYNCHRONIZED => {
                fingerprint.rendering.webgl.context_desynchronized = value;
            }
            profile_field::WEBGL_CONTEXT_FAIL_IF_MAJOR_PERFORMANCE_CAVEAT => {
                fingerprint
                    .rendering
                    .webgl
                    .context_fail_if_major_performance_caveat = value;
            }
            profile_field::WEBGL_CONTEXT_PREMULTIPLIED_ALPHA => {
                fingerprint.rendering.webgl.context_premultiplied_alpha = value;
            }
            profile_field::WEBGL_CONTEXT_PRESERVE_DRAWING_BUFFER => {
                fingerprint.rendering.webgl.context_preserve_drawing_buffer = value;
            }
            profile_field::WEBGL_CONTEXT_STENCIL => {
                fingerprint.rendering.webgl.context_stencil = value;
            }
            profile_field::WEBGL_CONTEXT_XR_COMPATIBLE => {
                fingerprint.rendering.webgl.context_xr_compatible = value;
            }
            profile_field::BLUETOOTH_AVAILABLE => {
                fingerprint.hardware_devices.bluetooth_available = value;
            }
            profile_field::MIDI_SYSEX_ENABLED => {
                fingerprint.hardware_devices.midi_sysex_enabled = value;
            }
            profile_field::WEBGPU_DEVELOPER_FEATURES => {
                fingerprint.rendering.webgpu.developer_features = value;
            }
            profile_field::WEBGPU_IS_FALLBACK_ADAPTER => {
                fingerprint.rendering.webgpu.is_fallback_adapter = value;
            }
            profile_field::SENSORS_AVAILABLE => {
                fingerprint.sensors.available = value;
            }
            profile_field::WEBGPU_AVAILABLE => {
                fingerprint.rendering.webgpu.available = value;
            }
            _ => return Err(format!("unknown bool profile field {field}")),
        }
        Ok(())
    })
}

/// Clears the User-Agent Client Hints brand list.
///
/// # Safety
///
/// `profile` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_ua_brands(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .navigator
            .user_agent_data
            .brands
            .clear();
        Ok(())
    })
}

/// Appends one User-Agent Client Hints brand tuple.
///
/// # Safety
///
/// Every string pointer must address its matching readable byte length,
/// `profile` must be live, and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_ua_brand(
    profile: *mut EdgeSandboxProfile,
    brand: *const u8,
    brand_len: usize,
    version: *const u8,
    version_len: usize,
    full_version: *const u8,
    full_version_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let brand = input_string(brand, brand_len, "User-Agent brand")?;
        let version = input_string(version, version_len, "User-Agent brand version")?;
        let full_version = input_string(
            full_version,
            full_version_len,
            "User-Agent brand full version",
        )?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .navigator
            .user_agent_data
            .brands
            .push(crate::UserAgentBrandFingerprint {
                brand,
                version,
                full_version,
            });
        Ok(())
    })
}

/// Clears the speech-synthesis voice list.
///
/// # Safety
///
/// `profile` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_speech_voices(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .speech
            .voices
            .clear();
        Ok(())
    })
}

/// Appends one speech-synthesis voice profile.
///
/// # Safety
///
/// Every string pointer must address its matching readable byte length,
/// `profile` must be live, and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_speech_voice(
    profile: *mut EdgeSandboxProfile,
    voice_uri: *const u8,
    voice_uri_len: usize,
    name: *const u8,
    name_len: usize,
    lang: *const u8,
    lang_len: usize,
    local_service: bool,
    is_default: bool,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let voice_uri = input_string(voice_uri, voice_uri_len, "speech voice URI")?;
        let name = input_string(name, name_len, "speech voice name")?;
        let lang = input_string(lang, lang_len, "speech voice language")?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .speech
            .voices
            .push(crate::SpeechVoiceFingerprint {
                voice_uri,
                name,
                lang,
                local_service,
                is_default,
            });
        Ok(())
    })
}

/// Clears the local-font inventory returned by `queryLocalFonts()`.
///
/// # Safety
///
/// `profile` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_local_fonts(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .fonts
            .local_fonts
            .clear();
        Ok(())
    })
}

/// Appends one local-font metadata record returned by `queryLocalFonts()`.
///
/// # Safety
///
/// Every string pointer must address its matching readable byte length,
/// `profile` must be live, and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_local_font(
    profile: *mut EdgeSandboxProfile,
    postscript_name: *const u8,
    postscript_name_len: usize,
    full_name: *const u8,
    full_name_len: usize,
    family: *const u8,
    family_len: usize,
    style: *const u8,
    style_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let postscript_name =
            input_string(postscript_name, postscript_name_len, "font PostScript name")?;
        let full_name = input_string(full_name, full_name_len, "font full name")?;
        let family = input_string(family, family_len, "font family")?;
        let style = input_string(style, style_len, "font style")?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .fonts
            .local_fonts
            .push(crate::LocalFontFingerprint {
                postscript_name,
                full_name,
                family,
                style,
            });
        Ok(())
    })
}

/// Clears configured canvas text metrics for individual font families.
///
/// # Safety
///
/// `profile` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_font_metrics(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .fonts
            .metrics
            .clear();
        Ok(())
    })
}

/// Appends canvas text metrics for one configured font family.
///
/// # Safety
///
/// `family` must address `family_len` readable bytes, `profile` must be live,
/// and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_font_metric(
    profile: *mut EdgeSandboxProfile,
    family: *const u8,
    family_len: usize,
    width_scale: f64,
    monospace: bool,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let family = input_string(family, family_len, "font metric family")?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .fonts
            .metrics
            .push(crate::FontMetricFingerprint {
                family,
                width_scale,
                monospace,
            });
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_media_devices(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .media
            .devices
            .clear();
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_media_device(
    profile: *mut EdgeSandboxProfile,
    device_id: *const u8,
    device_id_len: usize,
    kind: *const u8,
    kind_len: usize,
    label: *const u8,
    label_len: usize,
    group_id: *const u8,
    group_id_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let device_id = input_string(device_id, device_id_len, "media device id")?;
        let kind = input_string(kind, kind_len, "media device kind")?;
        let label = input_string(label, label_len, "media device label")?;
        let group_id = input_string(group_id, group_id_len, "media device group id")?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .media
            .devices
            .push(crate::MediaDeviceFingerprint {
                device_id,
                kind,
                label,
                group_id,
            });
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_webgl_compressed_texture_formats(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .rendering
            .webgl
            .compressed_texture_formats
            .clear();
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_webgl_compressed_texture_format(
    profile: *mut EdgeSandboxProfile,
    format: u32,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .rendering
            .webgl
            .compressed_texture_formats
            .push(format);
        Ok(())
    })
}

fn clear_rtc_codecs(profile: &mut EdgeSandboxProfile, audio: bool) {
    if audio {
        profile.fingerprint.media.rtc_audio_codecs.clear();
    } else {
        profile.fingerprint.media.rtc_video_codecs.clear();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_rtc_audio_codecs(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        clear_rtc_codecs(unsafe { profile_mut(profile)? }, true);
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_rtc_video_codecs(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        clear_rtc_codecs(unsafe { profile_mut(profile)? }, false);
        Ok(())
    })
}

unsafe fn append_rtc_codec(
    profile: *mut EdgeSandboxProfile,
    audio: bool,
    mime_type: *const u8,
    mime_type_len: usize,
    clock_rate: u32,
    channels: u16,
    has_channels: bool,
    sdp_fmtp_line: *const u8,
    sdp_fmtp_line_len: usize,
    has_sdp_fmtp_line: bool,
) -> Result<(), String> {
    let mime_type = input_string(mime_type, mime_type_len, "RTC codec MIME type")?;
    let sdp_fmtp_line = if has_sdp_fmtp_line {
        Some(input_string(
            sdp_fmtp_line,
            sdp_fmtp_line_len,
            "RTC codec fmtp line",
        )?)
    } else {
        None
    };
    let codec = crate::RtcCodecFingerprint {
        mime_type,
        clock_rate,
        channels: has_channels.then_some(channels),
        sdp_fmtp_line,
    };
    // SAFETY: inherited from the exported FFI caller.
    let profile = unsafe { profile_mut(profile)? };
    if audio {
        profile.fingerprint.media.rtc_audio_codecs.push(codec);
    } else {
        profile.fingerprint.media.rtc_video_codecs.push(codec);
    }
    Ok(())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_rtc_audio_codec(
    profile: *mut EdgeSandboxProfile,
    mime_type: *const u8,
    mime_type_len: usize,
    clock_rate: u32,
    channels: u16,
    has_channels: bool,
    sdp_fmtp_line: *const u8,
    sdp_fmtp_line_len: usize,
    has_sdp_fmtp_line: bool,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe {
            append_rtc_codec(
                profile,
                true,
                mime_type,
                mime_type_len,
                clock_rate,
                channels,
                has_channels,
                sdp_fmtp_line,
                sdp_fmtp_line_len,
                has_sdp_fmtp_line,
            )
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_rtc_video_codec(
    profile: *mut EdgeSandboxProfile,
    mime_type: *const u8,
    mime_type_len: usize,
    clock_rate: u32,
    channels: u16,
    has_channels: bool,
    sdp_fmtp_line: *const u8,
    sdp_fmtp_line_len: usize,
    has_sdp_fmtp_line: bool,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe {
            append_rtc_codec(
                profile,
                false,
                mime_type,
                mime_type_len,
                clock_rate,
                channels,
                has_channels,
                sdp_fmtp_line,
                sdp_fmtp_line_len,
                has_sdp_fmtp_line,
            )
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_rtc_header_extensions(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .media
            .rtc_header_extensions
            .clear();
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_rtc_header_extension(
    profile: *mut EdgeSandboxProfile,
    kind: *const u8,
    kind_len: usize,
    uri: *const u8,
    uri_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let kind = input_string(kind, kind_len, "RTC header extension kind")?;
        let uri = input_string(uri, uri_len, "RTC header extension URI")?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .media
            .rtc_header_extensions
            .push(crate::RtcHeaderExtensionFingerprint { kind, uri });
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_plugins(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .plugins
            .plugins
            .clear();
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_plugin(
    profile: *mut EdgeSandboxProfile,
    name: *const u8,
    name_len: usize,
    filename: *const u8,
    filename_len: usize,
    description: *const u8,
    description_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let name = input_string(name, name_len, "plugin name")?;
        let filename = input_string(filename, filename_len, "plugin filename")?;
        let description = input_string(description, description_len, "plugin description")?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .plugins
            .plugins
            .push(crate::PluginFingerprint {
                name,
                filename,
                description,
                mime_types: Vec::new(),
            });
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_plugin_mime_type(
    profile: *mut EdgeSandboxProfile,
    plugin_index: u32,
    mime_type: *const u8,
    mime_type_len: usize,
    suffixes: *const u8,
    suffixes_len: usize,
    description: *const u8,
    description_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let mime_type = input_string(mime_type, mime_type_len, "plugin MIME type")?;
        let suffixes = input_string(suffixes, suffixes_len, "plugin MIME suffixes")?;
        let description = input_string(description, description_len, "plugin MIME description")?;
        // SAFETY: guaranteed by this function's FFI contract.
        let plugins = &mut unsafe { profile_mut(profile)? }.fingerprint.plugins.plugins;
        let plugin = plugins
            .get_mut(plugin_index as usize)
            .ok_or_else(|| format!("plugin index {plugin_index} does not exist"))?;
        plugin.mime_types.push(crate::MimeTypeFingerprint {
            mime_type,
            suffixes,
            description,
        });
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_gamepads(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .gamepads
            .clear();
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_gamepad(
    profile: *mut EdgeSandboxProfile,
    id: *const u8,
    id_len: usize,
    index: u32,
    connected: bool,
    mapping: *const u8,
    mapping_len: usize,
    axes: *const f64,
    axes_len: usize,
    buttons: *const f64,
    buttons_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let id = input_string(id, id_len, "gamepad id")?;
        let mapping = input_string(mapping, mapping_len, "gamepad mapping")?;
        // SAFETY: guaranteed by this function's FFI contract.
        let axes = unsafe { input_f64_values(axes, axes_len, "gamepad axes")? };
        // SAFETY: guaranteed by this function's FFI contract.
        let buttons = unsafe { input_f64_values(buttons, buttons_len, "gamepad buttons")? };
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .gamepads
            .push(crate::GamepadFingerprint {
                id,
                index,
                connected,
                mapping,
                axes,
                buttons,
            });
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_usb_devices(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .usb_devices
            .clear();
        Ok(())
    })
}

#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn edge_sandbox_profile_append_usb_device(
    profile: *mut EdgeSandboxProfile,
    usb_version_major: u8,
    usb_version_minor: u8,
    usb_version_subminor: u8,
    device_class: u8,
    device_subclass: u8,
    device_protocol: u8,
    vendor_id: u16,
    product_id: u16,
    device_version_major: u8,
    device_version_minor: u8,
    device_version_subminor: u8,
    manufacturer_name: *const u8,
    manufacturer_name_len: usize,
    has_manufacturer_name: bool,
    product_name: *const u8,
    product_name_len: usize,
    has_product_name: bool,
    serial_number: *const u8,
    serial_number_len: usize,
    has_serial_number: bool,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let manufacturer_name = if has_manufacturer_name {
            Some(input_string(
                manufacturer_name,
                manufacturer_name_len,
                "USB manufacturer name",
            )?)
        } else {
            None
        };
        let product_name = if has_product_name {
            Some(input_string(
                product_name,
                product_name_len,
                "USB product name",
            )?)
        } else {
            None
        };
        let serial_number = if has_serial_number {
            Some(input_string(
                serial_number,
                serial_number_len,
                "USB serial number",
            )?)
        } else {
            None
        };
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .usb_devices
            .push(crate::UsbDeviceFingerprint {
                usb_version_major,
                usb_version_minor,
                usb_version_subminor,
                device_class,
                device_subclass,
                device_protocol,
                vendor_id,
                product_id,
                device_version_major,
                device_version_minor,
                device_version_subminor,
                manufacturer_name,
                product_name,
                serial_number,
            });
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_hid_devices(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .hid_devices
            .clear();
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_hid_device(
    profile: *mut EdgeSandboxProfile,
    vendor_id: u16,
    product_id: u16,
    product_name: *const u8,
    product_name_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let product_name = input_string(product_name, product_name_len, "HID product name")?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .hid_devices
            .push(crate::HidDeviceFingerprint {
                vendor_id,
                product_id,
                product_name,
            });
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_serial_ports(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .serial_ports
            .clear();
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_serial_port(
    profile: *mut EdgeSandboxProfile,
    usb_vendor_id: u16,
    usb_product_id: u16,
    connected: bool,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .serial_ports
            .push(crate::SerialPortFingerprint {
                usb_vendor_id,
                usb_product_id,
                connected,
            });
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_bluetooth_devices(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .bluetooth_devices
            .clear();
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_bluetooth_device(
    profile: *mut EdgeSandboxProfile,
    id: *const u8,
    id_len: usize,
    name: *const u8,
    name_len: usize,
    has_name: bool,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let id = input_string(id, id_len, "Bluetooth device id")?;
        let name = if has_name {
            Some(input_string(name, name_len, "Bluetooth device name")?)
        } else {
            None
        };
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .bluetooth_devices
            .push(crate::BluetoothDeviceFingerprint { id, name });
        Ok(())
    })
}

/// Clears the physical keyboard-layout entries returned by `getLayoutMap()`.
///
/// # Safety
///
/// `profile` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_keyboard_layout(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .keyboard_layout
            .clear();
        Ok(())
    })
}

/// Appends one physical key code and displayed value to the keyboard layout.
///
/// # Safety
///
/// Both string pointers must address their matching readable byte lengths,
/// `profile` must be live, and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_append_keyboard_layout_entry(
    profile: *mut EdgeSandboxProfile,
    code: *const u8,
    code_len: usize,
    value: *const u8,
    value_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let code = input_string(code, code_len, "keyboard code")?;
        let value = input_string(value, value_len, "keyboard layout value")?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .keyboard_layout
            .push(crate::KeyboardLayoutEntryFingerprint { code, value });
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_midi_inputs(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .midi_inputs
            .clear();
        Ok(())
    })
}

#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn edge_sandbox_profile_append_midi_input(
    profile: *mut EdgeSandboxProfile,
    id: *const u8,
    id_len: usize,
    manufacturer: *const u8,
    manufacturer_len: usize,
    name: *const u8,
    name_len: usize,
    version: *const u8,
    version_len: usize,
    state: *const u8,
    state_len: usize,
    connection: *const u8,
    connection_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let id = input_string(id, id_len, "MIDI input id")?;
        let manufacturer = input_string(manufacturer, manufacturer_len, "MIDI input manufacturer")?;
        let name = input_string(name, name_len, "MIDI input name")?;
        let version = input_string(version, version_len, "MIDI input version")?;
        let state = input_string(state, state_len, "MIDI input state")?;
        let connection = input_string(connection, connection_len, "MIDI input connection")?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .midi_inputs
            .push(crate::MidiPortFingerprint {
                id,
                manufacturer,
                name,
                version,
                state,
                connection,
            });
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_clear_midi_outputs(
    profile: *mut EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .midi_outputs
            .clear();
        Ok(())
    })
}

#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn edge_sandbox_profile_append_midi_output(
    profile: *mut EdgeSandboxProfile,
    id: *const u8,
    id_len: usize,
    manufacturer: *const u8,
    manufacturer_len: usize,
    name: *const u8,
    name_len: usize,
    version: *const u8,
    version_len: usize,
    state: *const u8,
    state_len: usize,
    connection: *const u8,
    connection_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let id = input_string(id, id_len, "MIDI output id")?;
        let manufacturer =
            input_string(manufacturer, manufacturer_len, "MIDI output manufacturer")?;
        let name = input_string(name, name_len, "MIDI output name")?;
        let version = input_string(version, version_len, "MIDI output version")?;
        let state = input_string(state, state_len, "MIDI output state")?;
        let connection = input_string(connection, connection_len, "MIDI output connection")?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_mut(profile)? }
            .fingerprint
            .hardware_devices
            .midi_outputs
            .push(crate::MidiPortFingerprint {
                id,
                manufacturer,
                name,
                version,
                state,
                connection,
            });
        Ok(())
    })
}

/// Validates all cross-field constraints without creating a worker.
///
/// # Safety
///
/// `profile` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_profile_validate(
    profile: *const EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { profile_ref(profile)? }.fingerprint.validate()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn edge_sandbox_options_schema_version() -> u32 {
    2
}

/// Allocates a complete runtime-options builder using the Chrome 150 defaults.
#[unsafe(no_mangle)]
pub extern "C" fn edge_sandbox_options_create(
    error_out: *mut EdgeSandboxBuffer,
) -> *mut EdgeSandboxOptions {
    reset_buffer(error_out);
    match catch_unwind(AssertUnwindSafe(|| {
        Box::into_raw(Box::new(EdgeSandboxOptions {
            options: crate::EdgeRuntimeOptions::default(),
        }))
    })) {
        Ok(options) => options,
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            ptr::null_mut()
        }
    }
}

/// Releases an options builder returned by [`edge_sandbox_options_create`].
///
/// # Safety
///
/// `options` must be null or a live pointer returned by this library, and it
/// must be destroyed at most once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_options_destroy(options: *mut EdgeSandboxOptions) {
    if !options.is_null() {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe {
            drop(Box::from_raw(options));
        }
    }
}

/// Copies a complete typed fingerprint into a runtime-options builder.
///
/// # Safety
///
/// Both handles must be live pointers returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_options_set_profile(
    options: *mut EdgeSandboxOptions,
    profile: *const EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        let fingerprint = unsafe { profile_ref(profile)? }.fingerprint.clone();
        fingerprint.validate()?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { options_mut(options)? }.options.fingerprint = fingerprint;
        Ok(())
    })
}

/// Configures the linked HTML page that is materialized before user code.
///
/// # Safety
///
/// `options` must be live and every string pointer must address its declared
/// UTF-8 byte length.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn edge_sandbox_options_set_page(
    options: *mut EdgeSandboxOptions,
    url: *const u8,
    url_len: usize,
    html: *const u8,
    html_len: usize,
    referrer: *const u8,
    referrer_len: usize,
    content_type: *const u8,
    content_type_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let page = crate::PageInit {
            url: input_string(url, url_len, "page URL")?,
            html: input_string(html, html_len, "page HTML")?,
            referrer: input_string(referrer, referrer_len, "page referrer")?,
            content_type: input_string(content_type, content_type_len, "page content type")?,
        };
        page.validate()?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { options_mut(options)? }.options.page = Some(page);
        Ok(())
    })
}

/// Removes page initialization from a runtime-options builder.
///
/// # Safety
///
/// `options` must be live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_options_clear_page(
    options: *mut EdgeSandboxOptions,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { options_mut(options)? }.options.page = None;
        Ok(())
    })
}

/// Clears every iframe preload hook from a runtime-options builder.
///
/// # Safety
///
/// `options` must be live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_options_clear_iframe_hooks(
    options: *mut EdgeSandboxOptions,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { options_mut(options)? }
            .options
            .iframe_hooks
            .clear();
        Ok(())
    })
}

/// Appends JavaScript that runs in every iframe realm before page scripts.
///
/// The source can replace realm-local prototype functions and then call the
/// private `__edgev8` binding. The binding is passed only to the preload
/// source, is never installed on Window, and provides V8-native `proxy()` and
/// `protectPrototypeFunction()` methods.
///
/// # Safety
///
/// `options` must be live and both strings must remain readable for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_options_append_iframe_hook(
    options: *mut EdgeSandboxOptions,
    name: *const u8,
    name_len: usize,
    source: *const u8,
    source_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let hook = crate::IframeHook::new(
            input_string(name, name_len, "iframe hook name")?,
            input_string(source, source_len, "iframe hook source")?,
        );
        hook.validate()?;
        // SAFETY: guaranteed by this function's FFI contract.
        let hooks = &mut unsafe { options_mut(options)? }.options.iframe_hooks;
        if hooks.len() >= 64 {
            return Err("iframe hook configuration contains more than 64 hooks".to_owned());
        }
        hooks.push(hook);
        crate::iframe_hook::validate_hooks(hooks)
    })
}

/// Clears all offline network replay entries.
///
/// # Safety
///
/// `options` must be live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_options_clear_network_replay(
    options: *mut EdgeSandboxOptions,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { options_mut(options)? }
            .options
            .network_replay
            .clear();
        Ok(())
    })
}

/// Appends one bounded offline response. Headers are appended separately with
/// [`edge_sandbox_options_append_network_replay_header`] using its zero-based
/// entry index.
///
/// # Safety
///
/// `options` must be live; string and body pointers must remain readable for
/// the duration of this synchronous call.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn edge_sandbox_options_append_network_replay(
    options: *mut EdgeSandboxOptions,
    url: *const u8,
    url_len: usize,
    method: *const u8,
    method_len: usize,
    status: u16,
    status_text: *const u8,
    status_text_len: usize,
    body: *const u8,
    body_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let entry = crate::NetworkReplayEntry {
            url: input_string(url, url_len, "network replay URL")?,
            method: input_string(method, method_len, "network replay method")?,
            status,
            status_text: input_string(status_text, status_text_len, "network replay status text")?,
            headers: Vec::new(),
            body: input_bytes(body, body_len)?.to_vec(),
        };
        entry.validate()?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { options_mut(options)? }
            .options
            .network_replay
            .push(entry);
        Ok(())
    })
}

/// Appends a header to an existing offline response.
///
/// # Safety
///
/// `options` must be live and both string pointers must address their declared
/// UTF-8 byte lengths.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn edge_sandbox_options_append_network_replay_header(
    options: *mut EdgeSandboxOptions,
    entry_index: u32,
    name: *const u8,
    name_len: usize,
    value: *const u8,
    value_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        let name = input_string(name, name_len, "network replay header name")?;
        let value = input_string(value, value_len, "network replay header value")?;
        // SAFETY: guaranteed by this function's FFI contract.
        let options = &mut unsafe { options_mut(options)? }.options;
        let entry = options
            .network_replay
            .get_mut(entry_index as usize)
            .ok_or_else(|| format!("network replay entry index {entry_index} is out of bounds"))?;
        entry.headers.push((name, value));
        entry.validate()
    })
}

/// Sets deterministic clock, random and task-turn configuration.
///
/// # Safety
///
/// `options` and `deterministic` must be live readable pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_options_set_deterministic(
    options: *mut EdgeSandboxOptions,
    deterministic: *const EdgeSandboxDeterministicOptions,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        let deterministic = unsafe {
            deterministic
                .as_ref()
                .copied()
                .ok_or_else(|| "deterministic options pointer is null".to_owned())?
        };
        if deterministic.has_clock_epoch_ms > 1
            || deterministic.has_random_seed > 1
            || deterministic.reserved != [0; 6]
        {
            return Err("deterministic options flags or reserved bytes are invalid".to_owned());
        }
        let value = crate::DeterministicExecution {
            clock_epoch_ms: (deterministic.has_clock_epoch_ms == 1)
                .then_some(deterministic.clock_epoch_ms),
            clock_step_ms: deterministic.clock_step_ms,
            random_seed: (deterministic.has_random_seed == 1).then_some(deterministic.random_seed),
            max_task_turns: deterministic.max_task_turns as usize,
        };
        value.validate()?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { options_mut(options)? }.options.deterministic = value;
        Ok(())
    })
}

fn optional_limit(value: u64, name: &str) -> Result<Option<usize>, String> {
    if value == 0 {
        return Ok(None);
    }
    usize::try_from(value)
        .map(Some)
        .map_err(|_| format!("{name} does not fit the host address space"))
}

/// Sets worker resource limits. Zero-valued fields retain isolated defaults.
///
/// # Safety
///
/// `options` and `limits` must be live readable pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_options_set_limits(
    options: *mut EdgeSandboxOptions,
    limits: *const EdgeSandboxLimits,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        let limits = unsafe {
            limits
                .as_ref()
                .copied()
                .ok_or_else(|| "sandbox limits pointer is null".to_owned())?
        };
        let value = crate::SandboxLimits {
            timeout: (limits.timeout_ms != 0)
                .then(|| std::time::Duration::from_millis(limits.timeout_ms)),
            max_heap_bytes: optional_limit(limits.max_heap_bytes, "max_heap_bytes")?,
            max_resident_bytes: optional_limit(limits.max_resident_bytes, "max_resident_bytes")?,
            max_source_bytes: optional_limit(limits.max_source_bytes, "max_source_bytes")?,
            max_output_bytes: optional_limit(limits.max_output_bytes, "max_output_bytes")?,
        };
        value.validate()?;
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { options_mut(options)? }.options.limits = value;
        Ok(())
    })
}

/// Validates a complete runtime-options builder without creating V8 or a
/// worker process.
///
/// # Safety
///
/// `options` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_options_validate(
    options: *const EdgeSandboxOptions,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    profile_operation(error_out, || {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe { options_ref(options)? }.options.validate()
    })
}

fn create_handle(
    worker_path: *const u8,
    worker_path_len: usize,
    options: crate::EdgeRuntimeOptions,
) -> Result<*mut EdgeSandboxHandle, String> {
    let path = input_string(worker_path, worker_path_len, "worker executable path")?;
    let runtime = crate::IsolatedEdgeRuntime::with_worker_executable(options, PathBuf::from(path))?;
    Ok(Box::into_raw(Box::new(EdgeSandboxHandle { runtime })))
}

fn create_self_hosted_handle(
    options: crate::EdgeRuntimeOptions,
) -> Result<*mut EdgeSandboxHandle, String> {
    let runtime = crate::IsolatedEdgeRuntime::self_hosted(options)?;
    Ok(Box::into_raw(Box::new(EdgeSandboxHandle { runtime })))
}

/// Creates a process-isolated runtime using only this loaded DLL/SO.
///
/// Windows starts the exported DLL worker entry through the operating
/// system's DLL loader. Linux forks directly into the already-loaded shared
/// object. No edge-sandbox executable is required.
///
/// # Safety
///
/// `error_out` must be null or point to a writable [`EdgeSandboxBuffer`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_create_self_hosted(
    error_out: *mut EdgeSandboxBuffer,
) -> *mut EdgeSandboxHandle {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        create_self_hosted_handle(crate::EdgeRuntimeOptions::default())
    }));
    match operation {
        Ok(Ok(handle)) => handle,
        Ok(Err(message)) => {
            report_error(error_out, message);
            ptr::null_mut()
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            ptr::null_mut()
        }
    }
}

/// Creates a self-hosted runtime from a typed options builder.
///
/// # Safety
///
/// `options` must be a live options pointer and `error_out` must be null or
/// point to a writable [`EdgeSandboxBuffer`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_create_self_hosted_with_options(
    options: *const EdgeSandboxOptions,
    error_out: *mut EdgeSandboxBuffer,
) -> *mut EdgeSandboxHandle {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let options = unsafe { options_ref(options)? }.options.clone();
        options.validate()?;
        create_self_hosted_handle(options)
    }));
    match operation {
        Ok(Ok(handle)) => handle,
        Ok(Err(message)) => {
            report_error(error_out, message);
            ptr::null_mut()
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            ptr::null_mut()
        }
    }
}

/// Creates a self-hosted runtime from a typed fingerprint profile.
///
/// # Safety
///
/// `profile` must be a live profile pointer and `error_out` must be null or
/// point to a writable [`EdgeSandboxBuffer`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_create_self_hosted_with_profile(
    profile: *const EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> *mut EdgeSandboxHandle {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let fingerprint = unsafe { profile_ref(profile)? }.fingerprint.clone();
        fingerprint.validate()?;
        create_self_hosted_handle(crate::EdgeRuntimeOptions {
            fingerprint,
            ..crate::EdgeRuntimeOptions::default()
        })
    }));
    match operation {
        Ok(Ok(handle)) => handle,
        Ok(Err(message)) => {
            report_error(error_out, message);
            ptr::null_mut()
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            ptr::null_mut()
        }
    }
}

/// Creates a self-hosted runtime from a typed WebAudio profile.
///
/// # Safety
///
/// `audio_profile` must point to a readable profile and `error_out` must be
/// null or point to a writable [`EdgeSandboxBuffer`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_create_self_hosted_with_audio_profile(
    audio_profile: *const EdgeSandboxAudioProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> *mut EdgeSandboxHandle {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let profile = unsafe {
            audio_profile
                .as_ref()
                .copied()
                .ok_or_else(|| "WebAudio profile pointer is null".to_owned())?
        };
        let mut options = crate::EdgeRuntimeOptions::default();
        options.fingerprint.rendering.audio = profile.into();
        create_self_hosted_handle(options)
    }));
    match operation {
        Ok(Ok(handle)) => handle,
        Ok(Err(message)) => {
            report_error(error_out, message);
            ptr::null_mut()
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            ptr::null_mut()
        }
    }
}

/// Creates an isolated edge-sandbox runtime.
///
/// # Safety
///
/// `worker_path` must address `worker_path_len` readable bytes. `error_out`
/// must be null or point to a writable [`EdgeSandboxBuffer`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_create(
    worker_path: *const u8,
    worker_path_len: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> *mut EdgeSandboxHandle {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        create_handle(
            worker_path,
            worker_path_len,
            crate::EdgeRuntimeOptions::default(),
        )
    }));
    match operation {
        Ok(Ok(handle)) => handle,
        Ok(Err(message)) => {
            report_error(error_out, message);
            ptr::null_mut()
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            ptr::null_mut()
        }
    }
}

/// Creates an isolated runtime from a complete typed options builder.
///
/// The options are cloned synchronously and then cross the controller/worker
/// boundary through the bounded binary IPC protocol.
///
/// # Safety
///
/// `worker_path` must address `worker_path_len` readable bytes, `options` must
/// be a live pointer returned by [`edge_sandbox_options_create`], and
/// `error_out` must be null or point to a writable [`EdgeSandboxBuffer`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_create_with_options(
    worker_path: *const u8,
    worker_path_len: usize,
    options: *const EdgeSandboxOptions,
    error_out: *mut EdgeSandboxBuffer,
) -> *mut EdgeSandboxHandle {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let options = unsafe { options_ref(options)? }.options.clone();
        options.validate()?;
        create_handle(worker_path, worker_path_len, options)
    }));
    match operation {
        Ok(Ok(handle)) => handle,
        Ok(Err(message)) => {
            report_error(error_out, message);
            ptr::null_mut()
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            ptr::null_mut()
        }
    }
}

/// Creates an isolated runtime from a complete typed profile builder.
///
/// The profile is cloned synchronously; the caller may destroy its builder as
/// soon as this function returns.
///
/// # Safety
///
/// `worker_path` must address `worker_path_len` readable bytes, `profile` must
/// be a live pointer returned by [`edge_sandbox_profile_create`], and
/// `error_out` must be null or point to a writable [`EdgeSandboxBuffer`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_create_with_profile(
    worker_path: *const u8,
    worker_path_len: usize,
    profile: *const EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> *mut EdgeSandboxHandle {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let fingerprint = unsafe { profile_ref(profile)? }.fingerprint.clone();
        fingerprint.validate()?;
        let options = crate::EdgeRuntimeOptions {
            fingerprint,
            ..crate::EdgeRuntimeOptions::default()
        };
        create_handle(worker_path, worker_path_len, options)
    }));
    match operation {
        Ok(Ok(handle)) => handle,
        Ok(Err(message)) => {
            report_error(error_out, message);
            ptr::null_mut()
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            ptr::null_mut()
        }
    }
}

/// Creates an isolated runtime with a strongly typed WebAudio profile.
///
/// # Safety
///
/// `worker_path` must address `worker_path_len` readable bytes,
/// `audio_profile` must point to a readable [`EdgeSandboxAudioProfile`], and
/// `error_out` must be null or point to a writable [`EdgeSandboxBuffer`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_create_with_audio_profile(
    worker_path: *const u8,
    worker_path_len: usize,
    audio_profile: *const EdgeSandboxAudioProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> *mut EdgeSandboxHandle {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let profile = unsafe {
            audio_profile
                .as_ref()
                .copied()
                .ok_or_else(|| "WebAudio profile pointer is null".to_owned())?
        };
        let mut options = crate::EdgeRuntimeOptions::default();
        options.fingerprint.rendering.audio = profile.into();
        create_handle(worker_path, worker_path_len, options)
    }));
    match operation {
        Ok(Ok(handle)) => handle,
        Ok(Err(message)) => {
            report_error(error_out, message);
            ptr::null_mut()
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            ptr::null_mut()
        }
    }
}

/// Destroys a runtime returned by [`edge_sandbox_create`].
///
/// # Safety
///
/// `handle` must be null or a live pointer returned by
/// [`edge_sandbox_create`], and it must be destroyed at most once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_destroy(handle: *mut EdgeSandboxHandle) {
    if !handle.is_null() {
        // SAFETY: guaranteed by this function's FFI contract.
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

/// Rebuilds the V8 isolate inside an existing worker process with a new typed
/// fingerprint. The worker PID and IPC channels remain unchanged. All prior
/// JavaScript globals, tasks, requests, stdout and trace entries are discarded.
///
/// # Safety
///
/// `handle` and `profile` must be live pointers created by this ABI and
/// `error_out` must be null or point to a writable buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_reinitialize_with_profile(
    handle: *mut EdgeSandboxHandle,
    profile: *const EdgeSandboxProfile,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let handle = unsafe {
            handle
                .as_ref()
                .ok_or_else(|| "edge-sandbox handle is null".to_owned())?
        };
        // SAFETY: guaranteed by this function's FFI contract.
        let fingerprint = unsafe { profile_ref(profile)? }.fingerprint.clone();
        handle.runtime.reinitialize_profile(fingerprint)
    }));
    match operation {
        Ok(Ok(())) => true,
        Ok(Err(message)) => {
            report_error(error_out, message);
            false
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            false
        }
    }
}

/// Returns the operating-system PID of the isolated Worker process.
///
/// # Safety
///
/// `handle` must be live and both output pointers must be writable when non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_process_id(
    handle: *mut EdgeSandboxHandle,
    process_id_out: *mut u32,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let handle = unsafe {
            handle
                .as_ref()
                .ok_or_else(|| "edge-sandbox handle is null".to_owned())?
        };
        if process_id_out.is_null() {
            return Err("worker process-id output pointer is null".to_owned());
        }
        let process_id = handle.runtime.process_id()?;
        // SAFETY: non-null writable output is guaranteed by the contract.
        unsafe { *process_id_out = process_id };
        Ok(())
    }));
    match operation {
        Ok(Ok(())) => true,
        Ok(Err(message)) => {
            report_error(error_out, message);
            false
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            false
        }
    }
}

/// Evaluates UTF-8 JavaScript and returns its display value as owned UTF-8.
///
/// # Safety
///
/// `handle` must be live, `source` must address `source_len` readable bytes,
/// and both output pointers must be null or writable buffer structures.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_evaluate(
    handle: *mut EdgeSandboxHandle,
    source: *const u8,
    source_len: usize,
    result_out: *mut EdgeSandboxBuffer,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    reset_buffer(result_out);
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let handle = unsafe {
            handle
                .as_ref()
                .ok_or_else(|| "edge-sandbox handle is null".to_owned())?
        };
        let source = input_string(source, source_len, "JavaScript source")?;
        let value = handle.runtime.evaluate(&source)?.to_string();
        write_buffer(result_out, value)
    }));
    match operation {
        Ok(Ok(())) => true,
        Ok(Err(message)) => {
            report_error(error_out, message);
            false
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            false
        }
    }
}

/// Evaluates UTF-8 JavaScript with an explicit V8 script resource URL.
/// The URL is used by Error.stack, syntax diagnostics and stack frames.
///
/// # Safety
///
/// `handle` must be live, both input pointers must address their declared
/// readable byte lengths, and both output pointers must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_evaluate_with_source_url(
    handle: *mut EdgeSandboxHandle,
    source: *const u8,
    source_len: usize,
    source_url: *const u8,
    source_url_len: usize,
    result_out: *mut EdgeSandboxBuffer,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    reset_buffer(result_out);
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let handle = unsafe {
            handle
                .as_ref()
                .ok_or_else(|| "edge-sandbox handle is null".to_owned())?
        };
        let source = input_string(source, source_len, "JavaScript source")?;
        let source_url = input_string(source_url, source_url_len, "JavaScript source URL")?;
        let value = handle
            .runtime
            .evaluate_with_source_url(&source, &source_url)?
            .to_string();
        write_buffer(result_out, value)
    }));
    match operation {
        Ok(Ok(())) => true,
        Ok(Err(message)) => {
            report_error(error_out, message);
            false
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            false
        }
    }
}

/// Enables V8-native API tracing for subsequent evaluations.
///
/// # Safety
///
/// `handle` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_enable_native_trace(
    handle: *mut EdgeSandboxHandle,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let handle = unsafe {
            handle
                .as_ref()
                .ok_or_else(|| "edge-sandbox handle is null".to_owned())?
        };
        handle.runtime.enable_native_trace()
    }));
    match operation {
        Ok(Ok(())) => true,
        Ok(Err(message)) => {
            report_error(error_out, message);
            false
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            false
        }
    }
}

/// Disables V8-native API tracing.
///
/// # Safety
///
/// `handle` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_disable_native_trace(
    handle: *mut EdgeSandboxHandle,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let handle = unsafe {
            handle
                .as_ref()
                .ok_or_else(|| "edge-sandbox handle is null".to_owned())?
        };
        handle.runtime.disable_native_trace()
    }));
    match operation {
        Ok(Ok(())) => true,
        Ok(Err(message)) => {
            report_error(error_out, message);
            false
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            false
        }
    }
}

/// Clears collected trace entries.
///
/// # Safety
///
/// `handle` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_clear_native_trace(
    handle: *mut EdgeSandboxHandle,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let handle = unsafe {
            handle
                .as_ref()
                .ok_or_else(|| "edge-sandbox handle is null".to_owned())?
        };
        handle.runtime.clear_native_trace()
    }));
    match operation {
        Ok(Ok(())) => true,
        Ok(Err(message)) => {
            report_error(error_out, message);
            false
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            false
        }
    }
}

/// Replaces the native Trace API exclusion rules.
///
/// Each UTF-8 rule matches one exact API path. A trailing `*` changes the
/// rule to a prefix match. Passing zero rules clears all exclusions. Rules
/// affect only subsequently recorded entries and do not use a JSON envelope.
///
/// # Safety
///
/// `handle` must be live, `exclusions` must address `exclusion_count`
/// readable string views, and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_set_native_trace_exclusions(
    handle: *mut EdgeSandboxHandle,
    exclusions: *const EdgeSandboxStringView,
    exclusion_count: usize,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let handle = unsafe {
            handle
                .as_ref()
                .ok_or_else(|| "edge-sandbox handle is null".to_owned())?
        };
        // SAFETY: guaranteed by this function's FFI contract.
        let exclusions =
            unsafe { input_string_views(exclusions, exclusion_count, "native trace exclusions")? };
        handle.runtime.set_native_trace_exclusions(&exclusions)
    }));
    match operation {
        Ok(Ok(())) => true,
        Ok(Err(message)) => {
            report_error(error_out, message);
            false
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            false
        }
    }
}

/// Returns trace entries as newline-separated UTF-8 text.
///
/// # Safety
///
/// `handle` must be live and both output pointers must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_native_trace(
    handle: *mut EdgeSandboxHandle,
    result_out: *mut EdgeSandboxBuffer,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    reset_buffer(result_out);
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let handle = unsafe {
            handle
                .as_ref()
                .ok_or_else(|| "edge-sandbox handle is null".to_owned())?
        };
        let entries = handle.runtime.native_trace()?;
        let text = entries
            .into_iter()
            .map(|entry| entry.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        write_buffer(result_out, text)
    }));
    match operation {
        Ok(Ok(())) => true,
        Ok(Err(message)) => {
            report_error(error_out, message);
            false
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            false
        }
    }
}

/// Returns trace entries containing `needle` as newline-separated UTF-8 text.
///
/// Filtering is performed inside the isolated worker, so callers can inspect
/// multi-million-entry traces without first copying the entire trace across
/// the bounded binary IPC channel.
///
/// # Safety
///
/// `handle` must be live, `needle` must describe a readable byte range, and
/// both output pointers must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_native_trace_matching(
    handle: *mut EdgeSandboxHandle,
    needle: *const u8,
    needle_len: usize,
    result_out: *mut EdgeSandboxBuffer,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    reset_buffer(result_out);
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let handle = unsafe {
            handle
                .as_ref()
                .ok_or_else(|| "edge-sandbox handle is null".to_owned())?
        };
        let needle = input_string(needle, needle_len, "native trace filter")?;
        let entries = handle.runtime.native_trace_matching(&needle)?;
        let text = entries
            .into_iter()
            .map(|entry| entry.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        write_buffer(result_out, text)
    }));
    match operation {
        Ok(Ok(())) => true,
        Ok(Err(message)) => {
            report_error(error_out, message);
            false
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            false
        }
    }
}

/// Returns all captured XHR and fetch requests in the versioned ESNR binary
/// format. Request capture is always active and does not require tracing.
///
/// The byte stream starts with `ESNR`, a little-endian u16 version, a reserved
/// u16 and a u32 record count. Version 1 records contain sequence, source,
/// method, URL, ordered request headers and exact body bytes.
///
/// # Safety
///
/// `handle` must be live and both output pointers must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_network_requests(
    handle: *mut EdgeSandboxHandle,
    result_out: *mut EdgeSandboxBuffer,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    reset_buffer(result_out);
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let handle = unsafe {
            handle
                .as_ref()
                .ok_or_else(|| "edge-sandbox handle is null".to_owned())?
        };
        let entries = handle.runtime.network_requests()?;
        let bytes = crate::network_capture::encode_binary(&entries)?;
        write_byte_buffer(result_out, bytes)
    }));
    match operation {
        Ok(Ok(())) => true,
        Ok(Err(message)) => {
            report_error(error_out, message);
            false
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            false
        }
    }
}

/// Clears all captured XHR and fetch request records.
///
/// # Safety
///
/// `handle` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_clear_network_requests(
    handle: *mut EdgeSandboxHandle,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let handle = unsafe {
            handle
                .as_ref()
                .ok_or_else(|| "edge-sandbox handle is null".to_owned())?
        };
        handle.runtime.clear_network_requests()
    }));
    match operation {
        Ok(Ok(())) => true,
        Ok(Err(message)) => {
            report_error(error_out, message);
            false
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            false
        }
    }
}

/// Returns captured console output in the versioned ESSO binary format.
/// Capture is always active for console output and does not require tracing.
///
/// # Safety
///
/// `handle` must be live and both output pointers must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_stdout(
    handle: *mut EdgeSandboxHandle,
    result_out: *mut EdgeSandboxBuffer,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    reset_buffer(result_out);
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let handle = unsafe {
            handle
                .as_ref()
                .ok_or_else(|| "edge-sandbox handle is null".to_owned())?
        };
        let entries = handle.runtime.stdout()?;
        let bytes = crate::console_capture::encode_binary(&entries)?;
        write_byte_buffer(result_out, bytes)
    }));
    match operation {
        Ok(Ok(())) => true,
        Ok(Err(message)) => {
            report_error(error_out, message);
            false
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            false
        }
    }
}

/// Clears captured console stdout records.
///
/// # Safety
///
/// `handle` must be live and `error_out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_clear_stdout(
    handle: *mut EdgeSandboxHandle,
    error_out: *mut EdgeSandboxBuffer,
) -> bool {
    reset_buffer(error_out);
    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: guaranteed by this function's FFI contract.
        let handle = unsafe {
            handle
                .as_ref()
                .ok_or_else(|| "edge-sandbox handle is null".to_owned())?
        };
        handle.runtime.clear_stdout()
    }));
    match operation {
        Ok(Ok(())) => true,
        Ok(Err(message)) => {
            report_error(error_out, message);
            false
        }
        Err(payload) => {
            report_error(error_out, panic_message(payload));
            false
        }
    }
}

/// Releases a buffer returned by this native API.
///
/// # Safety
///
/// `buffer` must be null or point to a buffer returned by this API, and each
/// allocation must be released at most once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn edge_sandbox_buffer_free(buffer: *mut EdgeSandboxBuffer) {
    if buffer.is_null() {
        return;
    }
    // SAFETY: guaranteed by this function's FFI contract.
    let buffer = unsafe { &mut *buffer };
    if !buffer.data.is_null() && buffer.len != 0 {
        let slice = ptr::slice_from_raw_parts_mut(buffer.data, buffer.len);
        // SAFETY: the allocation originated from Box<[u8]> in write_buffer.
        unsafe {
            drop(Box::from_raw(slice));
        }
    }
    *buffer = EdgeSandboxBuffer::default();
}

/// Returns the stable ABI version exposed by this library.
#[unsafe(no_mangle)]
pub extern "C" fn edge_sandbox_abi_version() -> u32 {
    1
}

// Keep the opaque handle ABI visibly pointer-sized in generated bindings.
const _: () =
    assert!(std::mem::size_of::<*mut EdgeSandboxHandle>() == std::mem::size_of::<*mut c_void>());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_document_media_and_activation_fields_cross_the_typed_ffi() {
        let mut profile = EdgeSandboxProfile {
            fingerprint: crate::EdgeFingerprint::default(),
        };
        let profile_ptr = &mut profile as *mut EdgeSandboxProfile;
        let mut error = EdgeSandboxBuffer::default();
        let high = b"high";

        // SAFETY: profile and error point to live stack values and `high`
        // remains readable for the duration of the synchronous call.
        unsafe {
            assert!(edge_sandbox_profile_set_u32(
                profile_ptr,
                profile_field::DOCUMENT_BODY_CHILD_ELEMENT_COUNT,
                5,
                &mut error,
            ));
            assert!(edge_sandbox_profile_set_f64(
                profile_ptr,
                profile_field::DOCUMENT_BODY_CLIENT_HEIGHT,
                23.0,
                &mut error,
            ));
            assert!(edge_sandbox_profile_set_bool(
                profile_ptr,
                profile_field::NAVIGATOR_USER_ACTIVATION_HAS_BEEN_ACTIVE,
                true,
                &mut error,
            ));
            assert!(edge_sandbox_profile_set_bool(
                profile_ptr,
                profile_field::NAVIGATOR_USER_ACTIVATION_IS_ACTIVE,
                true,
                &mut error,
            ));
            assert!(edge_sandbox_profile_set_bool(
                profile_ptr,
                profile_field::MEDIA_PREFERENCE_REDUCED_TRANSPARENCY,
                true,
                &mut error,
            ));
            assert!(edge_sandbox_profile_set_string(
                profile_ptr,
                profile_field::MEDIA_PREFERENCE_VIDEO_DYNAMIC_RANGE,
                high.as_ptr(),
                high.len(),
                &mut error,
            ));
        }

        assert_eq!(
            profile.fingerprint.document.body_child_element_count,
            Some(5)
        );
        assert_eq!(profile.fingerprint.document.body_client_height, Some(23.0));
        assert!(
            profile
                .fingerprint
                .navigator
                .user_activation_has_been_active
        );
        assert!(profile.fingerprint.navigator.user_activation_is_active);
        assert!(profile.fingerprint.media_preferences.reduced_transparency);
        assert_eq!(
            profile.fingerprint.media_preferences.video_dynamic_range,
            "high"
        );
        assert_eq!(profile.fingerprint.validate(), Ok(()));
        assert!(error.data.is_null());
        assert_eq!(error.len, 0);
    }

    #[test]
    fn performance_entry_crosses_the_typed_ffi_without_json() {
        let mut profile = EdgeSandboxProfile {
            fingerprint: crate::EdgeFingerprint::default(),
        };
        let profile_ptr = &mut profile as *mut EdgeSandboxProfile;
        let mut error = EdgeSandboxBuffer::default();
        let name = b"https://profile.example/ips.js";
        let entry_type = b"resource";
        let initiator_type = b"script";
        let rendering = b"non-blocking";
        let content_type = b"text/javascript";
        let encoding = b"zstd";
        // SAFETY: an all-zero value is valid for this C input structure; all
        // pointer fields have a zero length until replaced below.
        let mut entry: EdgeSandboxPerformanceEntryProfile = unsafe { std::mem::zeroed() };
        entry.name = EdgeSandboxStringView {
            data: name.as_ptr(),
            len: name.len(),
        };
        entry.entry_type = EdgeSandboxStringView {
            data: entry_type.as_ptr(),
            len: entry_type.len(),
        };
        entry.initiator_type = EdgeSandboxStringView {
            data: initiator_type.as_ptr(),
            len: initiator_type.len(),
        };
        entry.render_blocking_status = EdgeSandboxStringView {
            data: rendering.as_ptr(),
            len: rendering.len(),
        };
        entry.content_type = EdgeSandboxStringView {
            data: content_type.as_ptr(),
            len: content_type.len(),
        };
        entry.content_encoding = EdgeSandboxStringView {
            data: encoding.as_ptr(),
            len: encoding.len(),
        };
        entry.encoded_body_size = 291181;
        entry.decoded_body_size = 609863;
        entry.response_status = 200;
        entry.has_encoded_body_size = 1;
        entry.has_decoded_body_size = 1;
        entry.has_response_status = 1;

        // SAFETY: all pointers above remain live throughout the synchronous calls.
        unsafe {
            assert!(edge_sandbox_profile_clear_performance_entries(
                profile_ptr,
                &mut error,
            ));
            assert!(edge_sandbox_profile_append_performance_entry(
                profile_ptr,
                &entry,
                &mut error,
            ));
        }
        let configured = profile
            .fingerprint
            .performance
            .entries
            .as_ref()
            .expect("performance override");
        assert_eq!(configured.len(), 1);
        assert_eq!(configured[0].name, "https://profile.example/ips.js");
        assert_eq!(configured[0].content_encoding, "zstd");
        assert_eq!(
            configured[0].resolved_body_sizes(),
            (291481, 291181, 609863)
        );
        assert_eq!(profile.fingerprint.validate(), Ok(()));
        assert!(error.data.is_null());
        assert_eq!(error.len, 0);
    }
}
