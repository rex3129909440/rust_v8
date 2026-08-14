"""Android GPU-family WebGL/WebGPU capability profiles.

Pixel 4 is one captured device, not the Android default.  The remaining rows
are separate real-world capability tiers keyed by the concrete GPU family.
The numeric tiers are values observed in Web3D Survey's Android WebGL reports;
renderer/device associations come from vendor specifications in
``android_device_profile_catalog``.

Sources:
- build/chromium-android-version-surfaces/* (Pixel 4 Chromium 140-151 HTTPS)
- https://web3dsurvey.com/webgl/parameters/MAX_TEXTURE_SIZE
- https://web3dsurvey.com/webgl/parameters/MAX_RENDERBUFFER_SIZE
- https://developer.android.com/develop/ui/views/graphics/opengl/about-opengl
- https://chromium.googlesource.com/chromium/src/+/main/gpu/config/gpu_blocklist.json
"""

from __future__ import annotations


_WEBGL1_COMMON = (
    "ANGLE_instanced_arrays",
    "EXT_blend_minmax",
    "EXT_color_buffer_half_float",
    "EXT_float_blend",
    "EXT_frag_depth",
    "EXT_shader_texture_lod",
    "EXT_texture_filter_anisotropic",
    "OES_element_index_uint",
    "OES_fbo_render_mipmap",
    "OES_standard_derivatives",
    "OES_texture_float",
    "OES_texture_float_linear",
    "OES_texture_half_float",
    "OES_texture_half_float_linear",
    "OES_vertex_array_object",
    "WEBGL_color_buffer_float",
    "WEBGL_debug_renderer_info",
    "WEBGL_debug_shaders",
    "WEBGL_depth_texture",
    "WEBGL_draw_buffers",
    "WEBGL_lose_context",
)

_WEBGL2_COMMON = (
    "EXT_color_buffer_float",
    "EXT_color_buffer_half_float",
    "EXT_float_blend",
    "EXT_texture_filter_anisotropic",
    "OES_draw_buffers_indexed",
    "OES_texture_float_linear",
    "WEBGL_compressed_texture_etc",
    "WEBGL_debug_renderer_info",
    "WEBGL_debug_shaders",
    "WEBGL_lose_context",
)

_PIXEL4_WEBGL2_EXTENSIONS = (
    "EXT_clip_control",
    "EXT_color_buffer_float",
    "EXT_color_buffer_half_float",
    "EXT_float_blend",
    "EXT_texture_filter_anisotropic",
    "EXT_texture_norm16",
    "NV_shader_noperspective_interpolation",
    "OES_draw_buffers_indexed",
    "OES_sample_variables",
    "OES_shader_multisample_interpolation",
    "OES_texture_float_linear",
    "WEBGL_blend_func_extended",
    "WEBGL_clip_cull_distance",
    "WEBGL_compressed_texture_astc",
    "WEBGL_compressed_texture_etc",
    "WEBGL_compressed_texture_etc1",
    "WEBGL_debug_renderer_info",
    "WEBGL_debug_shaders",
    "WEBGL_lose_context",
    "WEBGL_multi_draw",
    "WEBGL_stencil_texturing",
)

_MODERN_WEBGL2_EXTENSIONS = tuple(dict.fromkeys((
    *_PIXEL4_WEBGL2_EXTENSIONS,
    "EXT_depth_clamp",
)))

_ETC1_FORMATS = (0x8D64,)
_ETC2_FORMATS = tuple(range(0x9274, 0x9279))
_ASTC_FORMATS = (
    *tuple(range(0x93B0, 0x93BE)),
    *tuple(range(0x93D0, 0x93DE)),
)
_BPTC_FORMATS = tuple(range(0x8E8C, 0x8E90))
_RGTC_FORMATS = tuple(range(0x8DBB, 0x8DBF))


def _profile(
    *,
    texture: int,
    cube: int,
    renderbuffer: int,
    viewport: int,
    vertex_uniforms: int,
    fragment_uniforms: int,
    varying_vectors: int,
    vertex_texture_units: int,
    combined_texture_units: int,
    array_layers: int,
    samples: int,
    uniform_block_size: int,
    anisotropy: float,
    webgl1_extensions: tuple[str, ...],
    webgl2_extensions: tuple[str, ...],
    compressed_formats: tuple[int, ...],
) -> dict[str, object]:
    combined_vertex_uniforms = 12 * uniform_block_size + vertex_uniforms * 4
    combined_fragment_uniforms = 12 * uniform_block_size + fragment_uniforms * 4
    return {
        "webgl1_extensions": webgl1_extensions,
        "webgl2_extensions": webgl2_extensions,
        "compressed_texture_formats": compressed_formats,
        "max_texture_size": texture,
        "max_cube_map_texture_size": cube,
        "max_renderbuffer_size": renderbuffer,
        "max_viewport_width": viewport,
        "max_viewport_height": viewport,
        "max_vertex_attribs": 16,
        "max_vertex_uniform_vectors": vertex_uniforms,
        "max_varying_vectors": varying_vectors,
        "max_fragment_uniform_vectors": fragment_uniforms,
        "max_vertex_texture_image_units": vertex_texture_units,
        "max_texture_image_units": 16,
        "max_combined_texture_image_units": combined_texture_units,
        "subpixel_bits": 4,
        "webgl2_max_3d_texture_size": 2_048,
        "webgl2_max_array_texture_layers": array_layers,
        "webgl2_max_draw_buffers": 8,
        "webgl2_max_color_attachments": 8,
        "webgl2_max_samples": samples,
        "webgl2_max_vertex_uniform_components": vertex_uniforms * 4,
        "webgl2_max_fragment_uniform_components": fragment_uniforms * 4,
        "webgl2_max_varying_components": varying_vectors * 4,
        "webgl2_max_vertex_output_components": 64,
        "webgl2_max_fragment_input_components": 60,
        "webgl2_max_vertex_uniform_blocks": 12,
        "webgl2_max_fragment_uniform_blocks": 12,
        "webgl2_max_combined_uniform_blocks": 24,
        "webgl2_max_uniform_buffer_bindings": 24,
        "webgl2_max_uniform_block_size": uniform_block_size,
        "webgl2_max_combined_vertex_uniform_components": combined_vertex_uniforms,
        "webgl2_max_combined_fragment_uniform_components": combined_fragment_uniforms,
        "webgl2_max_transform_feedback_separate_attribs": 4,
        "webgl2_max_transform_feedback_interleaved_components": 64,
        "webgl2_max_transform_feedback_separate_components": 4,
        "webgl2_max_program_texel_offset": 7,
        "webgl2_max_elements_vertices": 1_048_576,
        "webgl2_max_elements_indices": 1_048_576,
        "webgl2_max_element_index": 4_294_967_294,
        "webgl2_max_texture_lod_bias": 2.0,
        "max_anisotropy": anisotropy,
        "aliased_point_size_min": 1.0,
        "aliased_point_size_max": 1024.0,
        "aliased_line_width_min": 1.0,
        "aliased_line_width_max": 1.0,
    }


_ASTC_WEBGL1 = tuple(dict.fromkeys((
    *_WEBGL1_COMMON,
    "WEBGL_compressed_texture_astc",
    "WEBGL_compressed_texture_etc",
    "WEBGL_compressed_texture_etc1",
    "WEBGL_multi_draw",
)))

_ASTC_WEBGL2 = tuple(dict.fromkeys((
    *_MODERN_WEBGL2_EXTENSIONS,
    "WEBGL_compressed_texture_astc",
    "WEBGL_compressed_texture_etc1",
)))


ANDROID_GRAPHICS_CAPABILITY_PROFILES: dict[str, dict[str, object]] = {
    "pixel4-adreno-640-evidence": _profile(
        texture=4_096,
        cube=4_096,
        renderbuffer=16_384,
        viewport=16_384,
        vertex_uniforms=1_024,
        fragment_uniforms=1_024,
        varying_vectors=31,
        vertex_texture_units=16,
        combined_texture_units=32,
        array_layers=256,
        samples=4,
        uniform_block_size=16_384,
        anisotropy=16.0,
        webgl1_extensions=_ASTC_WEBGL1,
        webgl2_extensions=_PIXEL4_WEBGL2_EXTENSIONS,
        compressed_formats=(*_ETC1_FORMATS, *_ETC2_FORMATS, *_ASTC_FORMATS),
    ),
    "adreno-7xx-flagship": _profile(
        texture=16_384,
        cube=16_384,
        renderbuffer=16_384,
        viewport=16_384,
        vertex_uniforms=1_024,
        fragment_uniforms=1_024,
        varying_vectors=31,
        vertex_texture_units=32,
        combined_texture_units=64,
        array_layers=2_048,
        samples=8,
        uniform_block_size=65_536,
        anisotropy=16.0,
        webgl1_extensions=_ASTC_WEBGL1,
        webgl2_extensions=_ASTC_WEBGL2,
        compressed_formats=(*_ETC1_FORMATS, *_ETC2_FORMATS, *_ASTC_FORMATS),
    ),
    "adreno-6xx-mainstream": _profile(
        texture=8_192,
        cube=8_192,
        renderbuffer=16_384,
        viewport=16_384,
        vertex_uniforms=1_024,
        fragment_uniforms=1_024,
        varying_vectors=31,
        vertex_texture_units=16,
        combined_texture_units=32,
        array_layers=256,
        samples=4,
        uniform_block_size=16_384,
        anisotropy=16.0,
        webgl1_extensions=_ASTC_WEBGL1,
        webgl2_extensions=_ASTC_WEBGL2,
        compressed_formats=(*_ETC1_FORMATS, *_ETC2_FORMATS, *_ASTC_FORMATS),
    ),
    "mali-valhall-modern": _profile(
        texture=8_192,
        cube=8_192,
        renderbuffer=8_192,
        viewport=8_192,
        vertex_uniforms=1_024,
        fragment_uniforms=1_024,
        varying_vectors=31,
        vertex_texture_units=16,
        combined_texture_units=64,
        array_layers=256,
        samples=4,
        uniform_block_size=16_384,
        anisotropy=16.0,
        webgl1_extensions=_ASTC_WEBGL1,
        webgl2_extensions=_ASTC_WEBGL2,
        compressed_formats=(*_ETC1_FORMATS, *_ETC2_FORMATS, *_ASTC_FORMATS),
    ),
    "mali-bifrost": _profile(
        texture=8_192,
        cube=8_192,
        renderbuffer=8_192,
        viewport=8_192,
        vertex_uniforms=1_024,
        fragment_uniforms=1_024,
        varying_vectors=31,
        vertex_texture_units=16,
        combined_texture_units=32,
        array_layers=256,
        samples=4,
        uniform_block_size=16_384,
        anisotropy=16.0,
        webgl1_extensions=_ASTC_WEBGL1,
        webgl2_extensions=_ASTC_WEBGL2,
        compressed_formats=(*_ETC1_FORMATS, *_ETC2_FORMATS, *_ASTC_FORMATS),
    ),
    "xclipse-rdna2": _profile(
        texture=16_384,
        cube=16_384,
        renderbuffer=16_384,
        viewport=16_384,
        vertex_uniforms=1_024,
        fragment_uniforms=1_024,
        varying_vectors=31,
        vertex_texture_units=32,
        combined_texture_units=64,
        array_layers=2_048,
        samples=8,
        uniform_block_size=65_536,
        anisotropy=16.0,
        webgl1_extensions=tuple(dict.fromkeys((
            *_ASTC_WEBGL1,
            "EXT_texture_compression_bptc",
            "EXT_texture_compression_rgtc",
        ))),
        webgl2_extensions=tuple(dict.fromkeys((
            *_ASTC_WEBGL2,
            "EXT_texture_compression_bptc",
            "EXT_texture_compression_rgtc",
        ))),
        compressed_formats=(
            *_ETC1_FORMATS, *_ETC2_FORMATS, *_ASTC_FORMATS,
            *_BPTC_FORMATS, *_RGTC_FORMATS,
        ),
    ),
    "powervr-rogue": _profile(
        texture=4_096,
        cube=4_096,
        renderbuffer=8_192,
        viewport=8_192,
        vertex_uniforms=256,
        fragment_uniforms=256,
        varying_vectors=16,
        vertex_texture_units=16,
        combined_texture_units=32,
        array_layers=256,
        samples=4,
        uniform_block_size=16_384,
        anisotropy=4.0,
        webgl1_extensions=_ASTC_WEBGL1,
        webgl2_extensions=tuple(dict.fromkeys((
            *_WEBGL2_COMMON,
            "WEBGL_compressed_texture_astc",
            "WEBGL_compressed_texture_etc1",
        ))),
        compressed_formats=(*_ETC1_FORMATS, *_ETC2_FORMATS, *_ASTC_FORMATS),
    ),
    "adreno-5xx": _profile(
        texture=8_192,
        cube=8_192,
        renderbuffer=8_192,
        viewport=8_192,
        vertex_uniforms=512,
        fragment_uniforms=512,
        varying_vectors=16,
        vertex_texture_units=16,
        combined_texture_units=32,
        array_layers=256,
        samples=4,
        uniform_block_size=16_384,
        anisotropy=16.0,
        webgl1_extensions=_ASTC_WEBGL1,
        webgl2_extensions=_WEBGL2_COMMON,
        compressed_formats=(*_ETC1_FORMATS, *_ETC2_FORMATS, *_ASTC_FORMATS),
    ),
    "adreno-4xx": _profile(
        texture=4_096,
        cube=4_096,
        renderbuffer=4_096,
        viewport=4_096,
        vertex_uniforms=256,
        fragment_uniforms=256,
        varying_vectors=16,
        vertex_texture_units=16,
        combined_texture_units=32,
        array_layers=256,
        samples=4,
        uniform_block_size=16_384,
        anisotropy=4.0,
        webgl1_extensions=tuple(dict.fromkeys((
            *_WEBGL1_COMMON,
            "WEBGL_compressed_texture_etc",
            "WEBGL_compressed_texture_etc1",
        ))),
        webgl2_extensions=_WEBGL2_COMMON,
        compressed_formats=(*_ETC1_FORMATS, *_ETC2_FORMATS),
    ),
}


def build_android_graphics_capabilities(
    device: dict[str, object],
    chromium_major: int,
) -> tuple[dict[str, object], dict[str, object]]:
    """Return WebGL and WebGPU fields for one concrete device/OS row."""

    profile_id = str(device.get("graphicsProfileId", ""))
    if profile_id not in ANDROID_GRAPHICS_CAPABILITY_PROFILES:
        raise ValueError(f"unknown Android graphics capability profile {profile_id!r}")
    webgl = dict(ANDROID_GRAPHICS_CAPABILITY_PROFILES[profile_id])
    gpu = device.get("gpu", {})
    if not isinstance(gpu, dict):
        raise ValueError("Android device has no GPU object")
    webgpu = {
        "available": bool(device.get("webgpuSupported", False)),
        "vendor": str(gpu.get("vendor", "")),
        "architecture": str(gpu.get("webgpuArchitecture", "")),
        "device": str(gpu.get("model", "")),
        "description": str(gpu.get("model", "")),
        "developer_features": False,
        "subgroup_min_size": 32,
        "subgroup_max_size": 32,
        "is_fallback_adapter": False,
        "features": (
            "bgra8unorm-storage",
            "texture-compression-etc2",
            "texture-compression-astc",
        ),
        "max_compute_workgroup_storage_size": 16_384,
    }
    del chromium_major
    return webgl, webgpu


__all__ = [
    "ANDROID_GRAPHICS_CAPABILITY_PROFILES",
    "build_android_graphics_capabilities",
]
