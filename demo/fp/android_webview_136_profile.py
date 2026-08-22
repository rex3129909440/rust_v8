"""Compatibility import for the former version-specific module name.

All runtime implementation lives in ``android_webview_application_profile``;
no Chromium-136 geometry or profile branch remains here.
"""

try:
    from demo.fp.android_webview_application_profile import (
        WEBVIEW_136_APPLICATION_USER_AGENT,
        WEBVIEW_APPLICATION_USER_AGENT,
        WEBVIEW_DPR_CHOICES,
        apply_webview_device_profile,
        apply_webview_input_geometry_profile,
        apply_webview_136_device_profile,
        count_webview_device_mode_combinations,
        count_webview_136_device_mode_combinations,
        is_wizz_air_application_user_agent,
        is_android_webview_user_agent,
        supports_webview_device_profile,
        supports_webview_136_device_profile,
    )
except ModuleNotFoundError:
    from android_webview_application_profile import (  # type: ignore
        WEBVIEW_136_APPLICATION_USER_AGENT,
        WEBVIEW_APPLICATION_USER_AGENT,
        WEBVIEW_DPR_CHOICES,
        apply_webview_device_profile,
        apply_webview_input_geometry_profile,
        apply_webview_136_device_profile,
        count_webview_device_mode_combinations,
        count_webview_136_device_mode_combinations,
        is_wizz_air_application_user_agent,
        is_android_webview_user_agent,
        supports_webview_device_profile,
        supports_webview_136_device_profile,
    )


__all__ = [
    "WEBVIEW_136_APPLICATION_USER_AGENT",
    "WEBVIEW_APPLICATION_USER_AGENT",
    "WEBVIEW_DPR_CHOICES",
    "apply_webview_device_profile",
    "apply_webview_input_geometry_profile",
    "apply_webview_136_device_profile",
    "count_webview_device_mode_combinations",
    "count_webview_136_device_mode_combinations",
    "is_wizz_air_application_user_agent",
    "is_android_webview_user_agent",
    "supports_webview_device_profile",
    "supports_webview_136_device_profile",
]
