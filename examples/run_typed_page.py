"""Run an HTML-backed Edge sandbox from Python without a CLI wrapper."""

from __future__ import annotations

try:
    from .edge_runtime_options import (
        DeterministicExecution,
        EdgeRunOptions,
        NetworkReplayEntry,
        PageInit,
        SandboxLimits,
    )
    from .run_sandbox import EdgeSandbox
except ImportError:
    from edge_runtime_options import (
        DeterministicExecution,
        EdgeRunOptions,
        NetworkReplayEntry,
        PageInit,
        SandboxLimits,
    )
    from run_sandbox import EdgeSandbox


def run_javascript(source: str) -> str:
    """Evaluate JavaScript against a typed page and return its display value."""

    options = EdgeRunOptions(
        page=PageInit(
            url="https://example.test/app/index.html",
            html="""
                <!doctype html>
                <title>Typed page</title>
                <main id="app">
                  <a id="next" href="../next">Next</a>
                </main>
            """,
        ),
        network_replay=(
            NetworkReplayEntry(
                url="https://api.example.test/data",
                body="response from typed replay",
                headers=(("content-type", "text/plain; charset=utf-8"),),
            ),
        ),
        deterministic=DeterministicExecution(
            clock_epoch_ms=1_893_456_000_000,
            random_seed=150,
        ),
        limits=SandboxLimits(timeout_ms=3_000),
    )

    with EdgeSandbox(options=options) as sandbox:
        return sandbox.evaluate(source)


def example() -> str:
    """Show DOM lookup, URL resolution, fetch and a top-level Promise."""

    return run_javascript(
        """
        fetch("https://api.example.test/data").then(async response => [
          await response.text(),
          document.getElementById("next").href,
          document.all.namedItem("app") === document.getElementById("app"),
          location.href,
          Date.now() >= 1893456000000
        ].join("|"))
        """
    )


if __name__ == "__main__":
    print(example())
