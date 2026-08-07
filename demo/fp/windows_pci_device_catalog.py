"""Evidence-backed Windows GPU PCI/DXGI device variants.

Generated from the public PCI ID Repository snapshot and Chromium Dawn's
``gpu_info.json`` architecture mapping on 2026-08-07. NVIDIA Turing-and-newer
IDs are cross-checkable against NVIDIA's open GPU kernel module table.

Sources:
- https://pci-ids.ucw.cz/
- https://github.com/NVIDIA/open-gpu-kernel-modules/blob/main/README.md
- https://dawn.googlesource.com/dawn/+/refs/heads/main/src/dawn/gpu_info.json

Only rows in this mapping enter the Windows random pool. This prevents ANGLE
D3D11 renderers from inventing an unknown device ID.
"""

from __future__ import annotations


# profile id -> ((device id, public evidence name, Dawn adapter architecture), ...)
WINDOWS_GPU_DEVICE_VARIANTS: dict[str, tuple[tuple[str, str, str], ...]] = {

    'win_amd_r7_370': (
        ('6600', 'Mars [Radeon HD 8670A/8670M/8750M / R7 M370]', 'gcn-1'),
        ('6810', 'Curacao XT / Trinidad XT [Radeon R7 370 / R9 270X/370X]', 'gcn-1'),
        ('6811', 'Curacao PRO [Radeon R7 370 / R9 270/370 OEM]', 'gcn-1'),
    ),
    'win_amd_radeon_680m': (
        ('1681', 'Rembrandt [Radeon 680M]', 'rdna-2'),
    ),
    'win_amd_radeon_840m': (
        ('1114', 'Krackan [Radeon 840M / 860M Graphics]', 'rdna-3'),
    ),
    'win_amd_radeon_860m': (
        ('1114', 'Krackan [Radeon 840M / 860M Graphics]', 'rdna-3'),
    ),
    'win_amd_radeon_880m': (
        ('150E', 'Strix [Radeon 880M / 890M]', 'rdna-3'),
    ),
    'win_amd_radeon_890m': (
        ('150E', 'Strix [Radeon 880M / 890M]', 'rdna-3'),
    ),
    'win_amd_radeon_ai_pro_r9700': (
        ('7551', 'Navi 48 [Radeon AI PRO R9700]', 'rdna-4'),
    ),
    'win_amd_radeon_pro_w5700': (
        ('7310', 'Navi 10 [Radeon Pro W5700X]', 'rdna-1'),
        ('7312', 'Navi 10 [Radeon Pro W5700]', 'rdna-1'),
        ('7319', 'Navi 10 [Radeon Pro 5700 XT]', 'rdna-1'),
        ('731B', 'Navi 10 [Radeon Pro 5700]', 'rdna-1'),
        ('731F', 'Navi 10 [Radeon RX 5600 OEM/5600 XT / 5700/5700 XT]', 'rdna-1'),
    ),
    'win_amd_radeon_pro_w6400': (
        ('7422', 'Navi 24 [Radeon PRO W6400]', 'rdna-2'),
        ('743F', 'Navi 24 [Radeon RX 6400/6500 XT/6500M]', 'rdna-2'),
    ),
    'win_amd_radeon_pro_w6600': (
        ('73E1', 'Navi 23 WKS-XM [Radeon PRO W6600M]', 'rdna-2'),
        ('73E3', 'Navi 23 WKS-XL [Radeon PRO W6600]', 'rdna-2'),
        ('73FF', 'Navi 23 [Radeon RX 6600/6600 XT/6600M]', 'rdna-2'),
    ),
    'win_amd_radeon_pro_w6800': (
        ('73A3', 'Navi 21 GL-XL [Radeon PRO W6800]', 'rdna-2'),
        ('73AB', 'Navi 21 Pro-XLA [Radeon Pro W6800X/Radeon Pro W6800X Duo]', 'rdna-2'),
        ('73BF', 'Navi 21 [Radeon RX 6800/6800 XT / 6900 XT]', 'rdna-2'),
    ),
    'win_amd_radeon_pro_w7500': (
        ('7489', 'Navi 33 [Radeon Pro W7500]', 'rdna-3'),
    ),
    'win_amd_radeon_pro_w7600': (
        ('7480', 'Navi 33 [Radeon RX 7600/7600 XT/7600M XT/7600S/7700S / PRO W7600]', 'rdna-3'),
    ),
    'win_amd_radeon_pro_w7700': (
        ('7470', 'Navi 32 [Radeon PRO W7700]', 'rdna-3'),
        ('747E', 'Navi 32 [Radeon RX 7700 XT / 7800 XT]', 'rdna-3'),
    ),
    'win_amd_radeon_pro_w7800': (
        ('7449', 'Navi 31 [Radeon Pro W7800 48GB]', 'rdna-3'),
        ('745E', 'Navi 31 [Radeon Pro W7800]', 'rdna-3'),
        ('747E', 'Navi 32 [Radeon RX 7700 XT / 7800 XT]', 'rdna-3'),
    ),
    'win_amd_radeon_pro_w7900': (
        ('7448', 'Navi 31 [Radeon Pro W7900]', 'rdna-3'),
        ('744A', 'Navi 31 [Radeon Pro W7900 Dual Slot]', 'rdna-3'),
        ('744B', 'Navi 31 [Radeon Pro W7900D]', 'rdna-3'),
        ('744C', 'Navi 31 [Radeon RX 7900 XT/7900 XTX/7900 GRE/7900M]', 'rdna-3'),
    ),
    'win_amd_radeon_pro_wx_7100': (
        ('67C0', 'Ellesmere [Radeon Pro WX 7100 Mobile]', 'gcn-4'),
        ('67C4', 'Ellesmere [Radeon Pro WX 7100]', 'gcn-4'),
        ('67D4', 'Ellesmere [Radeon Pro WX 7100 / Barco MXRT-8700]', 'gcn-4'),
    ),
    'win_amd_rx_5300m': (
        ('7340', 'Navi 14 [Radeon RX 5500/5500M / Pro 5300/5300M/5500M]', 'rdna-1'),
        ('734F', 'Navi 14 [Radeon Pro W5300M]', 'rdna-1'),
    ),
    'win_amd_rx_550': (
        ('67FF', 'Baffin [Radeon RX 550 640SP / RX 560/560X]', 'gcn-4'),
        ('6987', 'Lexa [Radeon 540X/550X/630 / RX 640 / E9171 MCM]', 'gcn-4'),
        ('699F', 'Lexa PRO [Radeon 540/540X/550/550X / RX 540X/550/550X]', 'gcn-4'),
    ),
    'win_amd_rx_5500_xt': (
        ('7340', 'Navi 14 [Radeon RX 5500/5500M / Pro 5300/5300M/5500M]', 'rdna-1'),
        ('7341', 'Navi 14 [Radeon Pro W5500]', 'rdna-1'),
        ('7347', 'Navi 14 [Radeon Pro W5500M]', 'rdna-1'),
    ),
    'win_amd_rx_5500m': (
        ('7340', 'Navi 14 [Radeon RX 5500/5500M / Pro 5300/5300M/5500M]', 'rdna-1'),
        ('7341', 'Navi 14 [Radeon Pro W5500]', 'rdna-1'),
        ('7347', 'Navi 14 [Radeon Pro W5500M]', 'rdna-1'),
    ),
    'win_amd_rx_560': (
        ('67EF', 'Baffin [Radeon RX 460/560D / Pro 450/455/460/555/555X/560/560X]', 'gcn-4'),
        ('67FF', 'Baffin [Radeon RX 550 640SP / RX 560/560X]', 'gcn-4'),
    ),
    'win_amd_rx_5600_xt': (
        ('731F', 'Navi 10 [Radeon RX 5600 OEM/5600 XT / 5700/5700 XT]', 'rdna-1'),
    ),
    'win_amd_rx_5600m': (
        ('731F', 'Navi 10 [Radeon RX 5600 OEM/5600 XT / 5700/5700 XT]', 'rdna-1'),
        ('7360', 'Navi 12 [Radeon Pro 5600M/V520/BC-160]', 'rdna-1'),
    ),
    'win_amd_rx_570': (
        ('67DF', 'Ellesmere [Radeon RX 470/480/570/570X/580/580X/590]', 'gcn-4'),
    ),
    'win_amd_rx_5700': (
        ('7310', 'Navi 10 [Radeon Pro W5700X]', 'rdna-1'),
        ('7312', 'Navi 10 [Radeon Pro W5700]', 'rdna-1'),
        ('7319', 'Navi 10 [Radeon Pro 5700 XT]', 'rdna-1'),
        ('731B', 'Navi 10 [Radeon Pro 5700]', 'rdna-1'),
        ('731F', 'Navi 10 [Radeon RX 5600 OEM/5600 XT / 5700/5700 XT]', 'rdna-1'),
    ),
    'win_amd_rx_5700_xt': (
        ('7310', 'Navi 10 [Radeon Pro W5700X]', 'rdna-1'),
        ('7312', 'Navi 10 [Radeon Pro W5700]', 'rdna-1'),
        ('7319', 'Navi 10 [Radeon Pro 5700 XT]', 'rdna-1'),
        ('731B', 'Navi 10 [Radeon Pro 5700]', 'rdna-1'),
        ('731F', 'Navi 10 [Radeon RX 5600 OEM/5600 XT / 5700/5700 XT]', 'rdna-1'),
    ),
    'win_amd_rx_5700m': (
        ('7310', 'Navi 10 [Radeon Pro W5700X]', 'rdna-1'),
        ('7312', 'Navi 10 [Radeon Pro W5700]', 'rdna-1'),
        ('7319', 'Navi 10 [Radeon Pro 5700 XT]', 'rdna-1'),
        ('731B', 'Navi 10 [Radeon Pro 5700]', 'rdna-1'),
        ('731F', 'Navi 10 [Radeon RX 5600 OEM/5600 XT / 5700/5700 XT]', 'rdna-1'),
    ),
    'win_amd_rx_580': (
        ('67DF', 'Ellesmere [Radeon RX 470/480/570/570X/580/580X/590]', 'gcn-4'),
        ('6FDF', 'Polaris 20 XL [Radeon RX 580 2048SP]', 'gcn-4'),
    ),
    'win_amd_rx_580_2048sp': (
        ('67DF', 'Ellesmere [Radeon RX 470/480/570/570X/580/580X/590]', 'gcn-4'),
        ('6FDF', 'Polaris 20 XL [Radeon RX 580 2048SP]', 'gcn-4'),
    ),

    'win_amd_rx_590': (
        ('67DF', 'Ellesmere [Radeon RX 470/480/570/570X/580/580X/590]', 'gcn-4'),
    ),
    'win_amd_rx_6300m': (
        ('7423', 'Navi 24 [Radeon PRO W6300/W6300M]', 'rdna-2'),
        ('7424', 'Navi 24 [Radeon RX 6300]', 'rdna-2'),
    ),
    'win_amd_rx_6400': (
        ('7422', 'Navi 24 [Radeon PRO W6400]', 'rdna-2'),
        ('743F', 'Navi 24 [Radeon RX 6400/6500 XT/6500M]', 'rdna-2'),
    ),
    'win_amd_rx_6500_xt': (
        ('7421', 'Navi 24 [Radeon PRO W6500M]', 'rdna-2'),
        ('743F', 'Navi 24 [Radeon RX 6400/6500 XT/6500M]', 'rdna-2'),
    ),
    'win_amd_rx_6500m': (
        ('7421', 'Navi 24 [Radeon PRO W6500M]', 'rdna-2'),
        ('743F', 'Navi 24 [Radeon RX 6400/6500 XT/6500M]', 'rdna-2'),
    ),
    'win_amd_rx_6600': (
        ('73E1', 'Navi 23 WKS-XM [Radeon PRO W6600M]', 'rdna-2'),
        ('73E3', 'Navi 23 WKS-XL [Radeon PRO W6600]', 'rdna-2'),
        ('73FF', 'Navi 23 [Radeon RX 6600/6600 XT/6600M]', 'rdna-2'),
    ),
    'win_amd_rx_6600_xt': (
        ('73E1', 'Navi 23 WKS-XM [Radeon PRO W6600M]', 'rdna-2'),
        ('73E3', 'Navi 23 WKS-XL [Radeon PRO W6600]', 'rdna-2'),
        ('73FF', 'Navi 23 [Radeon RX 6600/6600 XT/6600M]', 'rdna-2'),
    ),
    'win_amd_rx_6600m': (
        ('73E1', 'Navi 23 WKS-XM [Radeon PRO W6600M]', 'rdna-2'),
        ('73E3', 'Navi 23 WKS-XL [Radeon PRO W6600]', 'rdna-2'),
        ('73FF', 'Navi 23 [Radeon RX 6600/6600 XT/6600M]', 'rdna-2'),
    ),
    'win_amd_rx_6650_xt': (
        ('73EF', 'Navi 23 [Radeon RX 6650 XT / 6700S / 6800S]', 'rdna-2'),
    ),
    'win_amd_rx_6650m': (
        ('73EF', 'Navi 23 [Radeon RX 6650 XT / 6700S / 6800S]', 'rdna-2'),
    ),
    'win_amd_rx_6700_xt': (
        ('73DF', 'Navi 22 [Radeon RX 6700/6700 XT/6750 XT / 6800M/6850M XT]', 'rdna-2'),
    ),
    'win_amd_rx_6700m': (
        ('73DF', 'Navi 22 [Radeon RX 6700/6700 XT/6750 XT / 6800M/6850M XT]', 'rdna-2'),
        ('73EF', 'Navi 23 [Radeon RX 6650 XT / 6700S / 6800S]', 'rdna-2'),
    ),
    'win_amd_rx_6750_xt': (
        ('73DF', 'Navi 22 [Radeon RX 6700/6700 XT/6750 XT / 6800M/6850M XT]', 'rdna-2'),
    ),
    'win_amd_rx_6800': (
        ('73A3', 'Navi 21 GL-XL [Radeon PRO W6800]', 'rdna-2'),
        ('73AB', 'Navi 21 Pro-XLA [Radeon Pro W6800X/Radeon Pro W6800X Duo]', 'rdna-2'),
        ('73BF', 'Navi 21 [Radeon RX 6800/6800 XT / 6900 XT]', 'rdna-2'),
    ),
    'win_amd_rx_6800_xt': (
        ('73A3', 'Navi 21 GL-XL [Radeon PRO W6800]', 'rdna-2'),
        ('73AB', 'Navi 21 Pro-XLA [Radeon Pro W6800X/Radeon Pro W6800X Duo]', 'rdna-2'),
        ('73BF', 'Navi 21 [Radeon RX 6800/6800 XT / 6900 XT]', 'rdna-2'),
    ),
    'win_amd_rx_6800m': (
        ('73A3', 'Navi 21 GL-XL [Radeon PRO W6800]', 'rdna-2'),
        ('73AB', 'Navi 21 Pro-XLA [Radeon Pro W6800X/Radeon Pro W6800X Duo]', 'rdna-2'),
        ('73BF', 'Navi 21 [Radeon RX 6800/6800 XT / 6900 XT]', 'rdna-2'),
        ('73DF', 'Navi 22 [Radeon RX 6700/6700 XT/6750 XT / 6800M/6850M XT]', 'rdna-2'),
        ('73EF', 'Navi 23 [Radeon RX 6650 XT / 6700S / 6800S]', 'rdna-2'),
    ),
    'win_amd_rx_6850m_xt': (
        ('73DF', 'Navi 22 [Radeon RX 6700/6700 XT/6750 XT / 6800M/6850M XT]', 'rdna-2'),
    ),
    'win_amd_rx_6900_xt': (
        ('73A2', 'Navi 21 Pro-XTA [Radeon Pro W6900X]', 'rdna-2'),
        ('73AF', 'Navi 21 [Radeon RX 6900 XT]', 'rdna-2'),
        ('73BF', 'Navi 21 [Radeon RX 6800/6800 XT / 6900 XT]', 'rdna-2'),
    ),
    'win_amd_rx_6950_xt': (
        ('73A5', 'Navi 21 [Radeon RX 6950 XT]', 'rdna-2'),
    ),
    'win_amd_rx_7600': (
        ('7480', 'Navi 33 [Radeon RX 7600/7600 XT/7600M XT/7600S/7700S / PRO W7600]', 'rdna-3'),
    ),
    'win_amd_rx_7600_xt': (
        ('7480', 'Navi 33 [Radeon RX 7600/7600 XT/7600M XT/7600S/7700S / PRO W7600]', 'rdna-3'),
    ),
    'win_amd_rx_7600m': (
        ('7480', 'Navi 33 [Radeon RX 7600/7600 XT/7600M XT/7600S/7700S / PRO W7600]', 'rdna-3'),
        ('7483', 'Navi 33 [Radeon RX 7600M/7600M XT]', 'rdna-3'),
    ),
    'win_amd_rx_7600m_xt': (
        ('7480', 'Navi 33 [Radeon RX 7600/7600 XT/7600M XT/7600S/7700S / PRO W7600]', 'rdna-3'),
        ('7483', 'Navi 33 [Radeon RX 7600M/7600M XT]', 'rdna-3'),
    ),
    'win_amd_rx_7600s': (
        ('7480', 'Navi 33 [Radeon RX 7600/7600 XT/7600M XT/7600S/7700S / PRO W7600]', 'rdna-3'),
        ('7483', 'Navi 33 [Radeon RX 7600M/7600M XT]', 'rdna-3'),
    ),
    'win_amd_rx_7700': (
        ('7470', 'Navi 32 [Radeon PRO W7700]', 'rdna-3'),
        ('747E', 'Navi 32 [Radeon RX 7700 XT / 7800 XT]', 'rdna-3'),
    ),
    'win_amd_rx_7700_xt': (
        ('7470', 'Navi 32 [Radeon PRO W7700]', 'rdna-3'),
        ('747E', 'Navi 32 [Radeon RX 7700 XT / 7800 XT]', 'rdna-3'),
    ),
    'win_amd_rx_7700s': (
        ('7470', 'Navi 32 [Radeon PRO W7700]', 'rdna-3'),
        ('747E', 'Navi 32 [Radeon RX 7700 XT / 7800 XT]', 'rdna-3'),
        ('7480', 'Navi 33 [Radeon RX 7600/7600 XT/7600M XT/7600S/7700S / PRO W7600]', 'rdna-3'),
    ),
    'win_amd_rx_7800_xt': (
        ('7449', 'Navi 31 [Radeon Pro W7800 48GB]', 'rdna-3'),
        ('745E', 'Navi 31 [Radeon Pro W7800]', 'rdna-3'),
        ('747E', 'Navi 32 [Radeon RX 7700 XT / 7800 XT]', 'rdna-3'),
    ),
    'win_amd_rx_7800m': (
        ('7449', 'Navi 31 [Radeon Pro W7800 48GB]', 'rdna-3'),
        ('745E', 'Navi 31 [Radeon Pro W7800]', 'rdna-3'),
        ('747E', 'Navi 32 [Radeon RX 7700 XT / 7800 XT]', 'rdna-3'),
    ),
    'win_amd_rx_7900_gre': (
        ('7448', 'Navi 31 [Radeon Pro W7900]', 'rdna-3'),
        ('744A', 'Navi 31 [Radeon Pro W7900 Dual Slot]', 'rdna-3'),
        ('744B', 'Navi 31 [Radeon Pro W7900D]', 'rdna-3'),
        ('744C', 'Navi 31 [Radeon RX 7900 XT/7900 XTX/7900 GRE/7900M]', 'rdna-3'),
    ),

    'win_amd_rx_7900_xt': (
        ('7448', 'Navi 31 [Radeon Pro W7900]', 'rdna-3'),
        ('744A', 'Navi 31 [Radeon Pro W7900 Dual Slot]', 'rdna-3'),
        ('744B', 'Navi 31 [Radeon Pro W7900D]', 'rdna-3'),
        ('744C', 'Navi 31 [Radeon RX 7900 XT/7900 XTX/7900 GRE/7900M]', 'rdna-3'),
    ),
    'win_amd_rx_7900_xtx': (
        ('7448', 'Navi 31 [Radeon Pro W7900]', 'rdna-3'),
        ('744A', 'Navi 31 [Radeon Pro W7900 Dual Slot]', 'rdna-3'),
        ('744B', 'Navi 31 [Radeon Pro W7900D]', 'rdna-3'),
        ('744C', 'Navi 31 [Radeon RX 7900 XT/7900 XTX/7900 GRE/7900M]', 'rdna-3'),
    ),
    'win_amd_rx_7900m': (
        ('7448', 'Navi 31 [Radeon Pro W7900]', 'rdna-3'),
        ('744A', 'Navi 31 [Radeon Pro W7900 Dual Slot]', 'rdna-3'),
        ('744B', 'Navi 31 [Radeon Pro W7900D]', 'rdna-3'),
        ('744C', 'Navi 31 [Radeon RX 7900 XT/7900 XTX/7900 GRE/7900M]', 'rdna-3'),
    ),
    'win_amd_rx_9060': (
        ('7590', 'Navi 44 [Radeon RX 9050 / 9060 XT]', 'rdna-4'),
    ),
    'win_amd_rx_9060_xt': (
        ('7590', 'Navi 44 [Radeon RX 9050 / 9060 XT]', 'rdna-4'),
    ),
    'win_amd_rx_9070': (
        ('7550', 'Navi 48 [Radeon RX 9070/9070 XT/9070 GRE]', 'rdna-4'),
    ),
    'win_amd_rx_9070_gre': (
        ('7550', 'Navi 48 [Radeon RX 9070/9070 XT/9070 GRE]', 'rdna-4'),
    ),
    'win_amd_rx_9070_xt': (
        ('7550', 'Navi 48 [Radeon RX 9070/9070 XT/9070 GRE]', 'rdna-4'),
    ),
    'win_google_swiftshader': (
        ('C0DE', 'Google SwiftShader', 'swiftshader'),
    ),
    'win_intel_arc_130t': (
        ('7D51', 'Arrow Lake-P [Arc Pro 130T/140T]', 'xe-lpg'),
    ),
    'win_intel_arc_130v': (
        ('64A0', 'Core Ultra 200V Series Processors Arc Graphics 130V/140V GPU', 'xe-2-lpg'),
    ),
    'win_intel_arc_140t': (
        ('7D51', 'Arrow Lake-P [Arc Pro 130T/140T]', 'xe-lpg'),
    ),
    'win_intel_arc_140v': (
        ('64A0', 'Core Ultra 200V Series Processors Arc Graphics 130V/140V GPU', 'xe-2-lpg'),
    ),
    'win_intel_arc_a310': (
        ('56A6', 'DG2 [Arc A310]', 'gen-12-hp'),
    ),
    'win_intel_arc_a350m': (
        ('5694', 'DG2 [Arc A350M]', 'gen-12-hp'),
    ),
    'win_intel_arc_a370m': (
        ('5693', 'DG2 [Arc A370M]', 'gen-12-hp'),
    ),
    'win_intel_arc_a380': (
        ('56A5', 'DG2 [Arc A380]', 'gen-12-hp'),
    ),
    'win_intel_arc_a530m': (
        ('5697', 'DG2 [Arc A530M]', 'gen-12-hp'),
    ),
    'win_intel_arc_a550m': (
        ('5692', 'DG2 [Arc A550M]', 'gen-12-hp'),
    ),
    'win_intel_arc_a570m': (
        ('5696', 'DG2 [Arc A570M]', 'gen-12-hp'),
    ),
    'win_intel_arc_a580': (
        ('56A2', 'DG2 [Arc A580]', 'gen-12-hp'),
    ),
    'win_intel_arc_a730m': (
        ('5691', 'DG2 [Arc A730M]', 'gen-12-hp'),
    ),
    'win_intel_arc_a750': (
        ('56A1', 'DG2 [Arc A750]', 'gen-12-hp'),
    ),
    'win_intel_arc_a770': (
        ('56A0', 'DG2 [Arc A770]', 'gen-12-hp'),
    ),
    'win_intel_arc_a770m': (
        ('5690', 'DG2 [Arc A770M]', 'gen-12-hp'),
    ),
    'win_intel_arc_b570': (
        ('E20C', 'Battlemage G21 [Arc B570]', 'xe-2-hpg'),
    ),
    'win_intel_arc_b580': (
        ('E20B', 'Battlemage G21 [Arc B580]', 'xe-2-hpg'),
    ),
    'win_intel_arc_graphics_lunar_lake': (
        ('6420', 'Lunar Lake [Intel Graphics]', 'xe-2-lpg'),
        ('64A0', 'Core Ultra 200V Series Processors Arc Graphics 130V/140V GPU', 'xe-2-lpg'),
        ('64B0', 'Lunar Lake [Intel Graphics]', 'xe-2-lpg'),
    ),
    'win_intel_arc_graphics_meteor_lake': (
        ('7D40', 'Meteor Lake-M [Intel Graphics]', 'xe-lpg'),
        ('7D41', 'Arrow Lake-U [Intel Graphics]', 'xe-lpg'),
        ('7D45', 'Meteor Lake-P [Intel Graphics]', 'xe-lpg'),
        ('7D51', 'Arrow Lake-P [Arc Pro 130T/140T]', 'xe-lpg'),
        ('7D55', 'Meteor Lake-P [Intel Arc Graphics]', 'xe-lpg'),
        ('7D60', 'Meteor Lake-M [Intel Graphics]', 'xe-lpg'),
    ),
    'win_intel_arc_graphics_plain': (
        ('7D40', 'Meteor Lake-M [Intel Graphics]', 'xe-lpg'),
        ('7D41', 'Arrow Lake-U [Intel Graphics]', 'xe-lpg'),
        ('7D45', 'Meteor Lake-P [Intel Graphics]', 'xe-lpg'),
        ('7D51', 'Arrow Lake-P [Arc Pro 130T/140T]', 'xe-lpg'),
        ('7D55', 'Meteor Lake-P [Intel Arc Graphics]', 'xe-lpg'),
        ('7D60', 'Meteor Lake-M [Intel Graphics]', 'xe-lpg'),
    ),

    'win_intel_arc_pro_a30m': (
        ('56B0', 'DG2 [Arc Pro A30M]', 'gen-12-hp'),
    ),
    'win_intel_arc_pro_a40': (
        ('56B1', 'DG2 [Arc Pro A40/A50]', 'gen-12-hp'),
    ),
    'win_intel_arc_pro_a50': (
        ('56B1', 'DG2 [Arc Pro A40/A50]', 'gen-12-hp'),
    ),
    'win_intel_arc_pro_a60': (
        ('56B3', 'DG2 [Arc Pro A60]', 'gen-12-hp'),
    ),
    'win_intel_arc_pro_b50': (
        ('E212', 'Battlemage G21 [Arc Pro B50]', 'xe-2-hpg'),
    ),
    'win_intel_arc_pro_b60': (
        ('E211', 'Battlemage G21 [Arc Pro B60]', 'xe-2-hpg'),
    ),
    'win_intel_graphics_generic': (
        ('7D40', 'Meteor Lake-M [Intel Graphics]', 'xe-lpg'),
        ('7D41', 'Arrow Lake-U [Intel Graphics]', 'xe-lpg'),
        ('7D45', 'Meteor Lake-P [Intel Graphics]', 'xe-lpg'),
        ('7D51', 'Arrow Lake-P [Arc Pro 130T/140T]', 'xe-lpg'),
        ('7D55', 'Meteor Lake-P [Intel Arc Graphics]', 'xe-lpg'),
        ('7D60', 'Meteor Lake-M [Intel Graphics]', 'xe-lpg'),
    ),
    'win_intel_hd_4000': (
        ('0162', 'IvyBridge GT2 [HD Graphics 4000]', 'gen-7'),
        ('0166', 'Ivy Bridge mobile GT2 [HD Graphics 4000]', 'gen-7'),
    ),
    'win_intel_hd_500': (
        ('5A85', 'Apollo Lake GT1 [HD Graphics 500]', 'gen-9'),
    ),
    'win_intel_hd_510': (
        ('1902', 'Skylake-S GT1 [HD Graphics 510]', 'gen-9'),
        ('1906', 'Skylake-U GT1 [HD Graphics 510]', 'gen-9'),
        ('190B', 'HD Graphics 510', 'gen-9'),
    ),
    'win_intel_hd_515': (
        ('191E', 'Skylake-Y GT2 [HD Graphics 515]', 'gen-9'),
    ),
    'win_intel_hd_520': (
        ('1916', 'Skylake-U GT2 [HD Graphics 520]', 'gen-9'),
        ('1921', 'HD Graphics 520', 'gen-9'),
    ),
    'win_intel_hd_520_plain': (
        ('1916', 'Skylake-U GT2 [HD Graphics 520]', 'gen-9'),
        ('1921', 'HD Graphics 520', 'gen-9'),
    ),
    'win_intel_hd_530': (
        ('1912', 'Skylake-S GT2 [HD Graphics 530]', 'gen-9'),
        ('191B', 'Skylake-H GT2 [HD Graphics 530]', 'gen-9'),
        ('191D', 'Skylake-DT/H GT2 [HD Graphics P530]', 'gen-9'),
    ),
    'win_intel_hd_5300': (
        ('161E', 'Broadwell-Y GT2 [HD Graphics 5300]', 'gen-8'),
    ),
    'win_intel_hd_5500': (
        ('1616', 'Broadwell-U GT2 [HD Graphics 5500]', 'gen-8'),
    ),
    'win_intel_hd_5600': (
        ('1612', 'Broadwell-H GT2 [HD Graphics 5600]', 'gen-8'),
    ),
    'win_intel_hd_6000': (
        ('1626', 'Broadwell-U GT3 [HD Graphics 6000]', 'gen-8'),
    ),
    'win_intel_hd_610': (
        ('3E90', 'CoffeeLake-S GT1 [UHD Graphics 610]', 'gen-9'),
        ('3E93', 'CoffeeLake-S GT1 [UHD Graphics 610]', 'gen-9'),
        ('3E9C', 'Coffee Lake-S GT1 [UHD Graphics 610]', 'gen-9'),
        ('3EA1', 'Whiskey Lake-U GT1 [UHD Graphics 610]', 'gen-9'),
        ('5902', 'Kaby Lake-S GT1 [HD Graphics 610]', 'gen-9'),
        ('5906', 'Kaby Lake-U GT1 [HD Graphics 610]', 'gen-9'),
    ),
    'win_intel_hd_615': (
        ('591C', 'UHD Graphics 615', 'gen-9'),
        ('591E', 'Kaby Lake-Y GT2 [HD Graphics 615]', 'gen-9'),
    ),
    'win_intel_hd_620': (
        ('3EA0', 'WhiskeyLake-U GT2 [UHD Graphics 620]', 'gen-9'),
        ('3EA9', 'Coffee Lake-U GT2 [UHD Graphics 620]', 'gen-9'),
        ('5916', 'Kaby Lake-U GT2 [HD Graphics 620]', 'gen-9'),
        ('5917', 'Kaby Lake-R GT2 [UHD Graphics 620]', 'gen-9'),
        ('5921', 'HD Graphics 620', 'gen-9'),
        ('9B21', 'Comet Lake-U GT2 [UHD Graphics 620]', 'gen-9'),
    ),
    'win_intel_hd_620_plain': (
        ('3EA0', 'WhiskeyLake-U GT2 [UHD Graphics 620]', 'gen-9'),
        ('3EA9', 'Coffee Lake-U GT2 [UHD Graphics 620]', 'gen-9'),
        ('5916', 'Kaby Lake-U GT2 [HD Graphics 620]', 'gen-9'),
        ('5917', 'Kaby Lake-R GT2 [UHD Graphics 620]', 'gen-9'),
        ('5921', 'HD Graphics 620', 'gen-9'),
        ('9B21', 'Comet Lake-U GT2 [UHD Graphics 620]', 'gen-9'),
    ),
    'win_intel_hd_630': (
        ('3E91', 'CoffeeLake-S GT2 [UHD Graphics 630]', 'gen-9'),
        ('3E92', 'CoffeeLake-S GT2 [UHD Graphics 630]', 'gen-9'),
        ('3E94', 'Coffee Lake-S GT2 [UHD Graphics P630]', 'gen-9'),
        ('3E96', 'CoffeeLake-S GT2 [UHD Graphics P630]', 'gen-9'),
        ('3E98', 'CoffeeLake-S GT2 [UHD Graphics 630]', 'gen-9'),
        ('3E9A', 'Coffee Lake-S GT2 [UHD Graphics P630]', 'gen-9'),
    ),
    'win_intel_iris_540': (
        ('1926', 'Skylake-U GT3 [Iris Graphics 540]', 'gen-9'),
    ),
    'win_intel_iris_550': (
        ('1927', 'Skylake-U GT3 [Iris Graphics 550]', 'gen-9'),
    ),
    'win_intel_iris_6100': (
        ('162B', 'Broadwell-U GT3 [Iris Graphics 6100]', 'gen-8'),
    ),
    'win_intel_iris_plus': (
        ('4555', 'Elkhart Lake [UHD Graphics Gen11 16EU]', 'gen-11'),
        ('4571', 'Elkhart Lake [UHD Graphics Gen11 32EU]', 'gen-11'),
        ('4E55', 'JasperLake [UHD Graphics]', 'gen-11'),
        ('4E61', 'JasperLake [UHD Graphics]', 'gen-11'),
        ('4E71', 'JasperLake [UHD Graphics]', 'gen-11'),
        ('8A51', 'Iris Plus Graphics G7 (Ice Lake)', 'gen-11'),
    ),
    'win_intel_iris_plus_640': (
        ('5926', 'Kaby Lake-U GT3 [Iris Plus Graphics 640]', 'gen-9'),
    ),
    'win_intel_iris_plus_650': (
        ('5927', 'Kaby Lake-U GT3 [Iris Plus Graphics 650]', 'gen-9'),
    ),
    'win_intel_iris_plus_655': (
        ('3EA5', 'CoffeeLake-U GT3e [Iris Plus Graphics 655]', 'gen-9'),
        ('3EA8', 'Coffee Lake-U GT3 [Iris Plus Graphics 655]', 'gen-9'),
    ),

    'win_intel_iris_plus_g4': (
        ('4555', 'Elkhart Lake [UHD Graphics Gen11 16EU]', 'gen-11'),
        ('4571', 'Elkhart Lake [UHD Graphics Gen11 32EU]', 'gen-11'),
        ('4E55', 'JasperLake [UHD Graphics]', 'gen-11'),
        ('4E61', 'JasperLake [UHD Graphics]', 'gen-11'),
        ('4E71', 'JasperLake [UHD Graphics]', 'gen-11'),
        ('8A51', 'Iris Plus Graphics G7 (Ice Lake)', 'gen-11'),
    ),
    'win_intel_iris_plus_g7': (
        ('4555', 'Elkhart Lake [UHD Graphics Gen11 16EU]', 'gen-11'),
        ('4571', 'Elkhart Lake [UHD Graphics Gen11 32EU]', 'gen-11'),
        ('4E55', 'JasperLake [UHD Graphics]', 'gen-11'),
        ('4E61', 'JasperLake [UHD Graphics]', 'gen-11'),
        ('4E71', 'JasperLake [UHD Graphics]', 'gen-11'),
        ('8A51', 'Iris Plus Graphics G7 (Ice Lake)', 'gen-11'),
    ),
    'win_intel_iris_pro_580': (
        ('1932', 'Iris Pro Graphics 580', 'gen-9'),
        ('193A', 'Skylake-H GT4 [Iris Pro Graphics P580]', 'gen-9'),
        ('193B', 'Skylake-H GT4 [Iris Pro Graphics 580]', 'gen-9'),
        ('193D', 'Skylake-H GT4 [Iris Pro Graphics P580]', 'gen-9'),
    ),
    'win_intel_iris_pro_6200': (
        ('1622', 'Broadwell-DT/H GT3 [Iris Pro Graphics 6200]', 'gen-8'),
    ),
    'win_intel_iris_xe': (
        ('4628', 'Alder Lake-UP3 GT2 [UHD Graphics]', 'gen-12-lp'),
        ('462A', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('4636', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('4638', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('463A', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('4680', 'Alder Lake-S GT1 [UHD Graphics 770]', 'gen-12-lp'),
    ),
    'win_intel_iris_xe_max': (
        ('4628', 'Alder Lake-UP3 GT2 [UHD Graphics]', 'gen-12-lp'),
        ('462A', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('4636', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('4638', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('463A', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('4680', 'Alder Lake-S GT1 [UHD Graphics 770]', 'gen-12-lp'),
    ),
    'win_intel_iris_xe_plain': (
        ('4628', 'Alder Lake-UP3 GT2 [UHD Graphics]', 'gen-12-lp'),
        ('462A', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('4636', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('4638', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('463A', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('4680', 'Alder Lake-S GT1 [UHD Graphics 770]', 'gen-12-lp'),
    ),
    'win_intel_uhd_600': (
        ('3185', 'GeminiLake [UHD Graphics 600]', 'gen-9'),
    ),
    'win_intel_uhd_605': (
        ('3184', 'GeminiLake [UHD Graphics 605]', 'gen-9'),
    ),
    'win_intel_uhd_610': (
        ('3E90', 'CoffeeLake-S GT1 [UHD Graphics 610]', 'gen-9'),
        ('3E93', 'CoffeeLake-S GT1 [UHD Graphics 610]', 'gen-9'),
        ('3E9C', 'Coffee Lake-S GT1 [UHD Graphics 610]', 'gen-9'),
        ('3EA1', 'Whiskey Lake-U GT1 [UHD Graphics 610]', 'gen-9'),
        ('5902', 'Kaby Lake-S GT1 [HD Graphics 610]', 'gen-9'),
        ('5906', 'Kaby Lake-U GT1 [HD Graphics 610]', 'gen-9'),
    ),
    'win_intel_uhd_615': (
        ('591C', 'UHD Graphics 615', 'gen-9'),
        ('591E', 'Kaby Lake-Y GT2 [HD Graphics 615]', 'gen-9'),
    ),
    'win_intel_uhd_617': (
        ('87C0', 'UHD Graphics 617', 'gen-9'),
        ('87CA', 'UHD Graphics 617', 'gen-9'),
    ),
    'win_intel_uhd_620': (
        ('3EA0', 'WhiskeyLake-U GT2 [UHD Graphics 620]', 'gen-9'),
        ('3EA9', 'Coffee Lake-U GT2 [UHD Graphics 620]', 'gen-9'),
        ('5916', 'Kaby Lake-U GT2 [HD Graphics 620]', 'gen-9'),
        ('5917', 'Kaby Lake-R GT2 [UHD Graphics 620]', 'gen-9'),
        ('5921', 'HD Graphics 620', 'gen-9'),
        ('9B21', 'Comet Lake-U GT2 [UHD Graphics 620]', 'gen-9'),
    ),
    'win_intel_uhd_620_plain': (
        ('3EA0', 'WhiskeyLake-U GT2 [UHD Graphics 620]', 'gen-9'),
        ('3EA9', 'Coffee Lake-U GT2 [UHD Graphics 620]', 'gen-9'),
        ('5916', 'Kaby Lake-U GT2 [HD Graphics 620]', 'gen-9'),
        ('5917', 'Kaby Lake-R GT2 [UHD Graphics 620]', 'gen-9'),
        ('5921', 'HD Graphics 620', 'gen-9'),
        ('9B21', 'Comet Lake-U GT2 [UHD Graphics 620]', 'gen-9'),
    ),
    'win_intel_uhd_630': (
        ('3E91', 'CoffeeLake-S GT2 [UHD Graphics 630]', 'gen-9'),
        ('3E92', 'CoffeeLake-S GT2 [UHD Graphics 630]', 'gen-9'),
        ('3E94', 'Coffee Lake-S GT2 [UHD Graphics P630]', 'gen-9'),
        ('3E96', 'CoffeeLake-S GT2 [UHD Graphics P630]', 'gen-9'),
        ('3E98', 'CoffeeLake-S GT2 [UHD Graphics 630]', 'gen-9'),
        ('3E9A', 'Coffee Lake-S GT2 [UHD Graphics P630]', 'gen-9'),
    ),
    'win_intel_uhd_630_plain': (
        ('3E91', 'CoffeeLake-S GT2 [UHD Graphics 630]', 'gen-9'),
        ('3E92', 'CoffeeLake-S GT2 [UHD Graphics 630]', 'gen-9'),
        ('3E94', 'Coffee Lake-S GT2 [UHD Graphics P630]', 'gen-9'),
        ('3E96', 'CoffeeLake-S GT2 [UHD Graphics P630]', 'gen-9'),
        ('3E98', 'CoffeeLake-S GT2 [UHD Graphics 630]', 'gen-9'),
        ('3E9A', 'Coffee Lake-S GT2 [UHD Graphics P630]', 'gen-9'),
    ),
    'win_intel_uhd_710': (
        ('4693', 'Alder Lake-S GT1 [UHD Graphics 710]', 'gen-12-lp'),
    ),
    'win_intel_uhd_730': (
        ('4682', 'Alder Lake-S GT1 [UHD Graphics 730]', 'gen-12-lp'),
        ('4692', 'Alder Lake-S GT1 [UHD Graphics 730]', 'gen-12-lp'),
        ('4C8B', 'RocketLake-S GT1 [UHD Graphics 730]', 'gen-12-lp'),
    ),
    'win_intel_uhd_750': (
        ('4C8A', 'RocketLake-S GT1 [UHD Graphics 750]', 'gen-12-lp'),
        ('4C90', 'RocketLake-S GT1 [UHD Graphics P750]', 'gen-12-lp'),
    ),
    'win_intel_uhd_770': (
        ('4680', 'Alder Lake-S GT1 [UHD Graphics 770]', 'gen-12-lp'),
        ('4688', 'Alder Lake-HX GT1 [UHD Graphics 770]', 'gen-12-lp'),
        ('4690', 'Alder Lake-S GT1 [UHD Graphics 770]', 'gen-12-lp'),
        ('A780', 'Raptor Lake-S GT1 [UHD Graphics 770]', 'gen-12-lp'),
    ),
    'win_intel_uhd_graphics': (
        ('4628', 'Alder Lake-UP3 GT2 [UHD Graphics]', 'gen-12-lp'),
        ('462A', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('4636', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('4638', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('463A', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('4680', 'Alder Lake-S GT1 [UHD Graphics 770]', 'gen-12-lp'),
    ),
    'win_intel_uhd_graphics_plain': (
        ('4628', 'Alder Lake-UP3 GT2 [UHD Graphics]', 'gen-12-lp'),
        ('462A', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('4636', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('4638', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('463A', 'AlderLake-P [UHD Graphics]', 'gen-12-lp'),
        ('4680', 'Alder Lake-S GT1 [UHD Graphics 770]', 'gen-12-lp'),
    ),
    'win_microsoft_basic_render': (
        ('008C', 'Microsoft Basic Render Driver', 'warp'),
    ),
    'win_nvidia_gt_1030': (
        ('1D01', 'GP108 [GeForce GT 1030]', 'pascal'),
    ),
    'win_nvidia_gt_710': (
        ('104D', 'GF119 [GeForce GT 710]', 'kepler'),
        ('1281', 'GK208 [GeForce GT 710]', 'kepler'),
        ('1289', 'GK208 [GeForce GT 710]', 'kepler'),
        ('128B', 'GK208B [GeForce GT 710]', 'kepler'),
    ),
    'win_nvidia_gt_730': (
        ('0F02', 'GF108 [GeForce GT 730]', 'kepler'),
        ('0F06', 'GF108 [GeForce GT 730]', 'kepler'),
        ('0FC9', 'GK107 [GeForce GT 730]', 'kepler'),
        ('1287', 'GK208B [GeForce GT 730]', 'kepler'),
    ),
    'win_nvidia_gtx_1050': (
        ('1C81', 'GP107 [GeForce GTX 1050]', 'pascal'),
        ('1C83', 'GP107 [GeForce GTX 1050 3GB]', 'pascal'),
    ),
    'win_nvidia_gtx_1050_laptop': (
        ('1C22', 'GP106M [GeForce GTX 1050 Mobile]', 'pascal'),
        ('1C62', 'GP106BM [GeForce GTX 1050 Mobile]', 'pascal'),
        ('1C8D', 'GP107M [GeForce GTX 1050 Mobile]', 'pascal'),
        ('1C91', 'GP107M [GeForce GTX 1050 3 GB Max-Q]', 'pascal'),
        ('1C92', 'GP107M [GeForce GTX 1050 Mobile]', 'pascal'),
        ('1CCD', 'GP107BM [GeForce GTX 1050 Mobile]', 'pascal'),
    ),
    'win_nvidia_gtx_1050_ti': (
        ('1C82', 'GP107 [GeForce GTX 1050 Ti]', 'pascal'),
    ),
    'win_nvidia_gtx_1050_ti_laptop': (
        ('1C21', 'GP106M [GeForce GTX 1050 Ti Mobile]', 'pascal'),
        ('1C61', 'GP106BM [GeForce GTX 1050 Ti Mobile]', 'pascal'),
        ('1C8C', 'GP107M [GeForce GTX 1050 Ti Mobile]', 'pascal'),
        ('1C8F', 'GP107M [GeForce GTX 1050 Ti Max-Q]', 'pascal'),
        ('1CCC', 'GP107BM [GeForce GTX 1050 Ti Mobile]', 'pascal'),
    ),

    'win_nvidia_gtx_1060': (
        ('1B83', 'GP104 [GeForce GTX 1060 6GB]', 'pascal'),
        ('1B84', 'GP104 [GeForce GTX 1060 3GB]', 'pascal'),
        ('1C02', 'GP106 [GeForce GTX 1060 3GB]', 'pascal'),
        ('1C03', 'GP106 [GeForce GTX 1060 6GB]', 'pascal'),
        ('1C04', 'GP106 [GeForce GTX 1060 5GB]', 'pascal'),
        ('1C06', 'GP106 [GeForce GTX 1060 6GB Rev. 2]', 'pascal'),
    ),
    'win_nvidia_gtx_1060_3gb': (
        ('1B83', 'GP104 [GeForce GTX 1060 6GB]', 'pascal'),
        ('1B84', 'GP104 [GeForce GTX 1060 3GB]', 'pascal'),
        ('1C02', 'GP106 [GeForce GTX 1060 3GB]', 'pascal'),
        ('1C03', 'GP106 [GeForce GTX 1060 6GB]', 'pascal'),
        ('1C04', 'GP106 [GeForce GTX 1060 5GB]', 'pascal'),
        ('1C06', 'GP106 [GeForce GTX 1060 6GB Rev. 2]', 'pascal'),
    ),
    'win_nvidia_gtx_1060_6gb': (
        ('1B83', 'GP104 [GeForce GTX 1060 6GB]', 'pascal'),
        ('1B84', 'GP104 [GeForce GTX 1060 3GB]', 'pascal'),
        ('1C02', 'GP106 [GeForce GTX 1060 3GB]', 'pascal'),
        ('1C03', 'GP106 [GeForce GTX 1060 6GB]', 'pascal'),
        ('1C04', 'GP106 [GeForce GTX 1060 5GB]', 'pascal'),
        ('1C06', 'GP106 [GeForce GTX 1060 6GB Rev. 2]', 'pascal'),
    ),
    'win_nvidia_gtx_1070': (
        ('1B81', 'GP104 [GeForce GTX 1070]', 'pascal'),
    ),
    'win_nvidia_gtx_1070_ti': (
        ('1B82', 'GP104 [GeForce GTX 1070 Ti]', 'pascal'),
    ),
    'win_nvidia_gtx_1080': (
        ('1B80', 'GP104 [GeForce GTX 1080]', 'pascal'),
    ),
    'win_nvidia_gtx_1080_ti': (
        ('1B01', 'GP102 [GeForce GTX 1080 Ti 10GB]', 'pascal'),
        ('1B06', 'GP102 [GeForce GTX 1080 Ti]', 'pascal'),
    ),
    'win_nvidia_gtx_1650': (
        ('1F0A', 'TU106 [GeForce GTX 1650]', 'turing'),
        ('1F82', 'TU117 [GeForce GTX 1650]', 'turing'),
        ('2188', 'TU116 [GeForce GTX 1650]', 'turing'),
    ),
    'win_nvidia_gtx_1650_laptop': (
        ('1F91', 'TU117M [GeForce GTX 1650 Mobile / Max-Q]', 'turing'),
        ('1F92', 'TU117M [GeForce GTX 1650 Mobile]', 'turing'),
        ('1F94', 'TU117M [GeForce GTX 1650 Mobile]', 'turing'),
        ('1F96', 'TU117M [GeForce GTX 1650 Mobile / Max-Q]', 'turing'),
        ('1F99', 'TU117M [GeForce GTX 1650 Mobile / Max-Q]', 'turing'),
        ('1F9D', 'TU117M [GeForce GTX 1650 Mobile / Max-Q]', 'turing'),
    ),
    'win_nvidia_gtx_1650_super': (
        ('2187', 'TU116 [GeForce GTX 1650 SUPER]', 'turing'),
    ),
    'win_nvidia_gtx_1650_ti_laptop': (
        ('1F95', 'TU117M [GeForce GTX 1650 Ti Mobile]', 'turing'),
        ('2192', 'TU116M [GeForce GTX 1650 Ti Mobile]', 'turing'),
    ),
    'win_nvidia_gtx_1660': (
        ('2184', 'TU116 [GeForce GTX 1660]', 'turing'),
    ),
    'win_nvidia_gtx_1660_super': (
        ('1F09', 'TU106 [GeForce GTX 1660 SUPER]', 'turing'),
        ('21C4', 'TU116 [GeForce GTX 1660 SUPER]', 'turing'),
    ),
    'win_nvidia_gtx_1660_ti': (
        ('2182', 'TU116 [GeForce GTX 1660 Ti]', 'turing'),
    ),
    'win_nvidia_gtx_1660_ti_laptop': (
        ('2191', 'TU116M [GeForce GTX 1660 Ti Mobile]', 'turing'),
        ('21D1', 'TU116BM [GeForce GTX 1660 Ti Mobile]', 'turing'),
    ),
    'win_nvidia_gtx_750': (
        ('1381', 'GM107 [GeForce GTX 750]', 'maxwell'),
        ('1407', 'GM206 [GeForce GTX 750 v2]', 'maxwell'),
    ),
    'win_nvidia_gtx_750_ti': (
        ('1380', 'GM107 [GeForce GTX 750 Ti]', 'maxwell'),
        ('139D', 'GM107M [GeForce GTX 750 Ti]', 'maxwell'),
    ),
    'win_nvidia_gtx_760': (
        ('1187', 'GK104 [GeForce GTX 760]', 'kepler'),
        ('118E', 'GK104 [GeForce GTX 760 OEM]', 'kepler'),
        ('1191', 'GK104 [GeForce GTX 760 Rev. 2]', 'kepler'),
    ),
    'win_nvidia_gtx_770': (
        ('1184', 'GK104 [GeForce GTX 770]', 'kepler'),
    ),
    'win_nvidia_gtx_780': (
        ('1004', 'GK110 [GeForce GTX 780]', 'kepler'),
        ('1007', 'GK110 [GeForce GTX 780 Rev. 2]', 'kepler'),
    ),
    'win_nvidia_gtx_780_ti': (
        ('1008', 'GK110 [GeForce GTX 780 Ti 6GB]', 'kepler'),
        ('100A', 'GK110B [GeForce GTX 780 Ti]', 'kepler'),
    ),
    'win_nvidia_gtx_960': (
        ('13D8', 'GM204M [GeForce GTX 960 OEM / 970M]', 'maxwell'),
        ('1401', 'GM206 [GeForce GTX 960]', 'maxwell'),
        ('1404', 'GM206 [GeForce GTX 960 FAKE]', 'maxwell'),
        ('1406', 'GM206 [GeForce GTX 960 OEM]', 'maxwell'),
    ),
    'win_nvidia_gtx_970': (
        ('13C2', 'GM204 [GeForce GTX 970]', 'maxwell'),
    ),
    'win_nvidia_gtx_980_ti': (
        ('17C8', 'GM200 [GeForce GTX 980 Ti]', 'maxwell'),
    ),
    'win_nvidia_mx130': (
        ('174D', 'GM108M [GeForce MX130]', 'maxwell'),
    ),
    'win_nvidia_mx150': (
        ('1C90', 'GP107M [GeForce MX150]', 'pascal'),
        ('1D10', 'GP108M [GeForce MX150]', 'pascal'),
        ('1D12', 'GP108M [GeForce MX150]', 'pascal'),
    ),
    'win_nvidia_mx230': (
        ('1D11', 'GP108M [GeForce MX230]', 'pascal'),
    ),
    'win_nvidia_mx250': (
        ('1D13', 'GP108M [GeForce MX250]', 'pascal'),
        ('1D52', 'GP108BM [GeForce MX250]', 'pascal'),
    ),
    'win_nvidia_mx330': (
        ('1D16', 'GP108M [GeForce MX330]', 'pascal'),
        ('1D56', 'GP108BM [GeForce MX330]', 'pascal'),
    ),
    'win_nvidia_mx350': (
        ('1C94', 'GP107M [GeForce MX350]', 'pascal'),
        ('1C96', 'GP107M [GeForce MX350]', 'pascal'),
    ),

    'win_nvidia_mx450': (
        ('1F97', 'TU117M [GeForce MX450]', 'turing'),
        ('1F98', 'TU117M [GeForce MX450]', 'turing'),
        ('1F9C', 'TU117M [GeForce MX450]', 'turing'),
    ),
    'win_nvidia_mx550': (
        ('1F9F', 'TU117M [GeForce MX550]', 'turing'),
        ('1FA0', 'TU117M [GeForce MX550]', 'turing'),
    ),
    'win_nvidia_quadro_p1000': (
        ('1CB1', 'GP107GL [Quadro P1000]', 'pascal'),
        ('1CFB', 'GP107GL [Quadro P1000]', 'pascal'),
    ),
    'win_nvidia_quadro_p2200': (
        ('1C31', 'GP106GL [Quadro P2200]', 'pascal'),
    ),
    'win_nvidia_quadro_p620': (
        ('1CB6', 'GP107GL [Quadro P620]', 'pascal'),
        ('1CBD', 'GP107GLM [Quadro P620]', 'pascal'),
    ),
    'win_nvidia_quadro_rtx_5000': (
        ('1EB0', 'TU104GL [Quadro RTX 5000]', 'turing'),
    ),
    'win_nvidia_rtx_2000_ada': (
        ('28B0', 'AD107GL [RTX 2000 / 2000E Ada Generation]', 'lovelace'),
        ('28F8', 'AD107GLM [RTX 2000 Ada Generation Embedded GPU]', 'lovelace'),
    ),
    'win_nvidia_rtx_2050': (
        ('25A9', 'GA107M [GeForce RTX 2050]', 'ampere'),
        ('25AD', 'GA107 [GeForce RTX 2050]', 'ampere'),
        ('25ED', 'GA107 [GeForce RTX 2050]', 'ampere'),
    ),
    'win_nvidia_rtx_2060': (
        ('1E89', 'TU104 [GeForce RTX 2060]', 'turing'),
        ('1F03', 'TU106 [GeForce RTX 2060 12GB]', 'turing'),
        ('1F08', 'TU106 [GeForce RTX 2060 Rev. A]', 'turing'),
    ),
    'win_nvidia_rtx_2060_laptop': (
        ('1F11', 'TU106M [GeForce RTX 2060 Mobile]', 'turing'),
        ('1F12', 'TU106M [GeForce RTX 2060 Max-Q]', 'turing'),
        ('1F15', 'TU106M [GeForce RTX 2060 Mobile]', 'turing'),
        ('1F51', 'TU106BM [GeForce RTX 2060 Mobile]', 'turing'),
        ('1F55', 'TU106BM [GeForce RTX 2060 Mobile]', 'turing'),
    ),
    'win_nvidia_rtx_2060_super': (
        ('1F06', 'TU106 [GeForce RTX 2060 SUPER]', 'turing'),
        ('1F42', 'TU106 [GeForce RTX 2060 SUPER]', 'turing'),
        ('1F47', 'TU106 [GeForce RTX 2060 SUPER]', 'turing'),
    ),
    'win_nvidia_rtx_2070': (
        ('1F02', 'TU106 [GeForce RTX 2070]', 'turing'),
        ('1F07', 'TU106 [GeForce RTX 2070 Rev. A]', 'turing'),
    ),
    'win_nvidia_rtx_2070_laptop': (
        ('1F10', 'TU106M [GeForce RTX 2070 Mobile]', 'turing'),
        ('1F14', 'TU106M [GeForce RTX 2070 Mobile / Max-Q Refresh]', 'turing'),
        ('1F50', 'TU106BM [GeForce RTX 2070 Mobile / Max-Q]', 'turing'),
        ('1F54', 'TU106BM [GeForce RTX 2070 Mobile]', 'turing'),
    ),
    'win_nvidia_rtx_2070_super': (
        ('1E84', 'TU104 [GeForce RTX 2070 SUPER]', 'turing'),
        ('1EC2', 'TU104 [GeForce RTX 2070 SUPER]', 'turing'),
        ('1EC7', 'TU104 [GeForce RTX 2070 SUPER]', 'turing'),
    ),
    'win_nvidia_rtx_2080': (
        ('1E82', 'TU104 [GeForce RTX 2080]', 'turing'),
        ('1E87', 'TU104 [GeForce RTX 2080 Rev. A]', 'turing'),
    ),
    'win_nvidia_rtx_2080_laptop': (
        ('1E90', 'TU104M [GeForce RTX 2080 Mobile]', 'turing'),
        ('1ED0', 'TU104BM [GeForce RTX 2080 Mobile]', 'turing'),
    ),
    'win_nvidia_rtx_2080_super': (
        ('1E81', 'TU104 [GeForce RTX 2080 SUPER]', 'turing'),
    ),
    'win_nvidia_rtx_2080_ti': (
        ('1E03', 'TU102 [GeForce RTX 2080 Ti 12GB]', 'turing'),
        ('1E04', 'TU102 [GeForce RTX 2080 Ti]', 'turing'),
        ('1E07', 'TU102 [GeForce RTX 2080 Ti Rev. A]', 'turing'),
    ),
    'win_nvidia_rtx_3050': (
        ('2507', 'GA106 [Geforce RTX 3050]', 'ampere'),
        ('2508', 'GA106 [GeForce RTX 3050 OEM]', 'ampere'),
        ('2582', 'GA107 [GeForce RTX 3050 8GB]', 'ampere'),
        ('2583', 'GA107 [GeForce RTX 3050 4GB]', 'ampere'),
        ('2584', 'GA107 [GeForce RTX 3050 6GB]', 'ampere'),
    ),
    'win_nvidia_rtx_3050_6gb_laptop': (
        ('25A2', 'GA107M [GeForce RTX 3050 Mobile]', 'ampere'),
        ('25A5', 'GA107M [GeForce RTX 3050 Mobile]', 'ampere'),
        ('25AB', 'GA107M [GeForce RTX 3050 4GB Laptop GPU]', 'ampere'),
        ('25AC', 'GA107BM / GN20-P0-R-K2 [GeForce RTX 3050 6GB Laptop GPU]', 'ampere'),
        ('25E2', 'GA107BM [GeForce RTX 3050 Mobile]', 'ampere'),
        ('25E5', 'GA107BM [GeForce RTX 3050 Mobile]', 'ampere'),
    ),
    'win_nvidia_rtx_3050_laptop': (
        ('25A2', 'GA107M [GeForce RTX 3050 Mobile]', 'ampere'),
        ('25A5', 'GA107M [GeForce RTX 3050 Mobile]', 'ampere'),
        ('25AB', 'GA107M [GeForce RTX 3050 4GB Laptop GPU]', 'ampere'),
        ('25AC', 'GA107BM / GN20-P0-R-K2 [GeForce RTX 3050 6GB Laptop GPU]', 'ampere'),
        ('25E2', 'GA107BM [GeForce RTX 3050 Mobile]', 'ampere'),
        ('25E5', 'GA107BM [GeForce RTX 3050 Mobile]', 'ampere'),
    ),
    'win_nvidia_rtx_3050_ti_laptop': (
        ('2523', 'GA106M [GeForce RTX 3050 Ti Mobile / Max-Q]', 'ampere'),
        ('2563', 'GA106M [GeForce RTX 3050 Ti Mobile / Max-Q]', 'ampere'),
        ('25A0', 'GA107M [GeForce RTX 3050 Ti Mobile]', 'ampere'),
        ('25E0', 'GA107BM [GeForce RTX 3050 Ti Mobile]', 'ampere'),
    ),
    'win_nvidia_rtx_3060': (
        ('2487', 'GA104 [GeForce RTX 3060]', 'ampere'),
        ('24C7', 'GA104 [GeForce RTX 3060 8GB]', 'ampere'),
        ('2501', 'GA106 [GeForce RTX 3060]', 'ampere'),
        ('2503', 'GA106 [GeForce RTX 3060]', 'ampere'),
        ('2504', 'GA106 [GeForce RTX 3060 Lite Hash Rate]', 'ampere'),
        ('2509', 'GA106 [GeForce RTX 3060 12GB Rev. 2]', 'ampere'),
    ),
    'win_nvidia_rtx_3060_laptop': (
        ('2520', 'GA106M [GeForce RTX 3060 Mobile / Max-Q]', 'ampere'),
        ('2521', 'GA106M [GeForce RTX 3060 Laptop GPU]', 'ampere'),
        ('2560', 'GA106M [GeForce RTX 3060 Mobile / Max-Q]', 'ampere'),
        ('2561', 'GA106M [GeForce RTX 3060 Laptop GPU]', 'ampere'),
    ),
    'win_nvidia_rtx_3060_ti': (
        ('2414', 'GA103 [GeForce RTX 3060 Ti]', 'ampere'),
        ('2486', 'GA104 [GeForce RTX 3060 Ti]', 'ampere'),
        ('2489', 'GA104 [GeForce RTX 3060 Ti Lite Hash Rate]', 'ampere'),
        ('248E', 'GA104 [GeForce RTX 3060 Ti]', 'ampere'),
        ('24C9', 'GA104 [GeForce RTX 3060 Ti GDDR6X]', 'ampere'),
    ),
    'win_nvidia_rtx_3070': (
        ('2484', 'GA104 [GeForce RTX 3070]', 'ampere'),
        ('2488', 'GA104 [GeForce RTX 3070 Lite Hash Rate]', 'ampere'),
        ('248D', 'GA104 [GeForce RTX 3070]', 'ampere'),
        ('24C8', 'GA104 [GeForce RTX 3070 GDDR6X]', 'ampere'),
    ),
    'win_nvidia_rtx_3070_laptop': (
        ('249D', 'GA104M [GeForce RTX 3070 Mobile / Max-Q]', 'ampere'),
        ('24DD', 'GA104M [GeForce RTX 3070 Mobile / Max-Q]', 'ampere'),
    ),
    'win_nvidia_rtx_3070_ti': (
        ('2207', 'GA102 [GeForce RTX 3070 Ti]', 'ampere'),
        ('2482', 'GA104 [GeForce RTX 3070 Ti]', 'ampere'),
        ('248C', 'GA104 [GeForce RTX 3070 Ti]', 'ampere'),
    ),
    'win_nvidia_rtx_3070_ti_laptop': (
        ('24A0', 'GA104 [Geforce RTX 3070 Ti Laptop GPU]', 'ampere'),
        ('24E0', 'GA104M [Geforce RTX 3070 Ti Laptop GPU]', 'ampere'),
    ),
    'win_nvidia_rtx_3080': (
        ('2206', 'GA102 [GeForce RTX 3080]', 'ampere'),
        ('220A', 'GA102 [GeForce RTX 3080 12GB]', 'ampere'),
        ('2216', 'GA102 [GeForce RTX 3080 Lite Hash Rate]', 'ampere'),
    ),

    'win_nvidia_rtx_3080_laptop': (
        ('249C', 'GA104M [GeForce RTX 3080 Mobile / Max-Q 8GB/16GB]', 'ampere'),
        ('24DC', 'GA104M [GeForce RTX 3080 Mobile / Max-Q 8GB/16GB]', 'ampere'),
    ),
    'win_nvidia_rtx_3080_ti': (
        ('2205', 'GA102 [GeForce RTX 3080 Ti 20GB]', 'ampere'),
        ('2208', 'GA102 [GeForce RTX 3080 Ti]', 'ampere'),
    ),
    'win_nvidia_rtx_3080_ti_laptop': (
        ('2420', 'GA103M [GeForce RTX 3080 Ti Mobile]', 'ampere'),
        ('2460', 'GA103M [GeForce RTX 3080 Ti Laptop GPU]', 'ampere'),
    ),
    'win_nvidia_rtx_3090': (
        ('2204', 'GA102 [GeForce RTX 3090]', 'ampere'),
    ),
    'win_nvidia_rtx_3090_ti': (
        ('2203', 'GA102 [GeForce RTX 3090 Ti]', 'ampere'),
    ),
    'win_nvidia_rtx_4000_ada': (
        ('27B0', 'AD104GL [RTX 4000 SFF Ada Generation]', 'lovelace'),
        ('27B2', 'AD104GL [RTX 4000 Ada Generation]', 'lovelace'),
        ('27FA', 'AD104GLM [RTX 4000 Ada Generation Embedded GPU]', 'lovelace'),
    ),
    'win_nvidia_rtx_4050_laptop': (
        ('28A1', 'AD107M [GeForce RTX 4050 Max-Q / Mobile]', 'lovelace'),
        ('28E1', 'AD107M [GeForce RTX 4050 Max-Q / Mobile]', 'lovelace'),
    ),
    'win_nvidia_rtx_4060': (
        ('2808', 'AD106 [GeForce RTX 4060]', 'lovelace'),
        ('2882', 'AD107 [GeForce RTX 4060]', 'lovelace'),
    ),
    'win_nvidia_rtx_4060_laptop': (
        ('28A0', 'AD107M [GeForce RTX 4060 Max-Q / Mobile]', 'lovelace'),
        ('28E0', 'AD107M [GeForce RTX 4060 Max-Q / Mobile]', 'lovelace'),
    ),
    'win_nvidia_rtx_4060_ti': (
        ('2788', 'AD104 [GeForce RTX 4060 Ti]', 'lovelace'),
        ('2803', 'AD106 [GeForce RTX 4060 Ti]', 'lovelace'),
        ('2805', 'AD106 [GeForce RTX 4060 Ti 16GB]', 'lovelace'),
    ),
    'win_nvidia_rtx_4070': (
        ('2709', 'AD103 [GeForce RTX 4070]', 'lovelace'),
        ('2786', 'AD104 [GeForce RTX 4070]', 'lovelace'),
    ),
    'win_nvidia_rtx_4070_laptop': (
        ('2820', 'AD106M [GeForce RTX 4070 Max-Q / Mobile]', 'lovelace'),
        ('2860', 'AD106M [GeForce RTX 4070 Max-Q / Mobile]', 'lovelace'),
    ),
    'win_nvidia_rtx_4070_super': (
        ('2783', 'AD104 [GeForce RTX 4070 SUPER]', 'lovelace'),
    ),
    'win_nvidia_rtx_4070_ti': (
        ('2782', 'AD104 [GeForce RTX 4070 Ti]', 'lovelace'),
    ),
    'win_nvidia_rtx_4070_ti_super': (
        ('2689', 'AD102 [GeForce RTX 4070 Ti SUPER]', 'lovelace'),
        ('2705', 'AD103 [GeForce RTX 4070 Ti SUPER]', 'lovelace'),
    ),
    'win_nvidia_rtx_4080': (
        ('2704', 'AD103 [GeForce RTX 4080]', 'lovelace'),
    ),
    'win_nvidia_rtx_4080_laptop': (
        ('27A0', 'AD104M [GeForce RTX 4080 Max-Q / Mobile]', 'lovelace'),
        ('27E0', 'AD104M [GeForce RTX 4080 Max-Q / Mobile]', 'lovelace'),
    ),
    'win_nvidia_rtx_4080_super': (
        ('2702', 'AD103 [GeForce RTX 4080 SUPER]', 'lovelace'),
        ('2703', 'AD103 [GeForce RTX 4080 SUPER]', 'lovelace'),
    ),
    'win_nvidia_rtx_4090': (
        ('2684', 'AD102 [GeForce RTX 4090]', 'lovelace'),
        ('2685', 'AD102 [GeForce RTX 4090 D]', 'lovelace'),
    ),
    'win_nvidia_rtx_4090_laptop': (
        ('2717', 'AD103M / GN21-X11 [GeForce RTX 4090 Laptop GPU]', 'lovelace'),
        ('2757', 'AD103M / GN21-X11 [GeForce RTX 4090 Laptop GPU]', 'lovelace'),
    ),
    'win_nvidia_rtx_4500_ada': (
        ('27B1', 'AD104GL [RTX 4500 Ada Generation]', 'lovelace'),
    ),
    'win_nvidia_rtx_5000_ada': (
        ('26B2', 'AD102GL [RTX 5000 Ada Generation]', 'lovelace'),
        ('2770', 'AD103GLM [RTX 5000 Ada Generation Embedded GPU]', 'lovelace'),
    ),
    'win_nvidia_rtx_5050': (
        ('2D83', 'GB207 [GeForce RTX 5050]', 'blackwell'),
    ),
    'win_nvidia_rtx_5050_laptop': (
        ('2D98', 'GB207M [GeForce RTX 5050 Max-Q / Mobile]', 'blackwell'),
        ('2DD8', 'GB207M [GeForce RTX 5050 Max-Q / Mobile]', 'blackwell'),
    ),
    'win_nvidia_rtx_5060': (
        ('2D05', 'GB206 [GeForce RTX 5060]', 'blackwell'),
        ('2F06', 'GB205 [GeForce RTX 5060]', 'blackwell'),
    ),
    'win_nvidia_rtx_5060_laptop': (
        ('2D19', 'GB206M [GeForce RTX 5060 Max-Q / Mobile]', 'blackwell'),
        ('2D59', 'GB206M [GeForce RTX 5060 Max-Q / Mobile]', 'blackwell'),
    ),
    'win_nvidia_rtx_5060_ti': (
        ('2D04', 'GB206 [GeForce RTX 5060 Ti]', 'blackwell'),
    ),
    'win_nvidia_rtx_5070': (
        ('2F04', 'GB205 [GeForce RTX 5070]', 'blackwell'),
    ),
    'win_nvidia_rtx_5070_laptop': (
        ('2D18', 'GB206M [GeForce RTX 5070 Max-Q / Mobile]', 'blackwell'),
        ('2D58', 'GB206M [GeForce RTX 5070 Max-Q / Mobile]', 'blackwell'),
    ),
    'win_nvidia_rtx_5070_ti': (
        ('2C05', 'GB203 [GeForce RTX 5070 Ti]', 'blackwell'),
    ),

    'win_nvidia_rtx_5070_ti_laptop': (
        ('2F18', 'GB205M [GeForce RTX 5070 Ti Mobile]', 'blackwell'),
        ('2F58', 'GB205M [GeForce RTX 5070 Ti Mobile]', 'blackwell'),
    ),
    'win_nvidia_rtx_5080': (
        ('2C02', 'GB203 [GeForce RTX 5080]', 'blackwell'),
    ),
    'win_nvidia_rtx_5080_laptop': (
        ('2C19', 'GB203M / GN22 [GeForce RTX 5080 Max-Q / Mobile]', 'blackwell'),
        ('2C59', 'GB203M / GN22-X9 [GeForce RTX 5080 Max-Q / Mobile]', 'blackwell'),
    ),
    'win_nvidia_rtx_5090': (
        ('2B85', 'GB202 [GeForce RTX 5090]', 'blackwell'),
        ('2B87', 'GB202 [GeForce RTX 5090 D]', 'blackwell'),
        ('2B8C', 'GB202 [GeForce RTX 5090 D V2]', 'blackwell'),
    ),
    'win_nvidia_rtx_5090_laptop': (
        ('2C18', 'GB203M / GN22 [GeForce RTX 5090 Max-Q / Mobile]', 'blackwell'),
        ('2C58', 'GB203M / GN22-X11 [GeForce RTX 5090 Max-Q / Mobile]', 'blackwell'),
    ),
    'win_nvidia_rtx_6000_ada': (
        ('26B1', 'AD102GL [RTX 6000 Ada Generation]', 'lovelace'),
    ),
    'win_nvidia_rtx_a2000': (
        ('2230', 'GA102GL [RTX A6000]', 'ampere'),
        ('2231', 'GA102GL [RTX A5000]', 'ampere'),
        ('2232', 'GA102GL [RTX A4500]', 'ampere'),
        ('2233', 'GA102GL [RTX A5500]', 'ampere'),
        ('24B0', 'GA104GL [RTX A4000]', 'ampere'),
        ('24B1', 'GA104GL [RTX A4000H]', 'ampere'),
    ),
    'win_nvidia_rtx_a4000': (
        ('2230', 'GA102GL [RTX A6000]', 'ampere'),
        ('2231', 'GA102GL [RTX A5000]', 'ampere'),
        ('2232', 'GA102GL [RTX A4500]', 'ampere'),
        ('2233', 'GA102GL [RTX A5500]', 'ampere'),
        ('24B0', 'GA104GL [RTX A4000]', 'ampere'),
        ('24B1', 'GA104GL [RTX A4000H]', 'ampere'),
    ),
    'win_nvidia_rtx_a4500': (
        ('2230', 'GA102GL [RTX A6000]', 'ampere'),
        ('2231', 'GA102GL [RTX A5000]', 'ampere'),
        ('2232', 'GA102GL [RTX A4500]', 'ampere'),
        ('2233', 'GA102GL [RTX A5500]', 'ampere'),
        ('24B0', 'GA104GL [RTX A4000]', 'ampere'),
        ('24B1', 'GA104GL [RTX A4000H]', 'ampere'),
    ),
    'win_nvidia_rtx_a5000': (
        ('2230', 'GA102GL [RTX A6000]', 'ampere'),
        ('2231', 'GA102GL [RTX A5000]', 'ampere'),
        ('2232', 'GA102GL [RTX A4500]', 'ampere'),
        ('2233', 'GA102GL [RTX A5500]', 'ampere'),
        ('24B0', 'GA104GL [RTX A4000]', 'ampere'),
        ('24B1', 'GA104GL [RTX A4000H]', 'ampere'),
    ),
    'win_nvidia_rtx_a5500': (
        ('2230', 'GA102GL [RTX A6000]', 'ampere'),
        ('2231', 'GA102GL [RTX A5000]', 'ampere'),
        ('2232', 'GA102GL [RTX A4500]', 'ampere'),
        ('2233', 'GA102GL [RTX A5500]', 'ampere'),
        ('24B0', 'GA104GL [RTX A4000]', 'ampere'),
        ('24B1', 'GA104GL [RTX A4000H]', 'ampere'),
    ),
    'win_nvidia_rtx_a6000': (
        ('2230', 'GA102GL [RTX A6000]', 'ampere'),
        ('2231', 'GA102GL [RTX A5000]', 'ampere'),
        ('2232', 'GA102GL [RTX A4500]', 'ampere'),
        ('2233', 'GA102GL [RTX A5500]', 'ampere'),
        ('24B0', 'GA104GL [RTX A4000]', 'ampere'),
        ('24B1', 'GA104GL [RTX A4000H]', 'ampere'),
    ),
    'win_nvidia_rtx_pro_2000_blackwell': (
        ('2D30', 'GB206GL [RTX PRO 2000 Blackwell]', 'blackwell'),
        ('2D79', 'GB206GLM [RTX PRO 2000 Blackwell Embedded GPU]', 'blackwell'),
    ),
    'win_nvidia_rtx_pro_4000_blackwell': (
        ('2C33', 'GB203GL [RTX PRO 4000 Blackwell SFF Edition]', 'blackwell'),
        ('2C34', 'GB203GL [RTX PRO 4000 Blackwell]', 'blackwell'),
        ('2C79', 'GB203GLM [RTX PRO 4000 Blackwell Embedded GPU]', 'blackwell'),
    ),
    'win_nvidia_rtx_pro_4500_blackwell': (
        ('2C31', 'GB203GL [RTX PRO 4500 Blackwell]', 'blackwell'),
        ('2C3A', 'GB203GL [RTX PRO 4500 Blackwell Server Edition]', 'blackwell'),
    ),
    'win_nvidia_rtx_pro_5000_blackwell': (
        ('2BB3', 'GB202GL [RTX PRO 5000 Blackwell / RTX PRO 5000 72GB Blackwell]', 'blackwell'),
        ('2C77', 'GB203GLM [RTX PRO 5000 Blackwell Embedded GPU]', 'blackwell'),
    ),
    'win_nvidia_rtx_pro_6000_blackwell': (
        ('2BB1', 'GB202GL [RTX PRO 6000 Blackwell Workstation Edition]', 'blackwell'),
        ('2BB5', 'GB202GL [RTX PRO 6000 Blackwell Server Edition]', 'blackwell'),
    ),
    'win_nvidia_t1000': (
        ('1FF0', 'TU117GL [T1000 8GB]', 'turing'),
    ),
    'win_nvidia_t400': (
        ('1FF2', 'TU117GL [T400 4GB / T400E]', 'turing'),
    ),
    'win_nvidia_t600': (
        ('1FB1', 'TU117GL [T600]', 'turing'),
    ),
    'win_qualcomm_adreno_x1_45': (
        ('000D', 'Qualcomm Adreno X1-45 DXGI adapter', 'adreno-7xx'),
    ),
    'win_qualcomm_adreno_x1_85': (
        ('0C36', 'Qualcomm Adreno X1-85 DXGI adapter', 'adreno-7xx'),
    ),
}

# Curated corrections for AMD families whose public PCI description contains
# several nearby desktop, mobile, or professional products. The generated
# number match is intentionally overridden here so a shared Navi family ID
# does not leak into a similarly named but different adapter.
WINDOWS_GPU_DEVICE_VARIANTS.update(
    {
        "win_amd_r7_370": (
            ("6810", "Curacao XT / Trinidad XT [Radeon R7 370 / R9 270X/370X]", "gcn-1"),
            ("6811", "Curacao PRO [Radeon R7 370 / R9 270/370 OEM]", "gcn-1"),
        ),
        "win_amd_radeon_pro_w6400": (
            ("7422", "Navi 24 [Radeon PRO W6400]", "rdna-2"),
        ),
        "win_amd_radeon_pro_w6600": (
            ("73E3", "Navi 23 WKS-XL [Radeon PRO W6600]", "rdna-2"),
        ),
        "win_amd_radeon_pro_w7700": (
            ("7470", "Navi 32 [Radeon PRO W7700]", "rdna-3"),
        ),
        "win_amd_radeon_pro_w7800": (
            ("7449", "Navi 31 [Radeon Pro W7800 48GB]", "rdna-3"),
            ("745E", "Navi 31 [Radeon Pro W7800]", "rdna-3"),
        ),
        "win_amd_radeon_pro_w7900": (
            ("7448", "Navi 31 [Radeon Pro W7900]", "rdna-3"),
            ("744A", "Navi 31 [Radeon Pro W7900 Dual Slot]", "rdna-3"),
            ("744B", "Navi 31 [Radeon Pro W7900D]", "rdna-3"),
        ),
        "win_amd_radeon_pro_wx_7100": (
            ("67C4", "Ellesmere [Radeon Pro WX 7100]", "gcn-4"),
            ("67D4", "Ellesmere [Radeon Pro WX 7100 / Barco MXRT-8700]", "gcn-4"),
        ),
        "win_amd_rx_5300m": (
            ("7340", "Navi 14 [Radeon RX 5500/5500M / Pro 5300/5300M/5500M]", "rdna-1"),
        ),
        "win_amd_rx_5500m": (
            ("7340", "Navi 14 [Radeon RX 5500/5500M / Pro 5300/5300M/5500M]", "rdna-1"),
        ),
        "win_amd_rx_5600m": (
            ("731F", "Navi 10 [Radeon RX 5600 OEM/5600 XT / 5700/5700 XT]", "rdna-1"),
        ),
        "win_amd_rx_5700m": (
            ("731F", "Navi 10 [Radeon RX 5600 OEM/5600 XT / 5700/5700 XT]", "rdna-1"),
        ),
        "win_amd_rx_6300m": (
            ("7424", "Navi 24 [Radeon RX 6300]", "rdna-2"),
        ),
        "win_amd_rx_6500m": (
            ("743F", "Navi 24 [Radeon RX 6400/6500 XT/6500M]", "rdna-2"),
        ),
        "win_amd_rx_6600m": (
            ("73FF", "Navi 23 [Radeon RX 6600/6600 XT/6600M]", "rdna-2"),
        ),
        "win_amd_rx_6700m": (
            ("73DF", "Navi 22 [Radeon RX 6700/6700 XT/6750 XT / 6800M/6850M XT]", "rdna-2"),
        ),
        "win_amd_rx_6800m": (
            ("73DF", "Navi 22 [Radeon RX 6700/6700 XT/6750 XT / 6800M/6850M XT]", "rdna-2"),
        ),
        "win_amd_rx_7700s": (
            ("7480", "Navi 33 [Radeon RX 7600/7600 XT/7600M XT/7600S/7700S / PRO W7600]", "rdna-3"),
        ),
        "win_amd_rx_7900m": (
            ("744C", "Navi 31 [Radeon RX 7900 XT/7900 XTX/7900 GRE/7900M]", "rdna-3"),
        ),
    }
)

# These names currently lack an unambiguous public PCI mapping. Keep their
# source model rows for future evidence, but exclude them from random output.
WINDOWS_GPU_DEVICE_VARIANTS.pop("win_amd_rx_6650m", None)
WINDOWS_GPU_DEVICE_VARIANTS.pop("win_amd_rx_7800m", None)

def count_windows_gpu_device_variants() -> int:
    return sum(len(items) for items in WINDOWS_GPU_DEVICE_VARIANTS.values())
