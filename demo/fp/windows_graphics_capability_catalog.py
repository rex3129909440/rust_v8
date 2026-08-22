"""Chromium 150 ANGLE/Dawn graphics capabilities for Windows profiles.

The adapter name and PCI Device ID come from ``windows_webgl_gpu_catalog``.
This module supplies the capability record that must accompany that identity.
It deliberately uses Chromium/Dawn's exposed tiers instead of inventing a
different limit table for every marketing name.

The pinned revisions are the revisions used by Chromium branch 7879.  ANGLE
queries multisample support from the active D3D11 device, so public product
lists are not sufficient to infer an arbitrary per-model value.  The hardware
profile therefore uses the verified eight-sample desktop row and does not
pretend that PCI identity alone proves a different value.
"""

from __future__ import annotations

from typing import Mapping


CHROMIUM_BRANCH = "7879"
CHROMIUM_COMMIT = "6b856e0afe5890e231137725eac0449907f4fdb2"
ANGLE_COMMIT = "e53ecb3f8dbd797748dd21eea0a5606b54d82802"
DAWN_COMMIT = "23cf554e645f61acabcd10aac24bfe6d6b0eeeec"

ANGLE_D3D11_CAPS_SOURCE = (
    "https://chromium.googlesource.com/angle/angle/+/"
    f"{ANGLE_COMMIT}/src/libANGLE/renderer/d3d/d3d11/renderer11_utils.cpp"
)
ANGLE_D3D11_RENDERER_SOURCE = (
    "https://chromium.googlesource.com/angle/angle/+/"
    f"{ANGLE_COMMIT}/src/libANGLE/renderer/d3d/d3d11/Renderer11.cpp"
)
DAWN_D3D12_DEVICE_SOURCE = (
    "https://dawn.googlesource.com/dawn/+/"
    f"{DAWN_COMMIT}/src/dawn/native/d3d12/PhysicalDeviceD3D12.cpp"
)
DAWN_LIMITS_SOURCE = (
    "https://dawn.googlesource.com/dawn/+/"
    f"{DAWN_COMMIT}/src/dawn/native/Limits.cpp"
)

CAPABILITY_SOURCES = (
    ANGLE_D3D11_CAPS_SOURCE,
    ANGLE_D3D11_RENDERER_SOURCE,
    DAWN_D3D12_DEVICE_SOURCE,
    DAWN_LIMITS_SOURCE,
)


WINDOWS_CHROMIUM150_WEBGL1_EXTENSIONS = (
    "ANGLE_instanced_arrays",
    "EXT_blend_minmax",
    "EXT_clip_control",
    "EXT_color_buffer_half_float",
    "EXT_depth_clamp",
    "EXT_disjoint_timer_query",
    "EXT_float_blend",
    "EXT_frag_depth",
    "EXT_polygon_offset_clamp",
    "EXT_shader_texture_lod",
    "EXT_texture_compression_bptc",
    "EXT_texture_compression_rgtc",
    "EXT_texture_filter_anisotropic",
    "EXT_texture_mirror_clamp_to_edge",
    "EXT_sRGB",
    "KHR_parallel_shader_compile",
    "OES_element_index_uint",
    "OES_fbo_render_mipmap",
    "OES_standard_derivatives",
    "OES_texture_float",
    "OES_texture_float_linear",
    "OES_texture_half_float",
    "OES_texture_half_float_linear",
    "OES_vertex_array_object",
    "WEBGL_blend_func_extended",
    "WEBGL_color_buffer_float",
    "WEBGL_compressed_texture_s3tc",
    "WEBGL_compressed_texture_s3tc_srgb",
    "WEBGL_debug_renderer_info",
    "WEBGL_debug_shaders",
    "WEBGL_depth_texture",
    "WEBGL_draw_buffers",
    "WEBGL_lose_context",
    "WEBGL_multi_draw",
    "WEBGL_polygon_mode",
)

WINDOWS_CHROMIUM150_WEBGL2_EXTENSIONS = (
    "EXT_clip_control",
    "EXT_color_buffer_float",
    "EXT_color_buffer_half_float",
    "EXT_conservative_depth",
    "EXT_depth_clamp",
    "EXT_disjoint_timer_query_webgl2",
    "EXT_float_blend",
    "EXT_polygon_offset_clamp",
    "EXT_render_snorm",
    "EXT_texture_compression_bptc",
    "EXT_texture_compression_rgtc",
    "EXT_texture_filter_anisotropic",
    "EXT_texture_mirror_clamp_to_edge",
    "EXT_texture_norm16",
    "KHR_parallel_shader_compile",
    "NV_shader_noperspective_interpolation",
    "OES_draw_buffers_indexed",
    "OES_sample_variables",
    "OES_shader_multisample_interpolation",
    "OES_texture_float_linear",
    "OVR_multiview2",
    "WEBGL_blend_func_extended",
    "WEBGL_clip_cull_distance",
    "WEBGL_compressed_texture_s3tc",
    "WEBGL_compressed_texture_s3tc_srgb",
    "WEBGL_debug_renderer_info",
    "WEBGL_debug_shaders",
    "WEBGL_lose_context",
    "WEBGL_multi_draw",
    "WEBGL_polygon_mode",
    "WEBGL_provoking_vertex",
    "WEBGL_stencil_texturing",
)

WINDOWS_BC_COMPRESSED_TEXTURE_FORMATS = (
    *range(0x83F0, 0x83F4),
    *range(0x8C4C, 0x8C50),
    *range(0x8E8C, 0x8E90),
)


_WEBGPU_BASE_FEATURES = (
    "depth32float-stencil8",
    "rg11b10ufloat-renderable",
    "texture-formats-tier1",
    "bgra8unorm-storage",
    "texture-compression-bc",
    "dual-source-blending",
    "core-features-and-limits",
    "float32-filterable",
    "indirect-first-instance",
    "float32-blendable",
    "depth-clip-control",
    "texture-compression-bc-sliced-3d",
    "timestamp-query",
    "texture-formats-tier2",
    "clip-distances",
    "primitive-index",
    "texture-component-swizzle",
)

_F16_ARCHITECTURES = frozenset(
    {
        "turing",
        "ampere",
        "ada",
        "lovelace",
        "blackwell",
        "rdna1",
        "rdna2",
        "rdna3",
        "rdna3.5",
        "rdna4",
        "xe-lp",
        "xe-lpg",
        "xe-hpg",
        "xe2",
        "xe2-battlemage",
    }
)

_SUBGROUP_ARCHITECTURES = _F16_ARCHITECTURES | frozenset(
    {
        "pascal",
        "gcn4",
        "gen-11",
        "gen-12",
        "gen-12lp",
    }
)

_FORMAT_TIER2_ARCHITECTURES = _SUBGROUP_ARCHITECTURES | frozenset(
    {"maxwell", "gen-9.5", "gen-9"}
)


def _webgpu_features(candidate: Mapping[str, object]) -> tuple[str, ...]:
    """Return stable Web-exposed features supported by the selected tier."""

    if not bool(candidate.get("webgpuSupported", True)):
        return ()
    architecture = str(candidate.get("architecture", "")).lower()
    tier = str(candidate.get("tier", "")).lower()
    features = list(_WEBGPU_BASE_FEATURES)
    if architecture not in _FORMAT_TIER2_ARCHITECTURES:
        features.remove("texture-formats-tier2")
    if tier == "virtual":
        # Timestamp support is an actual command-queue query and Chromium's
        # source explicitly calls out containers/vGPUs as a failure case.
        features.remove("timestamp-query")
    if architecture in _F16_ARCHITECTURES:
        features.insert(features.index("clip-distances"), "shader-f16")
    if architecture in _SUBGROUP_ARCHITECTURES:
        features.append("subgroups")
    return tuple(features)


def _subgroup_range(candidate: Mapping[str, object]) -> tuple[int, int]:
    vendor = str(candidate.get("vendor", "")).lower()
    architecture = str(candidate.get("architecture", "")).lower()
    if architecture not in _SUBGROUP_ARCHITECTURES:
        return 4, 128
    if vendor == "amd":
        return (64, 64) if architecture.startswith("gcn") else (32, 64)
    if vendor == "intel":
        return 8, 32
    return 32, 32


def build_windows_webgl_capabilities(
    candidate: Mapping[str, object],
    chromium_major: int,
) -> dict[str, object]:
    """Build a coherent ANGLE D3D11 WebGL record for one adapter."""

    vendor = str(candidate.get("vendor", "")).lower()
    # ANGLE reserves VS constant register zero only on NVIDIA.  That produces
    # the observable 4095-vector / 16380-component values.
    vertex_uniform_vectors = 4_095 if vendor == "nvidia" else 4_096
    vertex_uniform_components = vertex_uniform_vectors * 4
    patch: dict[str, object] = {
        "max_vertex_uniform_vectors": vertex_uniform_vectors,
        "webgl2_max_vertex_uniform_components": vertex_uniform_components,
        # ANGLE obtains this from CheckMultisampleQualityLevels. Eight is the
        # verified hardware-D3D11 row; do not inherit WARP's sixteen here.
        "webgl2_max_samples": 8,
    }
    if int(chromium_major) >= 150:
        patch.update(
            webgl1_extensions=WINDOWS_CHROMIUM150_WEBGL1_EXTENSIONS,
            webgl2_extensions=WINDOWS_CHROMIUM150_WEBGL2_EXTENSIONS,
            compressed_texture_formats=WINDOWS_BC_COMPRESSED_TEXTURE_FORMATS,
        )
    return patch


def build_windows_webgpu_capabilities(
    candidate: Mapping[str, object],
    chromium_major: int,
) -> dict[str, object]:
    """Build Chromium/Dawn's tiered Windows D3D12 adapter surface."""

    available = bool(candidate.get("webgpuSupported", True))
    subgroup_min, subgroup_max = _subgroup_range(candidate)
    # Chromium 150's pinned Dawn revision exposes the high desktop tiers after
    # applying privacy-preserving limit tiering. Older Chromium majors keep the
    # project's previous baseline until exact revision evidence is available.
    if int(chromium_major) < 150:
        return {
            "available": available,
            "vendor": str(candidate.get("vendor", "")),
            "architecture": str(candidate.get("webgpuArchitecture", "")),
            "developer_features": False,
            "subgroup_min_size": subgroup_min,
            "subgroup_max_size": subgroup_max,
            "is_fallback_adapter": False,
        }

    storage_buffer_size = (
        268_435_456
        if str(candidate.get("vendor", "")).lower() == "qualcomm"
        else 2_147_483_644
    )
    return {
        "available": available,
        "vendor": str(candidate.get("vendor", "")),
        "architecture": str(candidate.get("webgpuArchitecture", "")),
        "developer_features": False,
        "subgroup_min_size": subgroup_min,
        "subgroup_max_size": subgroup_max,
        "is_fallback_adapter": False,
        "features": _webgpu_features(candidate),
        "max_texture_dimension_1d": 16_384,
        "max_texture_dimension_2d": 16_384,
        "max_texture_dimension_3d": 2_048,
        "max_texture_array_layers": 2_048,
        "max_bind_groups": 4,
        "max_bind_groups_plus_vertex_buffers": 24,
        "max_bindings_per_bind_group": 1_000,
        "max_dynamic_uniform_buffers_per_pipeline_layout": 10,
        "max_dynamic_storage_buffers_per_pipeline_layout": 8,
        "max_sampled_textures_per_shader_stage": 48,
        "max_samplers_per_shader_stage": 16,
        "max_storage_buffers_per_shader_stage": 16,
        "max_storage_textures_per_shader_stage": 8,
        "max_uniform_buffers_per_shader_stage": 12,
        "max_uniform_buffer_binding_size": 65_536,
        "max_storage_buffer_binding_size": storage_buffer_size,
        "min_uniform_buffer_offset_alignment": 256,
        "min_storage_buffer_offset_alignment": 256,
        "max_vertex_buffers": 8,
        "max_buffer_size": 2_147_483_648,
        "max_vertex_attributes": 30,
        "max_vertex_buffer_array_stride": 2_048,
        "max_inter_stage_shader_variables": 28,
        "max_color_attachments": 8,
        "max_color_attachment_bytes_per_sample": 128,
        "max_compute_workgroup_storage_size": 32_768,
        "max_compute_invocations_per_workgroup": 1_024,
        "max_compute_workgroup_size_x": 1_024,
        "max_compute_workgroup_size_y": 1_024,
        "max_compute_workgroup_size_z": 64,
        "max_compute_workgroups_per_dimension": 65_535,
        "max_immediate_size": 64,
        "max_storage_buffers_in_fragment_stage": 16,
        "max_storage_textures_in_fragment_stage": 8,
        "max_storage_buffers_in_vertex_stage": 16,
        "max_storage_textures_in_vertex_stage": 8,
    }


def build_windows_graphics_capabilities(
    candidate: Mapping[str, object],
    chromium_major: int,
) -> tuple[dict[str, object], dict[str, object]]:
    return (
        build_windows_webgl_capabilities(candidate, chromium_major),
        build_windows_webgpu_capabilities(candidate, chromium_major),
    )


__all__ = [
    "ANGLE_COMMIT",
    "ANGLE_D3D11_CAPS_SOURCE",
    "ANGLE_D3D11_RENDERER_SOURCE",
    "CAPABILITY_SOURCES",
    "CHROMIUM_BRANCH",
    "CHROMIUM_COMMIT",
    "DAWN_COMMIT",
    "DAWN_D3D12_DEVICE_SOURCE",
    "DAWN_LIMITS_SOURCE",
    "WINDOWS_BC_COMPRESSED_TEXTURE_FORMATS",
    "WINDOWS_CHROMIUM150_WEBGL1_EXTENSIONS",
    "WINDOWS_CHROMIUM150_WEBGL2_EXTENSIONS",
    "build_windows_graphics_capabilities",
    "build_windows_webgl_capabilities",
    "build_windows_webgpu_capabilities",
]
