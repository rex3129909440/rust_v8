"""Collect one Android Chromium surface from an HTTPS page via ChromeDriver.

ChromeDriver is the supported way to launch Chrome/Chromium on Android.  The
browser page is deliberately an HTTPS origin; the JavaScript probe is evaluated
after navigation so secure-context-only APIs are represented in the evidence.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import subprocess
import time
import urllib.error
import urllib.request


ROOT = Path(__file__).resolve().parents[1]
PROBE = ROOT / "tools" / "chromium_version_surface_probe.js"
WEBDRIVER_WINDOW_ARTIFACT = re.compile(
    r"^cdc_[A-Za-z0-9]+_(?:Array|Object|Promise|Proxy|Symbol|JSON|Window)$"
)


def sanitize_webdriver_artifacts(evidence: dict[str, object]) -> list[str]:
    """Remove ChromeDriver transport globals from captured Window surfaces."""

    removed: list[str] = []

    def is_artifact(name: str) -> bool:
        return name == "ret_nodes" or WEBDRIVER_WINDOW_ARTIFACT.fullmatch(name) is not None

    for field in ("topWindowBeforeFrame", "topWindowWithFrame", "iframeWindow"):
        surface = evidence.get(field)
        if not isinstance(surface, dict):
            continue
        for list_name in ("names", "keys"):
            values = surface.get(list_name)
            if isinstance(values, list):
                artifacts = [
                    value for value in values
                    if isinstance(value, str) and is_artifact(value)
                ]
                removed.extend(artifacts)
                surface[list_name] = [
                    value for value in values
                    if not (isinstance(value, str) and is_artifact(value))
                ]
        descriptors = surface.get("descriptors")
        if isinstance(descriptors, dict):
            for name in list(descriptors):
                if is_artifact(name):
                    descriptors.pop(name)
    for field in (
        "constructorPrototypes",
        "constructorStatics",
        "globalObjects",
        "iframeConstructorPrototypes",
        "iframeConstructorStatics",
        "iframeGlobalObjects",
    ):
        owners = evidence.get(field)
        if isinstance(owners, dict):
            for name in list(owners):
                if is_artifact(name):
                    owners.pop(name)
                    removed.append(name)
    evidence["webdriverArtifactSanitization"] = {
        "rule": "ChromeDriver cdc_* transport globals and ret_nodes",
        "removed": sorted(set(removed)),
    }
    return removed


def request_json(
    method: str,
    url: str,
    body: dict[str, object] | None = None,
    timeout: float = 60,
) -> dict[str, object]:
    payload = None
    headers: dict[str, str] = {}
    if body is not None:
        payload = json.dumps(body, separators=(",", ":")).encode()
        headers["Content-Type"] = "application/json"
    request = urllib.request.Request(url, data=payload, headers=headers, method=method)
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            value = json.load(response)
    except urllib.error.HTTPError as error:
        detail = error.read().decode(errors="replace")
        raise RuntimeError(f"WebDriver HTTP {error.code}: {detail}") from error
    if isinstance(value, dict) and value.get("value") is not None:
        error = value["value"]
        if isinstance(error, dict) and error.get("error"):
            raise RuntimeError(
                f"WebDriver {error.get('error')}: {error.get('message', '')}"
            )
    return value


def webdriver_value(value: dict[str, object]) -> object:
    return value.get("value")


def wait_driver(port: int, timeout: float = 20) -> None:
    deadline = time.monotonic() + timeout
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            request_json("GET", f"http://127.0.0.1:{port}/status", timeout=2)
            return
        except Exception as error:
            last_error = error
            time.sleep(0.1)
    raise RuntimeError(f"ChromeDriver did not start: {last_error}")


def collect_android_surface(
    driver: Path,
    package: str,
    device: str,
    url: str,
    port: int = 9515,
) -> dict[str, object]:
    if not url.lower().startswith("https://"):
        raise ValueError("evidence URL must use HTTPS")

    process = subprocess.Popen(
        [str(driver), f"--port={port}", "--log-level=WARNING"],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    session_id: str | None = None
    base = f"http://127.0.0.1:{port}"
    try:
        wait_driver(port)
        created = request_json(
            "POST",
            f"{base}/session",
            {
                "capabilities": {
                    "alwaysMatch": {
                        "browserName": "chrome",
                        "goog:chromeOptions": {
                            "androidPackage": package,
                            "androidDeviceSerial": device,
                            "args": [
                                "--disable-fre",
                                "--no-default-browser-check",
                            ],
                        },
                    }
                }
            },
            timeout=120,
        )
        value = created.get("value")
        if not isinstance(value, dict) or not isinstance(value.get("sessionId"), str):
            raise RuntimeError(f"invalid ChromeDriver session response: {created}")
        session_id = value["sessionId"]
        session = f"{base}/session/{session_id}"
        request_json("POST", f"{session}/url", {"url": url}, timeout=120)
        actual_url = webdriver_value(request_json("GET", f"{session}/url"))
        if not isinstance(actual_url, str) or not actual_url.startswith("https://"):
            raise RuntimeError(f"evidence page did not remain on HTTPS: {actual_url!r}")
        source = PROBE.read_text(encoding="utf-8")
        evidence = webdriver_value(
            request_json(
                "POST",
                f"{session}/execute/async",
                {
                    "script": (
                        "const done=arguments[arguments.length-1];"
                        "Promise.resolve(eval(arguments[0])).then("
                        "value=>done(JSON.stringify(value)),"
                        "e=>done(JSON.stringify({probeError:String(e&&e.stack||e)})));"
                    ),
                    "args": [source],
                },
                timeout=180,
            )
        )
        if not isinstance(evidence, str):
            raise RuntimeError("probe returned a non-string transport result")
        evidence = json.loads(evidence)
        if not isinstance(evidence, dict):
            raise RuntimeError("probe returned a non-object result")
        sanitize_webdriver_artifacts(evidence)
        evidence["httpsEvidence"] = {
            "requestedUrl": url,
            "actualUrl": actual_url,
            "webdriverPackage": package,
            "deviceSerial": device,
        }
        runtime = evidence.get("runtimeEvidence")
        if not isinstance(runtime, dict) or runtime.get("secureContext") is not True:
            raise RuntimeError("probe did not run in a secure context")
        return evidence
    finally:
        if session_id is not None:
            try:
                request_json(
                    "DELETE",
                    f"{base}/session/{session_id}",
                    timeout=20,
                )
            except Exception:
                pass
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--driver", type=Path, required=True)
    parser.add_argument("--package", required=True)
    parser.add_argument("--device", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--port", type=int, default=9515)
    parser.add_argument(
        "--url",
        default="https://example.com/",
        help="HTTPS evidence origin; must use a valid trusted certificate",
    )
    arguments = parser.parse_args()
    evidence = collect_android_surface(
        arguments.driver,
        arguments.package,
        arguments.device,
        arguments.url,
        arguments.port,
    )
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    arguments.output.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    runtime = evidence["runtimeEvidence"]
    print(
        json.dumps(
            {
                "output": str(arguments.output),
                "url": evidence["httpsEvidence"]["actualUrl"],
                "userAgent": evidence.get("userAgent"),
                "windowCount": len(
                    evidence.get("topWindowBeforeFrame", {}).get("names", [])
                ),
                "secureContext": runtime.get("secureContext"),
            },
            separators=(",", ":"),
        )
    )


if __name__ == "__main__":
    main()
