"""Measure real V8 counters and Worker RSS for typed heap constraints."""

from __future__ import annotations

import csv
import json
import os
import statistics
import sys
import time
from pathlib import Path

import psutil


ROOT = Path(__file__).resolve().parents[1]
for value in (ROOT, ROOT / "demo"):
    if str(value) not in sys.path:
        sys.path.insert(0, str(value))

from demo.android_call_edge_sandbox import build_android_profile  # noqa: E402
from demo.w6_sandbox_executor_api import (  # noqa: E402
    _dv_preload_source,
    build_executor_runtime_options,
)
from examples.run_sandbox import EdgeSandbox  # noqa: E402


LIBRARY = Path(
    os.environ.get(
        "EDGE_SANDBOX_BENCHMARK_LIBRARY",
        ROOT / "target" / "release" / "edge_sandbox.dll",
    )
)
SOURCE = ROOT / "demo" / "ips.js"
OUTPUT = ROOT / "build" / "v8-memory-ab"
USER_AGENT = "wizz-air/8.1.9 (com.wizzair.WizzAirApp; build:2207; android 9)"
MIB = 1024 * 1024


VARIANTS = (
    ("v8-default", None, None),
    ("young-4m", 4 * MIB, None),
    ("young-8m", 8 * MIB, None),
    ("young-16m", 16 * MIB, None),
    ("young-32m", 32 * MIB, None),
    ("code-32m", None, 32 * MIB),
    ("code-64m", None, 64 * MIB),
    ("code-128m", None, 128 * MIB),
    ("young-8m-code-32m", 8 * MIB, 32 * MIB),
    ("young-8m-code-64m", 8 * MIB, 64 * MIB),
    ("young-16m-code-32m", 16 * MIB, 32 * MIB),
    ("young-16m-code-64m", 16 * MIB, 64 * MIB),
)


def one_run(
    name: str,
    young: int | None,
    code: int | None,
    repeat: int,
) -> dict[str, object]:
    profile = build_android_profile("US", USER_AGENT, seed=repeat, chromium_major=136)
    options = build_executor_runtime_options(
        timeout_ms=30_000,
        max_young_generation_bytes=young,
        max_code_range_bytes=code,
    )
    source = SOURCE.read_text(encoding="utf-8")
    with EdgeSandbox(library=LIBRARY, profile=profile, options=options) as sandbox:
        sandbox.set_stdout_capture_enabled(False)
        process = psutil.Process(sandbox.process_id())
        idle = sandbox.v8_memory_statistics()
        idle_rss = process.memory_info().rss
        sandbox.evaluate(
            _dv_preload_source("v8-memory-ab-dv"),
            source_url="https://sandbox.test/__v8_memory_preload__.js",
        )
        before_cpu = process.cpu_times()
        started = time.perf_counter()
        sandbox.evaluate(source, source_url=f"https://example.test/{name}-{repeat}.js")
        wall_seconds = time.perf_counter() - started
        after_cpu = process.cpu_times()
        cpu_seconds = (
            after_cpu.user + after_cpu.system - before_cpu.user - before_cpu.system
        )
        after = sandbox.v8_memory_statistics()
        after_rss = process.memory_info().rss
        gc_started = time.perf_counter()
        sandbox.low_memory_notification()
        gc_seconds = time.perf_counter() - gc_started
        after_gc = sandbox.v8_memory_statistics()
        after_gc_rss = process.memory_info().rss
    return {
        "variant": name,
        "repeat": repeat,
        "young_bytes": young or 0,
        "code_range_bytes": code or 0,
        "wall_ms": wall_seconds * 1000,
        "cpu_ms": cpu_seconds * 1000,
        "gc_ms": gc_seconds * 1000,
        "idle_rss": idle_rss,
        "after_rss": after_rss,
        "after_gc_rss": after_gc_rss,
        **{f"idle_{key}": value for key, value in idle.__dict__.items()},
        **{f"after_{key}": value for key, value in after.__dict__.items()},
        **{f"after_gc_{key}": value for key, value in after_gc.__dict__.items()},
    }


def main(*, repeats: int = 2) -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    rows = [
        one_run(name, young, code, repeat)
        for name, young, code in VARIANTS
        for repeat in range(repeats)
    ]
    (OUTPUT / "runs.json").write_text(
        json.dumps(rows, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )
    summary = []
    for name, young, code in VARIANTS:
        selected = [row for row in rows if row["variant"] == name]
        summary.append({
            "variant": name,
            "young_bytes": young or 0,
            "code_range_bytes": code or 0,
            **{
                key: round(statistics.mean(float(row[key]) for row in selected), 3)
                for key in (
                    "wall_ms", "cpu_ms", "gc_ms", "idle_rss", "after_rss",
                    "after_gc_rss", "after_total_heap_size", "after_used_heap_size",
                    "after_total_physical_size", "after_malloced_memory",
                    "after_external_memory", "after_code_and_metadata_size",
                    "after_bytecode_and_metadata_size", "after_external_script_source_size",
                    "after_gc_total_heap_size", "after_gc_used_heap_size",
                    "after_gc_total_physical_size",
                )
            },
        })
    with (OUTPUT / "summary.tsv").open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=tuple(summary[0]), delimiter="\t")
        writer.writeheader()
        writer.writerows(summary)


if __name__ == "__main__":
    main()
