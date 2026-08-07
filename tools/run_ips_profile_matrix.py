"""Run an opaque JavaScript file against a deterministic profile matrix.

This utility deliberately does not inspect or interpret the JavaScript source.
It records the sandbox's native API trace and typed console output so browser
environment values can be compared between profiles without loading large
trace files into an interactive session.
"""

from __future__ import annotations

import argparse
import csv
from dataclasses import replace
from pathlib import Path

from demo.call_edge_sandbox import build_evaluation_runtime_options
from demo.get_random_fp import RandomFingerprint, get_random_fp_details
from examples.edge_runtime_options import DeterministicExecution, PageInit
from examples.run_sandbox import (
    CapturedConsoleOutput,
    CapturedConsoleValue,
    EdgeSandbox,
    SandboxExecutionError,
)


DEFAULT_EXCLUSIONS = (
    "window.String*",
    "window.Number*",
    "window.Math*",
    "window.Object*",
    "window.Array*",
    "window.Boolean*",
    "window.BigInt*",
    "window.Symbol*",
    "window.JSON*",
    "window.Reflect*",
    "window.RegExp*",
    "window.Promise*",
    "window.Map*",
    "window.Set*",
    "window.WeakMap*",
    "window.WeakSet*",
)


TEXT_ENCODER_AUDIT_HOOK = r"""
(() => {
  if (typeof TextEncoder !== "function" ||
      typeof TextEncoder.prototype.encode !== "function") return;
  const originalEncode = TextEncoder.prototype.encode;
  TextEncoder.prototype.encode = function encode() {
    const encoded = Reflect.apply(originalEncode, this, arguments);
    console.log(
      "TextEncoder.prototype.encode",
      arguments,
      encoded,
      { input: arguments[0], byteLength: encoded.byteLength }
    );
    return encoded;
  };
})();
"""


def _escape(value: object) -> str:
    return (
        str(value)
        .replace("\\", "\\\\")
        .replace("\t", "\\t")
        .replace("\r", "\\r")
        .replace("\n", "\\n")
    )


def _flatten_value(
    value: CapturedConsoleValue,
    path: str,
) -> list[tuple[str, CapturedConsoleValue, object]]:
    if value.kind == "sequence":
        rows: list[tuple[str, CapturedConsoleValue, object]] = []
        for index, item in enumerate(value.value):
            rows.extend(_flatten_value(item, f"{path}[{index}]"))
        return rows
    if value.kind == "object":
        rows = []
        for key, item in value.value:
            rows.extend(_flatten_value(item, f"{path}.{key}"))
        return rows
    rendered = value.value.hex() if value.kind == "bytes" else value.value
    return [(path, value, rendered)]


def _write_stdout(path: Path, entries: tuple[CapturedConsoleOutput, ...]) -> int:
    rows = 0
    with path.open("w", encoding="utf-8", newline="") as output:
        writer = csv.writer(output, delimiter="\t", lineterminator="\n")
        writer.writerow(
            (
                "console_sequence",
                "frame_url",
                "argument_path",
                "kind",
                "type_name",
                "truncated",
                "value",
            )
        )
        for entry in entries:
            for index, argument in enumerate(entry.arguments):
                for argument_path, value, rendered in _flatten_value(
                    argument, f"arguments[{index}]"
                ):
                    writer.writerow(
                        (
                            entry.sequence,
                            _escape(entry.frame_url),
                            argument_path,
                            value.kind,
                            value.type_name or "",
                            value.truncated,
                            _escape(rendered),
                        )
                    )
                    rows += 1
    return rows


def _manifest_row(
    fingerprint: RandomFingerprint,
    *,
    trace_count: int,
    stdout_count: int,
    encoder_count: int,
    request_count: int,
    tl_count: int,
    error: str,
) -> tuple[object, ...]:
    return (
        fingerprint.seed,
        fingerprint.platform,
        fingerprint.user_agent_profile_id,
        fingerprint.navigator_hardware_profile_id,
        fingerprint.screen_profile_id,
        fingerprint.webgl_gpu_profile_id,
        fingerprint.speech_voice_profile_id,
        fingerprint.font_profile_id,
        fingerprint.cpu_logical_processors,
        fingerprint.device_memory_gb,
        fingerprint.physical_memory_gb,
        fingerprint.gpu_model,
        trace_count,
        stdout_count,
        encoder_count,
        request_count,
        tl_count,
        _escape(error),
    )


def run_sample(
    javascript_path: Path,
    output_directory: Path,
    library: Path,
    country_code: str,
    seed: int,
) -> tuple[RandomFingerprint, tuple[object, ...]]:
    fingerprint = get_random_fp_details(country_code, seed=seed)
    options = build_evaluation_runtime_options(timeout_ms=10_000)
    options = replace(
        options,
        page=PageInit(
            url="https://sandbox.test/",
            html="<!doctype html><html><head></head><body></body></html>",
            content_type="text/html; charset=utf-8",
        ),
        deterministic=DeterministicExecution(
            clock_epoch_ms=None,
            clock_step_ms=1,
            random_seed=150,
            max_task_turns=2_048,
        ),
    )
    # The file contents remain opaque: the audit adds only a generic API hook.
    javascript = TEXT_ENCODER_AUDIT_HOOK + javascript_path.read_text(
        encoding="utf-8"
    )
    trace_path = output_directory / f"windows-seed-{seed}-textencoder.trace.log"
    stdout_path = output_directory / f"windows-seed-{seed}-textencoder-stdout.tsv"
    error_text = ""
    requests = ()
    stdout = ()
    trace_count = 0

    with EdgeSandbox(
        library=library,
        profile=fingerprint.profile,
        options=options,
    ) as sandbox:
        sandbox.set_native_trace_exclusions(DEFAULT_EXCLUSIONS)
        sandbox.enable_native_trace()
        try:
            sandbox.evaluate(javascript, source_url="https://sandbox.test/input.js")
        except SandboxExecutionError as error:
            error_text = str(error)
        finally:
            sandbox.disable_native_trace()
        requests = sandbox.network_requests()
        stdout = sandbox.stdout()
        trace_count = sandbox.export_native_trace(
            trace_path,
            batch_size=8_192,
            overwrite=True,
        )

    stdout_count = _write_stdout(stdout_path, stdout)
    encoder_count = sum(
        1
        for entry in stdout
        if entry.arguments
        and entry.arguments[0].kind == "string"
        and entry.arguments[0].value == "TextEncoder.prototype.encode"
    )
    tl_count = sum(request.url.endswith("/tl") for request in requests)
    return fingerprint, _manifest_row(
        fingerprint,
        trace_count=trace_count,
        stdout_count=stdout_count,
        encoder_count=encoder_count,
        request_count=len(requests),
        tl_count=tl_count,
        error=error_text,
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("javascript", type=Path)
    parser.add_argument("output_directory", type=Path)
    parser.add_argument("--library", type=Path, required=True)
    parser.add_argument("--country", default="US")
    parser.add_argument("--seeds", type=int, nargs="+", required=True)
    arguments = parser.parse_args()

    output_directory = arguments.output_directory.resolve()
    output_directory.mkdir(parents=True, exist_ok=True)
    manifest_path = output_directory / "matrix-manifest.tsv"
    header = (
        "seed",
        "platform",
        "ua_profile",
        "hardware_profile",
        "screen_profile",
        "webgl_profile",
        "speech_profile",
        "font_profile",
        "hardware_concurrency",
        "device_memory_gb",
        "physical_memory_gb",
        "gpu_model",
        "trace_entries",
        "stdout_rows",
        "textencoder_calls",
        "request_count",
        "tl_count",
        "error",
    )
    with manifest_path.open("w", encoding="utf-8", newline="") as output:
        writer = csv.writer(output, delimiter="\t", lineterminator="\n")
        writer.writerow(header)
        for seed in arguments.seeds:
            fingerprint, row = run_sample(
                arguments.javascript.resolve(),
                output_directory,
                arguments.library.resolve(),
                arguments.country,
                seed,
            )
            writer.writerow(row)
            output.flush()
            print(
                f"seed={seed} platform={fingerprint.platform} "
                f"trace={row[12]} encoders={row[14]} "
                f"requests={row[15]} tl={row[16]} "
                f"error={'yes' if row[17] else 'no'}",
                flush=True,
            )


if __name__ == "__main__":
    main()
