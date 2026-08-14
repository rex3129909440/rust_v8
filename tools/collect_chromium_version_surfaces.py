"""Download official Chrome for Testing builds and collect 140-151 HTTPS surfaces."""

from __future__ import annotations

import argparse
import hashlib
import json
import msvcrt
from pathlib import Path
import shutil
import subprocess
import sys
import urllib.request
import zipfile

from playwright.sync_api import sync_playwright


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CACHE = ROOT / "build" / "browser-evidence" / "chrome-for-testing"
DEFAULT_OUTPUT = ROOT / "build" / "chromium-version-surfaces"
INDEX_URL = (
    "https://googlechromelabs.github.io/chrome-for-testing/"
    "known-good-versions-with-downloads.json"
)
PROBE = ROOT / "tools" / "chromium_version_surface_probe.js"
PROBE_URL = "https://example.com/"


class CollectionLock:
    def __init__(self, path: Path) -> None:
        self.path = path
        self.handle = None

    def __enter__(self) -> "CollectionLock":
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.handle = self.path.open("a+b")
        try:
            msvcrt.locking(self.handle.fileno(), msvcrt.LK_NBLCK, 1)
        except OSError as error:
            self.handle.close()
            raise RuntimeError("another Chromium surface collector is already running") from error
        return self

    def __exit__(self, *_: object) -> None:
        assert self.handle is not None
        self.handle.seek(0)
        msvcrt.locking(self.handle.fileno(), msvcrt.LK_UNLCK, 1)
        self.handle.close()


def requested_majors(value: str) -> list[int]:
    if value == "all":
        return list(range(140, 152))
    majors = sorted({int(item) for item in value.split(",")})
    if any(major < 140 or major > 151 for major in majors):
        raise ValueError("majors must be within 140-151")
    return majors


def load_index() -> dict[str, object]:
    with urllib.request.urlopen(INDEX_URL, timeout=60) as response:
        return json.load(response)


def select_builds(index: dict[str, object], majors: list[int]) -> dict[int, dict[str, str]]:
    selected: dict[int, dict[str, str]] = {}
    for entry in index["versions"]:
        version = str(entry["version"])
        major = int(version.split(".", 1)[0])
        if major not in majors:
            continue
        win64 = next(
            (item for item in entry["downloads"]["chrome"] if item["platform"] == "win64"),
            None,
        )
        if win64 is not None:
            selected[major] = {"version": version, "url": str(win64["url"])}
    missing = sorted(set(majors) - set(selected))
    if missing:
        raise RuntimeError(f"Chrome for Testing builds are missing for: {missing}")
    return selected


def download(url: str, target: Path) -> str:
    target.parent.mkdir(parents=True, exist_ok=True)
    temporary = target.with_suffix(target.suffix + ".part")
    # The public storage endpoint can stall before returning response headers
    # behind corporate proxies.  The official gvt1 entry point reaches the
    # identical object and curl gives us retries plus byte-range resumption.
    official_url = url.replace(
        "https://storage.googleapis.com/chrome-for-testing-public/",
        "https://edgedl.me.gvt1.com/edgedl/chrome/chrome-for-testing/",
    )
    command = [
            "curl.exe",
            "--fail",
            "--location",
            "--retry",
            "5",
            "--retry-delay",
            "2",
            "--speed-limit",
            "16384",
            "--speed-time",
            "30",
            "--continue-at",
            "-",
            "--output",
            str(temporary),
            official_url,
        ]
    subprocess.run(command, check=True)
    if not zipfile.is_zipfile(temporary):
        corrupt = temporary.with_suffix(temporary.suffix + ".corrupt")
        if corrupt.exists():
            corrupt = temporary.with_suffix(temporary.suffix + f".{int(temporary.stat().st_mtime)}.corrupt")
        temporary.replace(corrupt)
        subprocess.run(command, check=True)
    if not zipfile.is_zipfile(temporary):
        raise RuntimeError(f"downloaded archive is not a ZIP file: {temporary}")
    digest = hashlib.sha256()
    with temporary.open("rb") as source:
        while block := source.read(1024 * 1024):
            digest.update(block)
    temporary.replace(target)
    return digest.hexdigest()


def ensure_browser(cache: Path, major: int, build: dict[str, str]) -> tuple[Path, str]:
    version = build["version"]
    directory = cache / version
    executable = directory / "chrome-win64" / "chrome.exe"
    archive = cache / f"chrome-{version}-win64.zip"
    checksum_file = archive.with_suffix(".zip.sha256")
    if executable.is_file():
        checksum = checksum_file.read_text(encoding="ascii").strip() if checksum_file.exists() else ""
        return executable, checksum
    if archive.is_file() and zipfile.is_zipfile(archive):
        digest = hashlib.sha256()
        with archive.open("rb") as source:
            while block := source.read(1024 * 1024):
                digest.update(block)
        checksum = digest.hexdigest()
    else:
        checksum = download(build["url"], archive)
    checksum_file.write_text(checksum + "\n", encoding="ascii")
    if directory.exists():
        shutil.rmtree(directory)
    directory.mkdir(parents=True)
    with zipfile.ZipFile(archive) as package:
        package.extractall(directory)
    if not executable.is_file():
        raise RuntimeError(f"Chrome {major} executable missing after extraction")
    return executable, checksum


def collect(executable: Path) -> dict[str, object]:
    source = PROBE.read_text(encoding="utf-8")
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(executable_path=str(executable), headless=True)
        try:
            page = browser.new_page(viewport={"width": 1280, "height": 720})
            page.goto(PROBE_URL, wait_until="load")
            surface = page.evaluate(source)
            if surface.get("runtimeEvidence", {}).get("secureContext") is not True:
                raise RuntimeError("desktop evidence did not run in a secure context")
            surface["httpsEvidence"] = {
                "requestedUrl": PROBE_URL,
                "actualUrl": page.url,
                "headless": True,
            }
            return surface
        finally:
            browser.close()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--majors", default="all", help="all or comma-separated 140-151 majors")
    parser.add_argument("--cache", type=Path, default=DEFAULT_CACHE)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--refresh", action="store_true")
    arguments = parser.parse_args()

    with CollectionLock(arguments.cache / ".surface-collection.lock"):
        majors = requested_majors(arguments.majors)
        builds = select_builds(load_index(), majors)
        arguments.output.mkdir(parents=True, exist_ok=True)
        manifest: dict[str, object] = {"source": INDEX_URL, "builds": {}}
        for major in majors:
            output = arguments.output / f"chromium-{major}-surface.json"
            executable, checksum = ensure_browser(arguments.cache, major, builds[major])
            if arguments.refresh or not output.exists():
                print(f"collecting Chromium {major} from {builds[major]['version']}", flush=True)
                surface = collect(executable)
                output.write_text(
                    json.dumps(surface, ensure_ascii=False, indent=2, sort_keys=False) + "\n",
                    encoding="utf-8",
                )
            manifest["builds"][str(major)] = {
                **builds[major],
                "archiveSha256": checksum,
                "surface": output.name,
            }
        (arguments.output / "manifest.json").write_text(
            json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        print(json.dumps(manifest, ensure_ascii=False, separators=(",", ":")))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"surface collection failed: {error}", file=sys.stderr)
        raise
