"""Collect Chromium 140-151 Android web-platform evidence on a real device.

Official Chromium Android ARM64 snapshots are installed as ``org.chromium.chrome``.
The user's stable ``com.android.chrome`` package and its data are never replaced.
Each snapshot is uninstalled after capture so versions cannot share browser state.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import binascii
import contextlib
import hashlib
import http.server
import json
from pathlib import Path
import shutil
import socket
import struct
import subprocess
import threading
import time
import urllib.request
import zipfile
import zlib

from playwright.sync_api import sync_playwright


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CACHE = ROOT / "build" / "browser-evidence" / "chromium-android-arm64"
DEFAULT_OUTPUT = ROOT / "build" / "chromium-android-version-surfaces"
PROBE = ROOT / "tools" / "chromium_version_surface_probe.js"
SNAPSHOT_ROOT = (
    "https://commondatastorage.googleapis.com/chromium-browser-snapshots/"
    "Android_Arm64"
)
SNAPSHOTS: dict[int, tuple[str, int]] = {
    140: ("140.0.7339.208", 1496472),
    141: ("141.0.7390.123", 1509326),
    142: ("142.0.7444.173", 1522577),
    143: ("143.0.7499.194", 1536363),
    144: ("144.0.7559.133", 1552474),
    145: ("145.0.7632.161", 1568189),
    146: ("146.0.7680.178", 1582188),
    147: ("147.0.7727.138", 1596546),
    148: ("148.0.7778.217", 1610480),
    149: ("149.0.7827.201", 1625082),
    150: ("150.0.7871.189", 1639808),
    151: ("151.0.7922.138", 1654401),
}


def parse_majors(value: str) -> list[int]:
    if value == "all":
        return list(SNAPSHOTS)
    majors = sorted({int(item) for item in value.split(",")})
    unsupported = [major for major in majors if major not in SNAPSHOTS]
    if unsupported:
        raise ValueError(f"unsupported Android Chromium majors: {unsupported}")
    return majors


def find_adb(explicit: Path | None) -> Path:
    candidates = [
        explicit,
        Path(shutil.which("adb") or ""),
        Path(
            r"C:\Users\EDY\Downloads\QtScrcpy-win-x64-v3.3.3"
            r"\QtScrcpy-win-x64-v3.3.3\adb.exe"
        ),
    ]
    for candidate in candidates:
        if candidate and candidate.is_file():
            return candidate.resolve()
    raise FileNotFoundError("adb executable was not found")


def run_adb(adb: Path, *arguments: str, timeout: int = 120) -> str:
    completed = subprocess.run(
        [str(adb), *arguments],
        check=True,
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    return completed.stdout.strip()


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        while block := source.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def range_bytes(url: str, start: int, end: int, timeout: int = 60) -> bytes:
    request = urllib.request.Request(url, headers={"Range": f"bytes={start}-{end}"})
    with urllib.request.urlopen(request, timeout=timeout) as response:
        value = response.read()
    expected = end - start + 1
    if len(value) != expected:
        raise RuntimeError(
            f"HTTP range {start}-{end} returned {len(value)} bytes, expected {expected}"
        )
    return value


def download_parallel(
    url: str,
    target: Path,
    start_offset: int,
    size: int,
    workers: int = 64,
) -> tuple[Path, ...]:
    """Download one immutable byte range in small concurrent pieces."""

    part_size = 1024 * 1024
    parts = []
    for index, relative_start in enumerate(range(0, size, part_size)):
        relative_end = min(size - 1, relative_start + part_size - 1)
        start = start_offset + relative_start
        end = start_offset + relative_end
        parts.append((index, start, end, target.with_suffix(f".part.{index:04d}")))

    def fetch(part: tuple[int, int, int, Path]) -> Path:
        _, start, end, path = part
        expected = end - start + 1
        if path.is_file() and path.stat().st_size == expected:
            return path
        last_error: Exception | None = None
        for attempt in range(12):
            try:
                value = range_bytes(url, start, end, timeout=90)
                path.write_bytes(value)
                return path
            except Exception as error:  # pragma: no cover - network retries
                last_error = error
                time.sleep(min(20, 1 + attempt * 2))
        raise RuntimeError(f"cannot download range {start}-{end}: {last_error}")

    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as executor:
        completed = list(executor.map(fetch, parts))
    return tuple(completed)


def remote_zip_member(url: str, archive_size: int, member_name: str) -> dict[str, int]:
    tail_size = min(archive_size, 65_557)
    tail = range_bytes(url, archive_size - tail_size, archive_size - 1)
    eocd_offset = tail.rfind(b"PK\x05\x06")
    if eocd_offset < 0:
        raise RuntimeError("ZIP end-of-central-directory record is missing")
    eocd = struct.unpack_from("<4s4H2LH", tail, eocd_offset)
    central_size, central_offset = eocd[5], eocd[6]
    central = range_bytes(
        url, central_offset, central_offset + central_size - 1
    )
    offset = 0
    while offset < len(central):
        values = struct.unpack_from("<4s6H3L5H2L", central, offset)
        if values[0] != b"PK\x01\x02":
            raise RuntimeError("invalid ZIP central-directory entry")
        name_length, extra_length, comment_length = values[10:13]
        name = central[offset + 46 : offset + 46 + name_length].decode("utf-8")
        if name == member_name:
            local_offset = values[16]
            local = range_bytes(url, local_offset, local_offset + 65_535)
            local_values = struct.unpack_from("<4s5H3L2H", local, 0)
            if local_values[0] != b"PK\x03\x04":
                raise RuntimeError("invalid ZIP local entry")
            data_offset = local_offset + 30 + local_values[9] + local_values[10]
            return {
                "method": values[4],
                "crc32": values[7],
                "compressedSize": values[8],
                "uncompressedSize": values[9],
                "dataOffset": data_offset,
            }
        offset += 46 + name_length + extra_length + comment_length
    raise FileNotFoundError(f"{member_name} is missing from remote snapshot ZIP")


def ensure_apk(cache: Path, revision: int) -> tuple[Path, dict[str, object]]:
    cache.mkdir(parents=True, exist_ok=True)
    apk = cache / str(revision) / "ChromePublic.apk"
    metadata_file = apk.with_suffix(".apk.evidence.json")
    if apk.is_file() and metadata_file.is_file():
        metadata = json.loads(metadata_file.read_text(encoding="utf-8"))
        if apk.stat().st_size == metadata["uncompressedSize"] and sha256(apk) == metadata["apkSha256"]:
            return apk, metadata

    url = f"{SNAPSHOT_ROOT}/{revision}/chrome-android.zip"
    request = urllib.request.Request(url, method="HEAD")
    with urllib.request.urlopen(request, timeout=30) as response:
        archive_size = int(response.headers["Content-Length"])
        etag = response.headers.get("ETag", "").strip('"')
        generation = response.headers.get("x-goog-generation", "")
    member = remote_zip_member(url, archive_size, "chrome-android/apks/ChromePublic.apk")
    if member["method"] != 8:
        raise RuntimeError(f"unsupported remote ZIP method {member['method']}")
    apk.parent.mkdir(parents=True, exist_ok=True)
    compressed_target = apk.with_suffix(".apk.compressed")
    parts = download_parallel(
        url,
        compressed_target,
        int(member["dataOffset"]),
        int(member["compressedSize"]),
    )
    temporary = apk.with_suffix(".apk.assembling")
    decompressor = zlib.decompressobj(-zlib.MAX_WBITS)
    crc = 0
    written = 0
    with temporary.open("wb") as destination:
        for part in parts:
            with part.open("rb") as source:
                while block := source.read(1024 * 1024):
                    decoded = decompressor.decompress(block)
                    if decoded:
                        destination.write(decoded)
                        crc = binascii.crc32(decoded, crc)
                        written += len(decoded)
        decoded = decompressor.flush()
        if decoded:
            destination.write(decoded)
            crc = binascii.crc32(decoded, crc)
            written += len(decoded)
    for part in parts:
        part.unlink()
    if written != member["uncompressedSize"] or crc & 0xFFFFFFFF != member["crc32"]:
        raise RuntimeError("ChromePublic.apk size or CRC32 does not match official ZIP metadata")
    temporary.replace(apk)
    metadata: dict[str, object] = {
        "url": url,
        "archiveSize": archive_size,
        "archiveEtagMd5": etag,
        "archiveGeneration": generation,
        **member,
        "apkSha256": sha256(apk),
    }
    metadata_file.write_text(
        json.dumps(metadata, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    return apk, metadata


class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, _format: str, *_arguments: object) -> None:
        return


@contextlib.contextmanager
def evidence_server(port: int):
    handler = lambda *args, **kwargs: QuietHandler(  # noqa: E731
        *args, directory=str(ROOT), **kwargs
    )
    server = http.server.ThreadingHTTPServer(("127.0.0.1", port), handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)


def wait_for_devtools(port: int, expected_package: str, timeout: float = 30) -> dict[str, object]:
    deadline = time.monotonic() + timeout
    url = f"http://127.0.0.1:{port}/json/version"
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            with urllib.request.urlopen(url, timeout=2) as response:
                value = json.load(response)
            if value.get("Android-Package") == expected_package:
                return value
        except Exception as error:  # pragma: no cover - device timing
            last_error = error
        time.sleep(0.25)
    raise RuntimeError(f"Android DevTools endpoint did not start: {last_error}")


def collect_surface(port: int, probe_url: str) -> dict[str, object]:
    source = PROBE.read_text(encoding="utf-8")
    with sync_playwright() as playwright:
        browser = playwright.chromium.connect_over_cdp(f"http://127.0.0.1:{port}")
        context = browser.contexts[0]
        page = context.new_page()
        try:
            page.goto(probe_url, wait_until="load", timeout=30_000)
            return page.evaluate(source)
        finally:
            page.close()
            browser.close()


def collect_installed(
    adb: Path,
    package: str,
    output: Path,
    port: int,
    server_port: int,
    devtools_socket: str = "chrome_devtools_remote",
) -> dict[str, object]:
    run_adb(adb, "forward", f"tcp:{port}", f"localabstract:{devtools_socket}")
    run_adb(adb, "reverse", f"tcp:{server_port}", f"tcp:{server_port}")
    run_adb(adb, "shell", "monkey", "-p", package, "1")
    devtools = wait_for_devtools(port, package)
    surface = collect_surface(
        port,
        f"http://127.0.0.1:{server_port}/tools/chromium_version_surface_probe.html",
    )
    surface["androidEvidence"] = {
        "package": package,
        "devtools": devtools,
        "device": {
            "model": run_adb(adb, "shell", "getprop", "ro.product.model"),
            "product": run_adb(adb, "shell", "getprop", "ro.product.name"),
            "androidRelease": run_adb(
                adb, "shell", "getprop", "ro.build.version.release"
            ),
            "buildFingerprint": run_adb(
                adb, "shell", "getprop", "ro.build.fingerprint"
            ),
            "wmSize": run_adb(adb, "shell", "wm", "size"),
            "wmDensity": run_adb(adb, "shell", "wm", "density"),
        },
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(surface, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    run_adb(adb, "forward", "--remove", f"tcp:{port}")
    return surface


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--adb", type=Path)
    parser.add_argument("--majors", default="all")
    parser.add_argument("--cache", type=Path, default=DEFAULT_CACHE)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--port", type=int, default=9223)
    parser.add_argument("--server-port", type=int, default=38473)
    parser.add_argument("--installed-package")
    parser.add_argument("--installed-output", type=Path)
    parser.add_argument("--refresh", action="store_true")
    arguments = parser.parse_args()
    adb = find_adb(arguments.adb)
    if "device" not in run_adb(adb, "devices"):
        raise RuntimeError("no authorized Android device is connected")

    with evidence_server(arguments.server_port):
        if arguments.installed_package:
            if arguments.installed_output is None:
                raise ValueError("--installed-output is required")
            surface = collect_installed(
                adb,
                arguments.installed_package,
                arguments.installed_output,
                arguments.port,
                arguments.server_port,
                "chrome_devtools_remote",
            )
            print(
                json.dumps(
                    {
                        "userAgent": surface["userAgent"],
                        "windowCount": len(surface["topWindowBeforeFrame"]["names"]),
                        "output": str(arguments.installed_output),
                    },
                    separators=(",", ":"),
                )
            )
            return

        arguments.output.mkdir(parents=True, exist_ok=True)
        manifest: dict[str, object] = {
            "source": SNAPSHOT_ROOT,
            "deviceSerials": run_adb(adb, "devices", "-l").splitlines()[1:],
            "builds": {},
        }
        for major in parse_majors(arguments.majors):
            stable_version, revision = SNAPSHOTS[major]
            output = arguments.output / f"chromium-android-{major}-surface.json"
            apk, artifact = ensure_apk(arguments.cache, revision)
            if arguments.refresh or not output.is_file():
                # org.chromium.chrome is an isolated snapshot package. Never
                # uninstall or replace the user's com.android.chrome package.
                subprocess.run(
                    [str(adb), "uninstall", "org.chromium.chrome"],
                    capture_output=True,
                    text=True,
                    timeout=60,
                )
                run_adb(adb, "shell", "am", "force-stop", "com.android.chrome")
                run_adb(adb, "install", "-r", "-t", str(apk), timeout=300)
                try:
                    surface = collect_installed(
                        adb,
                        "org.chromium.chrome",
                        output,
                        arguments.port,
                        arguments.server_port,
                        "chrome_devtools_remote",
                    )
                finally:
                    subprocess.run(
                        [str(adb), "uninstall", "org.chromium.chrome"],
                        capture_output=True,
                        text=True,
                        timeout=60,
                    )
            else:
                surface = json.loads(output.read_text(encoding="utf-8"))
            actual_major = int(str(surface["userAgent"]).split("Chrome/", 1)[1].split(".", 1)[0])
            if actual_major != major:
                raise RuntimeError(
                    f"snapshot revision {revision} is Chromium {actual_major}, expected {major}"
                )
            manifest["builds"][str(major)] = {
                "stableVersionReference": stable_version,
                "revision": revision,
                "archiveUrl": artifact["url"],
                "archiveSize": artifact["archiveSize"],
                "archiveEtagMd5": artifact["archiveEtagMd5"],
                "archiveGeneration": artifact["archiveGeneration"],
                "apk": str(apk.relative_to(arguments.cache)),
                "apkSha256": artifact["apkSha256"],
                "runtimeUserAgent": surface["userAgent"],
                "surface": output.name,
            }
            (arguments.output / "manifest.json").write_text(
                json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
                encoding="utf-8",
            )
            print(
                json.dumps(
                    {
                        "major": major,
                        "revision": revision,
                        "windowCount": len(surface["topWindowBeforeFrame"]["names"]),
                        "output": str(output),
                    },
                    separators=(",", ":"),
                ),
                flush=True,
            )


if __name__ == "__main__":
    main()
