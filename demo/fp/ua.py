import json
import re
from typing import Any, Dict, List, Optional


MIN_SUPPORTED_CHROMIUM_MAJOR = 140
MAX_SUPPORTED_CHROMIUM_MAJOR = 151


CHROME_VERSION_MAP = {
    # Released Stable branch versions.  The previous table incremented an
    # invented build number and produced full-version UA-CH values that never
    # existed in Chrome.  Where desktop patches differ by one across systems,
    # use the release value common to Linux and at least one Windows/macOS
    # rollout.
    "140": "140.0.7339.80",
    "141": "141.0.7390.54",
    "142": "142.0.7444.59",
    "143": "143.0.7499.40",
    "144": "144.0.7559.225",
    "145": "145.0.7632.116",
    "146": "146.0.7680.177",
    "147": "147.0.7727.137",
    "148": "148.0.7778.216",
    "149": "149.0.7827.54",
    "150": "150.0.7871.187",
    "151": "151.0.7922.47",
}

EDGE_VERSION_MAP = {
    # Microsoft-published Stable/Extended Stable versions, one real version
    # per supported major.  UA strings supplied with an explicit full version
    # continue to take precedence over this reduced-UA fallback table.
    "140": "140.0.3485.94",
    "141": "141.0.3537.85",
    "142": "142.0.3595.90",
    "143": "143.0.3650.139",
    "144": "144.0.3719.151",
    "145": "145.0.3800.97",
    "146": "146.0.3856.130",
    "147": "147.0.3912.98",
    "148": "148.0.3967.96",
    "149": "149.0.4022.98",
    "150": "150.0.4078.99",
    "151": "151.0.4129.15",
}

# Android is released on its own train.  Reusing the desktop patch maps makes
# high-entropy UA-CH claim builds that were never shipped by the mobile
# browser.  Chrome values are the official Android/CfT builds used by the
# project's 140-151 Pixel evidence matrix.  Edge values are published Mobile
# Stable builds from Microsoft's mobile release notes.
ANDROID_CHROME_VERSION_MAP = {
    "140": "140.0.7339.207",
    "141": "141.0.7390.122",
    "142": "142.0.7444.175",
    "143": "143.0.7499.192",
    "144": "144.0.7559.133",
    "145": "145.0.7632.117",
    "146": "146.0.7680.165",
    "147": "147.0.7727.117",
    "148": "148.0.7778.178",
    "149": "149.0.7827.155",
    "150": "150.0.7871.124",
    "151": "151.0.7922.138",
}

ANDROID_EDGE_VERSION_MAP = {
    "140": "140.0.3485.98",
    "141": "141.0.3537.99",
    "142": "142.0.3595.107",
    "143": "143.0.3650.139",
    "144": "144.0.3719.115",
    "145": "145.0.3800.99",
    "146": "146.0.3856.102",
    "147": "147.0.3912.87",
    "148": "148.0.3967.97",
    "149": "149.0.4022.105",
    "150": "150.0.4078.96",
    "151": "151.0.4129.72",
}

GREASED_CHARS = [" ", "(", ":", "-", ".", "/", ")", ";", "=", "?", "_"]
GREASED_VERSIONS = ["8", "99", "24"]
BRAND_ORDERS = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
]


def _parse_int(value: Any, fallback: int = 0) -> int:
    try:
        return int(value)
    except (TypeError, ValueError):
        return fallback


def clamp_supported_chromium_major(value: Any) -> int:
    major = _parse_int(value, 143) or 143
    return max(MIN_SUPPORTED_CHROMIUM_MAJOR, min(MAX_SUPPORTED_CHROMIUM_MAJOR, major))


def has_specific_full_version(value: Any) -> bool:
    parts = str(value or "").split(".")
    if len(parts) < 4:
        return False
    # Chromium reduced UA strings use major.0.0.0. UA-CH full-version fields
    # must keep the concrete build/patch version from the version maps.
    return any(_parse_int(part, 0) for part in parts[2:4])


def parse_ua(ua_string: str) -> Dict[str, Any]:
    ua = str(ua_string or "")
    chrome_match = re.search(r"\b(?:Chrome|Chromium)/([\d.]+)", ua)
    # Desktop Edge uses Edg/, while Android Edge uses EdgA/. Both contribute
    # the Microsoft Edge UA-CH brand and full-version fields.
    edge_match = re.search(r"\b(?:Edg|EdgA)/([\d.]+)", ua)
    full_chrome_version = chrome_match.group(1) if chrome_match else "143.0.0.0"
    chrome_major_int = clamp_supported_chromium_major(str(full_chrome_version).split(".")[0])
    chrome_major = str(chrome_major_int)
    full_edge_version = edge_match.group(1) if edge_match else ""
    edge_major = str(clamp_supported_chromium_major(full_edge_version.split(".")[0])) if full_edge_version else ""
    return {
        "ua": ua,
        "fullChromeVersion": full_chrome_version,
        "chromeMajor": chrome_major,
        "chromeMajorInt": chrome_major_int,
        "fullEdgeVersion": full_edge_version,
        "edgeMajor": edge_major,
        "isEdge": edge_match is not None,
    }


def resolve_full_version(version: str, version_map: Dict[str, str]) -> str:
    text = str(version or "")
    raw_major = text.split(".")[0]
    major = str(clamp_supported_chromium_major(raw_major))
    if has_specific_full_version(text) and raw_major == major:
        return text
    return version_map[major]


def get_grease_data(seed: int, full_version_mode: bool = False) -> Dict[str, Any]:
    value = _parse_int(seed, 0) or 0
    order = BRAND_ORDERS[value % len(BRAND_ORDERS)]
    version = GREASED_VERSIONS[value % len(GREASED_VERSIONS)]
    if full_version_mode:
        version += ".0.0.0"
    return {
        "brand": f"Not{GREASED_CHARS[value % len(GREASED_CHARS)]}A{GREASED_CHARS[(value + 1) % len(GREASED_CHARS)]}Brand",
        "version": version,
        "order": order,
    }


def make_brand_lists(ua_string: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
    options = options or {}
    parsed = parse_ua(ua_string)
    android = get_platform(ua_string) == "Android"
    chrome_version_map = (
        ANDROID_CHROME_VERSION_MAP if android else CHROME_VERSION_MAP
    )
    edge_version_map = ANDROID_EDGE_VERSION_MAP if android else EDGE_VERSION_MAP
    chrome_full = resolve_full_version(
        options.get("chromeFullVersion") or parsed["fullChromeVersion"],
        chrome_version_map,
    )
    edge_full = (
        resolve_full_version(
            options.get("edgeFullVersion") or parsed["fullEdgeVersion"],
            edge_version_map,
        )
        if parsed["isEdge"]
        else ""
    )
    chrome_major = str(chrome_full).split(".")[0]
    product_name = "Microsoft Edge" if parsed["isEdge"] else "Google Chrome"
    product_major = str(edge_full).split(".")[0] if parsed["isEdge"] else parsed["chromeMajor"]

    grease = get_grease_data(chrome_major, False)
    grease_full = get_grease_data(chrome_major, True)
    brands: List[Optional[Dict[str, str]]] = [None, None, None]
    full_version_list: List[Optional[Dict[str, str]]] = [None, None, None]

    brands[grease["order"][0]] = {"brand": grease["brand"], "version": grease["version"]}
    brands[grease["order"][1]] = {"brand": "Chromium", "version": chrome_major}
    brands[grease["order"][2]] = {"brand": product_name, "version": product_major}

    full_version_list[grease["order"][0]] = {
        "brand": grease_full["brand"],
        "version": grease_full["version"],
    }
    full_version_list[grease["order"][1]] = {"brand": "Chromium", "version": chrome_full}
    full_version_list[grease["order"][2]] = {
        "brand": product_name,
        "version": edge_full or chrome_full,
    }

    return {
        "brands": [item for item in brands if item is not None],
        "fullVersionList": [item for item in full_version_list if item is not None],
        "uaFullVersion": edge_full or chrome_full,
    }


def get_platform(ua_string: str) -> str:
    ua = str(ua_string or "")
    if re.search(r"Macintosh|Mac OS X", ua, re.I):
        return "macOS"
    if re.search(r"Windows NT", ua, re.I):
        return "Windows"
    if re.search(r"Android", ua, re.I):
        return "Android"
    if re.search(r"Linux|X11", ua, re.I):
        return "Linux"
    return "Windows"


def get_base_platform(ua_string: str) -> str:
    platform = get_platform(ua_string)
    if platform == "macOS":
        return "MacIntel"
    if platform == "Windows":
        return "Win32"
    if platform == "Android":
        # Chromium exposes the ARM ISA revision here; the final character is
        # the digit one, not a lower-case L.
        return "Linux armv81"
    return "Linux x86_64"


def get_platform_defaults(ua_string: str, platform: Optional[str] = None) -> Dict[str, Any]:
    ua = str(ua_string or "")
    platform = platform or get_platform(ua_string)
    if platform == "Android":
        # HTTPS evidence from Chromium Android 151 returns empty high-entropy
        # architecture/bitness values even on the arm64 Pixel test device.
        return {"architecture": "", "bitness": "", "mobile": True}
    if platform == "Windows" and re.search(r"\bARM64\b|\bARM\b", ua, re.I):
        return {"architecture": "arm", "bitness": "64", "mobile": False}
    return {"architecture": "x86", "bitness": "64", "mobile": False}


def get_platform_version(ua_string: str, major_version: int, platform: Optional[str] = None) -> str:
    ua = str(ua_string or "")
    platform = platform or get_platform(ua_string)
    if platform == "macOS":
        mac_match = re.search(r"Mac OS X ([\d_]+)", ua)
        return mac_match.group(1).replace("_", ".") if mac_match else "10.15.7"
    if platform == "Android":
        android_match = re.search(r"Android ([\d.]+)", ua)
        if not android_match:
            return ""
        components = android_match.group(1).split(".")[:3]
        return ".".join(components + ["0"] * (3 - len(components)))
    if platform == "Linux":
        return ""

    nt_match = re.search(r"Windows NT ([\d.]+)", ua)
    if not nt_match:
        return "15.0.0"
    nt_version = nt_match.group(1)
    if nt_version == "10.0":
        return "15.0.0" if int(major_version) >= 120 else "10.0.0"
    if nt_version == "6.3":
        return "6.3.0"
    if nt_version == "6.2":
        return "6.2.0"
    if nt_version == "6.1":
        return "6.1.0"
    return "15.0.0"


def generate_ua_data_from_ua(ua_string: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
    options = options or {}
    parsed = parse_ua(ua_string)
    platform = options.get("platform") or get_platform(ua_string)
    defaults = get_platform_defaults(ua_string, platform)
    brand_data = make_brand_lists(ua_string, options)
    return {
        "architecture": (
            options["architecture"]
            if "architecture" in options
            else defaults["architecture"]
        ),
        "bitness": str(
            options["bitness"] if "bitness" in options else defaults["bitness"]
        ),
        "mobile": bool(options["mobile"]) if "mobile" in options else defaults["mobile"],
        "model": options.get("model") or "",
        "platform": platform,
        "platformVersion": (
            options["platformVersion"]
            if "platformVersion" in options
            else get_platform_version(ua_string, parsed["chromeMajorInt"], platform)
        ),
        "uaFullVersion": brand_data["uaFullVersion"],
        "wow64": bool(options.get("wow64")),
        "brands": brand_data["brands"],
        "fullVersionList": brand_data["fullVersionList"],
        "formFactors": options.get("formFactors") or ["Desktop"],
    }


def format_brand_list(items: List[Dict[str, str]]) -> str:
    return ", ".join(
        f'"{item.get("brand") or item.get("name")}";v="{item.get("version")}"'
        for item in (items or [])
    )


def ua_data_to_headers(ua_data: Dict[str, Any]) -> Dict[str, str]:
    return {
        "sec-ch-ua": format_brand_list(ua_data.get("brands", [])),
        "sec-ch-ua-full-version-list": format_brand_list(ua_data.get("fullVersionList", [])),
        "sec-ch-ua-platform": f'"{ua_data.get("platform", "")}"',
        "sec-ch-ua-arch": f'"{ua_data.get("architecture", "")}"',
        "sec-ch-ua-bitness": f'"{ua_data.get("bitness", "")}"',
        "sec-ch-ua-model": f'"{ua_data.get("model") or ""}"',
        "sec-ch-ua-platform-version": f'"{ua_data.get("platformVersion") or ""}"',
        "sec-ch-ua-mobile": "?1" if ua_data.get("mobile") else "?0",
        "sec-ch-ua-wow64": "?1" if ua_data.get("wow64") else "?0",
    }


def generate_headers_from_ua(ua_string: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, str]:
    return ua_data_to_headers(generate_ua_data_from_ua(ua_string, options))


def get_ua_sec(version: int, browser: int) -> str:
    version = clamp_supported_chromium_major(version)
    browser_token = "Edg" if browser == 0 else "Chrome"
    ua = (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
        f"(KHTML, like Gecko) Chrome/{version}.0.0.0 Safari/537.36"
    )
    if browser_token == "Edg":
        ua += f" Edg/{version}.0.0.0"
    return generate_headers_from_ua(ua)["sec-ch-ua"]


if __name__ == "__main__":
    import sys

    ua_arg = (
        sys.argv[1]
        if len(sys.argv) > 1
        else "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36"
    )
    ua_data = generate_ua_data_from_ua(ua_arg)
    print(
        json.dumps(
            {
                "uaData": ua_data,
                "headers": ua_data_to_headers(ua_data),
                "basePlatform": get_base_platform(ua_arg),
            },
            ensure_ascii=False,
            indent=2,
        )
    )
