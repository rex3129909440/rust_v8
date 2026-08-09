"""Additive current PC screen modes with per-row public evidence.

These rows only extend ``screen_profile_catalog``. Existing profile IDs,
values and weights are not replaced. Physical panel resolutions come from
manufacturer specifications; CSS dimensions are the native mode divided by a
Windows-supported scale factor.
"""

WINDOWS_SCALE_SOURCE = (
    "https://learn.microsoft.com/en-us/windows-hardware/design/"
    "component-guidelines/display"
)
WINDOWS_DEVICE_SCALE_SOURCE = (
    "https://learn.microsoft.com/en-us/windows/win32/api/shtypes/"
    "ne-shtypes-device_scale_factor"
)
SAMSUNG_DUAL_UHD_SOURCE = (
    "https://news.samsung.com/global/infographic-odyssey-neo-g9-specs-"
    "the-worlds-first-dual-uhd-monitor"
)
DELL_8K_SOURCE = (
    "https://i.dell.com/sites/csdocuments/Product_Docs/en/"
    "dell_ultrasharp_32_8k_monitor_up3218k_spec_sheet.pdf"
)
APPLE_6K_SOURCE = "https://support.apple.com/en-us/111892"
SURFACE_LAPTOP_SOURCE = (
    "https://www.microsoft.com/content/dam/microsoft/final/en-us/"
    "microsoft-product-and-services/surface/surface-laptop/"
    "surface-laptop-7th-edition/"
    "MSFT-Microsoft-Surface-Laptop-7th-Edition-Fact-Sheet.pdf"
)
SURFACE_STUDIO_SOURCE = (
    "https://www.microsoft.com/surface/business/surface-laptop-studio/"
)


# id, CSS width, CSS height, DPR, weight, tags, evidence sources.
PC_SCREEN_EXTENSION_ROWS = (
    # Samsung 57-inch Dual UHD 7680x2160 panel.
    ("pc_ext_7680x2160_1x_dual_uhd", 7680, 2160, 1.0, 2, ("desktop", "super_ultrawide", "windows"), (SAMSUNG_DUAL_UHD_SOURCE, WINDOWS_SCALE_SOURCE)),
    ("pc_ext_7680x2160_1p25_dual_uhd", 6144, 1728, 1.25, 2, ("desktop", "super_ultrawide", "scaled", "windows"), (SAMSUNG_DUAL_UHD_SOURCE, WINDOWS_SCALE_SOURCE)),
    ("pc_ext_7680x2160_1p5_dual_uhd", 5120, 1440, 1.5, 2, ("desktop", "super_ultrawide", "scaled", "windows"), (SAMSUNG_DUAL_UHD_SOURCE, WINDOWS_SCALE_SOURCE)),
    ("pc_ext_7680x2160_2x_dual_uhd", 3840, 1080, 2.0, 1, ("desktop", "super_ultrawide", "scaled", "windows"), (SAMSUNG_DUAL_UHD_SOURCE, WINDOWS_SCALE_SOURCE)),

    # Dell UP3218K 7680x4320 panel and supported Windows scale factors.
    ("pc_ext_7680x4320_1x_8k", 7680, 4320, 1.0, 1, ("desktop", "8k", "workstation", "windows"), (DELL_8K_SOURCE, WINDOWS_DEVICE_SCALE_SOURCE)),
    ("pc_ext_7680x4320_1p25_8k", 6144, 3456, 1.25, 1, ("desktop", "8k", "workstation", "scaled", "windows"), (DELL_8K_SOURCE, WINDOWS_DEVICE_SCALE_SOURCE)),
    ("pc_ext_7680x4320_1p5_8k", 5120, 2880, 1.5, 2, ("desktop", "8k", "workstation", "scaled", "windows"), (DELL_8K_SOURCE, WINDOWS_DEVICE_SCALE_SOURCE)),
    ("pc_ext_7680x4320_2x_8k", 3840, 2160, 2.0, 2, ("desktop", "8k", "workstation", "scaled", "windows"), (DELL_8K_SOURCE, WINDOWS_DEVICE_SCALE_SOURCE)),
    ("pc_ext_7680x4320_2p5_8k", 3072, 1728, 2.5, 1, ("desktop", "8k", "workstation", "scaled", "windows"), (DELL_8K_SOURCE, WINDOWS_DEVICE_SCALE_SOURCE)),
    ("pc_ext_7680x4320_3x_8k", 2560, 1440, 3.0, 1, ("desktop", "8k", "workstation", "scaled", "windows"), (DELL_8K_SOURCE, WINDOWS_DEVICE_SCALE_SOURCE)),

    # Apple Pro Display XDR physical 6K mode, usable as an external panel.
    ("pc_ext_6016x3384_1p25_6k", 4813, 2707, 1.25, 1, ("desktop", "6k", "workstation", "scaled", "windows"), (APPLE_6K_SOURCE, WINDOWS_DEVICE_SCALE_SOURCE)),
    ("pc_ext_6016x3384_1p5_6k", 4011, 2256, 1.5, 1, ("desktop", "6k", "workstation", "scaled", "windows"), (APPLE_6K_SOURCE, WINDOWS_DEVICE_SCALE_SOURCE)),
    ("pc_ext_6016x3384_2x_6k", 3008, 1692, 2.0, 2, ("desktop", "6k", "workstation", "hidpi", "windows"), (APPLE_6K_SOURCE, WINDOWS_DEVICE_SCALE_SOURCE)),
    ("pc_ext_6016x3384_2p5_6k", 2406, 1354, 2.5, 1, ("desktop", "6k", "workstation", "scaled", "windows"), (APPLE_6K_SOURCE, WINDOWS_DEVICE_SCALE_SOURCE)),

    # Current Microsoft 3:2 laptop panels with additional supported scales.
    ("pc_ext_2496x1664_1p25_surface_laptop", 1997, 1331, 1.25, 4, ("windows", "surface", "laptop", "scaled", "arm64"), (SURFACE_LAPTOP_SOURCE, WINDOWS_SCALE_SOURCE)),
    ("pc_ext_2496x1664_1p5_surface_laptop", 1664, 1109, 1.5, 6, ("windows", "surface", "laptop", "scaled", "arm64"), (SURFACE_LAPTOP_SOURCE, WINDOWS_SCALE_SOURCE)),
    ("pc_ext_2496x1664_2x_surface_laptop", 1248, 832, 2.0, 6, ("windows", "surface", "laptop", "hidpi", "arm64"), (SURFACE_LAPTOP_SOURCE, WINDOWS_SCALE_SOURCE)),
    ("pc_ext_2304x1536_1p25_surface_laptop", 1843, 1229, 1.25, 4, ("windows", "surface", "laptop", "scaled", "arm64"), (SURFACE_LAPTOP_SOURCE, WINDOWS_SCALE_SOURCE)),
    ("pc_ext_2400x1600_1p25_surface_studio", 1920, 1280, 1.25, 4, ("windows", "surface", "laptop", "convertible", "scaled"), (SURFACE_STUDIO_SOURCE, WINDOWS_SCALE_SOURCE)),
    ("pc_ext_2400x1600_1p5_surface_studio", 1600, 1067, 1.5, 6, ("windows", "surface", "laptop", "convertible", "scaled"), (SURFACE_STUDIO_SOURCE, WINDOWS_SCALE_SOURCE)),
)


def count_extension_screen_profiles() -> int:
    return len(PC_SCREEN_EXTENSION_ROWS)
