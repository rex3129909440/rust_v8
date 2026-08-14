"""Collect the Android Chromium 140-151 matrix from a real HTTPS page."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import urllib.request
import zipfile

from collect_android_chromium_surface_webdriver import collect_android_surface
from collect_android_chromium_surfaces import (
    DEFAULT_CACHE,
    DEFAULT_OUTPUT,
    SNAPSHOTS,
    ensure_apk,
    find_adb,
    parse_majors,
    run_adb,
)


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_DRIVER_CACHE = (
    ROOT / "build" / "browser-evidence" / "chromedriver-android-host"
)
DRIVER_VERSIONS = {
    140: "140.0.7339.207",
    141: "141.0.7390.122",
    142: "142.0.7444.175",
    143: "143.0.7499.192",
    144: "144.0.7559.133",
    145: "145.0.7632.117",
    146: "146.0.7680.165",
    147: "147.0.7727.117",
    148: "148.0.7778.178",
    149: "149.0.7827.155",
    150: "150.0.7871.124",
    151: "151.0.7922.138",
}


def digest(path: Path) -> str:
    result = hashlib.sha256()
    with path.open("rb") as source:
        while block := source.read(1024 * 1024):
            result.update(block)
    return result.hexdigest()


def ensure_driver(cache: Path, major: int) -> tuple[Path, dict[str, object]]:
    version = DRIVER_VERSIONS[major]
    directory = cache / version
    executable = directory / "chromedriver.exe"
    evidence_file = directory / "evidence.json"
    if executable.is_file() and evidence_file.is_file():
        evidence = json.loads(evidence_file.read_text(encoding="utf-8"))
        if digest(executable) == evidence.get("sha256"):
            return executable, evidence
    url = (
        "https://storage.googleapis.com/chrome-for-testing-public/"
        f"{version}/win64/chromedriver-win64.zip"
    )
    archive = directory / "chromedriver-win64.zip"
    directory.mkdir(parents=True, exist_ok=True)
    urllib.request.urlretrieve(url, archive)
    with zipfile.ZipFile(archive) as source:
        member = next(
            name for name in source.namelist() if name.endswith("/chromedriver.exe")
        )
        with source.open(member) as binary, executable.open("wb") as output:
            shutil.copyfileobj(binary, output)
    evidence: dict[str, object] = {
        "version": version,
        "url": url,
        "archiveSha256": digest(archive),
        "sha256": digest(executable),
    }
    evidence_file.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    return executable, evidence


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--adb", type=Path)
    parser.add_argument("--majors", default="all")
    parser.add_argument("--cache", type=Path, default=DEFAULT_CACHE)
    parser.add_argument("--driver-cache", type=Path, default=DEFAULT_DRIVER_CACHE)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--device")
    parser.add_argument("--url", default="https://example.com/")
    parser.add_argument("--refresh", action="store_true")
    parser.add_argument("--driver-port", type=int, default=9515)
    arguments = parser.parse_args()
    if not arguments.url.lower().startswith("https://"):
        raise ValueError("the formal evidence URL must use HTTPS")
    adb = find_adb(arguments.adb)
    devices = [
        line.split()[0]
        for line in run_adb(adb, "devices").splitlines()[1:]
        if "\tdevice" in line
    ]
    if not devices:
        raise RuntimeError("no authorized Android device is connected")
    device = arguments.device or devices[0]
    arguments.output.mkdir(parents=True, exist_ok=True)
    manifest_path = arguments.output / "android-https-manifest.json"
    manifest: dict[str, object] = {
        "evidenceOrigin": arguments.url,
        "device": device,
        "builds": {},
    }
    if manifest_path.is_file():
        previous = json.loads(manifest_path.read_text(encoding="utf-8"))
        if isinstance(previous, dict):
            manifest.update(previous)
            manifest["evidenceOrigin"] = arguments.url
            manifest["device"] = device
    try:
        for major in parse_majors(arguments.majors):
            output = arguments.output / f"chromium-android-{major}-https-surface.json"
            stable_reference, revision = SNAPSHOTS[major]
            apk, apk_evidence = ensure_apk(arguments.cache, revision)
            driver, driver_evidence = ensure_driver(arguments.driver_cache, major)
            if arguments.refresh or not output.is_file():
                subprocess.run(
                    [str(adb), "-s", device, "uninstall", "org.chromium.chrome"],
                    capture_output=True,
                    text=True,
                    timeout=60,
                )
                # The test Pixel can have Chrome processes in both user 0 and
                # the foreground secondary user. ChromeDriver refuses to
                # launch any Android browser while either remains alive.
                for user in ("0", "current"):
                    run_adb(
                        adb,
                        "-s",
                        device,
                        "shell",
                        "am",
                        "force-stop",
                        "--user",
                        user,
                        "com.android.chrome",
                    )
                run_adb(
                    adb,
                    "-s",
                    device,
                    "install",
                    "-r",
                    "-t",
                    str(apk),
                    timeout=300,
                )
                surface = collect_android_surface(
                    driver,
                    "org.chromium.chrome",
                    device,
                    arguments.url,
                    arguments.driver_port,
                )
                output.write_text(
                    json.dumps(surface, ensure_ascii=False, indent=2) + "\n",
                    encoding="utf-8",
                )
            else:
                surface = json.loads(output.read_text(encoding="utf-8"))
            actual_major = int(
                str(surface["userAgent"]).split("Chrome/", 1)[1].split(".", 1)[0]
            )
            runtime = surface.get("runtimeEvidence", {})
            if actual_major != major:
                raise RuntimeError(
                    f"snapshot revision {revision} is Chromium {actual_major}, expected {major}"
                )
            if not isinstance(runtime, dict) or runtime.get("secureContext") is not True:
                raise RuntimeError(f"Chromium {major} evidence is not a secure context")
            manifest.setdefault("builds", {})[str(major)] = {
                "stableVersionReference": stable_reference,
                "snapshotRevision": revision,
                "apk": apk_evidence,
                "driver": driver_evidence,
                "runtimeUserAgent": surface["userAgent"],
                "windowCount": len(surface["topWindowBeforeFrame"]["names"]),
                "surface": output.name,
                "https": surface["httpsEvidence"],
            }
            manifest_path.write_text(
                json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
                encoding="utf-8",
            )
            print(
                json.dumps(
                    {
                        "major": major,
                        "windowCount": len(surface["topWindowBeforeFrame"]["names"]),
                        "secureContext": True,
                        "output": str(output),
                    },
                    separators=(",", ":"),
                ),
                flush=True,
            )
    finally:
        subprocess.run(
            [str(adb), "-s", device, "uninstall", "org.chromium.chrome"],
            capture_output=True,
            text=True,
            timeout=60,
        )
        # Return the device to the user's stable Chrome after isolated evidence.
        subprocess.run(
            [
                str(adb),
                "-s",
                device,
                "shell",
                "monkey",
                "-p",
                "com.android.chrome",
                "1",
            ],
            capture_output=True,
            text=True,
            timeout=30,
        )


if __name__ == "__main__":
    main()
