"""Strongly typed Edge fingerprint profile values for the native sandbox."""

from __future__ import annotations

from dataclasses import dataclass
from enum import IntEnum


class ProfileField(IntEnum):
    ID = 1
    LOCALE = 2
    TIME_ZONE = 3

    NAVIGATOR_USER_AGENT = 10
    NAVIGATOR_APP_VERSION = 11
    NAVIGATOR_APP_CODE_NAME = 12
    NAVIGATOR_APP_NAME = 13
    NAVIGATOR_PLATFORM = 14
    NAVIGATOR_PRODUCT = 15
    NAVIGATOR_PRODUCT_SUB = 16
    NAVIGATOR_VENDOR = 17
    NAVIGATOR_VENDOR_SUB = 18
    NAVIGATOR_LANGUAGE = 19
    NAVIGATOR_DO_NOT_TRACK = 20

    UA_PLATFORM = 30
    UA_ARCHITECTURE = 31
    UA_BITNESS = 32
    UA_MODEL = 33
    UA_PLATFORM_VERSION = 34
    UA_FULL_VERSION = 35
    NETWORK_EFFECTIVE_TYPE = 40
    NETWORK_CONNECTION_TYPE = 41

    CANVAS_DATA_URL_SALT = 50
    WEBGL_VENDOR = 60
    WEBGL_RENDERER = 61
    WEBGL_UNMASKED_VENDOR = 62
    WEBGL_UNMASKED_RENDERER = 63
    WEBGL1_VERSION = 64
    WEBGL1_SHADING_LANGUAGE_VERSION = 65
    WEBGL2_VERSION = 66
    WEBGL2_SHADING_LANGUAGE_VERSION = 67
    WEBGL_CONTEXT_POWER_PREFERENCE = 68
    WEBGPU_VENDOR = 70
    WEBGPU_ARCHITECTURE = 71
    WEBGPU_DEVICE = 72
    WEBGPU_DESCRIPTION = 73
    RTC_OFFER_SDP = 74
    RTC_ANSWER_SDP = 75
    SCREEN_ORIENTATION_TYPE = 76
    DEVICE_POSTURE = 77
    CSS_BODY = 78
    CSS_INPUT_COMMON = 79
    CSS_INPUT_HIDDEN = 80
    CSS_INPUT_SEARCH = 81
    CSS_INPUT_CHECKBOX_RADIO = 82
    CSS_INPUT_RANGE = 83
    CSS_INPUT_COLOR = 84
    CSS_INPUT_DATE = 85
    CSS_INPUT_TIME = 86
    CSS_INPUT_DATETIME_LOCAL = 87
    CSS_INPUT_MONTH = 88
    CSS_INPUT_WEEK = 89
    CSS_INPUT_IMAGE = 90
    CSS_INPUT_BUTTON = 91
    CSS_INPUT_SUBMIT_RESET = 92
    CSS_INPUT_FILE = 93
    CSS_INPUT_TEXT = 94
    PERFORMANCE_EVALUATED_SCRIPT_CONTENT_ENCODING = 95
    DOCUMENT_VISIBILITY_STATE = 96

    PERMISSION_ACCELEROMETER = 800
    PERMISSION_BACKGROUND_SYNC = 801
    PERMISSION_CAMERA = 802
    PERMISSION_CLIPBOARD_READ = 803
    PERMISSION_CLIPBOARD_WRITE = 804
    PERMISSION_GEOLOCATION = 805
    PERMISSION_GYROSCOPE = 806
    PERMISSION_LOCAL_FONTS = 807
    PERMISSION_MAGNETOMETER = 808
    PERMISSION_MICROPHONE = 809
    PERMISSION_MIDI = 810
    PERMISSION_NOTIFICATIONS = 811
    PERMISSION_PAYMENT_HANDLER = 812
    PERMISSION_PERSISTENT_STORAGE = 813
    PERMISSION_SPEAKER_SELECTION = 814
    PERMISSION_STORAGE_ACCESS = 815
    PERMISSION_TOP_LEVEL_STORAGE_ACCESS = 816
    PERMISSION_WINDOW_MANAGEMENT = 817

    MEDIA_PREFERENCE_COLOR_SCHEME = 820
    MEDIA_PREFERENCE_CONTRAST = 821
    MEDIA_PREFERENCE_COLOR_GAMUT = 822
    MEDIA_PREFERENCE_POINTER = 823
    MEDIA_PREFERENCE_ANY_POINTER = 824
    MEDIA_PREFERENCE_HOVER = 825
    MEDIA_PREFERENCE_ANY_HOVER = 826
    MEDIA_PREFERENCE_DISPLAY_MODE = 827
    MEDIA_PREFERENCE_DYNAMIC_RANGE = 828
    MEDIA_PREFERENCE_SCRIPTING = 829
    MEDIA_PREFERENCE_VIDEO_DYNAMIC_RANGE = 830

    NAVIGATOR_LANGUAGES = 100
    UA_FORM_FACTORS = 101
    WEBGL1_EXTENSIONS = 102
    WEBGL2_EXTENSIONS = 103
    WEBGPU_FEATURES = 104
    FONT_FAMILIES = 105
    MEDIA_SUPPORTED_CONSTRAINTS = 106
    MEDIA_CAN_PLAY_PROBABLY_TYPES = 107
    MEDIA_CAN_PLAY_MAYBE_TYPES = 108
    MEDIA_SOURCE_TYPES = 109
    MEDIA_RECORDER_TYPES = 110
    MEDIA_DECODING_SUPPORTED_TYPES = 111
    MEDIA_DECODING_SMOOTH_TYPES = 112
    MEDIA_DECODING_POWER_EFFICIENT_TYPES = 113
    MEDIA_ENCODING_SUPPORTED_TYPES = 114
    MEDIA_ENCODING_SMOOTH_TYPES = 115
    MEDIA_ENCODING_POWER_EFFICIENT_TYPES = 116
    IMAGE_DECODER_TYPES = 117
    XR_SUPPORTED_SESSION_MODES = 118
    AUDIO_DECODER_CODECS = 119
    AUDIO_ENCODER_CODECS = 120
    VIDEO_DECODER_CODECS = 121
    VIDEO_ENCODER_CODECS = 122

    HARDWARE_CONCURRENCY = 200
    MAX_TOUCH_POINTS = 201
    NETWORK_RTT = 202
    WEBGPU_MAX_TEXTURE_DIMENSION_2D = 203
    AUDIO_MAX_CHANNEL_COUNT = 204
    MEDIA_PREFERENCE_MONOCHROME_BITS = 205
    WEBGPU_MAX_TEXTURE_DIMENSION_1D = 206
    WEBGPU_MAX_TEXTURE_DIMENSION_3D = 207
    WEBGPU_MAX_TEXTURE_ARRAY_LAYERS = 208
    WEBGPU_MAX_BIND_GROUPS = 209
    WEBGPU_MAX_BIND_GROUPS_PLUS_VERTEX_BUFFERS = 210
    WEBGPU_MAX_BINDINGS_PER_BIND_GROUP = 211
    WEBGPU_MAX_DYNAMIC_UNIFORM_BUFFERS_PER_PIPELINE_LAYOUT = 212
    WEBGPU_MAX_DYNAMIC_STORAGE_BUFFERS_PER_PIPELINE_LAYOUT = 213
    WEBGPU_MAX_SAMPLED_TEXTURES_PER_SHADER_STAGE = 214
    WEBGPU_MAX_SAMPLERS_PER_SHADER_STAGE = 215
    WEBGPU_MAX_STORAGE_BUFFERS_PER_SHADER_STAGE = 216
    WEBGPU_MAX_STORAGE_TEXTURES_PER_SHADER_STAGE = 217
    WEBGPU_MAX_UNIFORM_BUFFERS_PER_SHADER_STAGE = 218
    WEBGPU_MIN_UNIFORM_BUFFER_OFFSET_ALIGNMENT = 219
    WEBGPU_MIN_STORAGE_BUFFER_OFFSET_ALIGNMENT = 220
    WEBGPU_MAX_VERTEX_BUFFERS = 221
    WEBGPU_MAX_VERTEX_ATTRIBUTES = 222
    WEBGPU_MAX_VERTEX_BUFFER_ARRAY_STRIDE = 223
    WEBGPU_MAX_INTER_STAGE_SHADER_VARIABLES = 224
    WEBGPU_MAX_COLOR_ATTACHMENTS = 225
    WEBGPU_MAX_COLOR_ATTACHMENT_BYTES_PER_SAMPLE = 226
    WEBGPU_MAX_COMPUTE_WORKGROUP_STORAGE_SIZE = 227
    WEBGPU_MAX_COMPUTE_INVOCATIONS_PER_WORKGROUP = 228
    WEBGPU_MAX_COMPUTE_WORKGROUP_SIZE_X = 229
    WEBGPU_MAX_COMPUTE_WORKGROUP_SIZE_Y = 230
    WEBGPU_MAX_COMPUTE_WORKGROUP_SIZE_Z = 231
    WEBGPU_MAX_COMPUTE_WORKGROUPS_PER_DIMENSION = 232
    WEBGPU_MAX_IMMEDIATE_SIZE = 233
    WEBGPU_MAX_STORAGE_BUFFERS_IN_FRAGMENT_STAGE = 234
    WEBGPU_MAX_STORAGE_TEXTURES_IN_FRAGMENT_STAGE = 235
    WEBGPU_MAX_STORAGE_BUFFERS_IN_VERTEX_STAGE = 236
    WEBGPU_MAX_STORAGE_TEXTURES_IN_VERTEX_STAGE = 237
    SCREEN_ORIENTATION_ANGLE = 238
    WEBGPU_SUBGROUP_MIN_SIZE = 239
    WEBGPU_SUBGROUP_MAX_SIZE = 240
    DOCUMENT_BODY_CHILD_ELEMENT_COUNT = 241

    TIME_ZONE_OFFSET_MINUTES = 300
    SCREEN_WIDTH = 301
    SCREEN_HEIGHT = 302
    SCREEN_AVAIL_WIDTH = 303
    SCREEN_AVAIL_HEIGHT = 304
    SCREEN_AVAIL_LEFT = 305
    SCREEN_AVAIL_TOP = 306
    SCREEN_COLOR_DEPTH = 307
    SCREEN_PIXEL_DEPTH = 308
    WEBGL_MAX_TEXTURE_SIZE = 309
    WEBGL_MAX_CUBE_MAP_TEXTURE_SIZE = 310
    WEBGL_MAX_RENDERBUFFER_SIZE = 311
    WEBGL_MAX_VIEWPORT_WIDTH = 312
    WEBGL_MAX_VIEWPORT_HEIGHT = 313
    WEBGL_MAX_VERTEX_ATTRIBS = 314
    WEBGL_MAX_VERTEX_TEXTURE_IMAGE_UNITS = 315
    WEBGL_MAX_TEXTURE_IMAGE_UNITS = 316
    WEBGL_MAX_COMBINED_TEXTURE_IMAGE_UNITS = 317
    WEBGL2_MAX_DRAW_BUFFERS = 318
    WEBGL2_MAX_COLOR_ATTACHMENTS = 319
    WEBGL2_MAX_SAMPLES = 320
    WEBGL_SHADER_PRECISION_RANGE_MIN = 321
    WEBGL_SHADER_PRECISION_RANGE_MAX = 322
    WEBGL_SHADER_PRECISION_BITS = 323
    WEBGL_MAX_VERTEX_UNIFORM_VECTORS = 324
    WEBGL_MAX_VARYING_VECTORS = 325
    WEBGL_MAX_FRAGMENT_UNIFORM_VECTORS = 326
    WEBGL_SUBPIXEL_BITS = 327
    WEBGL2_MAX_3D_TEXTURE_SIZE = 328
    WEBGL2_MAX_ARRAY_TEXTURE_LAYERS = 329
    WEBGL2_MAX_VERTEX_UNIFORM_COMPONENTS = 330
    WEBGL2_MAX_FRAGMENT_UNIFORM_COMPONENTS = 331
    WEBGL2_MAX_VARYING_COMPONENTS = 332
    WEBGL2_MAX_VERTEX_OUTPUT_COMPONENTS = 333
    WEBGL2_MAX_FRAGMENT_INPUT_COMPONENTS = 334
    WEBGL2_MAX_VERTEX_UNIFORM_BLOCKS = 335
    WEBGL2_MAX_FRAGMENT_UNIFORM_BLOCKS = 336
    WEBGL2_MAX_COMBINED_UNIFORM_BLOCKS = 337
    WEBGL2_MAX_UNIFORM_BUFFER_BINDINGS = 338
    WEBGL2_MAX_UNIFORM_BLOCK_SIZE = 339
    WEBGL2_MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS = 340
    WEBGL2_MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS = 341
    WEBGL2_MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS = 342
    WEBGL2_MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS = 343
    WEBGL2_MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS = 344
    WEBGL2_MAX_PROGRAM_TEXEL_OFFSET = 345
    WEBGL2_MAX_ELEMENTS_VERTICES = 346
    WEBGL2_MAX_ELEMENTS_INDICES = 347
    WEBGL2_MAX_ELEMENT_INDEX = 348

    STORAGE_QUOTA_BYTES = 400
    STORAGE_USAGE_BYTES = 401
    AUDIO_NOISE_SEED = 402
    WEBGPU_MAX_UNIFORM_BUFFER_BINDING_SIZE = 403
    WEBGPU_MAX_STORAGE_BUFFER_BINDING_SIZE = 404
    WEBGPU_MAX_BUFFER_SIZE = 405
    TIMING_CLOCK_STEP_MS = 406
    TIMING_RANDOM_SEED = 407
    PERFORMANCE_JS_HEAP_SIZE_LIMIT = 408
    PERFORMANCE_TOTAL_JS_HEAP_SIZE = 409
    PERFORMANCE_USED_JS_HEAP_SIZE = 410
    CONSOLE_JS_HEAP_SIZE_LIMIT = 411
    CONSOLE_TOTAL_JS_HEAP_SIZE = 412
    CONSOLE_USED_JS_HEAP_SIZE = 413

    TIMING_CLOCK_EPOCH_MS = 900

    DEVICE_MEMORY_GB = 500
    NETWORK_DOWNLINK = 501
    NETWORK_DOWNLINK_MAX = 564
    SCREEN_VIEWPORT_WIDTH = 502
    SCREEN_VIEWPORT_HEIGHT = 503
    SCREEN_OUTER_WIDTH = 504
    SCREEN_OUTER_HEIGHT = 505
    WINDOW_INNER_WIDTH = 502
    WINDOW_INNER_HEIGHT = 503
    WINDOW_OUTER_WIDTH = 504
    WINDOW_OUTER_HEIGHT = 505
    SCREEN_X = 506
    SCREEN_Y = 507
    SCREEN_DEVICE_PIXEL_RATIO = 508
    CANVAS_TEXT_WIDTH_SCALE = 509
    WEBGL_MAX_ANISOTROPY = 510
    AUDIO_SAMPLE_RATE = 511
    AUDIO_BASE_LATENCY = 512
    AUDIO_OUTPUT_LATENCY = 513
    BATTERY_CHARGING_TIME = 514
    BATTERY_DISCHARGING_TIME = 515
    BATTERY_LEVEL = 516
    GEOLOCATION_LATITUDE = 517
    GEOLOCATION_LONGITUDE = 518
    GEOLOCATION_ALTITUDE = 519
    GEOLOCATION_ACCURACY = 520
    GEOLOCATION_ALTITUDE_ACCURACY = 521
    GEOLOCATION_HEADING = 522
    GEOLOCATION_SPEED = 523
    WEBGL_ALIASED_POINT_SIZE_MIN = 524
    WEBGL_ALIASED_POINT_SIZE_MAX = 525
    WEBGL_ALIASED_LINE_WIDTH_MIN = 526
    WEBGL_ALIASED_LINE_WIDTH_MAX = 527
    SENSOR_ACCELEROMETER_X = 528
    SENSOR_ACCELEROMETER_Y = 529
    SENSOR_ACCELEROMETER_Z = 530
    SENSOR_GRAVITY_X = 531
    SENSOR_GRAVITY_Y = 532
    SENSOR_GRAVITY_Z = 533
    SENSOR_LINEAR_ACCELERATION_X = 534
    SENSOR_LINEAR_ACCELERATION_Y = 535
    SENSOR_LINEAR_ACCELERATION_Z = 536
    SENSOR_GYROSCOPE_X = 537
    SENSOR_GYROSCOPE_Y = 538
    SENSOR_GYROSCOPE_Z = 539
    SENSOR_ABSOLUTE_ORIENTATION_X = 540
    SENSOR_ABSOLUTE_ORIENTATION_Y = 541
    SENSOR_ABSOLUTE_ORIENTATION_Z = 542
    SENSOR_ABSOLUTE_ORIENTATION_W = 543
    SENSOR_RELATIVE_ORIENTATION_X = 544
    SENSOR_RELATIVE_ORIENTATION_Y = 545
    SENSOR_RELATIVE_ORIENTATION_Z = 546
    SENSOR_RELATIVE_ORIENTATION_W = 547
    CANVAS_ACTUAL_BOUNDING_BOX_LEFT = 548
    CANVAS_ACTUAL_BOUNDING_BOX_RIGHT_SCALE = 549
    CANVAS_FONT_BOUNDING_BOX_ASCENT = 550
    CANVAS_FONT_BOUNDING_BOX_DESCENT = 551
    CANVAS_ACTUAL_BOUNDING_BOX_ASCENT = 552
    CANVAS_ACTUAL_BOUNDING_BOX_DESCENT = 553
    CANVAS_HANGING_BASELINE = 554
    CANVAS_ALPHABETIC_BASELINE = 555
    CANVAS_IDEOGRAPHIC_BASELINE = 556
    VISUAL_VIEWPORT_OFFSET_LEFT = 557
    VISUAL_VIEWPORT_OFFSET_TOP = 558
    VISUAL_VIEWPORT_PAGE_LEFT = 559
    VISUAL_VIEWPORT_PAGE_TOP = 560
    VISUAL_VIEWPORT_SCALE = 561
    WEBGL2_MAX_TEXTURE_LOD_BIAS = 562
    DOCUMENT_BODY_CLIENT_HEIGHT = 563
    WINDOW_IFRAME_OUTER_WIDTH = 565
    WINDOW_IFRAME_OUTER_HEIGHT = 566
    WINDOW_IFRAME_INNER_WIDTH = 567
    WINDOW_IFRAME_INNER_HEIGHT = 568

    AUDIO_CHANNEL_NOISE_AMPLITUDE = 600
    AUDIO_FREQUENCY_NOISE_AMPLITUDE = 601
    AUDIO_TIME_DOMAIN_NOISE_AMPLITUDE = 602

    NAVIGATOR_COOKIE_ENABLED = 700
    NAVIGATOR_ON_LINE = 701
    NAVIGATOR_WEBDRIVER = 702
    NAVIGATOR_PDF_VIEWER_ENABLED = 703
    UA_MOBILE = 704
    UA_WOW64 = 705
    NETWORK_SAVE_DATA = 706
    STORAGE_PERSISTED = 707
    FONT_ALLOW_UNKNOWN_FAMILIES = 708
    BATTERY_CHARGING = 709
    MEDIA_PREFERENCE_REDUCED_MOTION = 710
    MEDIA_PREFERENCE_REDUCED_DATA = 711
    MEDIA_PREFERENCE_FORCED_COLORS = 712
    MEDIA_PREFERENCE_INVERTED_COLORS = 713
    WEBGL_CONTEXT_ALPHA = 714
    WEBGL_CONTEXT_ANTIALIAS = 715
    WEBGL_CONTEXT_DEPTH = 716
    WEBGL_CONTEXT_DESYNCHRONIZED = 717
    WEBGL_CONTEXT_FAIL_IF_MAJOR_PERFORMANCE_CAVEAT = 718
    WEBGL_CONTEXT_PREMULTIPLIED_ALPHA = 719
    WEBGL_CONTEXT_PRESERVE_DRAWING_BUFFER = 720
    WEBGL_CONTEXT_STENCIL = 721
    WEBGL_CONTEXT_XR_COMPATIBLE = 722
    BLUETOOTH_AVAILABLE = 723
    MIDI_SYSEX_ENABLED = 724
    WEBGPU_DEVELOPER_FEATURES = 725
    WEBGPU_IS_FALLBACK_ADAPTER = 726
    SENSORS_AVAILABLE = 727
    WEBGPU_AVAILABLE = 728
    MEDIA_PREFERENCE_REDUCED_TRANSPARENCY = 729
    NAVIGATOR_USER_ACTIVATION_HAS_BEEN_ACTIVE = 730
    NAVIGATOR_USER_ACTIVATION_IS_ACTIVE = 731
    DOCUMENT_HAS_FOCUS = 732
    DOCUMENT_IS_POPUP = 733
    FONT_USE_SYSTEM_FONTS = 734


@dataclass(frozen=True, slots=True)
class LocaleProfile:
    locale: str | None = None
    time_zone: str | None = None
    time_zone_offset_minutes: int | None = None


@dataclass(frozen=True, slots=True)
class UserAgentBrandProfile:
    brand: str
    version: str
    full_version: str


@dataclass(frozen=True, slots=True)
class UserAgentDataProfile:
    brands: tuple[UserAgentBrandProfile, ...] | None = None
    mobile: bool | None = None
    platform: str | None = None
    architecture: str | None = None
    bitness: str | None = None
    model: str | None = None
    platform_version: str | None = None
    ua_full_version: str | None = None
    wow64: bool | None = None
    form_factors: tuple[str, ...] | None = None


@dataclass(frozen=True, slots=True)
class NetworkProfile:
    effective_type: str | None = None
    rtt: int | None = None
    downlink: float | None = None
    save_data: bool | None = None
    connection_type: str | None = None
    downlink_max: float | None = None


@dataclass(frozen=True, slots=True)
class NavigatorProfile:
    user_agent: str | None = None
    app_version: str | None = None
    app_code_name: str | None = None
    app_name: str | None = None
    platform: str | None = None
    product: str | None = None
    product_sub: str | None = None
    vendor: str | None = None
    vendor_sub: str | None = None
    language: str | None = None
    languages: tuple[str, ...] | None = None
    hardware_concurrency: int | None = None
    device_memory_gb: float | None = None
    max_touch_points: int | None = None
    cookie_enabled: bool | None = None
    on_line: bool | None = None
    webdriver: bool | None = None
    pdf_viewer_enabled: bool | None = None
    do_not_track: str | None = None
    user_activation_has_been_active: bool | None = None
    user_activation_is_active: bool | None = None
    user_agent_data: UserAgentDataProfile | None = None
    network: NetworkProfile | None = None


@dataclass(frozen=True, slots=True)
class ScreenProfile:
    width: int | None = None
    height: int | None = None
    avail_width: int | None = None
    avail_height: int | None = None
    avail_left: int | None = None
    avail_top: int | None = None
    color_depth: int | None = None
    pixel_depth: int | None = None
    viewport_width: float | None = None
    viewport_height: float | None = None
    outer_width: float | None = None
    outer_height: float | None = None
    screen_x: float | None = None
    screen_y: float | None = None
    device_pixel_ratio: float | None = None
    orientation_type: str | None = None
    orientation_angle: int | None = None
    visual_viewport_offset_left: float | None = None
    visual_viewport_offset_top: float | None = None
    visual_viewport_page_left: float | None = None
    visual_viewport_page_top: float | None = None
    visual_viewport_scale: float | None = None


@dataclass(frozen=True, slots=True)
class WindowProfile:
    """Window viewport dimensions, independent from the physical Screen.

    Explicit zero values are preserved. When a value is omitted, the binding
    derives it from its paired Window dimension or from ``ScreenProfile``.
    """

    inner_width: float | None = None
    inner_height: float | None = None
    outer_width: float | None = None
    outer_height: float | None = None
    iframe_inner_width: float | None = None
    iframe_inner_height: float | None = None
    iframe_outer_width: float | None = None
    iframe_outer_height: float | None = None


@dataclass(frozen=True, slots=True)
class CanvasProfile:
    data_url_salt: str | None = None
    text_width_scale: float | None = None
    actual_bounding_box_left: float | None = None
    actual_bounding_box_right_scale: float | None = None
    font_bounding_box_ascent: float | None = None
    font_bounding_box_descent: float | None = None
    actual_bounding_box_ascent: float | None = None
    actual_bounding_box_descent: float | None = None
    hanging_baseline: float | None = None
    alphabetic_baseline: float | None = None
    ideographic_baseline: float | None = None


@dataclass(frozen=True, slots=True)
class WebGlProfile:
    vendor: str | None = None
    renderer: str | None = None
    unmasked_vendor: str | None = None
    unmasked_renderer: str | None = None
    webgl1_version: str | None = None
    webgl1_shading_language_version: str | None = None
    webgl2_version: str | None = None
    webgl2_shading_language_version: str | None = None
    webgl1_extensions: tuple[str, ...] | None = None
    webgl2_extensions: tuple[str, ...] | None = None
    compressed_texture_formats: tuple[int, ...] | None = None
    max_texture_size: int | None = None
    max_cube_map_texture_size: int | None = None
    max_renderbuffer_size: int | None = None
    max_viewport_width: int | None = None
    max_viewport_height: int | None = None
    max_vertex_attribs: int | None = None
    max_vertex_uniform_vectors: int | None = None
    max_varying_vectors: int | None = None
    max_fragment_uniform_vectors: int | None = None
    max_vertex_texture_image_units: int | None = None
    max_texture_image_units: int | None = None
    max_combined_texture_image_units: int | None = None
    subpixel_bits: int | None = None
    webgl2_max_3d_texture_size: int | None = None
    webgl2_max_array_texture_layers: int | None = None
    webgl2_max_draw_buffers: int | None = None
    webgl2_max_color_attachments: int | None = None
    webgl2_max_samples: int | None = None
    webgl2_max_vertex_uniform_components: int | None = None
    webgl2_max_fragment_uniform_components: int | None = None
    webgl2_max_varying_components: int | None = None
    webgl2_max_vertex_output_components: int | None = None
    webgl2_max_fragment_input_components: int | None = None
    webgl2_max_vertex_uniform_blocks: int | None = None
    webgl2_max_fragment_uniform_blocks: int | None = None
    webgl2_max_combined_uniform_blocks: int | None = None
    webgl2_max_uniform_buffer_bindings: int | None = None
    webgl2_max_uniform_block_size: int | None = None
    webgl2_max_combined_vertex_uniform_components: int | None = None
    webgl2_max_combined_fragment_uniform_components: int | None = None
    webgl2_max_transform_feedback_separate_attribs: int | None = None
    webgl2_max_transform_feedback_interleaved_components: int | None = None
    webgl2_max_transform_feedback_separate_components: int | None = None
    webgl2_max_program_texel_offset: int | None = None
    webgl2_max_elements_vertices: int | None = None
    webgl2_max_elements_indices: int | None = None
    webgl2_max_element_index: int | None = None
    webgl2_max_texture_lod_bias: float | None = None
    max_anisotropy: float | None = None
    aliased_point_size_min: float | None = None
    aliased_point_size_max: float | None = None
    aliased_line_width_min: float | None = None
    aliased_line_width_max: float | None = None
    shader_precision_range_min: int | None = None
    shader_precision_range_max: int | None = None
    shader_precision_bits: int | None = None
    context_alpha: bool | None = None
    context_antialias: bool | None = None
    context_depth: bool | None = None
    context_desynchronized: bool | None = None
    context_fail_if_major_performance_caveat: bool | None = None
    context_premultiplied_alpha: bool | None = None
    context_preserve_drawing_buffer: bool | None = None
    context_stencil: bool | None = None
    context_xr_compatible: bool | None = None
    context_power_preference: str | None = None


@dataclass(frozen=True, slots=True)
class WebGpuProfile:
    available: bool | None = None
    vendor: str | None = None
    architecture: str | None = None
    device: str | None = None
    description: str | None = None
    developer_features: bool | None = None
    subgroup_min_size: int | None = None
    subgroup_max_size: int | None = None
    is_fallback_adapter: bool | None = None
    features: tuple[str, ...] | None = None
    max_texture_dimension_1d: int | None = None
    max_texture_dimension_2d: int | None = None
    max_texture_dimension_3d: int | None = None
    max_texture_array_layers: int | None = None
    max_bind_groups: int | None = None
    max_bind_groups_plus_vertex_buffers: int | None = None
    max_bindings_per_bind_group: int | None = None
    max_dynamic_uniform_buffers_per_pipeline_layout: int | None = None
    max_dynamic_storage_buffers_per_pipeline_layout: int | None = None
    max_sampled_textures_per_shader_stage: int | None = None
    max_samplers_per_shader_stage: int | None = None
    max_storage_buffers_per_shader_stage: int | None = None
    max_storage_textures_per_shader_stage: int | None = None
    max_uniform_buffers_per_shader_stage: int | None = None
    max_uniform_buffer_binding_size: int | None = None
    max_storage_buffer_binding_size: int | None = None
    min_uniform_buffer_offset_alignment: int | None = None
    min_storage_buffer_offset_alignment: int | None = None
    max_vertex_buffers: int | None = None
    max_buffer_size: int | None = None
    max_vertex_attributes: int | None = None
    max_vertex_buffer_array_stride: int | None = None
    max_inter_stage_shader_variables: int | None = None
    max_color_attachments: int | None = None
    max_color_attachment_bytes_per_sample: int | None = None
    max_compute_workgroup_storage_size: int | None = None
    max_compute_invocations_per_workgroup: int | None = None
    max_compute_workgroup_size_x: int | None = None
    max_compute_workgroup_size_y: int | None = None
    max_compute_workgroup_size_z: int | None = None
    max_compute_workgroups_per_dimension: int | None = None
    max_immediate_size: int | None = None
    max_storage_buffers_in_fragment_stage: int | None = None
    max_storage_textures_in_fragment_stage: int | None = None
    max_storage_buffers_in_vertex_stage: int | None = None
    max_storage_textures_in_vertex_stage: int | None = None


@dataclass(frozen=True, slots=True)
class WebAudioProfile:
    sample_rate: float = 48_000.0
    max_channel_count: int = 2
    base_latency: float = 0.01
    output_latency: float = 0.0
    noise_seed: int = 0x45444745
    channel_noise_amplitude: float = 0.0
    frequency_noise_amplitude: float = 0.0
    time_domain_noise_amplitude: float = 0.0


@dataclass(frozen=True, slots=True)
class StorageProfile:
    quota_bytes: int | None = None
    usage_bytes: int | None = None
    persisted: bool | None = None


@dataclass(frozen=True, slots=True)
class SpeechVoiceProfile:
    voice_uri: str
    name: str
    lang: str
    local_service: bool
    is_default: bool


@dataclass(frozen=True, slots=True)
class SpeechProfile:
    voices: tuple[SpeechVoiceProfile, ...] | None = None


@dataclass(frozen=True, slots=True)
class LocalFontProfile:
    postscript_name: str
    full_name: str
    family: str
    style: str


@dataclass(frozen=True, slots=True)
class FontMetricProfile:
    family: str
    width_scale: float = 1.0
    monospace: bool = False


@dataclass(frozen=True, slots=True)
class FontBinarySourceProfile:
    """Native OpenType source used for HarfBuzz-compatible text shaping.

    ``face_index`` selects a member of a TTC/OTC collection. TTF and OTF
    sources normally use index zero.
    """

    family: str
    path: str
    face_index: int = 0


@dataclass(frozen=True, slots=True)
class FontProfile:
    families: tuple[str, ...] | None = None
    allow_unknown_families: bool | None = None
    local_fonts: tuple[LocalFontProfile, ...] | None = None
    metrics: tuple[FontMetricProfile, ...] | None = None
    use_system_fonts: bool | None = None
    binary_sources: tuple[FontBinarySourceProfile, ...] | None = None


@dataclass(frozen=True, slots=True)
class CssProfile:
    body: str | None = None
    input_common: str | None = None
    input_hidden: str | None = None
    input_search: str | None = None
    input_checkbox_radio: str | None = None
    input_range: str | None = None
    input_color: str | None = None
    input_date: str | None = None
    input_time: str | None = None
    input_datetime_local: str | None = None
    input_month: str | None = None
    input_week: str | None = None
    input_image: str | None = None
    input_button: str | None = None
    input_submit_reset: str | None = None
    input_file: str | None = None
    input_text: str | None = None


@dataclass(frozen=True, slots=True)
class DocumentProfile:
    """Initial document state used before standalone ``evaluate`` runs.

    ``body_child_element_count`` is materialized as real placeholder ``div``
    nodes. ``body_client_height`` overrides only the initial BODY clientHeight;
    omitted values preserve normal HTML/CSS-derived behavior. The remaining
    values describe the initial page/document state shared by the matching
    document APIs and Performance visibility entry.
    """

    body_child_element_count: int | None = None
    body_client_height: float | None = None
    has_focus: bool | None = None
    visibility_state: str | None = None
    is_popup: bool | None = None


@dataclass(frozen=True, slots=True)
class MediaDeviceProfile:
    device_id: str
    kind: str
    label: str
    group_id: str


@dataclass(frozen=True, slots=True)
class RtcCodecProfile:
    mime_type: str
    clock_rate: int
    channels: int | None = None
    sdp_fmtp_line: str | None = None


@dataclass(frozen=True, slots=True)
class RtcHeaderExtensionProfile:
    kind: str
    uri: str


@dataclass(frozen=True, slots=True)
class MediaProfile:
    devices: tuple[MediaDeviceProfile, ...] | None = None
    supported_constraints: tuple[str, ...] | None = None
    can_play_probably_types: tuple[str, ...] | None = None
    can_play_maybe_types: tuple[str, ...] | None = None
    media_source_types: tuple[str, ...] | None = None
    media_recorder_types: tuple[str, ...] | None = None
    decoding_supported_types: tuple[str, ...] | None = None
    decoding_smooth_types: tuple[str, ...] | None = None
    decoding_power_efficient_types: tuple[str, ...] | None = None
    encoding_supported_types: tuple[str, ...] | None = None
    encoding_smooth_types: tuple[str, ...] | None = None
    encoding_power_efficient_types: tuple[str, ...] | None = None
    image_decoder_types: tuple[str, ...] | None = None
    audio_decoder_codecs: tuple[str, ...] | None = None
    audio_encoder_codecs: tuple[str, ...] | None = None
    video_decoder_codecs: tuple[str, ...] | None = None
    video_encoder_codecs: tuple[str, ...] | None = None
    rtc_audio_codecs: tuple[RtcCodecProfile, ...] | None = None
    rtc_video_codecs: tuple[RtcCodecProfile, ...] | None = None
    rtc_header_extensions: tuple[RtcHeaderExtensionProfile, ...] | None = None
    rtc_offer_sdp: str | None = None
    rtc_answer_sdp: str | None = None


@dataclass(frozen=True, slots=True)
class PermissionsProfile:
    accelerometer: str | None = None
    background_sync: str | None = None
    camera: str | None = None
    clipboard_read: str | None = None
    clipboard_write: str | None = None
    geolocation: str | None = None
    gyroscope: str | None = None
    local_fonts: str | None = None
    magnetometer: str | None = None
    microphone: str | None = None
    midi: str | None = None
    notifications: str | None = None
    payment_handler: str | None = None
    persistent_storage: str | None = None
    speaker_selection: str | None = None
    storage_access: str | None = None
    top_level_storage_access: str | None = None
    window_management: str | None = None


@dataclass(frozen=True, slots=True)
class BatteryProfile:
    charging: bool | None = None
    charging_time: float | None = None
    discharging_time: float | None = None
    level: float | None = None


@dataclass(frozen=True, slots=True)
class GeolocationProfile:
    latitude: float | None = None
    longitude: float | None = None
    altitude: float | None = None
    accuracy: float | None = None
    altitude_accuracy: float | None = None
    heading: float | None = None
    speed: float | None = None


@dataclass(frozen=True, slots=True)
class MediaPreferencesProfile:
    color_scheme: str | None = None
    contrast: str | None = None
    reduced_motion: bool | None = None
    reduced_transparency: bool | None = None
    reduced_data: bool | None = None
    forced_colors: bool | None = None
    inverted_colors: bool | None = None
    monochrome_bits: int | None = None
    color_gamut: str | None = None
    pointer: str | None = None
    any_pointer: str | None = None
    hover: str | None = None
    any_hover: str | None = None
    display_mode: str | None = None
    dynamic_range: str | None = None
    video_dynamic_range: str | None = None
    scripting: str | None = None


@dataclass(frozen=True, slots=True)
class MimeTypeProfile:
    mime_type: str
    suffixes: str
    description: str


@dataclass(frozen=True, slots=True)
class PluginProfile:
    name: str
    filename: str
    description: str
    mime_types: tuple[MimeTypeProfile, ...] = ()


@dataclass(frozen=True, slots=True)
class PluginListProfile:
    plugins: tuple[PluginProfile, ...] | None = None


@dataclass(frozen=True, slots=True)
class GamepadProfile:
    id: str
    index: int
    connected: bool = True
    mapping: str = "standard"
    axes: tuple[float, ...] = ()
    buttons: tuple[float, ...] = ()


@dataclass(frozen=True, slots=True)
class UsbDeviceProfile:
    usb_version_major: int = 1
    usb_version_minor: int = 0
    usb_version_subminor: int = 0
    device_class: int = 0
    device_subclass: int = 0
    device_protocol: int = 0
    vendor_id: int = 0x045E
    product_id: int = 1
    device_version_major: int = 1
    device_version_minor: int = 0
    device_version_subminor: int = 0
    manufacturer_name: str | None = "Microsoft"
    product_name: str | None = "Edge Sandbox USB"
    serial_number: str | None = "EDGE0001"


@dataclass(frozen=True, slots=True)
class HidDeviceProfile:
    vendor_id: int
    product_id: int
    product_name: str


@dataclass(frozen=True, slots=True)
class SerialPortProfile:
    usb_vendor_id: int
    usb_product_id: int
    connected: bool = True


@dataclass(frozen=True, slots=True)
class BluetoothDeviceProfile:
    id: str
    name: str | None = None


@dataclass(frozen=True, slots=True)
class KeyboardLayoutEntryProfile:
    code: str
    value: str


@dataclass(frozen=True, slots=True)
class MidiPortProfile:
    id: str
    manufacturer: str
    name: str
    version: str = "1.0"
    state: str = "connected"
    connection: str = "closed"


@dataclass(frozen=True, slots=True)
class HardwareDevicesProfile:
    gamepads: tuple[GamepadProfile, ...] | None = None
    usb_devices: tuple[UsbDeviceProfile, ...] | None = None
    hid_devices: tuple[HidDeviceProfile, ...] | None = None
    serial_ports: tuple[SerialPortProfile, ...] | None = None
    bluetooth_available: bool | None = None
    bluetooth_devices: tuple[BluetoothDeviceProfile, ...] | None = None
    keyboard_layout: tuple[KeyboardLayoutEntryProfile, ...] | None = None
    device_posture: str | None = None
    midi_inputs: tuple[MidiPortProfile, ...] | None = None
    midi_outputs: tuple[MidiPortProfile, ...] | None = None
    midi_sysex_enabled: bool | None = None


@dataclass(frozen=True, slots=True)
class SensorsProfile:
    available: bool | None = None
    accelerometer: tuple[float, float, float] | None = None
    gravity: tuple[float, float, float] | None = None
    linear_acceleration: tuple[float, float, float] | None = None
    gyroscope: tuple[float, float, float] | None = None
    absolute_orientation_quaternion: (
        tuple[float, float, float, float] | None
    ) = None
    relative_orientation_quaternion: (
        tuple[float, float, float, float] | None
    ) = None


@dataclass(frozen=True, slots=True)
class TimingProfile:
    clock_epoch_ms: int | None = None
    clock_step_ms: int | None = None
    random_seed: int | None = None


@dataclass(frozen=True, slots=True)
class XrProfile:
    supported_session_modes: tuple[str, ...] | None = None


@dataclass(frozen=True, slots=True)
class MemoryProfile:
    """One coherent V8 heap snapshot exposed by performance/console.memory.

    Values are transmitted exactly as configured. Keep ``used <= total <= limit``
    for ordinary profiles; evidence-specific test profiles may intentionally
    supply another ordering. ``total`` and ``used`` are transient allocation
    statistics, not physical-RAM identifiers.
    """

    performance_js_heap_size_limit: int | None = None
    performance_total_js_heap_size: int | None = None
    performance_used_js_heap_size: int | None = None
    console_js_heap_size_limit: int | None = None
    console_total_js_heap_size: int | None = None
    console_used_js_heap_size: int | None = None


@dataclass(frozen=True, slots=True)
class PerformanceEntryProfile:
    """One ordered Performance Timeline entry supplied without JSON."""

    name: str
    entry_type: str
    start_time: float = 0.0
    duration: float = 0.0

    initiator_type: str = ""
    delivery_type: str = ""
    next_hop_protocol: str = ""
    render_blocking_status: str = "non-blocking"
    content_type: str = ""
    content_encoding: str = ""
    worker_start: float = 0.0
    worker_router_evaluation_start: float = 0.0
    worker_cache_lookup_start: float = 0.0
    worker_matched_source_type: str = ""
    worker_final_source_type: str = ""
    redirect_start: float = 0.0
    redirect_end: float = 0.0
    fetch_start: float = 0.0
    domain_lookup_start: float = 0.0
    domain_lookup_end: float = 0.0
    connect_start: float = 0.0
    secure_connection_start: float = 0.0
    connect_end: float = 0.0
    request_start: float = 0.0
    response_start: float = 0.0
    first_interim_response_start: float = 0.0
    final_response_headers_start: float = 0.0
    response_end: float = 0.0
    transfer_size: int | None = None
    encoded_body_size: int | None = None
    decoded_body_size: int | None = None
    response_status: int | None = None

    unload_event_start: float = 0.0
    unload_event_end: float = 0.0
    dom_interactive: float = 0.0
    dom_content_loaded_event_start: float = 0.0
    dom_content_loaded_event_end: float = 0.0
    dom_complete: float = 0.0
    load_event_start: float = 0.0
    load_event_end: float = 0.0
    navigation_type: str = "navigate"
    redirect_count: int = 0
    critical_ch_restart: float = 0.0
    activation_start: float = 0.0

    paint_time: float = 0.0
    presentation_time: float = 0.0


@dataclass(frozen=True, slots=True)
class PerformanceProfile:
    """Performance Timeline inputs and evaluated-script transfer encoding.

    ``evaluated_script_content_encoding`` controls the automatic resource
    entry created by ``evaluate(..., source_url=...)``. Rust compresses the
    actual UTF-8 source to derive encodedBodySize; an empty string represents
    an uncompressed response.
    """

    entries: tuple[PerformanceEntryProfile, ...] | None = None
    evaluated_script_content_encoding: str = "zstd"


@dataclass(frozen=True, slots=True)
class EdgeProfile:
    """Partial typed override applied to the fixed Chrome 150 default profile."""

    id: str | None = None
    locale: LocaleProfile | None = None
    navigator: NavigatorProfile | None = None
    screen: ScreenProfile | None = None
    window: WindowProfile | None = None
    canvas: CanvasProfile | None = None
    webgl: WebGlProfile | None = None
    webgpu: WebGpuProfile | None = None
    audio: WebAudioProfile | None = None
    storage: StorageProfile | None = None
    speech: SpeechProfile | None = None
    fonts: FontProfile | None = None
    css: CssProfile | None = None
    document: DocumentProfile | None = None
    media: MediaProfile | None = None
    permissions: PermissionsProfile | None = None
    battery: BatteryProfile | None = None
    geolocation: GeolocationProfile | None = None
    media_preferences: MediaPreferencesProfile | None = None
    plugins: PluginListProfile | None = None
    hardware_devices: HardwareDevicesProfile | None = None
    sensors: SensorsProfile | None = None
    timing: TimingProfile | None = None
    xr: XrProfile | None = None
    memory: MemoryProfile | None = None
    performance: PerformanceProfile | None = None
