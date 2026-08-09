"""
Windows Chrome/ANGLE WebGL GPU catalog.

This module is intentionally separate from fingerprint_runtime_composer.py
because the Windows GPU catalog is large and will keep growing.

Data model:

- WINDOWS_GPU_MODEL_ROWS is the compact source table.
- WINDOWS_WEBGL_GPU_CANDIDATES is generated from that table at import time.
- Each candidate keeps the same variable paths as the local webgl_gpu template:
  gpu.adapter.* and webgl.unmaskedVendor/unmaskedRenderer.

The full webgl_gpu template contains large WebGL/WebGL2 parameter tables. Do not
duplicate those tables per GPU. Pick one candidate here, call
build_windows_webgl_gpu_patch(), then deep-merge the returned slice into a copy
of a captured webgl_gpu template.
"""

from __future__ import annotations

import random
from bisect import bisect_right
from itertools import accumulate
from typing import Iterable, Sequence

try:
    from .windows_pci_device_catalog import WINDOWS_GPU_DEVICE_VARIANTS
    from .nvidia_open_gpu_extension_catalog import (
        DAWN_GPU_INFO_SOURCE as NVIDIA_EXTENSION_DAWN_SOURCE,
        NVIDIA_OPEN_GPU_PRODUCT_ROWS,
        NVIDIA_OPEN_GPU_SOURCE,
    )
except ImportError:  # Support running this catalog directly from demo/fp.
    from windows_pci_device_catalog import WINDOWS_GPU_DEVICE_VARIANTS
    from nvidia_open_gpu_extension_catalog import (  # type: ignore
        DAWN_GPU_INFO_SOURCE as NVIDIA_EXTENSION_DAWN_SOURCE,
        NVIDIA_OPEN_GPU_PRODUCT_ROWS,
        NVIDIA_OPEN_GPU_SOURCE,
    )


WINDOWS_ANGLE_BACKEND = "Direct3D11 vs_5_0 ps_5_0, D3D11"

WEBGL_UNMASKED_VENDOR_BY_DRIVER_VENDOR: dict[str, str] = {
    "NVIDIA": "Google Inc. (NVIDIA)",
    "AMD": "Google Inc. (AMD)",
    "Intel": "Google Inc. (Intel)",
    "Qualcomm": "Google Inc. (Qualcomm)",
    "Microsoft": "Google Inc. (Microsoft)",
    "Google": "Google Inc. (Google)",
}

WEBGPU_ADAPTER_VENDOR_BY_DRIVER_VENDOR: dict[str, str] = {
    "NVIDIA": "nvidia",
    "AMD": "amd",
    "Intel": "intel",
    "Qualcomm": "qualcomm",
    "Microsoft": "microsoft",
    "Google": "google",
}


GPU_TIER_WEIGHTS: dict[str, int] = {
    "integrated": 18,
    "laptop": 16,
    "mainstream": 14,
    "entry": 10,
    "legacy": 7,
    "high": 6,
    "enthusiast": 3,
    "workstation": 1,
    "virtual": 1,
}

GPU_VENDOR_WEIGHTS: dict[str, int] = {
    "NVIDIA": 10,
    "Intel": 8,
    "AMD": 6,
    "Qualcomm": 2,
    "Microsoft": 1,
    "Google": 1,
}

COMMON_PC_GPU_MODEL_BOOSTS: tuple[tuple[str, float], ...] = (
    # Boost common Steam-observed gaming laptop/desktop rows.
    ("rtx 4060 laptop", 2.4),
    ("rtx 3060", 2.2),
    ("rtx 3050", 2.0),
    ("gtx 1650", 2.0),
    ("gtx 1060", 1.6),
    ("gtx 1050", 1.4),
    ("rtx 4070", 1.5),
    ("rtx 5060", 1.4),
    # Boost common office/laptop integrated rows.
    ("iris(r) xe", 2.0),
    ("iris xe graphics", 1.8),
    ("arc graphics", 1.5),
    ("uhd graphics", 1.8),
    ("hd graphics 520", 1.5),
    ("hd graphics 620", 1.5),
    ("radeon(tm) graphics", 1.8),
    ("radeon (tm) graphics", 1.8),
    ("radeon graphics", 1.6),
    ("vega 8", 1.5),
    ("rx 6600", 1.5),
    ("rx 580", 1.4),
)


# id, driver_vendor, architecture, tier, model, optional pci/device marker.
# Tiers are for weighted selection later, not a claim about benchmark ranking.
WINDOWS_GPU_MODEL_ROWS: tuple[tuple[str, str, str, str, str, str], ...] = (
    # NVIDIA Blackwell / RTX 50
    ("win_nvidia_rtx_5090", "NVIDIA", "blackwell", "enthusiast", "NVIDIA GeForce RTX 5090", "0x00002A01"),
    ("win_nvidia_rtx_5080", "NVIDIA", "blackwell", "enthusiast", "NVIDIA GeForce RTX 5080", ""),
    ("win_nvidia_rtx_5070_ti", "NVIDIA", "blackwell", "high", "NVIDIA GeForce RTX 5070 Ti", ""),
    ("win_nvidia_rtx_5070", "NVIDIA", "blackwell", "high", "NVIDIA GeForce RTX 5070", ""),
    ("win_nvidia_rtx_5060_ti", "NVIDIA", "blackwell", "mainstream", "NVIDIA GeForce RTX 5060 Ti", ""),
    ("win_nvidia_rtx_5060", "NVIDIA", "blackwell", "mainstream", "NVIDIA GeForce RTX 5060", ""),
    ("win_nvidia_rtx_5050", "NVIDIA", "blackwell", "entry", "NVIDIA GeForce RTX 5050", ""),
    ("win_nvidia_rtx_5090_laptop", "NVIDIA", "blackwell", "laptop", "NVIDIA GeForce RTX 5090 Laptop GPU", ""),
    ("win_nvidia_rtx_5080_laptop", "NVIDIA", "blackwell", "laptop", "NVIDIA GeForce RTX 5080 Laptop GPU", ""),
    ("win_nvidia_rtx_5070_ti_laptop", "NVIDIA", "blackwell", "laptop", "NVIDIA GeForce RTX 5070 Ti Laptop GPU", ""),
    ("win_nvidia_rtx_5070_laptop", "NVIDIA", "blackwell", "laptop", "NVIDIA GeForce RTX 5070 Laptop GPU", ""),
    ("win_nvidia_rtx_5060_laptop", "NVIDIA", "blackwell", "laptop", "NVIDIA GeForce RTX 5060 Laptop GPU", ""),
    ("win_nvidia_rtx_5050_laptop", "NVIDIA", "blackwell", "laptop", "NVIDIA GeForce RTX 5050 Laptop GPU", ""),

    # NVIDIA Ada / RTX 40
    ("win_nvidia_rtx_4090", "NVIDIA", "ada", "enthusiast", "NVIDIA GeForce RTX 4090", ""),
    ("win_nvidia_rtx_4080_super", "NVIDIA", "ada", "high", "NVIDIA GeForce RTX 4080 SUPER", ""),
    ("win_nvidia_rtx_4080", "NVIDIA", "ada", "high", "NVIDIA GeForce RTX 4080", ""),
    ("win_nvidia_rtx_4070_ti_super", "NVIDIA", "ada", "high", "NVIDIA GeForce RTX 4070 Ti SUPER", ""),
    ("win_nvidia_rtx_4070_ti", "NVIDIA", "ada", "high", "NVIDIA GeForce RTX 4070 Ti", ""),
    ("win_nvidia_rtx_4070_super", "NVIDIA", "ada", "high", "NVIDIA GeForce RTX 4070 SUPER", ""),
    ("win_nvidia_rtx_4070", "NVIDIA", "ada", "mainstream", "NVIDIA GeForce RTX 4070", ""),
    ("win_nvidia_rtx_4060_ti", "NVIDIA", "ada", "mainstream", "NVIDIA GeForce RTX 4060 Ti", ""),
    ("win_nvidia_rtx_4060", "NVIDIA", "ada", "mainstream", "NVIDIA GeForce RTX 4060", ""),
    ("win_nvidia_rtx_4090_laptop", "NVIDIA", "ada", "laptop", "NVIDIA GeForce RTX 4090 Laptop GPU", ""),
    ("win_nvidia_rtx_4080_laptop", "NVIDIA", "ada", "laptop", "NVIDIA GeForce RTX 4080 Laptop GPU", ""),
    ("win_nvidia_rtx_4070_laptop", "NVIDIA", "ada", "laptop", "NVIDIA GeForce RTX 4070 Laptop GPU", ""),
    ("win_nvidia_rtx_4060_laptop", "NVIDIA", "ada", "laptop", "NVIDIA GeForce RTX 4060 Laptop GPU", ""),
    ("win_nvidia_rtx_4050_laptop", "NVIDIA", "ada", "laptop", "NVIDIA GeForce RTX 4050 Laptop GPU", ""),

    # NVIDIA Ampere / RTX 30
    ("win_nvidia_rtx_3090_ti", "NVIDIA", "ampere", "enthusiast", "NVIDIA GeForce RTX 3090 Ti", ""),
    ("win_nvidia_rtx_3090", "NVIDIA", "ampere", "enthusiast", "NVIDIA GeForce RTX 3090", ""),
    ("win_nvidia_rtx_3080_ti", "NVIDIA", "ampere", "high", "NVIDIA GeForce RTX 3080 Ti", ""),
    ("win_nvidia_rtx_3080", "NVIDIA", "ampere", "high", "NVIDIA GeForce RTX 3080", ""),
    ("win_nvidia_rtx_3070_ti", "NVIDIA", "ampere", "high", "NVIDIA GeForce RTX 3070 Ti", ""),
    ("win_nvidia_rtx_3070", "NVIDIA", "ampere", "mainstream", "NVIDIA GeForce RTX 3070", ""),
    ("win_nvidia_rtx_3060_ti", "NVIDIA", "ampere", "mainstream", "NVIDIA GeForce RTX 3060 Ti", ""),
    ("win_nvidia_rtx_3060", "NVIDIA", "ampere", "mainstream", "NVIDIA GeForce RTX 3060", ""),
    ("win_nvidia_rtx_3050", "NVIDIA", "ampere", "entry", "NVIDIA GeForce RTX 3050", ""),
    ("win_nvidia_rtx_3080_ti_laptop", "NVIDIA", "ampere", "laptop", "NVIDIA GeForce RTX 3080 Ti Laptop GPU", ""),
    ("win_nvidia_rtx_3080_laptop", "NVIDIA", "ampere", "laptop", "NVIDIA GeForce RTX 3080 Laptop GPU", ""),
    ("win_nvidia_rtx_3070_ti_laptop", "NVIDIA", "ampere", "laptop", "NVIDIA GeForce RTX 3070 Ti Laptop GPU", ""),
    ("win_nvidia_rtx_3070_laptop", "NVIDIA", "ampere", "laptop", "NVIDIA GeForce RTX 3070 Laptop GPU", ""),
    ("win_nvidia_rtx_3060_laptop", "NVIDIA", "ampere", "laptop", "NVIDIA GeForce RTX 3060 Laptop GPU", ""),
    ("win_nvidia_rtx_3050_ti_laptop", "NVIDIA", "ampere", "laptop", "NVIDIA GeForce RTX 3050 Ti Laptop GPU", ""),
    ("win_nvidia_rtx_3050_6gb_laptop", "NVIDIA", "ampere", "laptop", "NVIDIA GeForce RTX 3050 6GB Laptop GPU", ""),
    ("win_nvidia_rtx_3050_laptop", "NVIDIA", "ampere", "laptop", "NVIDIA GeForce RTX 3050 Laptop GPU", ""),

    # NVIDIA Turing / RTX 20 + GTX 16
    ("win_nvidia_rtx_2080_ti", "NVIDIA", "turing", "high", "NVIDIA GeForce RTX 2080 Ti", ""),
    ("win_nvidia_rtx_2080_super", "NVIDIA", "turing", "high", "NVIDIA GeForce RTX 2080 SUPER", ""),
    ("win_nvidia_rtx_2080", "NVIDIA", "turing", "high", "NVIDIA GeForce RTX 2080", ""),
    ("win_nvidia_rtx_2070_super", "NVIDIA", "turing", "mainstream", "NVIDIA GeForce RTX 2070 SUPER", ""),
    ("win_nvidia_rtx_2070", "NVIDIA", "turing", "mainstream", "NVIDIA GeForce RTX 2070", ""),
    ("win_nvidia_rtx_2060_super", "NVIDIA", "turing", "mainstream", "NVIDIA GeForce RTX 2060 SUPER", ""),
    ("win_nvidia_rtx_2060", "NVIDIA", "turing", "mainstream", "NVIDIA GeForce RTX 2060", ""),
    ("win_nvidia_rtx_2080_laptop", "NVIDIA", "turing", "laptop", "NVIDIA GeForce RTX 2080 Laptop GPU", ""),
    ("win_nvidia_rtx_2070_laptop", "NVIDIA", "turing", "laptop", "NVIDIA GeForce RTX 2070 Laptop GPU", ""),
    ("win_nvidia_rtx_2060_laptop", "NVIDIA", "turing", "laptop", "NVIDIA GeForce RTX 2060 Laptop GPU", ""),
    ("win_nvidia_rtx_2050", "NVIDIA", "ampere", "laptop", "NVIDIA GeForce RTX 2050", ""),
    ("win_nvidia_gtx_1660_ti", "NVIDIA", "turing", "mainstream", "NVIDIA GeForce GTX 1660 Ti", ""),
    ("win_nvidia_gtx_1660_super", "NVIDIA", "turing", "mainstream", "NVIDIA GeForce GTX 1660 SUPER", ""),
    ("win_nvidia_gtx_1660", "NVIDIA", "turing", "mainstream", "NVIDIA GeForce GTX 1660", ""),
    ("win_nvidia_gtx_1650_super", "NVIDIA", "turing", "entry", "NVIDIA GeForce GTX 1650 SUPER", ""),
    ("win_nvidia_gtx_1650_ti", "NVIDIA", "turing", "entry", "NVIDIA GeForce GTX 1650 Ti", ""),
    ("win_nvidia_gtx_1650", "NVIDIA", "turing", "entry", "NVIDIA GeForce GTX 1650", ""),
    ("win_nvidia_gtx_1660_ti_laptop", "NVIDIA", "turing", "laptop", "NVIDIA GeForce GTX 1660 Ti Laptop GPU", ""),
    ("win_nvidia_gtx_1650_ti_laptop", "NVIDIA", "turing", "laptop", "NVIDIA GeForce GTX 1650 Ti Laptop GPU", ""),
    ("win_nvidia_gtx_1650_laptop", "NVIDIA", "turing", "laptop", "NVIDIA GeForce GTX 1650 Laptop GPU", ""),

    # NVIDIA Pascal / Maxwell still common in Windows fleets
    ("win_nvidia_gtx_1080_ti", "NVIDIA", "pascal", "high", "NVIDIA GeForce GTX 1080 Ti", ""),
    ("win_nvidia_gtx_1080", "NVIDIA", "pascal", "high", "NVIDIA GeForce GTX 1080", ""),
    ("win_nvidia_gtx_1070_ti", "NVIDIA", "pascal", "mainstream", "NVIDIA GeForce GTX 1070 Ti", ""),
    ("win_nvidia_gtx_1070", "NVIDIA", "pascal", "mainstream", "NVIDIA GeForce GTX 1070", ""),
    ("win_nvidia_gtx_1060", "NVIDIA", "pascal", "mainstream", "NVIDIA GeForce GTX 1060", ""),
    ("win_nvidia_gtx_1060_6gb", "NVIDIA", "pascal", "mainstream", "NVIDIA GeForce GTX 1060 6GB", ""),
    ("win_nvidia_gtx_1060_3gb", "NVIDIA", "pascal", "mainstream", "NVIDIA GeForce GTX 1060 3GB", ""),
    ("win_nvidia_gtx_1050_ti", "NVIDIA", "pascal", "entry", "NVIDIA GeForce GTX 1050 Ti", ""),
    ("win_nvidia_gtx_1050", "NVIDIA", "pascal", "entry", "NVIDIA GeForce GTX 1050", ""),
    ("win_nvidia_gtx_1050_ti_laptop", "NVIDIA", "pascal", "laptop", "NVIDIA GeForce GTX 1050 Ti Laptop GPU", ""),
    ("win_nvidia_gtx_1050_laptop", "NVIDIA", "pascal", "laptop", "NVIDIA GeForce GTX 1050 Laptop GPU", ""),
    ("win_nvidia_gt_1030", "NVIDIA", "pascal", "entry", "NVIDIA GeForce GT 1030", ""),
    ("win_nvidia_gtx_980_ti", "NVIDIA", "maxwell", "legacy", "NVIDIA GeForce GTX 980 Ti", ""),
    ("win_nvidia_gtx_970", "NVIDIA", "maxwell", "legacy", "NVIDIA GeForce GTX 970", ""),
    ("win_nvidia_gtx_960", "NVIDIA", "maxwell", "legacy", "NVIDIA GeForce GTX 960", "0x00001401"),
    ("win_nvidia_gtx_750_ti", "NVIDIA", "maxwell", "legacy", "NVIDIA GeForce GTX 750 Ti", ""),
    ("win_nvidia_gtx_750", "NVIDIA", "maxwell", "legacy", "NVIDIA GeForce GTX 750", ""),
    ("win_nvidia_gtx_780_ti", "NVIDIA", "kepler", "legacy", "NVIDIA GeForce GTX 780 Ti", ""),
    ("win_nvidia_gtx_780", "NVIDIA", "kepler", "legacy", "NVIDIA GeForce GTX 780", ""),
    ("win_nvidia_gtx_770", "NVIDIA", "kepler", "legacy", "NVIDIA GeForce GTX 770", ""),
    ("win_nvidia_gtx_760", "NVIDIA", "kepler", "legacy", "NVIDIA GeForce GTX 760", ""),
    ("win_nvidia_mx550", "NVIDIA", "turing", "entry", "NVIDIA GeForce MX550", ""),
    ("win_nvidia_mx450", "NVIDIA", "turing", "entry", "NVIDIA GeForce MX450", ""),
    ("win_nvidia_mx350", "NVIDIA", "pascal", "entry", "NVIDIA GeForce MX350", ""),
    ("win_nvidia_mx330", "NVIDIA", "pascal", "entry", "NVIDIA GeForce MX330", ""),
    ("win_nvidia_mx250", "NVIDIA", "pascal", "entry", "NVIDIA GeForce MX250", ""),
    ("win_nvidia_mx230", "NVIDIA", "pascal", "entry", "NVIDIA GeForce MX230", ""),
    ("win_nvidia_mx150", "NVIDIA", "pascal", "entry", "NVIDIA GeForce MX150", ""),
    ("win_nvidia_mx130", "NVIDIA", "maxwell", "entry", "NVIDIA GeForce MX130", ""),
    ("win_nvidia_940mx", "NVIDIA", "maxwell", "legacy", "NVIDIA GeForce 940MX", ""),
    ("win_nvidia_gt_730", "NVIDIA", "kepler", "legacy", "NVIDIA GeForce GT 730", ""),
    ("win_nvidia_gt_710", "NVIDIA", "kepler", "legacy", "NVIDIA GeForce GT 710", ""),

    # NVIDIA workstation / professional Windows systems
    ("win_nvidia_rtx_pro_6000_blackwell", "NVIDIA", "blackwell", "workstation", "NVIDIA RTX PRO 6000 Blackwell Workstation Edition", ""),
    ("win_nvidia_rtx_pro_5000_blackwell", "NVIDIA", "blackwell", "workstation", "NVIDIA RTX PRO 5000 Blackwell", ""),
    ("win_nvidia_rtx_pro_4500_blackwell", "NVIDIA", "blackwell", "workstation", "NVIDIA RTX PRO 4500 Blackwell Workstation Edition", ""),
    ("win_nvidia_rtx_pro_4000_blackwell", "NVIDIA", "blackwell", "workstation", "NVIDIA RTX PRO 4000 Blackwell", ""),
    ("win_nvidia_rtx_pro_2000_blackwell", "NVIDIA", "blackwell", "workstation", "NVIDIA RTX PRO 2000 Blackwell", ""),
    ("win_nvidia_rtx_6000_ada", "NVIDIA", "ada", "workstation", "NVIDIA RTX 6000 Ada Generation", ""),
    ("win_nvidia_rtx_5000_ada", "NVIDIA", "ada", "workstation", "NVIDIA RTX 5000 Ada Generation", ""),
    ("win_nvidia_rtx_4500_ada", "NVIDIA", "ada", "workstation", "NVIDIA RTX 4500 Ada Generation", ""),
    ("win_nvidia_rtx_4000_ada", "NVIDIA", "ada", "workstation", "NVIDIA RTX 4000 Ada Generation", ""),
    ("win_nvidia_rtx_2000_ada", "NVIDIA", "ada", "workstation", "NVIDIA RTX 2000 Ada Generation", ""),
    ("win_nvidia_rtx_a6000", "NVIDIA", "ampere", "workstation", "NVIDIA RTX A6000", ""),
    ("win_nvidia_rtx_a5500", "NVIDIA", "ampere", "workstation", "NVIDIA RTX A5500", ""),
    ("win_nvidia_rtx_a5000", "NVIDIA", "ampere", "workstation", "NVIDIA RTX A5000", ""),
    ("win_nvidia_rtx_a4500", "NVIDIA", "ampere", "workstation", "NVIDIA RTX A4500", ""),
    ("win_nvidia_rtx_a4000", "NVIDIA", "ampere", "workstation", "NVIDIA RTX A4000", ""),
    ("win_nvidia_rtx_a2000", "NVIDIA", "ampere", "workstation", "NVIDIA RTX A2000", ""),
    ("win_nvidia_quadro_rtx_5000", "NVIDIA", "turing", "workstation", "NVIDIA Quadro RTX 5000", ""),
    ("win_nvidia_t1000", "NVIDIA", "turing", "workstation", "NVIDIA T1000", ""),
    ("win_nvidia_t600", "NVIDIA", "turing", "workstation", "NVIDIA T600", ""),
    ("win_nvidia_t400", "NVIDIA", "turing", "workstation", "NVIDIA T400", ""),
    ("win_nvidia_quadro_p2200", "NVIDIA", "pascal", "workstation", "NVIDIA Quadro P2200", ""),
    ("win_nvidia_quadro_p1000", "NVIDIA", "pascal", "workstation", "NVIDIA Quadro P1000", ""),
    ("win_nvidia_quadro_p620", "NVIDIA", "pascal", "workstation", "NVIDIA Quadro P620", ""),

    # AMD RDNA 4 / RX 9000
    ("win_amd_rx_9070_xt", "AMD", "rdna4", "high", "AMD Radeon RX 9070 XT", ""),
    ("win_amd_rx_9070", "AMD", "rdna4", "high", "AMD Radeon RX 9070", ""),
    ("win_amd_rx_9070_gre", "AMD", "rdna4", "high", "AMD Radeon RX 9070 GRE", ""),
    ("win_amd_rx_9060_xt", "AMD", "rdna4", "mainstream", "AMD Radeon RX 9060 XT", ""),
    ("win_amd_rx_9060", "AMD", "rdna4", "mainstream", "AMD Radeon RX 9060", ""),

    # AMD RDNA 3 / RX 7000
    ("win_amd_rx_7900_xtx", "AMD", "rdna3", "enthusiast", "AMD Radeon RX 7900 XTX", ""),
    ("win_amd_rx_7900_xt", "AMD", "rdna3", "high", "AMD Radeon RX 7900 XT", ""),
    ("win_amd_rx_7900_gre", "AMD", "rdna3", "high", "AMD Radeon RX 7900 GRE", ""),
    ("win_amd_rx_7800_xt", "AMD", "rdna3", "high", "AMD Radeon RX 7800 XT", ""),
    ("win_amd_rx_7700_xt", "AMD", "rdna3", "mainstream", "AMD Radeon RX 7700 XT", ""),
    ("win_amd_rx_7700", "AMD", "rdna3", "mainstream", "AMD Radeon RX 7700", ""),
    ("win_amd_rx_7600_xt", "AMD", "rdna3", "mainstream", "AMD Radeon RX 7600 XT", ""),
    ("win_amd_rx_7600", "AMD", "rdna3", "mainstream", "AMD Radeon RX 7600", ""),
    ("win_amd_rx_7400", "AMD", "rdna3", "entry", "AMD Radeon RX 7400", ""),
    ("win_amd_rx_7900m", "AMD", "rdna3", "laptop", "AMD Radeon RX 7900M", ""),
    ("win_amd_rx_7800m", "AMD", "rdna3", "laptop", "AMD Radeon RX 7800M", ""),
    ("win_amd_rx_7700s", "AMD", "rdna3", "laptop", "AMD Radeon RX 7700S", ""),
    ("win_amd_rx_7600m_xt", "AMD", "rdna3", "laptop", "AMD Radeon RX 7600M XT", ""),
    ("win_amd_rx_7600m", "AMD", "rdna3", "laptop", "AMD Radeon RX 7600M", ""),
    ("win_amd_rx_7600s", "AMD", "rdna3", "laptop", "AMD Radeon RX 7600S", ""),

    # AMD RDNA 2 / RX 6000
    ("win_amd_rx_6950_xt", "AMD", "rdna2", "enthusiast", "AMD Radeon RX 6950 XT", ""),
    ("win_amd_rx_6900_xt", "AMD", "rdna2", "enthusiast", "AMD Radeon RX 6900 XT", ""),
    ("win_amd_rx_6800_xt", "AMD", "rdna2", "high", "AMD Radeon RX 6800 XT", ""),
    ("win_amd_rx_6800", "AMD", "rdna2", "high", "AMD Radeon RX 6800", ""),
    ("win_amd_rx_6750_xt", "AMD", "rdna2", "mainstream", "AMD Radeon RX 6750 XT", ""),
    ("win_amd_rx_6700_xt", "AMD", "rdna2", "mainstream", "AMD Radeon RX 6700 XT", ""),
    ("win_amd_rx_6650_xt", "AMD", "rdna2", "mainstream", "AMD Radeon RX 6650 XT", ""),
    ("win_amd_rx_6600_xt", "AMD", "rdna2", "mainstream", "AMD Radeon RX 6600 XT", ""),
    ("win_amd_rx_6600", "AMD", "rdna2", "mainstream", "AMD Radeon RX 6600", ""),
    ("win_amd_rx_6500_xt", "AMD", "rdna2", "entry", "AMD Radeon RX 6500 XT", ""),
    ("win_amd_rx_6400", "AMD", "rdna2", "entry", "AMD Radeon RX 6400", ""),
    ("win_amd_rx_6850m_xt", "AMD", "rdna2", "laptop", "AMD Radeon RX 6850M XT", ""),
    ("win_amd_rx_6800m", "AMD", "rdna2", "laptop", "AMD Radeon RX 6800M", ""),
    ("win_amd_rx_6700m", "AMD", "rdna2", "laptop", "AMD Radeon RX 6700M", ""),
    ("win_amd_rx_6650m", "AMD", "rdna2", "laptop", "AMD Radeon RX 6650M", ""),
    ("win_amd_rx_6600m", "AMD", "rdna2", "laptop", "AMD Radeon RX 6600M", ""),
    ("win_amd_rx_6500m", "AMD", "rdna2", "laptop", "AMD Radeon RX 6500M", ""),
    ("win_amd_rx_6300m", "AMD", "rdna2", "laptop", "AMD Radeon RX 6300M", ""),

    # AMD RDNA 1 / GCN legacy
    ("win_amd_rx_5700_xt", "AMD", "rdna1", "mainstream", "AMD Radeon RX 5700 XT", ""),
    ("win_amd_rx_5700", "AMD", "rdna1", "mainstream", "AMD Radeon RX 5700", ""),
    ("win_amd_rx_5700m", "AMD", "rdna1", "laptop", "AMD Radeon RX 5700M", ""),
    ("win_amd_rx_5600_xt", "AMD", "rdna1", "mainstream", "AMD Radeon RX 5600 XT", ""),
    ("win_amd_rx_5600m", "AMD", "rdna1", "laptop", "AMD Radeon RX 5600M", ""),
    ("win_amd_rx_5500_xt", "AMD", "rdna1", "entry", "AMD Radeon RX 5500 XT", ""),
    ("win_amd_rx_5500m", "AMD", "rdna1", "laptop", "AMD Radeon RX 5500M", ""),
    ("win_amd_rx_5300m", "AMD", "rdna1", "laptop", "AMD Radeon RX 5300M", ""),
    ("win_amd_radeon_vii", "AMD", "gcn5", "legacy", "AMD Radeon VII", ""),
    ("win_amd_rx_590", "AMD", "gcn4", "legacy", "AMD Radeon RX 590", ""),
    ("win_amd_rx_580", "AMD", "gcn4", "legacy", "AMD Radeon RX 580", ""),
    ("win_amd_rx_580_2048sp", "AMD", "gcn4", "legacy", "AMD Radeon RX 580 2048SP", ""),
    ("win_amd_rx_570", "AMD", "gcn4", "legacy", "AMD Radeon RX 570", ""),
    ("win_amd_rx_560", "AMD", "gcn4", "legacy", "AMD Radeon RX 560", ""),
    ("win_amd_rx_550", "AMD", "gcn4", "legacy", "AMD Radeon RX 550", ""),
    ("win_amd_r7_370", "AMD", "gcn1", "legacy", "AMD Radeon R7 370", ""),

    # AMD workstation / professional Windows systems.
    ("win_amd_radeon_ai_pro_r9700", "AMD", "rdna4", "workstation", "AMD Radeon AI PRO R9700", ""),
    ("win_amd_radeon_pro_w7900", "AMD", "rdna3", "workstation", "AMD Radeon PRO W7900", ""),
    ("win_amd_radeon_pro_w7800", "AMD", "rdna3", "workstation", "AMD Radeon PRO W7800", ""),
    ("win_amd_radeon_pro_w7700", "AMD", "rdna3", "workstation", "AMD Radeon PRO W7700", ""),
    ("win_amd_radeon_pro_w7600", "AMD", "rdna3", "workstation", "AMD Radeon PRO W7600", ""),
    ("win_amd_radeon_pro_w7500", "AMD", "rdna3", "workstation", "AMD Radeon PRO W7500", ""),
    ("win_amd_radeon_pro_w7400", "AMD", "rdna3", "workstation", "AMD Radeon PRO W7400", ""),
    ("win_amd_radeon_pro_w6800", "AMD", "rdna2", "workstation", "AMD Radeon PRO W6800", ""),
    ("win_amd_radeon_pro_w6600", "AMD", "rdna2", "workstation", "AMD Radeon PRO W6600", ""),
    ("win_amd_radeon_pro_w6400", "AMD", "rdna2", "workstation", "AMD Radeon PRO W6400", ""),
    ("win_amd_radeon_pro_w5700", "AMD", "rdna1", "workstation", "AMD Radeon Pro W5700", ""),
    ("win_amd_radeon_pro_wx_7100", "AMD", "gcn4", "workstation", "AMD Radeon Pro WX 7100", ""),

    # AMD integrated / mobile Windows laptops
    ("win_amd_radeon_890m", "AMD", "rdna3.5", "integrated", "AMD Radeon 890M Graphics", ""),
    ("win_amd_radeon_880m", "AMD", "rdna3.5", "integrated", "AMD Radeon 880M Graphics", ""),
    ("win_amd_radeon_860m", "AMD", "rdna3.5", "integrated", "AMD Radeon 860M Graphics", ""),
    ("win_amd_radeon_840m", "AMD", "rdna3.5", "integrated", "AMD Radeon 840M Graphics", ""),
    ("win_amd_radeon_780m", "AMD", "rdna3", "integrated", "AMD Radeon 780M Graphics", ""),
    ("win_amd_radeon_760m", "AMD", "rdna3", "integrated", "AMD Radeon 760M Graphics", ""),
    ("win_amd_radeon_740m", "AMD", "rdna3", "integrated", "AMD Radeon 740M Graphics", ""),
    ("win_amd_radeon_680m", "AMD", "rdna2", "integrated", "AMD Radeon 680M Graphics", ""),
    ("win_amd_radeon_660m", "AMD", "rdna2", "integrated", "AMD Radeon 660M Graphics", ""),
    ("win_amd_radeon_610m", "AMD", "rdna2", "integrated", "AMD Radeon 610M Graphics", ""),
    ("win_amd_raphael", "AMD", "rdna2", "integrated", "AMD Raphael", ""),
    ("win_amd_radeon_graphics_plain", "AMD", "vega", "integrated", "AMD Radeon Graphics", ""),
    ("win_amd_radeon_graphics_vega", "AMD", "vega", "integrated", "AMD Radeon(TM) Graphics", ""),
    ("win_amd_radeon_graphics_spaced", "AMD", "vega", "integrated", "AMD Radeon (TM) Graphics", ""),
    ("win_amd_vega_10", "AMD", "vega", "integrated", "AMD Radeon Vega 10 Graphics", ""),
    ("win_amd_vega_8", "AMD", "vega", "integrated", "AMD Radeon Vega 8 Graphics", ""),
    ("win_amd_vega_7", "AMD", "vega", "integrated", "AMD Radeon Vega 7 Graphics", ""),
    ("win_amd_vega_6", "AMD", "vega", "integrated", "AMD Radeon Vega 6 Graphics", ""),
    ("win_amd_vega_3", "AMD", "vega", "integrated", "AMD Radeon Vega 3 Graphics", ""),

    # Intel Arc Battlemage / Alchemist discrete
    ("win_intel_arc_b580", "Intel", "xe2-battlemage", "mainstream", "Intel(R) Arc(TM) B580 Graphics", ""),
    ("win_intel_arc_b570", "Intel", "xe2-battlemage", "mainstream", "Intel(R) Arc(TM) B570 Graphics", ""),
    ("win_intel_arc_pro_b60", "Intel", "xe2-battlemage", "workstation", "Intel(R) Arc(TM) Pro B60 Graphics", ""),
    ("win_intel_arc_pro_b50", "Intel", "xe2-battlemage", "workstation", "Intel(R) Arc(TM) Pro B50 Graphics", ""),
    ("win_intel_arc_a770", "Intel", "xe-hpg", "mainstream", "Intel(R) Arc(TM) A770 Graphics", ""),
    ("win_intel_arc_a750", "Intel", "xe-hpg", "mainstream", "Intel(R) Arc(TM) A750 Graphics", ""),
    ("win_intel_arc_a580", "Intel", "xe-hpg", "mainstream", "Intel(R) Arc(TM) A580 Graphics", ""),
    ("win_intel_arc_a380", "Intel", "xe-hpg", "entry", "Intel(R) Arc(TM) A380 Graphics", ""),
    ("win_intel_arc_a310", "Intel", "xe-hpg", "entry", "Intel(R) Arc(TM) A310 Graphics", ""),
    ("win_intel_arc_a770m", "Intel", "xe-hpg", "laptop", "Intel(R) Arc(TM) A770M Graphics", ""),
    ("win_intel_arc_a730m", "Intel", "xe-hpg", "laptop", "Intel(R) Arc(TM) A730M Graphics", ""),
    ("win_intel_arc_a570m", "Intel", "xe-hpg", "laptop", "Intel(R) Arc(TM) A570M Graphics", ""),
    ("win_intel_arc_a550m", "Intel", "xe-hpg", "laptop", "Intel(R) Arc(TM) A550M Graphics", ""),
    ("win_intel_arc_a530m", "Intel", "xe-hpg", "laptop", "Intel(R) Arc(TM) A530M Graphics", ""),
    ("win_intel_arc_a370m", "Intel", "xe-hpg", "laptop", "Intel(R) Arc(TM) A370M Graphics", ""),
    ("win_intel_arc_a350m", "Intel", "xe-hpg", "laptop", "Intel(R) Arc(TM) A350M Graphics", ""),
    ("win_intel_arc_pro_a60", "Intel", "xe-hpg", "workstation", "Intel(R) Arc(TM) Pro A60 Graphics", ""),
    ("win_intel_arc_pro_a50", "Intel", "xe-hpg", "workstation", "Intel(R) Arc(TM) Pro A50 Graphics", ""),
    ("win_intel_arc_pro_a40", "Intel", "xe-hpg", "workstation", "Intel(R) Arc(TM) Pro A40 Graphics", ""),
    ("win_intel_arc_pro_a30m", "Intel", "xe-hpg", "workstation", "Intel(R) Arc(TM) Pro A30M Graphics", ""),

    # Intel modern integrated GPUs
    ("win_intel_arc_140v", "Intel", "xe2", "integrated", "Intel(R) Arc(TM) 140V GPU", ""),
    ("win_intel_arc_130v", "Intel", "xe2", "integrated", "Intel(R) Arc(TM) 130V GPU", ""),
    ("win_intel_arc_140t", "Intel", "xe-lpg", "integrated", "Intel(R) Arc(TM) 140T GPU", ""),
    ("win_intel_arc_130t", "Intel", "xe-lpg", "integrated", "Intel(R) Arc(TM) 130T GPU", ""),
    ("win_intel_arc_graphics_plain", "Intel", "xe-lpg", "integrated", "Intel Arc Graphics", ""),
    ("win_intel_arc_graphics_lunar_lake", "Intel", "xe2", "integrated", "Intel(R) Arc(TM) Graphics", ""),
    ("win_intel_arc_graphics_meteor_lake", "Intel", "xe-lpg", "integrated", "Intel(R) Arc(TM) Graphics", ""),
    ("win_intel_graphics_generic", "Intel", "xe-lpg", "integrated", "Intel(R) Graphics", ""),
    # Iris Xe MAX shipped as a mobile discrete GPU; classifying it as generic
    # integrated graphics allowed it to pair with desktop-only hardware rows.
    ("win_intel_iris_xe_max", "Intel", "xe-lp", "laptop", "Intel(R) Iris(R) Xe MAX Graphics", ""),
    ("win_intel_iris_xe_plain", "Intel", "gen-12lp", "integrated", "Intel Iris Xe Graphics", ""),
    ("win_intel_iris_xe", "Intel", "gen-12lp", "integrated", "Intel(R) Iris(R) Xe Graphics", ""),
    ("win_intel_iris_plus", "Intel", "gen-11", "integrated", "Intel(R) Iris(R) Plus Graphics", ""),
    ("win_intel_iris_plus_g7", "Intel", "gen-11", "integrated", "Intel(R) Iris(R) Plus Graphics G7", ""),
    ("win_intel_iris_plus_g4", "Intel", "gen-11", "integrated", "Intel(R) Iris(R) Plus Graphics G4", ""),
    ("win_intel_iris_plus_655", "Intel", "gen-9.5", "integrated", "Intel(R) Iris(R) Plus Graphics 655", ""),
    ("win_intel_iris_plus_650", "Intel", "gen-9.5", "integrated", "Intel(R) Iris(R) Plus Graphics 650", ""),
    ("win_intel_iris_plus_640", "Intel", "gen-9.5", "integrated", "Intel(R) Iris(R) Plus Graphics 640", ""),
    ("win_intel_iris_550", "Intel", "gen-9", "integrated", "Intel(R) Iris(TM) Graphics 550", ""),
    ("win_intel_iris_540", "Intel", "gen-9", "integrated", "Intel(R) Iris(TM) Graphics 540", ""),
    ("win_intel_iris_6100", "Intel", "gen-8", "legacy", "Intel(R) Iris(TM) Graphics 6100", ""),
    ("win_intel_iris_pro_6200", "Intel", "gen-8", "legacy", "Intel(R) Iris(TM) Pro Graphics 6200", ""),
    ("win_intel_iris_pro_580", "Intel", "gen-9", "integrated", "Intel(R) Iris(R) Pro Graphics 580", ""),
    ("win_intel_uhd_graphics", "Intel", "gen-12", "integrated", "Intel(R) UHD Graphics", ""),
    ("win_intel_uhd_graphics_plain", "Intel", "gen-12", "integrated", "Intel UHD Graphics", ""),
    ("win_intel_uhd_770", "Intel", "gen-12", "integrated", "Intel(R) UHD Graphics 770", ""),
    ("win_intel_uhd_750", "Intel", "gen-12", "integrated", "Intel(R) UHD Graphics 750", ""),
    ("win_intel_uhd_730", "Intel", "gen-12", "integrated", "Intel(R) UHD Graphics 730", ""),
    ("win_intel_uhd_710", "Intel", "gen-12", "integrated", "Intel(R) UHD Graphics 710", ""),
    ("win_intel_uhd_630", "Intel", "gen-9.5", "integrated", "Intel(R) UHD Graphics 630", ""),
    ("win_intel_uhd_630_plain", "Intel", "gen-9.5", "integrated", "Intel UHD Graphics 630", ""),
    ("win_intel_uhd_620", "Intel", "gen-9.5", "integrated", "Intel(R) UHD Graphics 620", ""),
    ("win_intel_uhd_620_plain", "Intel", "gen-9.5", "integrated", "Intel UHD Graphics 620", ""),
    ("win_intel_uhd_617", "Intel", "gen-9.5", "integrated", "Intel(R) UHD Graphics 617", ""),
    ("win_intel_uhd_615", "Intel", "gen-9.5", "integrated", "Intel(R) UHD Graphics 615", ""),
    ("win_intel_uhd_610", "Intel", "gen-9.5", "integrated", "Intel(R) UHD Graphics 610", ""),
    ("win_intel_uhd_605", "Intel", "gen-9.5", "integrated", "Intel(R) UHD Graphics 605", ""),
    ("win_intel_uhd_600", "Intel", "gen-9.5", "integrated", "Intel(R) UHD Graphics 600", ""),
    ("win_intel_hd_630", "Intel", "gen-9.5", "legacy", "Intel(R) HD Graphics 630", ""),
    ("win_intel_hd_620", "Intel", "gen-9.5", "legacy", "Intel(R) HD Graphics 620", ""),
    ("win_intel_hd_620_plain", "Intel", "gen-9.5", "legacy", "Intel HD Graphics 620", ""),
    ("win_intel_hd_615", "Intel", "gen-9", "legacy", "Intel(R) HD Graphics 615", ""),
    ("win_intel_hd_610", "Intel", "gen-9.5", "legacy", "Intel(R) HD Graphics 610", ""),
    ("win_intel_hd_5600", "Intel", "gen-8", "legacy", "Intel(R) HD Graphics 5600", ""),
    ("win_intel_hd_5500", "Intel", "gen-8", "legacy", "Intel(R) HD Graphics 5500", ""),
    ("win_intel_hd_5300", "Intel", "gen-8", "legacy", "Intel(R) HD Graphics 5300", ""),
    ("win_intel_hd_530", "Intel", "gen-9", "legacy", "Intel(R) HD Graphics 530", ""),
    ("win_intel_hd_520", "Intel", "gen-9", "legacy", "Intel(R) HD Graphics 520", ""),
    ("win_intel_hd_520_plain", "Intel", "gen-9", "legacy", "Intel HD Graphics 520", ""),
    ("win_intel_hd_515", "Intel", "gen-9", "legacy", "Intel(R) HD Graphics 515", ""),
    ("win_intel_hd_510", "Intel", "gen-9", "legacy", "Intel(R) HD Graphics 510", ""),
    ("win_intel_hd_500", "Intel", "gen-9", "legacy", "Intel(R) HD Graphics 500", ""),
    ("win_intel_hd_6000", "Intel", "gen-8", "legacy", "Intel(R) HD Graphics 6000", ""),
    ("win_intel_hd_5000", "Intel", "gen-7.5", "legacy", "Intel(R) HD Graphics 5000", ""),
    ("win_intel_hd_4600", "Intel", "gen-7.5", "legacy", "Intel(R) HD Graphics 4600", ""),
    ("win_intel_hd_4400", "Intel", "gen-7.5", "legacy", "Intel(R) HD Graphics 4400", ""),
    ("win_intel_hd_4000", "Intel", "gen-7", "legacy", "Intel(R) HD Graphics 4000", ""),
    ("win_intel_hd_3000", "Intel", "gen-6", "legacy", "Intel(R) HD Graphics 3000", ""),

    # Qualcomm Windows on ARM laptops/desktops. Keep these as real Windows PC
    # GPUs, but the generator correlates them with ARM64 UA/hardware profiles
    # instead of mixing them into the default x64 Windows path.
    ("win_qualcomm_adreno_x1_45", "Qualcomm", "adreno-x1", "integrated", "Qualcomm(R) Adreno(TM) X1-45 GPU", ""),
    ("win_qualcomm_adreno_x1_85", "Qualcomm", "adreno-x1", "integrated", "Qualcomm(R) Adreno(TM) X1-85 GPU", ""),
    ("win_qualcomm_adreno_x2_45", "Qualcomm", "adreno-x2", "integrated", "Qualcomm(R) Adreno(TM) X2-45 GPU", ""),
    ("win_qualcomm_adreno_x2_85", "Qualcomm", "adreno-x2", "integrated", "Qualcomm(R) Adreno(TM) X2-85 GPU", ""),
    ("win_qualcomm_adreno_x2_90", "Qualcomm", "adreno-x2", "integrated", "Qualcomm(R) Adreno(TM) X2-90 GPU", ""),
    ("win_qualcomm_adreno_generic", "Qualcomm", "adreno", "integrated", "Qualcomm(R) Adreno(TM) GPU", ""),

    # Virtual/software paths seen on Windows automation, VM, or fallback setups.
    ("win_microsoft_basic_render", "Microsoft", "software", "virtual", "Microsoft Basic Render Driver", ""),
    ("win_google_swiftshader", "Google", "swiftshader", "virtual", "Google SwiftShader", ""),
)


def build_angle_renderer(driver_vendor: str, model: str, device_marker: str = "") -> str:
    marker = f" ({device_marker})" if device_marker else ""
    return f"ANGLE ({driver_vendor}, {model}{marker} {WINDOWS_ANGLE_BACKEND})"


_WINDOWS_MOBILE_INTEGRATED_GPU_IDS = frozenset((
    "win_intel_arc_140v",
    "win_intel_arc_130v",
    "win_intel_iris_xe_plain",
    "win_intel_iris_xe",
    "win_intel_iris_plus",
    "win_intel_iris_plus_g7",
    "win_intel_iris_plus_g4",
    "win_intel_iris_plus_655",
    "win_intel_iris_plus_650",
    "win_intel_iris_plus_640",
    "win_intel_iris_550",
    "win_intel_iris_540",
    "win_intel_uhd_620",
    "win_intel_uhd_620_plain",
    "win_intel_uhd_617",
    "win_intel_uhd_615",
    "win_intel_uhd_605",
    "win_intel_uhd_600",
))

_WINDOWS_DESKTOP_INTEGRATED_GPU_IDS = frozenset((
    "win_amd_raphael",
    "win_intel_uhd_770",
    "win_intel_uhd_750",
    "win_intel_uhd_730",
    "win_intel_uhd_710",
))

# The native sandbox currently models a successful requestAdapter() result.
# Keep adapters that cannot satisfy Dawn's Windows D3D12 minimum out of the
# generated pool rather than inventing a successful adapter for them. Intel's
# own API table lists HD 4000 as DirectX 11 and HD 5300 as DirectX 11.2;
# Dawn requires D3D12 feature level 11.1 (or 11.0 with binding tier 2).
_WINDOWS_WEBGPU_UNAVAILABLE_BASE_PROFILE_IDS = frozenset((
    "win_intel_hd_4000",
    "win_intel_hd_5300",
))


def _windows_gpu_form_factor(
    profile_id: str,
    driver_vendor: str,
    tier: str,
    model: str,
) -> str:
    """Classify only hardware whose product form factor is unambiguous."""

    model_key = model.lower()
    if tier == "laptop":
        return "portable"
    if driver_vendor == "Qualcomm":
        return "portable"
    if profile_id in _WINDOWS_MOBILE_INTEGRATED_GPU_IDS:
        return "portable"
    if "geforce mx" in model_key or model_key.endswith("mx"):
        return "portable"
    if profile_id in _WINDOWS_DESKTOP_INTEGRATED_GPU_IDS:
        return "desktop"
    if tier in {"entry", "mainstream", "high", "enthusiast", "workstation"}:
        return "desktop"
    return "mixed"


def build_windows_webgl_gpu_candidate(
    row: tuple[str, str, str, str, str, str],
    device_variant: tuple[str, str, str],
    *,
    variant_index: int,
    expose_device_id: bool,
    variant_count: int,
) -> dict[str, object]:
    item_id, driver_vendor, architecture, tier, model, _legacy_marker = row
    device_id, evidence_name, webgpu_architecture = device_variant
    canonical_device_id = f"0x{int(device_id, 16):08X}"
    device_marker = canonical_device_id if expose_device_id else ""
    if variant_index == 0 and expose_device_id:
        candidate_id = item_id
    else:
        renderer_shape = "id" if expose_device_id else "noid"
        candidate_id = f"{item_id}__pci_{device_id.lower()}_{renderer_shape}"
    adapter_vendor = WEBGPU_ADAPTER_VENDOR_BY_DRIVER_VENDOR[driver_vendor]
    unmasked_vendor = WEBGL_UNMASKED_VENDOR_BY_DRIVER_VENDOR[driver_vendor]
    renderer = build_angle_renderer(driver_vendor, model, device_marker)
    return {
        "id": candidate_id,
        "baseProfileId": item_id,
        "os": "windows",
        "backend": "angle-d3d11",
        "driverVendor": driver_vendor,
        "vendor": adapter_vendor,
        "architecture": architecture,
        "webgpuArchitecture": webgpu_architecture,
        "webgpuSupported": (
            item_id not in _WINDOWS_WEBGPU_UNAVAILABLE_BASE_PROFILE_IDS
        ),
        "tier": tier,
        "formFactor": _windows_gpu_form_factor(
            item_id,
            driver_vendor,
            tier,
            model,
        ),
        "model": model,
        "deviceMarker": device_marker,
        "deviceId": canonical_device_id,
        "rendererDeviceIdExposed": expose_device_id,
        "rendererVariantWeight": 0.75 if expose_device_id else 0.25,
        "deviceIdVariantCount": variant_count,
        "evidenceName": evidence_name,
        "evidenceSource": "https://pci-ids.ucw.cz/",
        "architectureSource": NVIDIA_EXTENSION_DAWN_SOURCE,
        "gpu": {
            "adapter": {
                "vendor": adapter_vendor,
                "architecture": webgpu_architecture,
                "device": "",
                "description": "",
            }
        },
        "webgl": {
            "unmaskedVendor": unmasked_vendor,
            "unmaskedRenderer": renderer,
        },
    }


_BASE_WINDOWS_WEBGL_GPU_CANDIDATES: tuple[dict[str, object], ...] = tuple(
    build_windows_webgl_gpu_candidate(
        row,
        device_variant,
        variant_index=variant_index,
        expose_device_id=expose_device_id,
        variant_count=len(WINDOWS_GPU_DEVICE_VARIANTS[row[0]]),
    )
    for row in WINDOWS_GPU_MODEL_ROWS
    if row[0] in WINDOWS_GPU_DEVICE_VARIANTS
    for variant_index, device_variant in enumerate(
        WINDOWS_GPU_DEVICE_VARIANTS[row[0]]
    )
    for expose_device_id in (True, False)
)


def _build_nvidia_open_gpu_extension_candidates() -> tuple[dict[str, object], ...]:
    """Append current NVIDIA evidence without modifying any existing row."""

    existing_pairs = {
        (
            str(candidate.get("model", "")).casefold(),
            str(candidate.get("deviceId", "")).upper(),
        )
        for candidate in _BASE_WINDOWS_WEBGL_GPU_CANDIDATES
    }
    output: list[dict[str, object]] = []
    for (
        profile_id,
        model,
        architecture,
        tier,
        form_factor,
        browser_eligible,
        device_ids,
    ) in NVIDIA_OPEN_GPU_PRODUCT_ROWS:
        if not browser_eligible:
            continue
        variants = tuple(
            (
                str(device_id),
                str(model),
                str(architecture),
            )
            for device_id in device_ids
            if (
                str(model).casefold(),
                f"0x{int(str(device_id), 16):08X}",
            ) not in existing_pairs
        )
        for variant_index, device_variant in enumerate(variants):
            for expose_device_id in (True, False):
                candidate = build_windows_webgl_gpu_candidate(
                    (
                        str(profile_id),
                        "NVIDIA",
                        str(architecture),
                        str(tier),
                        str(model),
                        "",
                    ),
                    device_variant,
                    variant_index=variant_index,
                    expose_device_id=expose_device_id,
                    variant_count=len(variants),
                )
                candidate["formFactor"] = str(form_factor)
                candidate["catalog"] = "nvidia-open-gpu-extension"
                candidate["evidenceSource"] = NVIDIA_OPEN_GPU_SOURCE
                candidate["architectureSource"] = NVIDIA_EXTENSION_DAWN_SOURCE
                output.append(candidate)
    return tuple(output)


NVIDIA_OPEN_GPU_EXTENSION_CANDIDATES = (
    _build_nvidia_open_gpu_extension_candidates()
)

# The existing catalog remains byte-for-byte first in the pool. New evidence
# is additive and de-duplicated by exact browser adapter name + Device ID.
WINDOWS_WEBGL_GPU_CANDIDATES: tuple[dict[str, object], ...] = (
    _BASE_WINDOWS_WEBGL_GPU_CANDIDATES
    + NVIDIA_OPEN_GPU_EXTENSION_CANDIDATES
)


def get_windows_webgl_gpu_candidate_weight(candidate: dict[str, object]) -> float:
    """Return a PC-realistic sampling weight without changing the candidate."""
    tier = str(candidate.get("tier", "") or "").lower()
    driver_vendor = str(candidate.get("driverVendor", "") or "")
    model = str(candidate.get("model", "") or "").lower()
    weight = GPU_TIER_WEIGHTS.get(tier, 4) * GPU_VENDOR_WEIGHTS.get(driver_vendor, 4)

    if "laptop gpu" in model:
        weight = int(weight * 1.2)
    for needle, multiplier in COMMON_PC_GPU_MODEL_BOOSTS:
        if needle in model:
            weight = int(weight * multiplier)
            break
    # Several real PCI IDs can map to the same browser-visible adapter name.
    # Divide the base model's probability among those variants so generic
    # Intel/AMD rows do not dominate merely because they have more IDs.
    variant_count = max(1, int(candidate.get("deviceIdVariantCount", 1)))
    renderer_weight = float(candidate.get("rendererVariantWeight", 1.0))
    return max(0.01, float(weight) * renderer_weight / variant_count)


def choose_weighted_windows_webgl_gpu_candidate(
    rng: random.Random,
    candidates: Sequence[dict[str, object]],
) -> dict[str, object]:
    if not candidates:
        raise ValueError("no Windows WebGL GPU candidates available")
    # Candidate catalogs are immutable, long-lived tuples. Build their
    # cumulative table once, then select in O(log n) instead of recalculating
    # 1,300+ weights on every profile request.
    cache_key = id(candidates)
    cached = _WEIGHTED_POOL_CACHE.get(cache_key)
    if cached is None or cached[0] is not candidates:
        cumulative = tuple(
            accumulate(
                get_windows_webgl_gpu_candidate_weight(item)
                for item in candidates
            )
        )
        cached = (candidates, cumulative)
        _WEIGHTED_POOL_CACHE[cache_key] = cached
    cumulative = cached[1]
    selected = bisect_right(cumulative, rng.random() * cumulative[-1])
    return candidates[min(selected, len(candidates) - 1)]


_WEIGHTED_POOL_CACHE: dict[
    int,
    tuple[Sequence[dict[str, object]], tuple[float, ...]],
] = {}


def get_windows_webgl_gpu_candidates(
    vendor: str | None = None,
    architecture: str | None = None,
    tier: str | None = None,
    include_virtual: bool = False,
    webgpu_supported_only: bool = False,
) -> tuple[dict[str, object], ...]:
    vendor_key = str(vendor or "").strip().lower()
    architecture_key = str(architecture or "").strip().lower()
    tier_key = str(tier or "").strip().lower()
    output = []
    for item in WINDOWS_WEBGL_GPU_CANDIDATES:
        if not include_virtual and str(item.get("tier", "")).lower() == "virtual":
            continue
        if webgpu_supported_only and not bool(item.get("webgpuSupported", False)):
            continue
        if vendor_key and str(item.get("vendor", "")).lower() != vendor_key:
            continue
        if architecture_key and str(item.get("architecture", "")).lower() != architecture_key:
            continue
        if tier_key and str(item.get("tier", "")).lower() != tier_key:
            continue
        output.append(item)
    return tuple(output)


def choose_windows_webgl_gpu_candidate(
    rng: random.Random,
    vendor: str | None = None,
    architecture: str | None = None,
    tier: str | None = None,
    include_virtual: bool = False,
    webgpu_supported_only: bool = False,
) -> dict[str, object]:
    candidates = get_windows_webgl_gpu_candidates(
        vendor=vendor,
        architecture=architecture,
        tier=tier,
        include_virtual=include_virtual,
        webgpu_supported_only=webgpu_supported_only,
    )
    if not candidates:
        candidates = get_windows_webgl_gpu_candidates(
            include_virtual=include_virtual,
            webgpu_supported_only=webgpu_supported_only,
        )
    if not candidates:
        raise ValueError("no Windows WebGL GPU candidates available")
    return choose_weighted_windows_webgl_gpu_candidate(rng, candidates)


def build_windows_webgl_gpu_patch(candidate: dict[str, object]) -> dict[str, object]:
    gpu = candidate.get("gpu")
    webgl = candidate.get("webgl")
    return {
        "gpu": gpu if isinstance(gpu, dict) else {},
        "webgl": webgl if isinstance(webgl, dict) else {},
        "webgl2": webgl if isinstance(webgl, dict) else {},
        "webglGpuId": candidate.get("id", ""),
        "webglGpuVendor": candidate.get("vendor", ""),
        "webglGpuArchitecture": candidate.get("architecture", ""),
        "webglGpuTier": candidate.get("tier", ""),
        "webglGpuModel": candidate.get("model", ""),
    }


def iter_windows_webgl_gpu_models(vendor: str | None = None) -> Iterable[str]:
    for item in get_windows_webgl_gpu_candidates(vendor=vendor, include_virtual=True):
        yield str(item.get("model", ""))


def count_windows_webgl_gpu_candidates() -> int:
    return len(WINDOWS_WEBGL_GPU_CANDIDATES)
