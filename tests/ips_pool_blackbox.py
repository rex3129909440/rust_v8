from __future__ import annotations

import sys
import unittest
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(PROJECT_ROOT / "examples"))

from edge_sandbox_pool import EdgeSandboxPool  # noqa: E402


class OpaqueIpsPoolBlackBoxTests(unittest.TestCase):
    def test_ten_workers_export_tl_and_release_after_each_response(self) -> None:
        fixture = PROJECT_ROOT / "demo" / "ips.js"
        self.assertTrue(fixture.is_file())
        # Opaque execution fixture: load only to pass it into the sandbox. The
        # test never prints, searches, transforms, or inspects its source.
        source = fixture.read_text(encoding="utf-8")

        with EdgeSandboxPool(
            workers=10,
            timeout_ms=120_000,
            close_worker_after_network_requests=True,
        ) as sandbox:
            tasks = tuple(sandbox.submit(source) for _ in range(10))

            def collect_after_response(task):
                task.result()
                requests = sandbox.network_requests(task.task_id)
                matching = tuple(
                    request
                    for request in requests
                    if request.method == "POST" and request.url.endswith("/tl")
                )
                self.assertEqual(len(matching), 1)
                self.assertTrue(matching[0].headers)
                self.assertTrue(matching[0].body)
                return matching[0]

            with ThreadPoolExecutor(max_workers=10) as observers:
                completed = tuple(
                    future.result(timeout=150.0)
                    for future in as_completed(
                        observers.submit(collect_after_response, task)
                        for task in tasks
                    )
                )

            self.assertEqual(len(completed), 10)
            self.assertEqual(len({request.worker_id for request in completed}), 10)
            self.assertEqual(len({request.task_id for request in completed}), 10)
            self.assertEqual(sandbox.live_worker_count, 0)


if __name__ == "__main__":
    unittest.main()
