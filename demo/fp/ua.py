import json
import re
from typing import Any, Dict, List, Optional


MIN_SUPPORTED_CHROMIUM_MAJOR = 140
MAX_SUPPORTED_CHROMIUM_MAJOR = 150


CHROME_VERSION_MAP = {
    "140": "140.0.7301.21",
    "141": "141.0.7350.44",
    "142": "142.0.7399.78",
    "143": "143.0.7445.112",
    "144": "144.0.7492.33",
    "145": "145.0.7541.65",
    "146": "146.0.7590.99",
    "147": "147.0.7640.22",
    "148": "148.0.7778.97",
    "149": "149.0.7827.54",
    "150": "150.0.7879.54",
}

EDGE_VERSION_MAP = {
    "140": "140.0.3451.12",
    "141": "141.0.3498.44",
    "142": "142.0.3545.78",
    "143": "143.0.3592.112",
    "144": "144.0.3639.33",
    "145": "145.0.3686.65",
    "146": "146.0.3733.99",
    "147": "147.0.3780.22",
    "148": "148.0.3967.54",
    "149": "149.0.4022.52",
    "150": "150.0.4078.48",
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
    chrome_full = resolve_full_version(
        options.get("chromeFullVersion") or parsed["fullChromeVersion"],
        CHROME_VERSION_MAP,
    )
    edge_full = (
        resolve_full_version(
            options.get("edgeFullVersion") or parsed["fullEdgeVersion"],
            EDGE_VERSION_MAP,
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
        return "Linux armv8l"
    return "Linux x86_64"


def get_platform_defaults(ua_string: str, platform: Optional[str] = None) -> Dict[str, Any]:
    ua = str(ua_string or "")
    platform = platform or get_platform(ua_string)
    if platform == "Android":
        return {"architecture": "arm", "bitness": "64", "mobile": True}
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
        return android_match.group(1) if android_match else ""
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
        "architecture": options.get("architecture") or defaults["architecture"],
        "bitness": str(options.get("bitness") or defaults["bitness"]),
        "mobile": bool(options["mobile"]) if "mobile" in options else defaults["mobile"],
        "model": options.get("model") or "",
        "platform": platform,
        "platformVersion": options.get("platformVersion")
        or get_platform_version(ua_string, parsed["chromeMajorInt"], platform),
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
