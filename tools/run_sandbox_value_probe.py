"""Run the local Edge evidence page inside the sandbox for direct comparison."""

from __future__ import annotations

import argparse
from pathlib import Path
from urllib.parse import unquote

from demo.get_random_fp import get_random_fp_details
from examples.edge_runtime_options import EdgeRunOptions, PageInit, SandboxLimits
from examples.run_sandbox import EdgeSandbox


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("html", type=Path)
    parser.add_argument("--library", type=Path, required=True)
    parser.add_argument("--country", default="US")
    parser.add_argument("--seed", type=int, default=1)
    arguments = parser.parse_args()

    fingerprint = get_random_fp_details(arguments.country, seed=arguments.seed)
    html = arguments.html.resolve().read_text(encoding="utf-8")
    before_script, script_and_tail = html.split("<script>", 1)
    script, after_script = script_and_tail.split("</script>", 1)
    page_html = f"{before_script}{after_script}"
    options = EdgeRunOptions(
        page=PageInit(
            url="https://sandbox.test/edge-value-probe.html",
            html=page_html,
            content_type="text/html; charset=utf-8",
        ),
        limits=SandboxLimits(
            timeout_ms=10_000,
            max_heap_bytes=256 * 1024 * 1024,
            max_resident_bytes=768 * 1024 * 1024,
            max_source_bytes=4 * 1024 * 1024,
            max_output_bytes=4 * 1024 * 1024,
        ),
    )
    with EdgeSandbox(
        library=arguments.library.resolve(),
        profile=fingerprint.profile,
        options=options,
    ) as sandbox:
        output = sandbox.evaluate(
            f"(() => {{\n{script}\nreturn lines.join('\\n');\n}})()",
            source_url="https://sandbox.test/read-edge-value-probe.js",
        )
        keyboard = sandbox.evaluate(
            "navigator.keyboard.getLayoutMap().then(layout => "
            "'keyboard.size=' + layout.size + '\\nkeyboard.entries=' + "
            "Array.from(layout.entries(), ([code, value]) => "
            "code + ':' + value).join('|'))",
            source_url="https://sandbox.test/read-keyboard-layout.js",
        )
        output = f"{output}\n{keyboard}"
    for line in output.splitlines():
        name, separator, value = line.partition("=")
        print(f"{name}{separator}{unquote(value) if separator else value}")


if __name__ == "__main__":
    main()
