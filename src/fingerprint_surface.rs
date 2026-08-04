#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct LocaleFingerprint {
    pub locale: String,
    pub time_zone: String,
    pub time_zone_offset_minutes: i32,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ScreenFingerprint {
    pub width: i32,
    pub height: i32,
    pub avail_width: i32,
    pub avail_height: i32,
    pub avail_left: i32,
    pub avail_top: i32,
    pub color_depth: i32,
    pub pixel_depth: i32,
    pub viewport_width: f64,
    pub viewport_height: f64,
    pub outer_width: f64,
    pub outer_height: f64,
    pub screen_x: f64,
    pub screen_y: f64,
    pub device_pixel_ratio: f64,
    pub orientation_type: String,
    pub orientation_angle: u16,
    pub visual_viewport_offset_left: f64,
    pub visual_viewport_offset_top: f64,
    pub visual_viewport_page_left: f64,
    pub visual_viewport_page_top: f64,
    pub visual_viewport_scale: f64,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CanvasFingerprint {
    pub data_url_salt: String,
    pub text_width_scale: f64,
    pub actual_bounding_box_left: f64,
    pub actual_bounding_box_right_scale: f64,
    pub font_bounding_box_ascent: f64,
    pub font_bounding_box_descent: f64,
    pub actual_bounding_box_ascent: f64,
    pub actual_bounding_box_descent: f64,
    pub hanging_baseline: f64,
    pub alphabetic_baseline: f64,
    pub ideographic_baseline: f64,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WebGlFingerprint {
    pub vendor: String,
    pub renderer: String,
    pub unmasked_vendor: String,
    pub unmasked_renderer: String,
    pub webgl1_version: String,
    pub webgl1_shading_language_version: String,
    pub webgl2_version: String,
    pub webgl2_shading_language_version: String,
    pub webgl1_extensions: Vec<String>,
    pub webgl2_extensions: Vec<String>,
    pub compressed_texture_formats: Vec<u32>,
    pub max_texture_size: i32,
    pub max_cube_map_texture_size: i32,
    pub max_renderbuffer_size: i32,
    pub max_viewport_width: i32,
    pub max_viewport_height: i32,
    pub max_vertex_attribs: i32,
    pub max_vertex_uniform_vectors: i32,
    pub max_varying_vectors: i32,
    pub max_fragment_uniform_vectors: i32,
    pub max_vertex_texture_image_units: i32,
    pub max_texture_image_units: i32,
    pub max_combined_texture_image_units: i32,
    pub subpixel_bits: i32,
    pub webgl2_max_3d_texture_size: i32,
    pub webgl2_max_array_texture_layers: i32,
    pub webgl2_max_draw_buffers: i32,
    pub webgl2_max_color_attachments: i32,
    pub webgl2_max_samples: i32,
    pub webgl2_max_vertex_uniform_components: i32,
    pub webgl2_max_fragment_uniform_components: i32,
    pub webgl2_max_varying_components: i32,
    pub webgl2_max_vertex_output_components: i32,
    pub webgl2_max_fragment_input_components: i32,
    pub webgl2_max_vertex_uniform_blocks: i32,
    pub webgl2_max_fragment_uniform_blocks: i32,
    pub webgl2_max_combined_uniform_blocks: i32,
    pub webgl2_max_uniform_buffer_bindings: i32,
    pub webgl2_max_uniform_block_size: i32,
    pub webgl2_max_combined_vertex_uniform_components: i32,
    pub webgl2_max_combined_fragment_uniform_components: i32,
    pub webgl2_max_transform_feedback_separate_attribs: i32,
    pub webgl2_max_transform_feedback_interleaved_components: i32,
    pub webgl2_max_transform_feedback_separate_components: i32,
    pub webgl2_max_program_texel_offset: i32,
    pub webgl2_max_elements_vertices: i32,
    pub webgl2_max_elements_indices: i32,
    pub webgl2_max_element_index: u32,
    pub webgl2_max_texture_lod_bias: f64,
    pub max_anisotropy: f64,
    pub aliased_point_size_min: f64,
    pub aliased_point_size_max: f64,
    pub aliased_line_width_min: f64,
    pub aliased_line_width_max: f64,
    pub shader_precision_range_min: i32,
    pub shader_precision_range_max: i32,
    pub shader_precision_bits: i32,
    pub context_alpha: bool,
    pub context_antialias: bool,
    pub context_depth: bool,
    pub context_desynchronized: bool,
    pub context_fail_if_major_performance_caveat: bool,
    pub context_premultiplied_alpha: bool,
    pub context_preserve_drawing_buffer: bool,
    pub context_stencil: bool,
    pub context_xr_compatible: bool,
    pub context_power_preference: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct WebGpuFingerprint {
    pub vendor: String,
    pub architecture: String,
    pub device: String,
    pub description: String,
    pub developer_features: bool,
    pub subgroup_min_size: u32,
    pub subgroup_max_size: u32,
    pub is_fallback_adapter: bool,
    pub features: Vec<String>,
    pub max_texture_dimension_1d: u32,
    pub max_texture_dimension_2d: u32,
    pub max_texture_dimension_3d: u32,
    pub max_texture_array_layers: u32,
    pub max_bind_groups: u32,
    pub max_bind_groups_plus_vertex_buffers: u32,
    pub max_bindings_per_bind_group: u32,
    pub max_dynamic_uniform_buffers_per_pipeline_layout: u32,
    pub max_dynamic_storage_buffers_per_pipeline_layout: u32,
    pub max_sampled_textures_per_shader_stage: u32,
    pub max_samplers_per_shader_stage: u32,
    pub max_storage_buffers_per_shader_stage: u32,
    pub max_storage_textures_per_shader_stage: u32,
    pub max_uniform_buffers_per_shader_stage: u32,
    pub max_uniform_buffer_binding_size: u64,
    pub max_storage_buffer_binding_size: u64,
    pub min_uniform_buffer_offset_alignment: u32,
    pub min_storage_buffer_offset_alignment: u32,
    pub max_vertex_buffers: u32,
    pub max_buffer_size: u64,
    pub max_vertex_attributes: u32,
    pub max_vertex_buffer_array_stride: u32,
    pub max_inter_stage_shader_variables: u32,
    pub max_color_attachments: u32,
    pub max_color_attachment_bytes_per_sample: u32,
    pub max_compute_workgroup_storage_size: u32,
    pub max_compute_invocations_per_workgroup: u32,
    pub max_compute_workgroup_size_x: u32,
    pub max_compute_workgroup_size_y: u32,
    pub max_compute_workgroup_size_z: u32,
    pub max_compute_workgroups_per_dimension: u32,
    pub max_immediate_size: u32,
    pub max_storage_buffers_in_fragment_stage: u32,
    pub max_storage_textures_in_fragment_stage: u32,
    pub max_storage_buffers_in_vertex_stage: u32,
    pub max_storage_textures_in_vertex_stage: u32,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AudioFingerprint {
    pub sample_rate: f64,
    pub max_channel_count: u32,
    pub base_latency: f64,
    pub output_latency: f64,
    pub noise_seed: u64,
    pub channel_noise_amplitude: f32,
    pub frequency_noise_amplitude: f32,
    pub time_domain_noise_amplitude: f32,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RenderingFingerprint {
    pub canvas: CanvasFingerprint,
    pub webgl: WebGlFingerprint,
    pub webgpu: WebGpuFingerprint,
    pub audio: AudioFingerprint,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct StorageFingerprint {
    pub quota_bytes: u64,
    pub usage_bytes: u64,
    pub persisted: bool,
}

impl Default for LocaleFingerprint {
    fn default() -> Self {
        Self {
            locale: "zh-CN".to_owned(),
            time_zone: "Asia/Shanghai".to_owned(),
            time_zone_offset_minutes: -480,
        }
    }
}

impl Default for ScreenFingerprint {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            avail_width: 1280,
            avail_height: 720,
            avail_left: 0,
            avail_top: 0,
            color_depth: 24,
            pixel_depth: 24,
            viewport_width: 1280.0,
            viewport_height: 720.0,
            outer_width: 1280.0,
            outer_height: 720.0,
            screen_x: 10.0,
            screen_y: 10.0,
            device_pixel_ratio: 1.0,
            orientation_type: "landscape-primary".to_owned(),
            orientation_angle: 0,
            visual_viewport_offset_left: 0.0,
            visual_viewport_offset_top: 0.0,
            visual_viewport_page_left: 0.0,
            visual_viewport_page_top: 0.0,
            visual_viewport_scale: 1.0,
        }
    }
}

impl Default for CanvasFingerprint {
    fn default() -> Self {
        Self {
            data_url_salt: String::new(),
            text_width_scale: 1.0,
            actual_bounding_box_left: 0.0,
            actual_bounding_box_right_scale: 1.0,
            font_bounding_box_ascent: 12.0,
            font_bounding_box_descent: 3.0,
            actual_bounding_box_ascent: 8.0,
            actual_bounding_box_descent: 1.0,
            hanging_baseline: 9.600_000_381_469_727,
            alphabetic_baseline: 0.0,
            ideographic_baseline: -1.199_996_948_242_187_5,
        }
    }
}

impl Default for WebGlFingerprint {
    fn default() -> Self {
        Self {
            vendor: "WebKit".to_owned(),
            renderer: "WebKit WebGL".to_owned(),
            unmasked_vendor: "Google Inc. (Microsoft)".to_owned(),
            unmasked_renderer: concat!(
                "ANGLE (Microsoft, Microsoft Basic Render Driver (0x0000008C) ",
                "Direct3D11 vs_5_0 ps_5_0, D3D11)"
            )
            .to_owned(),
            webgl1_version: "WebGL 1.0 (OpenGL ES 2.0 Chromium)".to_owned(),
            webgl1_shading_language_version: "WebGL GLSL ES 1.0 (OpenGL ES GLSL ES 1.0 Chromium)"
                .to_owned(),
            webgl2_version: "WebGL 2.0 (OpenGL ES 3.0 Chromium)".to_owned(),
            webgl2_shading_language_version: "WebGL GLSL ES 3.00 (OpenGL ES GLSL ES 3.0 Chromium)"
                .to_owned(),
            webgl1_extensions: vec![
                "ANGLE_instanced_arrays".to_owned(),
                "EXT_blend_minmax".to_owned(),
                "EXT_color_buffer_half_float".to_owned(),
                "EXT_float_blend".to_owned(),
                "EXT_frag_depth".to_owned(),
                "EXT_shader_texture_lod".to_owned(),
                "EXT_texture_compression_bptc".to_owned(),
                "EXT_texture_filter_anisotropic".to_owned(),
                "OES_element_index_uint".to_owned(),
                "OES_fbo_render_mipmap".to_owned(),
                "OES_standard_derivatives".to_owned(),
                "OES_texture_float".to_owned(),
                "OES_texture_float_linear".to_owned(),
                "OES_texture_half_float".to_owned(),
                "OES_texture_half_float_linear".to_owned(),
                "OES_vertex_array_object".to_owned(),
                "WEBGL_color_buffer_float".to_owned(),
                "WEBGL_compressed_texture_s3tc".to_owned(),
                "WEBGL_compressed_texture_s3tc_srgb".to_owned(),
                "WEBGL_debug_renderer_info".to_owned(),
                "WEBGL_debug_shaders".to_owned(),
                "WEBGL_depth_texture".to_owned(),
                "WEBGL_draw_buffers".to_owned(),
                "WEBGL_lose_context".to_owned(),
                "WEBGL_multi_draw".to_owned(),
            ],
            webgl2_extensions: vec![
                "EXT_color_buffer_float".to_owned(),
                "EXT_color_buffer_half_float".to_owned(),
                "EXT_float_blend".to_owned(),
                "EXT_texture_compression_bptc".to_owned(),
                "EXT_texture_filter_anisotropic".to_owned(),
                "OES_draw_buffers_indexed".to_owned(),
                "OES_texture_float_linear".to_owned(),
                "WEBGL_compressed_texture_s3tc".to_owned(),
                "WEBGL_compressed_texture_s3tc_srgb".to_owned(),
                "WEBGL_debug_renderer_info".to_owned(),
                "WEBGL_debug_shaders".to_owned(),
                "WEBGL_lose_context".to_owned(),
                "WEBGL_multi_draw".to_owned(),
            ],
            compressed_texture_formats: Vec::new(),
            max_texture_size: 16_384,
            max_cube_map_texture_size: 16_384,
            max_renderbuffer_size: 16_384,
            max_viewport_width: 32_767,
            max_viewport_height: 32_767,
            max_vertex_attribs: 16,
            max_vertex_uniform_vectors: 4_096,
            max_varying_vectors: 30,
            max_fragment_uniform_vectors: 1_024,
            max_vertex_texture_image_units: 16,
            max_texture_image_units: 16,
            max_combined_texture_image_units: 32,
            subpixel_bits: 4,
            webgl2_max_3d_texture_size: 2_048,
            webgl2_max_array_texture_layers: 2_048,
            webgl2_max_draw_buffers: 8,
            webgl2_max_color_attachments: 8,
            webgl2_max_samples: 16,
            webgl2_max_vertex_uniform_components: 16_384,
            webgl2_max_fragment_uniform_components: 4_096,
            webgl2_max_varying_components: 120,
            webgl2_max_vertex_output_components: 120,
            webgl2_max_fragment_input_components: 120,
            webgl2_max_vertex_uniform_blocks: 12,
            webgl2_max_fragment_uniform_blocks: 12,
            webgl2_max_combined_uniform_blocks: 24,
            webgl2_max_uniform_buffer_bindings: 24,
            webgl2_max_uniform_block_size: 65_536,
            webgl2_max_combined_vertex_uniform_components: 212_992,
            webgl2_max_combined_fragment_uniform_components: 200_704,
            webgl2_max_transform_feedback_separate_attribs: 4,
            webgl2_max_transform_feedback_interleaved_components: 120,
            webgl2_max_transform_feedback_separate_components: 4,
            webgl2_max_program_texel_offset: 7,
            webgl2_max_elements_vertices: i32::MAX,
            webgl2_max_elements_indices: i32::MAX,
            webgl2_max_element_index: u32::MAX - 1,
            webgl2_max_texture_lod_bias: 2.0,
            max_anisotropy: 16.0,
            aliased_point_size_min: 1.0,
            aliased_point_size_max: 1_024.0,
            aliased_line_width_min: 1.0,
            aliased_line_width_max: 1.0,
            shader_precision_range_min: 127,
            shader_precision_range_max: 127,
            shader_precision_bits: 23,
            context_alpha: true,
            context_antialias: true,
            context_depth: true,
            context_desynchronized: false,
            context_fail_if_major_performance_caveat: false,
            context_premultiplied_alpha: true,
            context_preserve_drawing_buffer: false,
            context_stencil: false,
            context_xr_compatible: false,
            context_power_preference: "default".to_owned(),
        }
    }
}

impl Default for WebGpuFingerprint {
    fn default() -> Self {
        Self {
            vendor: "Microsoft".to_owned(),
            architecture: "D3D12".to_owned(),
            device: "Edge WebGPU Adapter".to_owned(),
            description: "Microsoft Edge WebGPU software adapter".to_owned(),
            developer_features: true,
            subgroup_min_size: 4,
            subgroup_max_size: 128,
            is_fallback_adapter: true,
            features: vec!["bgra8unorm-storage".to_owned()],
            max_texture_dimension_1d: 8_192,
            max_texture_dimension_2d: 8_192,
            max_texture_dimension_3d: 2_048,
            max_texture_array_layers: 256,
            max_bind_groups: 4,
            max_bind_groups_plus_vertex_buffers: 24,
            max_bindings_per_bind_group: 1_000,
            max_dynamic_uniform_buffers_per_pipeline_layout: 8,
            max_dynamic_storage_buffers_per_pipeline_layout: 4,
            max_sampled_textures_per_shader_stage: 16,
            max_samplers_per_shader_stage: 16,
            max_storage_buffers_per_shader_stage: 8,
            max_storage_textures_per_shader_stage: 4,
            max_uniform_buffers_per_shader_stage: 12,
            max_uniform_buffer_binding_size: 65_536,
            max_storage_buffer_binding_size: 134_217_728,
            min_uniform_buffer_offset_alignment: 256,
            min_storage_buffer_offset_alignment: 256,
            max_vertex_buffers: 8,
            max_buffer_size: 268_435_456,
            max_vertex_attributes: 16,
            max_vertex_buffer_array_stride: 2_048,
            max_inter_stage_shader_variables: 16,
            max_color_attachments: 8,
            max_color_attachment_bytes_per_sample: 32,
            max_compute_workgroup_storage_size: 16_384,
            max_compute_invocations_per_workgroup: 256,
            max_compute_workgroup_size_x: 256,
            max_compute_workgroup_size_y: 256,
            max_compute_workgroup_size_z: 64,
            max_compute_workgroups_per_dimension: 65_535,
            max_immediate_size: 0,
            max_storage_buffers_in_fragment_stage: 8,
            max_storage_textures_in_fragment_stage: 4,
            max_storage_buffers_in_vertex_stage: 8,
            max_storage_textures_in_vertex_stage: 4,
        }
    }
}

impl Default for AudioFingerprint {
    fn default() -> Self {
        Self {
            sample_rate: 44_100.0,
            max_channel_count: 2,
            base_latency: 0.01,
            output_latency: 0.0,
            noise_seed: 0x4544_4745,
            channel_noise_amplitude: 0.0,
            frequency_noise_amplitude: 0.0,
            time_domain_noise_amplitude: 0.0,
        }
    }
}

impl Default for StorageFingerprint {
    fn default() -> Self {
        Self {
            quota_bytes: 1_073_741_824,
            usage_bytes: 0,
            persisted: false,
        }
    }
}

impl LocaleFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.locale.is_empty()
            || self.locale.len() > 64
            || self.time_zone.is_empty()
            || self.time_zone.len() > 128
            || !(-1_440..=1_440).contains(&self.time_zone_offset_minutes)
        {
            return Err("locale fingerprint is outside supported bounds".to_owned());
        }
        Ok(())
    }
}

impl ScreenFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.width <= 0
            || self.height <= 0
            || self.avail_width <= 0
            || self.avail_height <= 0
            || self.avail_width > self.width
            || self.avail_height > self.height
            || !matches!(self.color_depth, 1..=64)
            || !matches!(self.pixel_depth, 1..=64)
            || !self.viewport_width.is_finite()
            || self.viewport_width < 0.0
            || !self.viewport_height.is_finite()
            || self.viewport_height < 0.0
            || !self.outer_width.is_finite()
            || self.outer_width < 0.0
            || !self.outer_height.is_finite()
            || self.outer_height < 0.0
            || !self.device_pixel_ratio.is_finite()
            || self.device_pixel_ratio <= 0.0
            || self.device_pixel_ratio > 16.0
            || !matches!(
                self.orientation_type.as_str(),
                "portrait-primary"
                    | "portrait-secondary"
                    | "landscape-primary"
                    | "landscape-secondary"
            )
            || !matches!(self.orientation_angle, 0 | 90 | 180 | 270)
            || [
                self.visual_viewport_offset_left,
                self.visual_viewport_offset_top,
                self.visual_viewport_page_left,
                self.visual_viewport_page_top,
            ]
            .into_iter()
            .any(|value| !value.is_finite())
            || !self.visual_viewport_scale.is_finite()
            || !(0.01..=100.0).contains(&self.visual_viewport_scale)
        {
            return Err("screen fingerprint is outside supported bounds".to_owned());
        }
        Ok(())
    }
}

impl RenderingFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.canvas.data_url_salt.len() > 256
            || !self.canvas.text_width_scale.is_finite()
            || !(0.25..=4.0).contains(&self.canvas.text_width_scale)
            || [
                self.canvas.actual_bounding_box_left,
                self.canvas.actual_bounding_box_right_scale,
                self.canvas.font_bounding_box_ascent,
                self.canvas.font_bounding_box_descent,
                self.canvas.actual_bounding_box_ascent,
                self.canvas.actual_bounding_box_descent,
                self.canvas.hanging_baseline,
                self.canvas.alphabetic_baseline,
                self.canvas.ideographic_baseline,
            ]
            .into_iter()
            .any(|value| !value.is_finite() || !(-100_000.0..=100_000.0).contains(&value))
            || self.webgl.vendor.is_empty()
            || self.webgl.renderer.is_empty()
            || self.webgl.unmasked_vendor.is_empty()
            || self.webgl.unmasked_renderer.is_empty()
            || self.webgl.webgl1_version.is_empty()
            || self.webgl.webgl1_shading_language_version.is_empty()
            || self.webgl.webgl2_version.is_empty()
            || self.webgl.webgl2_shading_language_version.is_empty()
            || self.webgl.max_texture_size <= 0
            || self.webgl.max_cube_map_texture_size <= 0
            || self.webgl.max_renderbuffer_size <= 0
            || self.webgl.max_viewport_width <= 0
            || self.webgl.max_viewport_height <= 0
            || self.webgl.max_vertex_attribs <= 0
            || self.webgl.max_vertex_uniform_vectors <= 0
            || self.webgl.max_varying_vectors <= 0
            || self.webgl.max_fragment_uniform_vectors <= 0
            || self.webgl.max_vertex_texture_image_units < 0
            || self.webgl.max_texture_image_units <= 0
            || self.webgl.max_combined_texture_image_units <= 0
            || self.webgl.subpixel_bits < 0
            || self.webgl.webgl2_max_3d_texture_size <= 0
            || self.webgl.webgl2_max_array_texture_layers <= 0
            || self.webgl.webgl2_max_draw_buffers <= 0
            || self.webgl.webgl2_max_color_attachments <= 0
            || self.webgl.webgl2_max_samples < 0
            || self.webgl.webgl2_max_vertex_uniform_components <= 0
            || self.webgl.webgl2_max_fragment_uniform_components <= 0
            || self.webgl.webgl2_max_varying_components <= 0
            || self.webgl.webgl2_max_vertex_output_components <= 0
            || self.webgl.webgl2_max_fragment_input_components <= 0
            || self.webgl.webgl2_max_vertex_uniform_blocks <= 0
            || self.webgl.webgl2_max_fragment_uniform_blocks <= 0
            || self.webgl.webgl2_max_combined_uniform_blocks <= 0
            || self.webgl.webgl2_max_uniform_buffer_bindings <= 0
            || self.webgl.webgl2_max_uniform_block_size <= 0
            || self.webgl.webgl2_max_combined_vertex_uniform_components <= 0
            || self.webgl.webgl2_max_combined_fragment_uniform_components <= 0
            || self.webgl.webgl2_max_transform_feedback_separate_attribs <= 0
            || self
                .webgl
                .webgl2_max_transform_feedback_interleaved_components
                <= 0
            || self.webgl.webgl2_max_transform_feedback_separate_components <= 0
            || self.webgl.webgl2_max_program_texel_offset < 0
            || self.webgl.webgl2_max_elements_vertices <= 0
            || self.webgl.webgl2_max_elements_indices <= 0
            || self.webgl.webgl2_max_element_index == 0
            || !self.webgl.webgl2_max_texture_lod_bias.is_finite()
            || self.webgl.webgl2_max_texture_lod_bias < 0.0
            || !self.webgl.max_anisotropy.is_finite()
            || self.webgl.max_anisotropy < 1.0
            || !self.webgl.aliased_point_size_min.is_finite()
            || !self.webgl.aliased_point_size_max.is_finite()
            || self.webgl.aliased_point_size_min > self.webgl.aliased_point_size_max
            || !self.webgl.aliased_line_width_min.is_finite()
            || !self.webgl.aliased_line_width_max.is_finite()
            || self.webgl.aliased_line_width_min > self.webgl.aliased_line_width_max
            || self.webgl.shader_precision_range_min < 0
            || self.webgl.shader_precision_range_max < self.webgl.shader_precision_range_min
            || self.webgl.shader_precision_bits < 0
            || !matches!(
                self.webgl.context_power_preference.as_str(),
                "default" | "high-performance" | "low-power"
            )
            || self.webgpu.vendor.is_empty()
            || self.webgpu.device.is_empty()
            || self.webgpu.subgroup_min_size == 0
            || self.webgpu.subgroup_max_size < self.webgpu.subgroup_min_size
            || self.webgpu.max_texture_dimension_1d == 0
            || self.webgpu.max_texture_dimension_2d == 0
            || self.webgpu.max_texture_dimension_3d == 0
            || self.webgpu.max_texture_array_layers == 0
            || self.webgpu.max_bind_groups == 0
            || self.webgpu.max_bind_groups_plus_vertex_buffers == 0
            || self.webgpu.max_bindings_per_bind_group == 0
            || self.webgpu.max_uniform_buffer_binding_size == 0
            || self.webgpu.max_storage_buffer_binding_size == 0
            || self.webgpu.min_uniform_buffer_offset_alignment == 0
            || self.webgpu.min_storage_buffer_offset_alignment == 0
            || self.webgpu.max_vertex_buffers == 0
            || self.webgpu.max_buffer_size == 0
            || self.webgpu.max_vertex_attributes == 0
            || self.webgpu.max_vertex_buffer_array_stride == 0
            || self.webgpu.max_compute_workgroup_storage_size == 0
            || self.webgpu.max_compute_invocations_per_workgroup == 0
            || self.webgpu.max_compute_workgroup_size_x == 0
            || self.webgpu.max_compute_workgroup_size_y == 0
            || self.webgpu.max_compute_workgroup_size_z == 0
            || self.webgpu.max_compute_workgroups_per_dimension == 0
            || !self.audio.sample_rate.is_finite()
            || !(3_000.0..=384_000.0).contains(&self.audio.sample_rate)
            || self.audio.max_channel_count == 0
            || self.audio.max_channel_count > 64
            || !self.audio.base_latency.is_finite()
            || !(0.0..=60.0).contains(&self.audio.base_latency)
            || !self.audio.output_latency.is_finite()
            || !(0.0..=60.0).contains(&self.audio.output_latency)
            || !self.audio.channel_noise_amplitude.is_finite()
            || !(0.0..=1.0).contains(&self.audio.channel_noise_amplitude)
            || !self.audio.frequency_noise_amplitude.is_finite()
            || !(0.0..=1.0).contains(&self.audio.frequency_noise_amplitude)
            || !self.audio.time_domain_noise_amplitude.is_finite()
            || !(0.0..=1.0).contains(&self.audio.time_domain_noise_amplitude)
        {
            return Err("rendering fingerprint is outside supported bounds".to_owned());
        }
        if self.webgl.webgl1_extensions.len() > 256
            || self.webgl.webgl2_extensions.len() > 256
            || self.webgl.compressed_texture_formats.len() > 256
            || self.webgpu.features.len() > 256
        {
            return Err("rendering fingerprint contains too many capabilities".to_owned());
        }
        Ok(())
    }
}

impl StorageFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.quota_bytes == 0 || self.usage_bytes > self.quota_bytes {
            return Err("storage usage must not exceed a non-zero quota".to_owned());
        }
        Ok(())
    }
}
