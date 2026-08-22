"""Additional public Android device/SoC rows shared by Chrome and WebView.

Physical display resolutions from OEM specifications are normalized to CSS
Screen dimensions with the device density scale.  GPU names are bound to the
published SoC/device model; WebGL limits are supplied by architecture-family
defaults in ``android_graphics_capability_catalog``.

Representative primary sources:
- https://en-us.support.motorola.com/app/answers/detail/a_id/143888/~/specifications--moto-g7-play
- https://www.mi.com/global/product/mi-a2/
- https://www.mi.com/global/product/redmi-note-8/
- https://www.samsung.com/us/smartphones/galaxy-a52-5g/
- https://www.oneplus.com/global/9/specs
- https://www.po.co/global/product/poco-f5/specs
- https://www.samsung.com/global/galaxy/galaxy-s22/specs/
- https://www.samsung.com/global/galaxy/galaxy-s24/specs/
- https://www.hmd.com/en_int/nokia-1-plus/
- https://corp.mediatek.com/news-events/press-releases/full-featured-mediatek-smartphone-chipset-mt6739-debuts-in-india
"""

from __future__ import annotations


# Same typed row format as ANDROID_DEVICE_ROWS in android_device_profile_catalog.
ANDROID_EXTENDED_DEVICE_ROWS: tuple[tuple[object, ...], ...] = (
    ("android_moto_e6", "moto e6", "9", 8, (2,), 360, 720, 2.0, "Qualcomm", "Adreno (TM) 505", "adreno-5xx", "3.2", 4, ("phone", "entry", "motorola")),
    ("android_moto_g7_play", "moto g(7) play", "9", 8, (2,), 360, 756, 2.0, "Qualcomm", "Adreno (TM) 506", "adreno-5xx", "3.2", 5, ("phone", "entry", "motorola")),
    ("android_xiaomi_mi_a2", "Mi A2", "8", 8, (4, 6), 360, 720, 3.0, "Qualcomm", "Adreno (TM) 512", "adreno-5xx", "3.2", 4, ("phone", "legacy", "xiaomi")),
    ("android_redmi_note_8", "Redmi Note 8", "9", 8, (4, 6), 360, 780, 3.0, "Qualcomm", "Adreno (TM) 610", "adreno-6xx", "3.2", 7, ("phone", "mainstream", "xiaomi")),
    ("android_redmi_note_7_pro", "Redmi Note 7 Pro", "9", 8, (4, 6), 360, 780, 3.0, "Qualcomm", "Adreno (TM) 612", "adreno-6xx", "3.2", 5, ("phone", "mainstream", "xiaomi")),
    ("android_redmi_12_5g", "23076RN4BI", "13", 8, (4, 6, 8), 360, 820, 3.0, "Qualcomm", "Adreno (TM) 613", "adreno-6xx", "3.2", 6, ("phone", "mainstream", "xiaomi")),
    ("android_galaxy_a52_5g", "SM-A526B", "11", 8, (6, 8), 360, 800, 3.0, "Qualcomm", "Adreno (TM) 619", "adreno-6xx", "3.2", 7, ("phone", "mainstream", "samsung")),
    ("android_galaxy_a52s", "SM-A528B", "11", 8, (6, 8), 360, 800, 3.0, "Qualcomm", "Adreno (TM) 642L", "adreno-6xx", "3.2", 6, ("phone", "mainstream", "samsung")),
    ("android_xiaomi_13_lite", "2210129SG", "13", 8, (8,), 360, 800, 3.0, "Qualcomm", "Adreno (TM) 644", "adreno-6xx", "3.2", 5, ("phone", "mainstream", "xiaomi")),
    ("android_oneplus_9", "LE2113", "11", 8, (8, 12), 412, 915, 2.625, "Qualcomm", "Adreno (TM) 660", "adreno-6xx", "3.2", 7, ("phone", "flagship", "oneplus")),
    ("android_poco_f5", "23049PCD8G", "13", 8, (8, 12), 393, 873, 2.75, "Qualcomm", "Adreno (TM) 725", "adreno-7xx", "3.2", 7, ("phone", "flagship", "xiaomi")),
    ("android_oneplus_10_pro", "NE2213", "12", 8, (8, 12), 480, 1072, 3.0, "Qualcomm", "Adreno (TM) 730", "adreno-7xx", "3.2", 6, ("phone", "flagship", "oneplus")),
    ("android_redmi_9c", "M2006C3MG", "10", 8, (2, 3, 4), 360, 800, 2.0, "ARM", "Mali-G31", "bifrost", "3.2", 6, ("phone", "entry", "xiaomi")),
    ("android_galaxy_a12", "SM-A125F", "10", 8, (3, 4, 6), 360, 800, 2.0, "ARM", "Mali-G52", "bifrost", "3.2", 7, ("phone", "entry", "samsung")),
    ("android_redmi_note_10_5g", "M2103K19G", "11", 8, (4, 6, 8), 360, 800, 3.0, "ARM", "Mali-G57", "valhall", "3.2", 7, ("phone", "mainstream", "xiaomi")),
    ("android_galaxy_a14_5g", "SM-A146B", "13", 8, (4, 6, 8), 360, 803, 3.0, "ARM", "Mali-G68", "valhall", "3.2", 7, ("phone", "mainstream", "samsung")),
    ("android_redmi_note_8_pro", "Redmi Note 8 Pro", "9", 8, (6, 8), 360, 780, 3.0, "ARM", "Mali-G76", "bifrost", "3.2", 6, ("phone", "mainstream", "xiaomi")),
    ("android_poco_x4_gt", "22041216G", "12", 8, (8,), 393, 895, 2.75, "ARM", "Mali-G610", "valhall", "3.2", 6, ("phone", "mainstream", "xiaomi")),
    ("android_galaxy_s22_exynos", "SM-S901B", "12", 8, (8,), 360, 780, 3.0, "Samsung", "Xclipse 920", "rdna-2", "3.2", 6, ("phone", "flagship", "samsung")),
    ("android_galaxy_s24_exynos", "SM-S921B", "14", 10, (8, 12), 360, 780, 3.0, "Samsung", "Xclipse 940", "rdna-3", "3.2", 7, ("phone", "flagship", "samsung")),
    ("android_nokia_1_plus", "Nokia 1 Plus", "9", 4, (1,), 320, 640, 1.5, "Imagination Technologies", "PowerVR GE8100", "rogue", "3.2", 4, ("phone", "entry", "aosp")),
    ("android_oppo_reno_z", "CPH1979", "9", 8, (6, 8), 360, 780, 3.0, "Imagination Technologies", "PowerVR GM9446", "rogue", "3.2", 4, ("phone", "mainstream", "oppo")),
)


ANDROID_EXTENDED_OS_RANGES: dict[str, tuple[int, int]] = {
    "android_moto_e6": (9, 10),
    "android_moto_g7_play": (9, 10),
    "android_xiaomi_mi_a2": (8, 10),
    "android_redmi_note_8": (9, 11),
    "android_redmi_note_7_pro": (9, 11),
    "android_redmi_12_5g": (13, 15),
    "android_galaxy_a52_5g": (11, 14),
    "android_galaxy_a52s": (11, 14),
    "android_xiaomi_13_lite": (13, 15),
    "android_oneplus_9": (11, 14),
    "android_poco_f5": (13, 15),
    "android_oneplus_10_pro": (12, 15),
    "android_redmi_9c": (10, 12),
    "android_galaxy_a12": (10, 13),
    "android_redmi_note_10_5g": (11, 13),
    "android_galaxy_a14_5g": (13, 15),
    "android_redmi_note_8_pro": (9, 12),
    "android_poco_x4_gt": (12, 15),
    "android_galaxy_s22_exynos": (12, 16),
    "android_galaxy_s24_exynos": (14, 16),
    "android_nokia_1_plus": (9, 10),
    "android_oppo_reno_z": (9, 11),
}


ANDROID_EXTENDED_OEMS: dict[str, str] = {
    key: (
        "samsung" if "galaxy" in key else
        "motorola" if "moto" in key else
        "oppo" if "oppo" in key else
        "oneplus" if "oneplus" in key else
        "aosp" if "nokia" in key else
        "xiaomi"
    )
    for key in ANDROID_EXTENDED_OS_RANGES
}


ANDROID_EXTENDED_GRAPHICS: dict[str, str] = {
    "android_moto_e6": "adreno-5xx",
    "android_moto_g7_play": "adreno-5xx",
    "android_xiaomi_mi_a2": "adreno-5xx",
    "android_redmi_note_8": "adreno-6xx-entry",
    "android_redmi_note_7_pro": "adreno-6xx-entry",
    "android_redmi_12_5g": "adreno-6xx-entry",
    "android_galaxy_a52_5g": "adreno-6xx-entry",
    "android_galaxy_a52s": "adreno-6xx-mainstream",
    "android_xiaomi_13_lite": "adreno-6xx-mainstream",
    "android_oneplus_9": "adreno-6xx-mainstream",
    "android_poco_f5": "adreno-7xx-flagship",
    "android_oneplus_10_pro": "adreno-7xx-flagship",
    "android_redmi_9c": "mali-bifrost-entry",
    "android_galaxy_a12": "mali-bifrost-entry",
    "android_redmi_note_10_5g": "mali-valhall-entry",
    "android_galaxy_a14_5g": "mali-valhall-entry",
    "android_redmi_note_8_pro": "mali-bifrost-entry",
    "android_poco_x4_gt": "mali-valhall-modern",
    "android_galaxy_s22_exynos": "xclipse-rdna2",
    "android_galaxy_s24_exynos": "xclipse-rdna3",
    "android_nokia_1_plus": "powervr-rogue",
    "android_oppo_reno_z": "powervr-rogue",
}


__all__ = [
    "ANDROID_EXTENDED_DEVICE_ROWS",
    "ANDROID_EXTENDED_GRAPHICS",
    "ANDROID_EXTENDED_OEMS",
    "ANDROID_EXTENDED_OS_RANGES",
]
