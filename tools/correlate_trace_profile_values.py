"""Correlate literal browser API trace values with captured TextEncoder inputs.

This is deliberately not a JavaScript or protocol analyzer.  It consumes only
the sandbox's native trace and typed stdout export, and reports direct literal
matches that can be audited as possible profile inputs.
"""

from __future__ import annotations

import argparse
import csv
import re
from collections import Counter
from dataclasses import dataclass
from pathlib import Path


QUOTED_VALUE = re.compile(r'"((?:\\.|[^"\\])*)"')
IGNORED_LITERALS = {
    "true",
    "false",
    "null",
    "undefined",
    "deferred",
    "prompt",
    "granted",
    "denied",
}


@dataclass(frozen=True, slots=True)
class EncoderInput:
    sequence: int
    frame_url: str
    value: str


def load_encoder_inputs(path: Path) -> tuple[EncoderInput, ...]:
    values: list[EncoderInput] = []
    with path.open("r", encoding="utf-8", newline="") as source:
        for row in csv.DictReader(source, delimiter="\t"):
            if row["argument_path"] != "arguments[1][0]":
                continue
            values.append(
                EncoderInput(
                    sequence=int(row["console_sequence"]),
                    frame_url=row["frame_url"],
                    value=row["value"],
                )
            )
    return tuple(values)


def unescape_trace_string(value: str) -> str:
    return (
        value.replace(r"\t", "\t")
        .replace(r"\r", "\r")
        .replace(r"\n", "\n")
        .replace(r'\"', '"')
        .replace(r"\\", "\\")
    )


def candidate_literals(args: str, result: str) -> tuple[tuple[str, str], ...]:
    candidates: list[tuple[str, str]] = []
    for source_name, value in (("result", result), ("args", args)):
        for match in QUOTED_VALUE.finditer(value):
            candidates.append((source_name, unescape_trace_string(match.group(1))))
        stripped = value.strip()
        if re.fullmatch(r"-?\d+(?:\.\d+)?", stripped):
            candidates.append((source_name, stripped))

    unique: list[tuple[str, str]] = []
    seen: set[tuple[str, str]] = set()
    for source_name, literal in candidates:
        key = source_name, literal
        if (
            key in seen
            or len(literal) < 4
            or literal.lower() in IGNORED_LITERALS
        ):
            continue
        seen.add(key)
        unique.append(key)
    return tuple(unique)


def correlate(trace_path: Path, stdout_path: Path, destination: Path) -> None:
    inputs = load_encoder_inputs(stdout_path)
    matches: list[tuple[EncoderInput, int, str, str, str, str]] = []
    per_input_counts: Counter[int] = Counter()

    with trace_path.open("r", encoding="utf-8", errors="replace") as source:
        for line in source:
            fields = line.rstrip("\r\n").split("\t")
            if len(fields) < 6 or fields[0] != "TRACE":
                continue
            trace_sequence = int(fields[1])
            operation, api_path = fields[2], fields[3]
            args = next(
                (field.removeprefix("args=") for field in fields[4:] if field.startswith("args=")),
                "",
            )
            result = next(
                (field.removeprefix("result=") for field in reversed(fields[4:]) if field.startswith("result=")),
                "",
            )
            for source_name, literal in candidate_literals(args, result):
                for encoder_input in inputs:
                    if literal not in encoder_input.value:
                        continue
                    # Bound pathological repetition while retaining enough
                    # provenance for each captured input.
                    if per_input_counts[encoder_input.sequence] >= 500:
                        continue
                    per_input_counts[encoder_input.sequence] += 1
                    matches.append(
                        (
                            encoder_input,
                            trace_sequence,
                            operation,
                            api_path,
                            source_name,
                            literal,
                        )
                    )

    destination.parent.mkdir(parents=True, exist_ok=True)
    with destination.open("w", encoding="utf-8", newline="\n") as output:
        output.write(
            "console_sequence\tframe_url\tinput_length\ttrace_sequence\t"
            "operation\tapi_path\tmatch_source\tliteral\n"
        )
        for item, trace_sequence, operation, api_path, source_name, literal in matches:
            escaped = (
                literal.replace("\\", "\\\\")
                .replace("\t", "\\t")
                .replace("\r", "\\r")
                .replace("\n", "\\n")
            )
            output.write(
                f"{item.sequence}\t{item.frame_url}\t{len(item.value)}\t"
                f"{trace_sequence}\t{operation}\t{api_path}\t"
                f"{source_name}\t{escaped}\n"
            )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("trace", type=Path)
    parser.add_argument("stdout", type=Path)
    parser.add_argument("destination", type=Path)
    arguments = parser.parse_args()
    correlate(arguments.trace, arguments.stdout, arguments.destination)


if __name__ == "__main__":
    main()
