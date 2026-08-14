"""Collect Android-only API behavior on a real HTTPS page with ChromeDriver."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import time

from collect_android_chromium_https_matrix import ensure_driver
from collect_android_chromium_surface_webdriver import (
    request_json,
    wait_driver,
    webdriver_value,
)
from collect_android_chromium_surfaces import DEFAULT_CACHE, SNAPSHOTS, ensure_apk, find_adb, run_adb


ROOT = Path(__file__).resolve().parents[1]
PROBE = ROOT / "tools" / "android_mobile_api_behavior_probe.js"
OUTPUT = ROOT / "build" / "chromium-android-version-surfaces" / "android-151-https-api-behavior.json"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--adb", type=Path)
    parser.add_argument("--device", default="9C181021C0D7D6")
    parser.add_argument("--major", type=int, default=151)
    parser.add_argument("--url", default="https://example.com/")
    parser.add_argument("--output", type=Path, default=OUTPUT)
    parser.add_argument("--port", type=int, default=9515)
    arguments = parser.parse_args()
    if not arguments.url.startswith("https://"):
        raise ValueError("formal behavior evidence must use HTTPS")
    adb = find_adb(arguments.adb)
    _, revision = SNAPSHOTS[arguments.major]
    apk, _ = ensure_apk(DEFAULT_CACHE, revision)
    driver, _ = ensure_driver(
        ROOT / "build" / "browser-evidence" / "chromedriver-android-host",
        arguments.major,
    )
    package = "org.chromium.chrome"
    process: subprocess.Popen[str] | None = None
    session_id: str | None = None
    base = f"http://127.0.0.1:{arguments.port}"
    try:
        subprocess.run([str(adb), "-s", arguments.device, "uninstall", package], capture_output=True, timeout=60)
        for user in ("0", "current"):
            run_adb(adb, "-s", arguments.device, "shell", "am", "force-stop", "--user", user, "com.android.chrome")
        run_adb(adb, "-s", arguments.device, "install", "-r", "-t", str(apk), timeout=300)
        process = subprocess.Popen(
            [str(driver), f"--port={arguments.port}", "--log-level=WARNING"],
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )
        wait_driver(arguments.port)
        created = request_json("POST", f"{base}/session", {"capabilities": {"alwaysMatch": {
            "browserName": "chrome",
            "goog:chromeOptions": {
                "androidPackage": package,
                "androidDeviceSerial": arguments.device,
                "args": ["--disable-fre", "--no-default-browser-check"],
            },
        }}}, timeout=120)
        value = created["value"]
        session_id = value["sessionId"]
        session = f"{base}/session/{session_id}"
        request_json("POST", f"{session}/url", {"url": arguments.url}, timeout=120)
        source = PROBE.read_text(encoding="utf-8")
        raw = webdriver_value(request_json("POST", f"{session}/execute/async", {
            "script": (
                "const done=arguments[arguments.length-1];"
                "Promise.resolve(eval(arguments[0])).then("
                "value=>done(JSON.stringify(value)),"
                "error=>done(JSON.stringify({probeError:String(error&&error.stack||error)})));"
            ),
            "args": [source],
        }, timeout=180))
        evidence = json.loads(raw)
        if evidence.get("secureContext") is not True:
            raise RuntimeError("behavior probe did not run in a secure context")
        evidence["capture"] = {
            "deviceSerial": arguments.device,
            "snapshotRevision": revision,
            "requestedUrl": arguments.url,
            "capturedAtUnix": time.time(),
        }
        arguments.output.parent.mkdir(parents=True, exist_ok=True)
        arguments.output.write_text(json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(arguments.output)
    finally:
        if session_id is not None:
            try:
                request_json("DELETE", f"{base}/session/{session_id}", timeout=20)
            except Exception:
                pass
        if process is not None:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
        subprocess.run([str(adb), "-s", arguments.device, "uninstall", package], capture_output=True, timeout=60)
        subprocess.run([
            str(adb), "-s", arguments.device, "shell", "monkey", "-p", "com.android.chrome", "1"
        ], capture_output=True, timeout=30)


if __name__ == "__main__":
    main()
