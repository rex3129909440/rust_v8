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
    from .run_sandbox import (
        CapturedConsoleOutput,
        CapturedNetworkRequest,
        EdgeSandbox,
        find_native_artifacts,
    )
except ImportError:
    from edge_profile import EdgeProfile
    from edge_runtime_options import EdgeRunOptions
    from run_sandbox import (
        CapturedConsoleOutput,
        CapturedNetworkRequest,
        EdgeSandbox,
        find_native_artifacts,
    )


@dataclass(frozen=True, slots=True)
class PooledNetworkRequest:
    task_id: int
    worker_id: int
    worker_process_id: int
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

    @property
    def future(self) -> Future[str]:
        """Underlying completion handle for completion-order coordination."""

        return self._future

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
    process_id: int
    sandbox: EdgeSandbox
    profile: EdgeProfile | None
    options: EdgeRunOptions
    busy: bool = True
    close_when_idle: bool = False


class EdgeSandboxPool:
    """One Python facade over multiple process-isolated V8 workers.

    Each worker executes only one top-level evaluation at a time. Independent
    tasks execute concurrently across workers. In ``one_shot_workers`` mode,
    every prewarmed process accepts one fresh profile and one JavaScript task,
    is destroyed, and is synchronously replaced before another task is served.
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
        one_shot_workers: bool = False,
        prewarm: bool = False,
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
        self._one_shot_workers = one_shot_workers
        self._condition = Condition()
        self._workers: list[_PoolWorker] = []
        self._creating_workers = 0
        self._closed = False
        self._worker_ids = count(1)
        self._task_ids = count(1)
        self._requests: list[PooledNetworkRequest] = []
        self._stdout_by_task: dict[int, tuple[CapturedConsoleOutput, ...]] = {}
        self._completed_worker_by_task: dict[int, int] = {}
        self._completed_process_by_task: dict[int, int] = {}
        self._collected_tasks: set[int] = set()
        self._executor = ThreadPoolExecutor(
            max_workers=workers,
            thread_name_prefix="edge-sandbox",
        )
        self._one_shot_options = self._effective_options(None, None)
        if prewarm:
            self.prewarm()

    @property
    def maximum_workers(self) -> int:
        return self._maximum_workers

    @property
    def live_worker_count(self) -> int:
        with self._condition:
            return len(self._workers) + self._creating_workers

    @property
    def worker_process_ids(self) -> tuple[int, ...]:
        with self._condition:
            return tuple(worker.process_id for worker in self._workers)

    def prewarm(self) -> None:
        """Create all configured Worker processes before accepting tasks."""

        try:
            while True:
                with self._condition:
                    if self._closed:
                        raise RuntimeError("edge sandbox pool is closed")
                    if (
                        len(self._workers) + self._creating_workers
                        >= self._maximum_workers
                    ):
                        return
                self._spawn_blank_worker()
        except BaseException:
            self.close()
            raise

    def submit(
        self,
        source: str,
        *,
        preload_javascript: str | None = None,
        preload_source_url: str = "https://sandbox.test/__pool_preload__.js",
        source_url: str | None = None,
        profile: EdgeProfile | None = None,
        options: EdgeRunOptions | None = None,
        timeout_ms: int | None = None,
        capture_stdout: bool = False,
    ) -> SandboxTask:
        if not isinstance(source, str):
            raise TypeError("source must be a string")
        if preload_javascript is not None:
            if not isinstance(preload_javascript, str):
                raise TypeError("preload_javascript must be a string or None")
            if not preload_javascript.strip():
                raise ValueError("preload_javascript must not be blank")
        if not isinstance(capture_stdout, bool):
            raise TypeError("capture_stdout must be a bool")
        effective_profile = profile if profile is not None else self._default_profile
        effective_options = self._effective_options(options, timeout_ms)
        if self._one_shot_workers and effective_options != self._one_shot_options:
            raise ValueError(
                "one-shot Worker tasks must use the pool's fixed runtime options; "
                "configure timeout and limits when creating the pool"
            )
        with self._condition:
            if self._closed:
                raise RuntimeError("edge sandbox pool is closed")
            task_id = next(self._task_ids)
        future = self._executor.submit(
            self._run_task,
            task_id,
            source,
            preload_javascript,
            preload_source_url,
            source_url,
            effective_profile,
            effective_options,
            capture_stdout,
        )
        return SandboxTask(task_id, future)

    def evaluate(
        self,
        source: str,
        *,
        preload_javascript: str | None = None,
        preload_source_url: str = "https://sandbox.test/__pool_preload__.js",
        source_url: str | None = None,
        profile: EdgeProfile | None = None,
        options: EdgeRunOptions | None = None,
        timeout_ms: int | None = None,
        capture_stdout: bool = False,
    ) -> str:
        return self.submit(
            source,
            preload_javascript=preload_javascript,
            preload_source_url=preload_source_url,
            source_url=source_url,
            profile=profile,
            options=options,
            timeout_ms=timeout_ms,
            capture_stdout=capture_stdout,
        ).result()

    def evaluate_many(
        self,
        sources: Sequence[str],
        *,
        source_urls: Sequence[str | None] | None = None,
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
        source_url_values = (
            (None,) * len(source_values)
            if source_urls is None
            else tuple(source_urls)
        )
        if len(source_url_values) != len(source_values):
            raise ValueError("source_urls must have the same length as sources")
        tasks = tuple(
            self.submit(
                source,
                source_url=source_url,
                profile=profile,
                options=options,
                timeout_ms=timeout_ms,
            )
            for source, source_url, profile in zip(
                source_values,
                source_url_values,
                profile_values,
                strict=True,
            )
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

    def completed_worker_id(self, task_id: int) -> int | None:
        """Return the native Worker id recorded for a completed task."""

        with self._condition:
            return self._completed_worker_by_task.get(task_id)

    def completed_worker_process_id(self, task_id: int) -> int | None:
        """Return the OS PID of the one Worker that executed a task."""

        with self._condition:
            return self._completed_process_by_task.get(task_id)

    def clear_network_requests(self, task_id: int | None = None) -> None:
        with self._condition:
            if task_id is None:
                self._requests.clear()
                self._completed_worker_by_task.clear()
                self._completed_process_by_task.clear()
                self._collected_tasks.clear()
            else:
                self._requests = [
                    request for request in self._requests if request.task_id != task_id
                ]
                self._completed_worker_by_task.pop(task_id, None)
                self._completed_process_by_task.pop(task_id, None)
                self._collected_tasks.discard(task_id)

    def stdout(self, task_id: int | None = None) -> tuple[CapturedConsoleOutput, ...]:
        """Return console output only for tasks submitted with capture enabled."""

        with self._condition:
            if task_id is not None:
                return self._stdout_by_task.get(task_id, ())
            return tuple(
                entry
                for completed_task in sorted(self._stdout_by_task)
                for entry in self._stdout_by_task[completed_task]
            )

    def clear_stdout(self, task_id: int | None = None) -> None:
        with self._condition:
            if task_id is None:
                self._stdout_by_task.clear()
            else:
                self._stdout_by_task.pop(task_id, None)

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
        preload_javascript: str | None,
        preload_source_url: str,
        source_url: str | None,
        profile: EdgeProfile | None,
        options: EdgeRunOptions,
        capture_stdout: bool,
    ) -> str:
        worker = self._acquire_worker(profile, options)
        discard = False
        try:
            worker.sandbox.clear_network_requests()
            if capture_stdout:
                worker.sandbox.set_stdout_capture_enabled(True)
            if preload_javascript is not None:
                worker.sandbox.evaluate(
                    preload_javascript,
                    source_url=preload_source_url,
                )
            value = worker.sandbox.evaluate(source, source_url=source_url)
            captured = worker.sandbox.network_requests()
            worker.sandbox.clear_network_requests()
            stdout = worker.sandbox.stdout() if capture_stdout else ()
            if capture_stdout:
                worker.sandbox.clear_stdout()
            self._store_requests(
                task_id,
                worker.worker_id,
                worker.process_id,
                captured,
            )
            if capture_stdout:
                self._store_stdout(task_id, stdout)
            return value
        except BaseException:
            # Timeouts and native failures must release the process and its V8
            # heap instead of putting a potentially damaged worker back.
            discard = True
            raise
        finally:
            if capture_stdout and not self._one_shot_workers and not discard:
                try:
                    worker.sandbox.set_stdout_capture_enabled(False)
                except BaseException:
                    discard = True
            self._release_worker(worker, discard)

    def _store_stdout(
        self,
        task_id: int,
        entries: tuple[CapturedConsoleOutput, ...],
    ) -> None:
        with self._condition:
            self._stdout_by_task[task_id] = entries

    def _store_requests(
        self,
        task_id: int,
        worker_id: int,
        worker_process_id: int,
        requests: tuple[CapturedNetworkRequest, ...],
    ) -> None:
        pooled = [
            PooledNetworkRequest(
                task_id=task_id,
                worker_id=worker_id,
                worker_process_id=worker_process_id,
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
            self._completed_process_by_task[task_id] = worker_process_id

    def _acquire_worker(
        self,
        profile: EdgeProfile | None,
        options: EdgeRunOptions,
    ) -> _PoolWorker:
        if self._one_shot_workers:
            return self._acquire_one_shot_worker(profile)
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
            sandbox.set_stdout_capture_enabled(False)
        except BaseException:
            with self._condition:
                self._creating_workers -= 1
                self._condition.notify_all()
            raise
        created = _PoolWorker(
            worker_id,
            sandbox.process_id(),
            sandbox,
            profile,
            options,
        )
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

    def _acquire_one_shot_worker(
        self,
        profile: EdgeProfile | None,
    ) -> _PoolWorker:
        while True:
            spawn_missing = False
            with self._condition:
                if self._closed:
                    raise RuntimeError("edge sandbox pool is closed")
                available = next(
                    (worker for worker in self._workers if not worker.busy),
                    None,
                )
                if available is not None:
                    available.busy = True
                    break
                if (
                    len(self._workers) + self._creating_workers
                    < self._maximum_workers
                ):
                    spawn_missing = True
                else:
                    self._condition.wait()
            if spawn_missing:
                self._spawn_blank_worker()

        try:
            available.sandbox.reinitialize_profile(profile or EdgeProfile())
            # Reinitialization constructs a fresh isolate whose direct-caller
            # compatibility default is capture-on. Pools always reset it to
            # capture-off before deciding whether this task opted in.
            available.sandbox.set_stdout_capture_enabled(False)
            available.profile = profile
            return available
        except BaseException:
            with self._condition:
                if available in self._workers:
                    self._workers.remove(available)
                self._condition.notify_all()
            available.sandbox.close()
            self._spawn_blank_worker()
            raise

    def _spawn_blank_worker(self) -> _PoolWorker:
        with self._condition:
            if self._closed:
                raise RuntimeError("edge sandbox pool is closed")
            self._creating_workers += 1
            worker_id = next(self._worker_ids)
        try:
            sandbox = EdgeSandbox(
                library=self._library_path,
                worker=self._worker_path,
                profile=self._default_profile,
                options=self._one_shot_options,
            )
            sandbox.set_stdout_capture_enabled(False)
            created = _PoolWorker(
                worker_id=worker_id,
                process_id=sandbox.process_id(),
                sandbox=sandbox,
                profile=None,
                options=self._one_shot_options,
                busy=False,
            )
        except BaseException:
            with self._condition:
                self._creating_workers -= 1
                self._condition.notify_all()
            raise
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
        if self._one_shot_workers:
            with self._condition:
                if worker in self._workers:
                    self._workers.remove(worker)
                should_replace = not self._closed
                self._condition.notify_all()
            worker.sandbox.close()
            if should_replace:
                self._spawn_blank_worker()
            return
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
