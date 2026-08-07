"""Build compact, streaming indexes for an Edge sandbox native trace.

The input JavaScript is intentionally out of scope.  This tool reads only the
tab-separated native trace and keeps bounded result samples, so later audits do
not have to load or repeatedly scan a large trace file.
"""

from __future__ import annotations

import argparse
from collections import Counter
from dataclasses import dataclass, field
from pathlib import Path


PROFILE_PATH_TOKENS = (
    "navigator",
    "screen",
    "innerwidth",
    "innerheight",
    "outerwidth",
    "outerheight",
    "devicepixelratio",
    "visualviewport",
    "performance",
    "date.",
    "intl.",
    "timezone",
    "webgl",
    "webgpu",
    "canvas",
    "audiocontext",
    "offlineaudiocontext",
    "media",
    "permission",
    "battery",
    "storage",
    "cookie",
    "getboundingclientrect",
    "getclientrects",
    "clientwidth",
    "clientheight",
    "offsetwidth",
    "offsetheight",
    "getcomputedstyle",
    "matchmedia",
    "font",
    "plugin",
    "mimetype",
    "speech",
    "crypto",
)


@dataclass(slots=True)
class PathStats:
    count: int = 0
    results: Counter[str] = field(default_factory=Counter)
    samples: list[str] = field(default_factory=list)

    def add(self, result: str) -> None:
        self.count += 1
        result_kind = classify_result(result)
        self.results[result_kind] += 1
        bounded = result.replace("\r", "\\r").replace("\n", "\\n")[:240]
        if bounded not in self.samples and len(self.samples) < 5:
            self.samples.append(bounded)


def classify_result(value: str) -> str:
    if value in {"deferred", "undefined", "null", "true", "false", ""}:
        return value or "empty"
    if value.startswith("throws "):
        return "throws"
    if value.startswith("[object "):
        return "object"
    if value.startswith("[function "):
        return "function"
    if value.startswith('"'):
        return "string"
    if value.startswith("["):
        return "array"
    try:
        float(value)
    except ValueError:
        return "other"
    return "number"


def parse_trace_line(line: str) -> tuple[str, str, str] | None:
    fields = line.rstrip("\r\n").split("\t")
    if len(fields) < 6 or fields[0] != "TRACE":
        return None
    operation = fields[2]
    path = fields[3]
    result_field = next(
        (field for field in reversed(fields[4:]) if field.startswith("result=")),
        None,
    )
    if result_field is None:
        return None
    return operation, path, result_field.removeprefix("result=")


def write_rows(
    destination: Path,
    rows: list[tuple[str, str, PathStats]],
) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    with destination.open("w", encoding="utf-8", newline="\n") as output:
        output.write(
            "operation\tpath\tcount\tdeferred\tundefined\tnull\tthrows\t"
            "number\tstring\tobject\tfunction\tarray\ttrue\tfalse\tsamples\n"
        )
        for operation, path, stats in rows:
            counts = stats.results
            samples = " || ".join(stats.samples).replace("\t", "\\t")
            output.write(
                f"{operation}\t{path}\t{stats.count}\t"
                f"{counts['deferred']}\t{counts['undefined']}\t"
                f"{counts['null']}\t{counts['throws']}\t{counts['number']}\t"
                f"{counts['string']}\t{counts['object']}\t"
                f"{counts['function']}\t{counts['array']}\t"
                f"{counts['true']}\t{counts['false']}\t{samples}\n"
            )


def analyze(trace_path: Path, output_directory: Path) -> None:
    stats: dict[tuple[str, str], PathStats] = {}
    operation_counts: Counter[str] = Counter()
    parsed_entries = 0
    with trace_path.open("r", encoding="utf-8", errors="replace") as source:
        for line in source:
            parsed = parse_trace_line(line)
            if parsed is None:
                continue
            operation, path, result = parsed
            parsed_entries += 1
            operation_counts[operation] += 1
            stats.setdefault((operation, path), PathStats()).add(result)

    rows = sorted(
        ((operation, path, value) for (operation, path), value in stats.items()),
        key=lambda row: (-row[2].count, row[0], row[1]),
    )
    write_rows(output_directory / "path-summary.tsv", rows)

    deferred_only = [
        row
        for row in rows
        if row[0] == "get"
        and row[2].results["deferred"] == row[2].count
    ]
    write_rows(output_directory / "deferred-only-getters.tsv", deferred_only)

    undefined_calls = [
        row
        for row in rows
        if row[0] in {"call", "construct"}
        and row[2].results["undefined"] > 0
    ]
    write_rows(output_directory / "undefined-calls.tsv", undefined_calls)

    profile_rows = [
        row
        for row in rows
        if any(token in row[1].lower() for token in PROFILE_PATH_TOKENS)
    ]
    write_rows(output_directory / "profile-hits.tsv", profile_rows)

    with (output_directory / "overview.txt").open(
        "w", encoding="utf-8", newline="\n"
    ) as output:
        output.write(f"trace={trace_path.resolve()}\n")
        output.write(f"entries={parsed_entries}\n")
        output.write(f"unique_operation_paths={len(rows)}\n")
        output.write(f"deferred_only_getters={len(deferred_only)}\n")
        output.write(f"undefined_calls={len(undefined_calls)}\n")
        output.write(f"profile_operation_paths={len(profile_rows)}\n")
        for operation, count in operation_counts.most_common():
            output.write(f"operation.{operation}={count}\n")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("trace", type=Path)
    parser.add_argument("output_directory", type=Path)
    arguments = parser.parse_args()
    analyze(arguments.trace, arguments.output_directory)


if __name__ == "__main__":
    main()
