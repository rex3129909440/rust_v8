"""Chromium 150 Metal graphics capabilities used by macOS profiles.

The values in this module are derived from the exact ANGLE and Dawn revisions
used by Chromium branch 7879, plus Apple's Metal feature-set table.  This is a
local, reviewable data catalog: it performs no device probing and contains no
fallback or guessed adapter values.

Only Apple7 through Apple10 are accepted.  Intel/AMD Metal properties such as
``supportsTextureSampleCount`` and ``maxBufferLength`` are runtime device
queries; public product specifications do not publish the result for every
adapter in the legacy catalog, so those candidates must not be synthesized.
"""

from __future__ import annotations

from typing import Mapping

try:
    from .mac_chromium150_capture_catalog import (
        MAC_CHROMIUM150_WEBGL1_EXTENSIONS,
        MAC_CHROMIUM150_WEBGL2_EXTENSIONS,
        MAC_CHROMIUM150_WEBGPU_FEATURES,
        MAC_CHROMIUM150_WEBGPU_LIMITS,
    )
except ImportError:  # Direct import from demo/fp.
    from mac_chromium150_capture_catalog import (  # type: ignore
        MAC_CHROMIUM150_WEBGL1_EXTENSIONS,
        MAC_CHROMIUM150_WEBGL2_EXTENSIONS,
        MAC_CHROMIUM150_WEBGPU_FEATURES,
        MAC_CHROMIUM150_WEBGPU_LIMITS,
    )


CHROMIUM_BRANCH = "7879"
CHROMIUM_COMMIT = "6b856e0afe5890e231137725eac0449907f4fdb2"
ANGLE_COMMIT = "e53ecb3f8dbd797748dd21eea0a5606b54d82802"
DAWN_COMMIT = "23cf554e645f61acabcd10aac24bfe6d6b0eeeec"

ANGLE_DISPLAY_MTL_SOURCE = (
    "https://chromium.googlesource.com/angle/angle/+/"
    f"{ANGLE_COMMIT}/src/libANGLE/renderer/metal/DisplayMtl.mm"
)
ANGLE_MTL_COMMON_SOURCE = (
    "https://chromium.googlesource.com/angle/angle/+/"
    f"{ANGLE_COMMIT}/src/libANGLE/renderer/metal/mtl_common.h"
)
ANGLE_CONSTANTS_SOURCE = (
    "https://chromium.googlesource.com/angle/angle/+/"
    f"{ANGLE_COMMIT}/src/libANGLE/Constants.h"
)
DAWN_METAL_DEVICE_SOURCE = (
    "https://dawn.googlesource.com/dawn/+/"
    f"{DAWN_COMMIT}/src/dawn/native/metal/PhysicalDeviceMTL.mm"
)
DAWN_LIMITS_SOURCE = (
    "https://dawn.googlesource.com/dawn/+/"
    f"{DAWN_COMMIT}/src/dawn/native/Limits.cpp"
)
CHROMIUM_WEBGPU_DECODER_SOURCE = (
    "https://chromium.googlesource.com/chromium/src/+/"
    f"{CHROMIUM_COMMIT}/gpu/command_buffer/service/webgpu_decoder_impl.cc"
)
APPLE_METAL_CAPABILITIES_SOURCE = "https://developer.apple.com/metal/capabilities/"

CAPABILITY_SOURCES = (
    ANGLE_DISPLAY_MTL_SOURCE,
    ANGLE_MTL_COMMON_SOURCE,
    ANGLE_CONSTANTS_SOURCE,
    DAWN_METAL_DEVICE_SOURCE,
    DAWN_LIMITS_SOURCE,
    CHROMIUM_WEBGPU_DECODER_SOURCE,
    APPLE_METAL_CAPABILITIES_SOURCE,
)


# OpenGL compressed texture enums.  All Apple7/Apple8 macOS devices support BC
# compression, and Apple9+ supports it across the family.  Apple silicon also
# supports ETC2/EAC and ASTC.
BC_S3TC_FORMATS = tuple(range(0x83F0, 0x83F4))
BC_S3TC_SRGB_FORMATS = tuple(range(0x8C4C, 0x8C50))
BC_BPTC_FORMATS = tuple(range(0x8E8C, 0x8E90))
ETC2_EAC_FORMATS = tuple(range(0x9270, 0x927A))
ASTC_LINEAR_FORMATS = tuple(range(0x93B0, 0x93BE))
ASTC_SRGB_FORMATS = tuple(range(0x93D0, 0x93DE))

APPLE_SILICON_COMPRESSED_TEXTURE_FORMATS = (
    *BC_S3TC_FORMATS,
    *BC_S3TC_SRGB_FORMATS,
    *BC_BPTC_FORMATS,
    *ETC2_EAC_FORMATS,
    *ASTC_LINEAR_FORMATS,
    *ASTC_SRGB_FORMATS,
)

APPLE_SILICON_WEBGL1_EXTENSIONS = MAC_CHROMIUM150_WEBGL1_EXTENSIONS

APPLE_SILICON_WEBGL2_EXTENSIONS = MAC_CHROMIUM150_WEBGL2_EXTENSIONS


# Features enabled by Dawn's Metal backend for an Apple7+ device on modern
# macOS without relying on runtime counter-set availability.  Timestamp-query
# and texture-formats-tier2 are deliberately absent because Dawn queries those
# capabilities from the active MTLDevice at runtime.
APPLE_SILICON_WEBGPU_FEATURES = MAC_CHROMIUM150_WEBGPU_FEATURES


_SUPPORTED_APPLE_FAMILIES = frozenset({"apple7", "apple8", "apple9", "apple10"})


def is_verified_mac_graphics_candidate(candidate: Mapping[str, object]) -> bool:
    """Return whether public sources fully cover this profile's graphics path."""

    return (
        str(candidate.get("vendor", "")).lower() == "apple"
        and str(candidate.get("architecture", "")).lower()
        in _SUPPORTED_APPLE_FAMILIES
    )


def _apple_family(candidate: Mapping[str, object]) -> str:
    family = str(candidate.get("architecture", "")).lower()
    if not is_verified_mac_graphics_candidate(candidate):
        profile_id = str(candidate.get("id", "unknown"))
        raise ValueError(
            f"Mac GPU candidate {profile_id!r} has no complete public "
            "Chromium 150 Metal capability record"
        )
    return family


def build_mac_webgl_capabilities(
    candidate: Mapping[str, object],
) -> dict[str, object]:
    """Build WebGL limits exposed by Chromium 150's ANGLE Metal backend."""

    family = _apple_family(candidate)
    max_samples = 8 if family == "apple10" else 4
    return {
        "webgl1_extensions": APPLE_SILICON_WEBGL1_EXTENSIONS,
        "webgl2_extensions": APPLE_SILICON_WEBGL2_EXTENSIONS,
        "compressed_texture_formats": APPLE_SILICON_COMPRESSED_TEXTURE_FORMATS,
        "max_texture_size": 16_384,
        "max_cube_map_texture_size": 16_384,
        "max_renderbuffer_size": 16_384,
        "max_viewport_width": 16_384,
        "max_viewport_height": 16_384,
        "max_vertex_attribs": 16,
        "max_vertex_uniform_vectors": 1_024,
        "max_varying_vectors": 30,
        "max_fragment_uniform_vectors": 1_024,
        "max_vertex_texture_image_units": 16,
        "max_texture_image_units": 16,
        "max_combined_texture_image_units": 32,
        "webgl2_max_3d_texture_size": 2_048,
        "webgl2_max_array_texture_layers": 2_048,
        "webgl2_max_draw_buffers": 8,
        "webgl2_max_color_attachments": 8,
        "webgl2_max_samples": max_samples,
        "webgl2_max_vertex_uniform_components": 4_096,
        "webgl2_max_fragment_uniform_components": 4_096,
        "webgl2_max_varying_components": 120,
        "webgl2_max_vertex_output_components": 120,
        "webgl2_max_fragment_input_components": 120,
        "webgl2_max_vertex_uniform_blocks": 16,
        "webgl2_max_fragment_uniform_blocks": 16,
        "webgl2_max_combined_uniform_blocks": 32,
        "webgl2_max_uniform_buffer_bindings": 32,
        "webgl2_max_uniform_block_size": 16_384,
        "webgl2_max_combined_vertex_uniform_components": 69_632,
        "webgl2_max_combined_fragment_uniform_components": 69_632,
        "webgl2_max_transform_feedback_separate_attribs": 4,
        "webgl2_max_transform_feedback_interleaved_components": 128,
        "webgl2_max_transform_feedback_separate_components": 4,
        "webgl2_max_program_texel_offset": 7,
        "webgl2_max_elements_vertices": 2_147_483_647,
        "webgl2_max_elements_indices": 2_147_483_647,
        "webgl2_max_element_index": 4_294_967_294,
        "webgl2_max_texture_lod_bias": 15.0,
        "max_anisotropy": 16.0,
        "aliased_point_size_min": 1.0,
        "aliased_point_size_max": 511.0,
        "aliased_line_width_min": 1.0,
        "aliased_line_width_max": 1.0,
        "shader_precision_range_min": 127,
        "shader_precision_range_max": 127,
        "shader_precision_bits": 23,
    }


def build_mac_webgpu_capabilities(
    candidate: Mapping[str, object],
) -> dict[str, object]:
    """Build Chrome-tiered WebGPU limits for an Apple-silicon Mac2 device."""

    _apple_family(candidate)
    # Dawn selects Mac2 before AppleN when computing physical WebGPU limits on
    # macOS.  Chromium enables Dawn's tiered adapter limits by default.
    return {
        "vendor": "apple",
        "architecture": "metal-3",
        "developer_features": False,
        "subgroup_min_size": 32,
        "subgroup_max_size": 32,
        "is_fallback_adapter": False,
        "features": APPLE_SILICON_WEBGPU_FEATURES,
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
        "max_storage_buffers_per_shader_stage": 10,
        "max_storage_textures_per_shader_stage": 8,
        "max_uniform_buffers_per_shader_stage": 12,
        "max_uniform_buffer_binding_size": 65_536,
        "max_storage_buffer_binding_size": 4_294_967_292,
        "min_uniform_buffer_offset_alignment": 256,
        "min_storage_buffer_offset_alignment": 256,
        "max_vertex_buffers": 8,
        "max_buffer_size": 4_294_967_292,
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
        "max_immediate_size": int(
            MAC_CHROMIUM150_WEBGPU_LIMITS["max_immediate_size"]
        ),
        "max_storage_buffers_in_fragment_stage": 10,
        "max_storage_textures_in_fragment_stage": 8,
        "max_storage_buffers_in_vertex_stage": 10,
        "max_storage_textures_in_vertex_stage": 8,
    }


def build_mac_graphics_capabilities(
    candidate: Mapping[str, object],
) -> tuple[dict[str, object], dict[str, object]]:
    """Return the complete WebGL and WebGPU patches for one candidate."""

    return (
        build_mac_webgl_capabilities(candidate),
        build_mac_webgpu_capabilities(candidate),
    )


__all__ = [
    "ANGLE_COMMIT",
    "ANGLE_CONSTANTS_SOURCE",
    "ANGLE_DISPLAY_MTL_SOURCE",
    "ANGLE_MTL_COMMON_SOURCE",
    "APPLE_METAL_CAPABILITIES_SOURCE",
    "APPLE_SILICON_COMPRESSED_TEXTURE_FORMATS",
    "APPLE_SILICON_WEBGL1_EXTENSIONS",
    "APPLE_SILICON_WEBGL2_EXTENSIONS",
    "APPLE_SILICON_WEBGPU_FEATURES",
    "CAPABILITY_SOURCES",
    "CHROMIUM_BRANCH",
    "CHROMIUM_COMMIT",
    "CHROMIUM_WEBGPU_DECODER_SOURCE",
    "DAWN_COMMIT",
    "DAWN_LIMITS_SOURCE",
    "DAWN_METAL_DEVICE_SOURCE",
    "build_mac_graphics_capabilities",
    "build_mac_webgl_capabilities",
    "build_mac_webgpu_capabilities",
    "is_verified_mac_graphics_candidate",
]
