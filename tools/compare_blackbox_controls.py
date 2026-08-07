"""Compare opaque sandbox samples using trace/stdout evidence only.

The tool does not open JavaScript files.  It compares already exported native
trace and typed stdout files, normalizes runtime-generated frame identifiers,
and reports only direct literal relationships.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import re
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path


DYNAMIC_IDS = re.compile(r"\b(iframe|worker)\[\d+\]")
SEED_IN_NAME = re.compile(r"windows-seed-(\d+)-textencoder-stdout\.tsv$")
QUOTED_RESULT = re.compile(r'"((?:\\.|[^"\\])*)"')
PROFILE_TOKENS = (
    "navigator",
    "screen",
    "devicepixelratio",
    "innerwidth",
    "innerheight",
    "outerwidth",
    "outerheight",
    "visualviewport",
    "webgl",
    "webgpu",
    "canvas",
    "audio",
    "media",
    "keyboard",
    "storage",
    "speech",
    "font",
    "timezone",
    "intl",
    "battery",
    "permission",
    "connection",
    "getboundingclientrect",
    "clientwidth",
    "clientheight",
    "offsetwidth",
    "offsetheight",
    "matchmedia",
)


@dataclass(frozen=True, slots=True)
class Run:
    sample: str
    seed: int
    stdout: Path
    trace: Path
    correlations: Path


def normalize_path(path: str) -> str:
    return DYNAMIC_IDS.sub(lambda match: f"{match.group(1)}[*]", path)


def parse_sample_argument(value: str) -> tuple[str, Path]:
    label, separator, directory = value.partition("=")
    if not separator or not label or not directory:
        raise argparse.ArgumentTypeError("sample must use LABEL=DIRECTORY")
    return label, Path(directory)


def discover(samples: list[tuple[str, Path]]) -> tuple[Run, ...]:
    runs: list[Run] = []
    for label, directory in samples:
        for stdout in directory.resolve().glob(
            "windows-seed-*-textencoder-stdout.tsv"
        ):
            match = SEED_IN_NAME.match(stdout.name)
            if match is None:
                continue
            base = stdout.name.removesuffix("-stdout.tsv")
            trace = stdout.with_name(f"{base}.trace.log")
            correlations = stdout.with_name(f"{base}-correlations.tsv")
            if not trace.is_file() or not correlations.is_file():
                raise FileNotFoundError(f"incomplete audit run: {stdout}")
            runs.append(
                Run(label, int(match.group(1)), stdout, trace, correlations)
            )
    return tuple(sorted(runs, key=lambda run: (run.sample, run.seed)))


def encoder_inputs(run: Run) -> tuple[tuple[int, str, str], ...]:
    values: list[tuple[int, str, str]] = []
    with run.stdout.open("r", encoding="utf-8", newline="") as source:
        for row in csv.DictReader(source, delimiter="\t"):
            if row["argument_path"] == "arguments[1][0]":
                values.append(
                    (int(row["console_sequence"]), row["frame_url"], row["value"])
                )
    return tuple(values)


def correlation_set(run: Run) -> set[tuple[str, str, str, str]]:
    values: set[tuple[str, str, str, str]] = set()
    with run.correlations.open("r", encoding="utf-8", newline="") as source:
        for row in csv.DictReader(source, delimiter="\t"):
            values.add(
                (
                    row["operation"],
                    normalize_path(row["api_path"]),
                    row["match_source"],
                    row["literal"],
                )
            )
    return values


def _result_literals(result: str) -> tuple[str, ...]:
    values = [match.group(1) for match in QUOTED_RESULT.finditer(result)]
    if re.fullmatch(r"-?\d+(?:\.\d+)?", result):
        values.append(result)
    return tuple(dict.fromkeys(value for value in values if value))


def _literal_in_input(literal: str, encoder_input: str) -> bool:
    if re.fullmatch(r"-?\d+(?:\.\d+)?", literal):
        return (
            re.search(
                rf"(?<![0-9.]){re.escape(literal)}(?![0-9.])",
                encoder_input,
            )
            is not None
        )
    return len(literal) >= 2 and literal in encoder_input


def profile_result_correlations(run: Run) -> set[tuple[str, str, str]]:
    inputs = tuple(value for _sequence, _frame, value in encoder_inputs(run))
    matches: set[tuple[str, str, str]] = set()
    with run.trace.open("r", encoding="utf-8", errors="replace") as source:
        for line in source:
            fields = line.rstrip("\r\n").split("\t")
            if len(fields) < 6 or fields[0] != "TRACE":
                continue
            api_path = normalize_path(fields[3])
            if not any(token in api_path.lower() for token in PROFILE_TOKENS):
                continue
            result = next(
                (
                    field.removeprefix("result=")
                    for field in reversed(fields[4:])
                    if field.startswith("result=")
                ),
                "",
            )
            for literal in _result_literals(result):
                if any(_literal_in_input(literal, value) for value in inputs):
                    matches.add((fields[2], api_path, literal))
    return matches


def write_encoder_summary(runs: tuple[Run, ...], destination: Path) -> None:
    grouped: dict[int, list[tuple[Run, int, str, str]]] = defaultdict(list)
    for run in runs:
        for ordinal, (sequence, frame_url, value) in enumerate(
            encoder_inputs(run), start=1
        ):
            grouped[ordinal].append((run, sequence, frame_url, value))

    with destination.open("w", encoding="utf-8", newline="") as output:
        writer = csv.writer(output, delimiter="\t", lineterminator="\n")
        writer.writerow(
            (
                "ordinal",
                "run_count",
                "unique_values",
                "fixed_across_all_runs",
                "lengths",
                "sha256_prefixes",
                "preview",
            )
        )
        for ordinal in sorted(grouped):
            rows = grouped[ordinal]
            values = {row[3] for row in rows}
            lengths = sorted({len(value) for value in values})
            hashes = sorted(
                hashlib.sha256(value.encode("utf-8")).hexdigest()[:16]
                for value in values
            )
            preview = next(iter(values))[:120].replace("\t", "\\t").replace(
                "\n", "\\n"
            )
            writer.writerow(
                (
                    ordinal,
                    len(rows),
                    len(values),
                    len(values) == 1 and len(rows) == len(runs),
                    ",".join(str(value) for value in lengths),
                    ",".join(hashes),
                    preview,
                )
            )


def write_common_correlations(runs: tuple[Run, ...], destination: Path) -> None:
    correlation_sets = [correlation_set(run) for run in runs]
    common = set.intersection(*correlation_sets) if correlation_sets else set()
    with destination.open("w", encoding="utf-8", newline="") as output:
        writer = csv.writer(output, delimiter="\t", lineterminator="\n")
        writer.writerow(
            (
                "operation",
                "api_path",
                "match_source",
                "profile_related_path",
                "literal",
            )
        )
        for operation, api_path, match_source, literal in sorted(common):
            if match_source != "result":
                continue
            lower_path = api_path.lower()
            writer.writerow(
                (
                    operation,
                    api_path,
                    match_source,
                    any(token in lower_path for token in PROFILE_TOKENS),
                    literal,
                )
            )


def write_common_trace_results(runs: tuple[Run, ...], destination: Path) -> None:
    per_run: list[set[tuple[str, str, str]]] = []
    for run in runs:
        values: set[tuple[str, str, str]] = set()
        with run.trace.open("r", encoding="utf-8", errors="replace") as source:
            for line in source:
                fields = line.rstrip("\r\n").split("\t")
                if len(fields) < 6 or fields[0] != "TRACE":
                    continue
                result = next(
                    (
                        field.removeprefix("result=")
                        for field in reversed(fields[4:])
                        if field.startswith("result=")
                    ),
                    None,
                )
                if result is None or result == "deferred":
                    continue
                values.add((fields[2], normalize_path(fields[3]), result))
        per_run.append(values)
    common = set.intersection(*per_run) if per_run else set()
    with destination.open("w", encoding="utf-8", newline="") as output:
        writer = csv.writer(output, delimiter="\t", lineterminator="\n")
        writer.writerow(("operation", "api_path", "profile_related_path", "result"))
        for operation, api_path, result in sorted(common):
            lower_path = api_path.lower()
            writer.writerow(
                (
                    operation,
                    api_path,
                    any(token in lower_path for token in PROFILE_TOKENS),
                    result,
                )
            )


def write_common_profile_result_correlations(
    runs: tuple[Run, ...], destination: Path
) -> None:
    per_run = [profile_result_correlations(run) for run in runs]
    common = set.intersection(*per_run) if per_run else set()
    with destination.open("w", encoding="utf-8", newline="") as output:
        writer = csv.writer(output, delimiter="\t", lineterminator="\n")
        writer.writerow(("operation", "api_path", "literal"))
        writer.writerows(sorted(common))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--sample",
        action="append",
        type=parse_sample_argument,
        required=True,
        help="LABEL=DIRECTORY; may be repeated for split directories",
    )
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    runs = discover(arguments.sample)
    if not runs:
        raise SystemExit("no complete audit runs found")
    samples = {run.sample for run in runs}
    if len(samples) < 2:
        raise SystemExit("at least two black-box sample labels are required")
    output = arguments.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    write_encoder_summary(runs, output / "encoder-input-summary.tsv")
    write_common_correlations(runs, output / "common-direct-api-correlations.tsv")
    write_common_trace_results(runs, output / "common-trace-results.tsv")
    write_common_profile_result_correlations(
        runs,
        output / "common-profile-result-correlations.tsv",
    )
    with (output / "overview.txt").open(
        "w", encoding="utf-8", newline="\n"
    ) as overview:
        overview.write(f"samples={','.join(sorted(samples))}\n")
        overview.write(f"runs={len(runs)}\n")
        for sample in sorted(samples):
            seeds = ",".join(
                str(run.seed) for run in runs if run.sample == sample
            )
            overview.write(f"sample.{sample}.seeds={seeds}\n")


if __name__ == "__main__":
    main()
