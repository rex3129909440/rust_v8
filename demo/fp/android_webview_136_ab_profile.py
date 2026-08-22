"""Diagnostic A/B profiles for isolating WebView 136 profile differences.

This module is test-only.  Production random profiles do not import it and do
not depend on the historical fixed reference.  Each diagnostic profile starts
from the known successful fixed profile and replaces exactly one named field
group from one real-device random profile.
"""

from __future__ import annotations

import random
import secrets
from dataclasses import replace

from examples.edge_profile import EdgeProfile
from demo.fp.android_webview_136_profile import (
    WEBVIEW_136_APPLICATION_USER_AGENT,
)


WEBVIEW_136_AB_GROUPS: tuple[str, ...] = (
    "hardware",
    "locale",
    "network",
    "screen",
    "screen-object",
    "window",
    "window-root-outer",
    "window-iframe-outer",
    "css",
    "graphics",
    "audio",
    "memory",
    "input",
    "fonts",
    "media",
    "runtime",
)

_GROUP_SALT = 0x5756423133364142


def _resolved_seed(seed: int | None) -> int:
    if seed is None:
        return secrets.randbits(63)
    if isinstance(seed, bool) or not isinstance(seed, int):
        raise TypeError("seed must be an integer or None")
    return seed & ((1 << 63) - 1)


def _select_group(seed: int, group: str | None) -> str:
    selected = "random" if group is None else str(group).strip().lower()
    if selected == "random":
        return random.Random(seed ^ _GROUP_SALT).choice(WEBVIEW_136_AB_GROUPS)
    if selected == "screen-random":
        return random.Random(seed ^ _GROUP_SALT).choice(
            ("screen-object", "window", "css")
        )
    if selected == "window-random":
        return random.Random(seed ^ _GROUP_SALT).choice(
            ("window-root-outer", "window-iframe-outer")
        )
    if selected not in {*WEBVIEW_136_AB_GROUPS, "baseline", "all-real"}:
        raise ValueError(
            "group must be random, baseline, all-real, or one of: "
            + ", ".join(WEBVIEW_136_AB_GROUPS)
        )
    return selected


def _apply_group(
    baseline: EdgeProfile,
    donor: EdgeProfile,
    group: str,
) -> EdgeProfile:
    if group == "baseline":
        return baseline
    if group == "all-real":
        return donor
    if group == "hardware":
        return replace(
            baseline,
            navigator=replace(
                baseline.navigator,
                hardware_concurrency=donor.navigator.hardware_concurrency,
                device_memory_gb=donor.navigator.device_memory_gb,
                max_touch_points=donor.navigator.max_touch_points,
                user_agent_data=donor.navigator.user_agent_data,
            ),
        )
    if group == "locale":
        return replace(
            baseline,
            locale=donor.locale,
            geolocation=donor.geolocation,
            navigator=replace(
                baseline.navigator,
                language=donor.navigator.language,
                languages=donor.navigator.languages,
            ),
        )
    if group == "network":
        return replace(
            baseline,
            navigator=replace(
                baseline.navigator,
                network=donor.navigator.network,
            ),
            media_preferences=replace(
                baseline.media_preferences,
                reduced_data=donor.media_preferences.reduced_data,
            ),
        )
    if group == "screen":
        return replace(
            baseline,
            screen=donor.screen,
            window=donor.window,
            css=donor.css,
        )
    if group == "screen-object":
        return replace(baseline, screen=donor.screen)
    if group == "window":
        return replace(baseline, window=donor.window)
    if group == "window-root-outer":
        return replace(
            baseline,
            window=replace(
                baseline.window,
                outer_width=donor.window.outer_width,
                outer_height=donor.window.outer_height,
            ),
        )
    if group == "window-iframe-outer":
        return replace(
            baseline,
            window=replace(
                baseline.window,
                iframe_outer_width=donor.window.iframe_outer_width,
                iframe_outer_height=donor.window.iframe_outer_height,
            ),
        )
    if group == "css":
        return replace(baseline, css=donor.css)
    if group == "graphics":
        return replace(
            baseline,
            webgl=donor.webgl,
            webgpu=donor.webgpu,
        )
    if group == "audio":
        return replace(baseline, audio=donor.audio)
    if group == "memory":
        return replace(baseline, memory=donor.memory)
    if group == "input":
        return replace(
            baseline,
            media_preferences=replace(
                baseline.media_preferences,
                color_scheme=donor.media_preferences.color_scheme,
                contrast=donor.media_preferences.contrast,
                reduced_motion=donor.media_preferences.reduced_motion,
                reduced_transparency=donor.media_preferences.reduced_transparency,
                forced_colors=donor.media_preferences.forced_colors,
                inverted_colors=donor.media_preferences.inverted_colors,
                monochrome_bits=donor.media_preferences.monochrome_bits,
                color_gamut=donor.media_preferences.color_gamut,
                pointer=donor.media_preferences.pointer,
                any_pointer=donor.media_preferences.any_pointer,
                hover=donor.media_preferences.hover,
                any_hover=donor.media_preferences.any_hover,
                dynamic_range=donor.media_preferences.dynamic_range,
                video_dynamic_range=donor.media_preferences.video_dynamic_range,
            ),
        )
    if group == "fonts":
        return replace(baseline, fonts=donor.fonts)
    if group == "media":
        return replace(
            baseline,
            media=donor.media,
            permissions=donor.permissions,
            sensors=donor.sensors,
            plugins=donor.plugins,
            speech=donor.speech,
        )
    if group == "runtime":
        return replace(
            baseline,
            battery=donor.battery,
            storage=donor.storage,
            timing=donor.timing,
            document=donor.document,
            hardware_devices=donor.hardware_devices,
            canvas=donor.canvas,
        )
    raise AssertionError(f"unhandled WebView 136 A/B group {group!r}")


def build_webview_136_ab_profile(
    *,
    country_code: str = "US",
    seed: int | None = None,
    group: str | None = "random",
    baseline_sample_index: int = 1,
) -> EdgeProfile:
    """Return one fixed-baseline profile with one randomized field group."""

    from demo.android_call_edge_sandbox import build_android_profile

    resolved_seed = _resolved_seed(seed)
    selected_group = _select_group(resolved_seed, group)
    donor = build_android_profile(
        country_code,
        WEBVIEW_136_APPLICATION_USER_AGENT,
        seed=resolved_seed,
        chromium_major=136,
    )
    if selected_group == "all-real":
        profile = donor
    else:
        from demo.android_call_edge_sandbox import (
            build_webview_136_success_reference_profile,
        )

        baseline = build_webview_136_success_reference_profile(
            sample_index=baseline_sample_index,
        )
        profile = _apply_group(baseline, donor, selected_group)
    donor_marker = donor.id.removeprefix("random-android-webview-136-").removesuffix(
        f"-{resolved_seed:016x}"
    )
    return replace(
        profile,
        id=(
            f"webview136-ab-{selected_group}-"
            f"{donor_marker}-"
            f"{resolved_seed:016x}"
        ),
    )


__all__ = [
    "WEBVIEW_136_AB_GROUPS",
    "build_webview_136_ab_profile",
]
