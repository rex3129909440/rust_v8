"""Concurrent process pool for independent edge-sandbox evaluations."""

from __future__ import annotations

from concurrent.futures import Future, ThreadPoolExecutor
from dataclasses import dataclass, replace
from itertools import count
from pathlib import Path
from threading import Condition
from types import TracebackType
from typing import Sequence, Self

try:
    from .edge_profile import EdgeProfile
    from .edge_runtime_options import EdgeRunOptions
    from .run_sandbox import CapturedNetworkRequest, EdgeSandbox, find_native_artifacts
except ImportError:
    from edge_profile import EdgeProfile
    from edge_runtime_options import EdgeRunOptions
    from run_sandbox import CapturedNetworkRequest, EdgeSandbox, find_native_artifacts


@dataclass(frozen=True, slots=True)
class PooledNetworkRequest:
    task_id: int
    worker_id: int
    sequence: int
    source: str
    method: str
    url: str
    headers: tuple[tuple[str, str], ...]
    body: bytes


class SandboxTask:
    """One submitted evaluation and its stable pool task identifier."""

    def __init__(self, task_id: int, future: Future[str]) -> None:
        self.task_id = task_id
        self._future = future

    def result(self, timeout: float | None = None) -> str:
        return self._future.result(timeout=timeout)

    def done(self) -> bool:
        return self._future.done()

    def cancel(self) -> bool:
        return self._future.cancel()

    def cancelled(self) -> bool:
        return self._future.cancelled()

    def exception(self, timeout: float | None = None) -> BaseException | None:
        return self._future.exception(timeout=timeout)


@dataclass(slots=True)
class _PoolWorker:
    worker_id: int
    sandbox: EdgeSandbox
    profile: EdgeProfile | None
    options: EdgeRunOptions
    busy: bool = True
    close_when_idle: bool = False


class EdgeSandboxPool:
    """One Python facade over multiple process-isolated V8 workers.

    Each worker executes only one top-level evaluation at a time. Independent
    tasks execute concurrently across workers. A worker is reused only when
    the next task has exactly the same typed fingerprint and runtime options.
    """

    def __init__(
        self,
        *,
        workers: int = 4,
        timeout_ms: int = 30_000,
        close_worker_after_network_requests: bool = False,
        library: Path | None = None,
        worker: Path | None = None,
        default_profile: EdgeProfile | None = None,
        default_options: EdgeRunOptions | None = None,
    ) -> None:
        if workers < 1:
            raise ValueError("workers must be at least 1")
        if timeout_ms < 1:
            raise ValueError("timeout_ms must be at least 1")
        self._library_path, self._worker_path = find_native_artifacts(library, worker)
        self._maximum_workers = workers
        self._default_timeout_ms = timeout_ms
        self._close_after_requests = close_worker_after_network_requests
        self._default_profile = default_profile
        self._default_options = default_options or EdgeRunOptions()
        self._condition = Condition()
        self._workers: list[_PoolWorker] = []
        self._creating_workers = 0
        self._closed = False
        self._worker_ids = count(1)
        self._task_ids = count(1)
        self._requests: list[PooledNetworkRequest] = []
        self._completed_worker_by_task: dict[int, int] = {}
        self._collected_tasks: set[int] = set()
        self._executor = ThreadPoolExecutor(
            max_workers=workers,
            thread_name_prefix="edge-sandbox",
        )

    @property
    def maximum_workers(self) -> int:
        return self._maximum_workers

    @property
    def live_worker_count(self) -> int:
        with self._condition:
            return len(self._workers) + self._creating_workers

    def submit(
        self,
        source: str,
        *,
        profile: EdgeProfile | None = None,
        options: EdgeRunOptions | None = None,
        timeout_ms: int | None = None,
    ) -> SandboxTask:
        if not isinstance(source, str):
            raise TypeError("source must be a string")
        effective_profile = profile if profile is not None else self._default_profile
        effective_options = self._effective_options(options, timeout_ms)
        with self._condition:
            if self._closed:
                raise RuntimeError("edge sandbox pool is closed")
            task_id = next(self._task_ids)
        future = self._executor.submit(
            self._run_task,
            task_id,
            source,
            effective_profile,
            effective_options,
        )
        return SandboxTask(task_id, future)

    def evaluate(
        self,
        source: str,
        *,
        profile: EdgeProfile | None = None,
        options: EdgeRunOptions | None = None,
        timeout_ms: int | None = None,
    ) -> str:
        return self.submit(
            source,
            profile=profile,
            options=options,
            timeout_ms=timeout_ms,
        ).result()

    def evaluate_many(
        self,
        sources: Sequence[str],
        *,
        profiles: Sequence[EdgeProfile | None] | None = None,
        options: EdgeRunOptions | None = None,
        timeout_ms: int | None = None,
    ) -> tuple[str, ...]:
        source_values = tuple(sources)
        profile_values = (
            (None,) * len(source_values) if profiles is None else tuple(profiles)
        )
        if len(profile_values) != len(source_values):
            raise ValueError("profiles must have the same length as sources")
        tasks = tuple(
            self.submit(
                source,
                profile=profile,
                options=options,
                timeout_ms=timeout_ms,
            )
            for source, profile in zip(source_values, profile_values, strict=True)
        )
        return tuple(task.result() for task in tasks)

    def network_requests(
        self,
        task_id: int | None = None,
    ) -> tuple[PooledNetworkRequest, ...]:
        workers_to_close: list[_PoolWorker] = []
        with self._condition:
            selected = tuple(
                request
                for request in self._requests
                if task_id is None or request.task_id == task_id
            )
            if self._close_after_requests:
                completed_tasks = {
                    completed_task
                    for completed_task in self._completed_worker_by_task
                    if (task_id is None or completed_task == task_id)
                    and completed_task not in self._collected_tasks
                }
                worker_ids = {
                    self._completed_worker_by_task[completed_task]
                    for completed_task in completed_tasks
                }
                self._collected_tasks.update(completed_tasks)
                for worker in tuple(self._workers):
                    if worker.worker_id not in worker_ids:
                        continue
                    if worker.busy:
                        worker.close_when_idle = True
                    else:
                        self._workers.remove(worker)
                        workers_to_close.append(worker)
                self._condition.notify_all()
        for worker in workers_to_close:
            worker.sandbox.close()
        return selected

    def clear_network_requests(self, task_id: int | None = None) -> None:
        with self._condition:
            if task_id is None:
                self._requests.clear()
                self._completed_worker_by_task.clear()
                self._collected_tasks.clear()
            else:
                self._requests = [
                    request for request in self._requests if request.task_id != task_id
                ]
                self._completed_worker_by_task.pop(task_id, None)
                self._collected_tasks.discard(task_id)

    def close(self) -> None:
        with self._condition:
            if self._closed:
                return
            self._closed = True
            self._condition.notify_all()
        self._executor.shutdown(wait=True, cancel_futures=True)
        with self._condition:
            workers = tuple(self._workers)
            self._workers.clear()
        for worker in workers:
            worker.sandbox.close()

    def _effective_options(
        self,
        options: EdgeRunOptions | None,
        timeout_ms: int | None,
    ) -> EdgeRunOptions:
        effective = options or self._default_options
        effective_timeout = (
            timeout_ms
            if timeout_ms is not None
            else effective.limits.timeout_ms
            if effective.limits.timeout_ms is not None
            else self._default_timeout_ms
        )
        if effective_timeout < 1:
            raise ValueError("timeout_ms must be at least 1")
        return replace(
            effective,
            limits=replace(effective.limits, timeout_ms=effective_timeout),
        )

    def _run_task(
        self,
        task_id: int,
        source: str,
        profile: EdgeProfile | None,
        options: EdgeRunOptions,
    ) -> str:
        worker = self._acquire_worker(profile, options)
        discard = False
        try:
            worker.sandbox.clear_network_requests()
            value = worker.sandbox.evaluate(source)
            captured = worker.sandbox.network_requests()
            worker.sandbox.clear_network_requests()
            self._store_requests(task_id, worker.worker_id, captured)
            return value
        except BaseException:
            # Timeouts and native failures must release the process and its V8
            # heap instead of putting a potentially damaged worker back.
            discard = True
            raise
        finally:
            self._release_worker(worker, discard)

    def _store_requests(
        self,
        task_id: int,
        worker_id: int,
        requests: tuple[CapturedNetworkRequest, ...],
    ) -> None:
        pooled = [
            PooledNetworkRequest(
                task_id=task_id,
                worker_id=worker_id,
                sequence=request.sequence,
                source=request.source,
                method=request.method,
                url=request.url,
                headers=request.headers,
                body=request.body,
            )
            for request in requests
        ]
        with self._condition:
            self._requests.extend(pooled)
            self._completed_worker_by_task[task_id] = worker_id

    def _acquire_worker(
        self,
        profile: EdgeProfile | None,
        options: EdgeRunOptions,
    ) -> _PoolWorker:
        retired: _PoolWorker | None = None
        while True:
            with self._condition:
                if self._closed:
                    raise RuntimeError("edge sandbox pool is closed")
                for worker in self._workers:
                    if (
                        not worker.busy
                        and not worker.close_when_idle
                        and worker.profile == profile
                        and worker.options == options
                    ):
                        worker.busy = True
                        return worker
                if len(self._workers) + self._creating_workers < self._maximum_workers:
                    self._creating_workers += 1
                    worker_id = next(self._worker_ids)
                    break
                retired = next(
                    (worker for worker in self._workers if not worker.busy),
                    None,
                )
                if retired is not None:
                    self._workers.remove(retired)
                    self._creating_workers += 1
                    worker_id = next(self._worker_ids)
                    break
                self._condition.wait()
        if retired is not None:
            retired.sandbox.close()
        try:
            sandbox = EdgeSandbox(
                library=self._library_path,
                worker=self._worker_path,
                profile=profile,
                options=options,
            )
        except BaseException:
            with self._condition:
                self._creating_workers -= 1
                self._condition.notify_all()
            raise
        created = _PoolWorker(worker_id, sandbox, profile, options)
        with self._condition:
            self._creating_workers -= 1
            if self._closed:
                close_created = True
            else:
                close_created = False
                self._workers.append(created)
            self._condition.notify_all()
        if close_created:
            created.sandbox.close()
            raise RuntimeError("edge sandbox pool is closed")
        return created

    def _release_worker(self, worker: _PoolWorker, discard: bool) -> None:
        close_worker = False
        with self._condition:
            worker.busy = False
            if discard or worker.close_when_idle or self._closed:
                if worker in self._workers:
                    self._workers.remove(worker)
                close_worker = True
            self._condition.notify_all()
        if close_worker:
            worker.sandbox.close()

    def __enter__(self) -> Self:
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except (AttributeError, RuntimeError):
            pass
