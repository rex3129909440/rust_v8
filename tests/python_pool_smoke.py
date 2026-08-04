from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(PROJECT_ROOT / "examples"))

from edge_profile import EdgeProfile, NavigatorProfile  # noqa: E402
from edge_sandbox_pool import EdgeSandboxPool  # noqa: E402
from run_sandbox import EdgeSandbox, SandboxExecutionError  # noqa: E402


class EdgeSandboxPoolTests(unittest.TestCase):
    def test_native_trace_user_exclusions_filter_before_storage(self) -> None:
        with EdgeSandbox() as sandbox:
            sandbox.set_native_trace_exclusions(
                ["window.String", "window.Number"]
            )
            sandbox.enable_native_trace()
            try:
                sandbox.evaluate(
                    'String(1); Number("2"); '
                    'new Blob(["alpha", 2, [true, null]]); '
                    "navigator.userAgent"
                )
            finally:
                sandbox.disable_native_trace()
            entries = sandbox.native_trace()
            self.assertFalse(any("window.String" in entry for entry in entries))
            self.assertFalse(any("window.Number" in entry for entry in entries))
            self.assertTrue(
                any(
                    'args=["alpha",2,[true,null]]' in entry
                    for entry in entries
                )
            )

            sandbox.set_native_trace_exclusions([])
            sandbox.clear_native_trace()
            sandbox.enable_native_trace()
            try:
                sandbox.evaluate("String(3)")
            finally:
                sandbox.disable_native_trace()
            self.assertTrue(
                any("window.String" in entry for entry in sandbox.native_trace())
            )

    def test_native_trace_is_exported_in_bounded_sequence_batches(self) -> None:
        with EdgeSandbox() as sandbox:
            sandbox.clear_native_trace()
            sandbox.enable_native_trace()
            try:
                sandbox.evaluate(
                    "for (let index = 0; index < 20; index++) "
                    "navigator.userAgent"
                )
            finally:
                sandbox.disable_native_trace()

            batches = tuple(sandbox.native_trace_batches(batch_size=7))
            flattened = tuple(entry for batch in batches for entry in batch)
            self.assertGreater(len(batches), 1)
            self.assertEqual(sandbox.native_trace(), flattened)
            self.assertTrue(
                any("window.navigator.userAgent" in entry for entry in flattened)
            )

            with tempfile.TemporaryDirectory() as directory:
                destination = Path(directory) / "nested" / "native-trace.log"
                count = sandbox.export_native_trace(destination, batch_size=7)
                self.assertEqual(count, len(flattened))
                self.assertEqual(
                    tuple(destination.read_text(encoding="utf-8").splitlines()),
                    flattened,
                )
                with self.assertRaises(FileExistsError):
                    sandbox.export_native_trace(destination)
                self.assertEqual(
                    sandbox.export_native_trace(destination, overwrite=True),
                    len(flattened),
                )

    def test_parallel_profiles_requests_and_release_switch(self) -> None:
        profile_a = EdgeProfile(
            id="pool-a",
            navigator=NavigatorProfile(user_agent="Pool-UA-A"),
        )
        profile_b = EdgeProfile(
            id="pool-b",
            navigator=NavigatorProfile(user_agent="Pool-UA-B"),
        )
        with EdgeSandboxPool(
            workers=2,
            timeout_ms=2_000,
            close_worker_after_network_requests=True,
        ) as sandbox:
            task_a = sandbox.submit(
                'const x=new XMLHttpRequest();x.open("POST",'
                '"https://pool.example/a");x.send("a");navigator.userAgent',
                profile=profile_a,
            )
            task_b = sandbox.submit(
                'const x=new XMLHttpRequest();x.open("POST",'
                '"https://pool.example/b");x.send("b");navigator.userAgent',
                profile=profile_b,
            )
            self.assertEqual(task_a.result(), "Pool-UA-A")
            self.assertEqual(task_b.result(), "Pool-UA-B")
            self.assertEqual(sandbox.live_worker_count, 2)
            requests = sandbox.network_requests()
            self.assertEqual(len(requests), 2)
            self.assertEqual(len({request.worker_id for request in requests}), 2)
            self.assertEqual(
                {request.task_id for request in requests},
                {task_a.task_id, task_b.task_id},
            )
            self.assertEqual(sandbox.live_worker_count, 0)

    def test_timeout_discards_worker_and_pool_recovers(self) -> None:
        with EdgeSandboxPool(workers=1, timeout_ms=2_000) as sandbox:
            with self.assertRaisesRegex(
                SandboxExecutionError,
                "configured timeout",
            ):
                sandbox.evaluate("while (true) {}", timeout_ms=100)
            self.assertEqual(sandbox.live_worker_count, 0)
            self.assertEqual(sandbox.evaluate("1 + 1"), "2")
            self.assertEqual(sandbox.live_worker_count, 1)


if __name__ == "__main__":
    unittest.main()
