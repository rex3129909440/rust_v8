"""Expose the existing business flow through a local FastAPI endpoint.

Each ``POST /v1/run`` request supplies a country code, user agent and proxy.
The complete business flow runs inside the service: it loads a new JavaScript
source, executes that source in one prewarmed one-shot sandbox Worker, exports
the captured ``/tl`` request, and then continues the original business HTTP
request.  The FastAPI process owns one fixed Worker pool; HTTP requests never
create their own sandbox controller process.
"""

from __future__ import annotations
from sdkDt import get_dt
import asyncio
import re
from contextlib import asynccontextmanager
from dataclasses import dataclass, replace
from pathlib import Path
from typing import AsyncIterator
from urllib.parse import urlencode

import execjs
from curl_cffi import requests
from fastapi import FastAPI, HTTPException, Request, status
from pydantic import BaseModel, ConfigDict, Field

try:  # Installed wheel.
    from edge_sandbox import EdgeSandboxPool, create_country_profile_details
    from edge_sandbox.edge_runtime_options import EdgeRunOptions
except ImportError:  # Source checkout.
    from examples import create_country_profile_details
    from examples.edge_runtime_options import EdgeRunOptions
    from examples.edge_sandbox_pool import EdgeSandboxPool

# This business-owned preset remains in the source checkout and is deliberately
# excluded from the wheel. Core sandbox imports above work with pip installation.
from examples.run_complete_iframe_hook import build_runtime_options


DEFAULT_WORKERS = 10
DEFAULT_TIMEOUT_MS = 30_000
DEFAULT_X_KPSDK_V = "j-1.2.543"
PAGE_ORIGIN = "https://"
PAGE_PATH = (
    f"{PAGE_ORIGIN}/149e9513-01fa-4fb0-aad4-566afd725d1b/"
    "2d206a39-8ed7-437e-a3be-862e0f06eea3/fp"
)
DT_JAVASCRIPT_PATH = Path(__file__).with_name("copli_dt.js")


def _page_url(x_kpsdk_v: str) -> str:
    return f"{PAGE_PATH}?{urlencode({'x-kpsdk-v': x_kpsdk_v})}"


def build_evaluation_runtime_options(*, timeout_ms: int) -> EdgeRunOptions:
    """Keep the complete iframe hooks and use an advancing Edge clock."""

    options = build_runtime_options()
    return replace(
        options,
        deterministic=replace(options.deterministic, clock_epoch_ms=None),
        limits=replace(options.limits, timeout_ms=timeout_ms),
    )


class BusinessRequest(BaseModel):
    """Caller-owned inputs for one complete business execution."""

    model_config = ConfigDict(populate_by_name=True)

    country_code: str = Field(alias="countryCode", min_length=2, max_length=2)
    user_agent: str = Field(alias="userAgent", min_length=1)
    proxy: str = Field(min_length=1)
    x_kpsdk_v: str = Field(
        default=DEFAULT_X_KPSDK_V,
        alias="x-kpsdk-v",
        min_length=1,
    )


class FingerprintResponse(BaseModel):
    seed: int
    country_code: str
    platform: str
    time_zone: str
    user_agent: str
    language: str
    cpu_logical_processors: int
    device_memory_gb: float
    gpu_model: str
    screen: str


class BusinessResponse(BaseModel):
    worker_id: int
    worker_process_id: int
    replacement_worker_process_ids: list[int]
    fingerprint: FingerprintResponse
    javascript_value: str
    sec_headers: dict[str, str]
    accept_language: str
    tl_status_code: int
    tl_response_text: str
    x_kpsdk_ct: str
    x_kpsdk_st: int


class HealthResponse(BaseModel):
    status: str
    maximum_workers: int
    live_workers: int
    worker_process_ids: list[int]
    trace_enabled: bool


class BusinessFailure(RuntimeError):
    """The sandbox completed, but the downstream business result failed."""


@dataclass(frozen=True, slots=True)
class _BusinessResult:
    worker_id: int
    worker_process_id: int
    replacement_worker_process_ids: tuple[int, ...]
    fingerprint: object
    javascript_value: str
    sec_headers: tuple[tuple[str, str], ...]
    accept_language: str
    tl_status_code: int
    tl_response_text: str
    x_kpsdk_ct: str
    x_kpsdk_st: int


def _validate_inputs(job: BusinessRequest) -> tuple[str, str, str, str]:
    country_code = job.country_code.strip().upper()
    user_agent = job.user_agent.strip()
    proxy = job.proxy.strip()
    x_kpsdk_v = job.x_kpsdk_v.strip()
    if not user_agent:
        raise ValueError("userAgent must not be blank")
    if not proxy:
        raise ValueError("proxy must not be blank")
    if not x_kpsdk_v:
        raise ValueError("x-kpsdk-v must not be blank")
    return country_code, user_agent, proxy, x_kpsdk_v


def _linked_browser_headers(fingerprint: object) -> dict[str, str]:
    generated = {
        str(name).lower(): str(value)
        for name, value in fingerprint.request_headers
    }
    required = (
        "accept-language",
        "sec-ch-ua",
        "sec-ch-ua-mobile",
        "sec-ch-ua-platform",
        "user-agent",
    )
    missing = [name for name in required if not generated.get(name)]
    if missing:
        raise ValueError(
            "generated profile is missing request headers: "
            + ", ".join(missing)
        )
    return {name: generated[name] for name in required}


def _document_headers(fingerprint: object) -> dict[str, str]:
    headers = _linked_browser_headers(fingerprint)
    headers.update(
        {
            "accept": (
                "text/html,application/xhtml+xml,application/xml;q=0.9,"
                "image/avif,image/webp,image/apng,*/*;q=0.8,"
                "application/signed-exchange;v=b3;q=0.7"
            ),
            "priority": "u=0, i",
            "sec-fetch-dest": "document",
            "sec-fetch-mode": "navigate",
            "sec-fetch-site": "none",
            "sec-fetch-user": "?1",
            "upgrade-insecure-requests": "1",
        }
    )
    return headers


def _tl_headers(
    fingerprint: object,
    captured_headers: dict[str, str],
    *,
    page_url: str,
    x_kpsdk_v: str,
) -> dict[str, str]:
    try:
        x_kpsdk_ct = captured_headers["x-kpsdk-ct"]
        x_kpsdk_im = captured_headers["x-kpsdk-im"]
    except KeyError as exc:
        raise RuntimeError(
            f"sandbox /tl request is missing {exc.args[0]}"
        ) from exc

    headers = _linked_browser_headers(fingerprint)
    headers.update(
        {
            "accept": "*/*",
            "content-type": "application/octet-stream",
            "origin": PAGE_ORIGIN,
            "priority": "u=1, i",
            "referer": page_url,
            "sec-fetch-dest": "empty",
            "sec-fetch-mode": "cors",
            "sec-fetch-site": "same-origin",
            "x-kpsdk-ct": x_kpsdk_ct,
            "x-kpsdk-dt": get_dt(),
            "x-kpsdk-im": x_kpsdk_im,
            "x-kpsdk-v": x_kpsdk_v,
        }
    )
    return headers


def _fingerprint_response(fingerprint: object) -> FingerprintResponse:
    navigator = fingerprint.profile.navigator
    screen = fingerprint.profile.screen
    return FingerprintResponse(
        seed=fingerprint.seed,
        country_code=fingerprint.country_code,
        platform=fingerprint.platform,
        time_zone=fingerprint.time_zone,
        user_agent=navigator.user_agent,
        language=navigator.language,
        cpu_logical_processors=fingerprint.cpu_logical_processors,
        device_memory_gb=fingerprint.device_memory_gb,
        gpu_model=fingerprint.gpu_model,
        screen=f"{screen.width}x{screen.height}",
    )


async def run_business_job(
    pool: EdgeSandboxPool,
    job: BusinessRequest,
) -> _BusinessResult:
    """Asynchronously execute one complete business request and Worker task."""

    country_code, user_agent, proxy, x_kpsdk_v = _validate_inputs(job)
    page_url = _page_url(x_kpsdk_v)
    fingerprint = create_country_profile_details(country_code, user_agent)
    fingerprint = replace(
        fingerprint,
        profile=replace(
            fingerprint.profile,
            window=replace(
                fingerprint.profile.window,
                inner_width=0.0,
                inner_height=0.0,
            ),
        ),
    )
    headers = _document_headers(fingerprint)

    # The proxy and HTTP session belong to this request only. Concurrent API
    # calls never share cookies, headers or upstream connection state.
    async with requests.AsyncSession(
        impersonate="chrome146",
        proxy=proxy,
        max_clients=1,
    ) as session:
        response = await session.get(page_url, headers=headers)
        ak_bm_vw_1 = response.headers["x-kpsdk-ct"]
        cookies = {
            "ak_bm_vw_1.1": ak_bm_vw_1,
            "ak_bm_vw_1.1-ssn": ak_bm_vw_1,
        }

        match = re.search(r'src="(.*?)"></script>', response.text)
        if match is None:
            raise RuntimeError(
                "business page did not contain the expected script"
            )
        js_path = match.group(1)
        js_url = PAGE_ORIGIN + js_path.replace("&amp;", "&")
        js_response = await session.get(js_url, headers=headers)
        js_response.raise_for_status()

        # This is the sandbox step already present in the business. The pool
        # was prewarmed once at FastAPI startup. This request loads its own
        # profile, executes this freshly fetched JavaScript once, then destroys
        # that Worker and creates a blank replacement before result() returns.
        task = pool.submit(
            js_response.text,
            source_url=js_url,
            profile=fingerprint.profile,
        )
        try:
            javascript_value = await asyncio.wrap_future(task.future)
        except asyncio.CancelledError:
            def discard_cancelled_task(_future: object) -> None:
                pool.network_requests(task.task_id)
                pool.clear_network_requests(task.task_id)

            task.future.add_done_callback(discard_cancelled_task)
            raise
        worker_id = pool.completed_worker_id(task.task_id)
        worker_process_id = pool.completed_worker_process_id(task.task_id)
        captured = pool.network_requests(task.task_id)
        pool.clear_network_requests(task.task_id)
        if worker_id is None or worker_process_id is None:
            raise RuntimeError(
                "sandbox task completed without Worker identity"
            )

        tl_requests = tuple(
            request
            for request in captured
            if request.url.rstrip("/").endswith("/tl")
        )
        if not tl_requests:
            raise RuntimeError("sandbox did not export a /tl request")
        tl_request = tl_requests[-1]
        captured_headers = {
            name.lower(): value for name, value in tl_request.headers
        }

        # The original business continues here after the sandbox JavaScript
        # execution and request export.
        tl_headers = await asyncio.to_thread(
            _tl_headers,
            fingerprint,
            captured_headers,
            page_url=page_url,
            x_kpsdk_v=x_kpsdk_v,
        )
        sec_headers = tuple(
            (name, value)
            for name, value in tl_headers.items()
            if name.lower().startswith("sec-")
        )
        accept_language = tl_headers["accept-language"]
        tl_response = await session.post(
            tl_request.url,
            data=tl_request.body,
            headers=tl_headers,
            cookies=cookies,
        )
        if len(tl_response.text) == 0:
            raise BusinessFailure(
                "business failed: /tl response text is empty"
            )
        x_kpsdk_ct = tl_response.headers["x-kpsdk-ct"]
        x_kpsdk_st = int(tl_response.headers["x-kpsdk-st"])

    return _BusinessResult(
        worker_id=worker_id,
        worker_process_id=worker_process_id,
        replacement_worker_process_ids=pool.worker_process_ids,
        fingerprint=fingerprint,
        javascript_value=javascript_value,
        sec_headers=sec_headers,
        accept_language=accept_language,
        tl_status_code=tl_response.status_code,
        tl_response_text=tl_response.text,
        x_kpsdk_ct=x_kpsdk_ct,
        x_kpsdk_st=x_kpsdk_st,
    )


def create_app(
    *,
    maximum_workers: int = DEFAULT_WORKERS,
    default_timeout_ms: int = DEFAULT_TIMEOUT_MS,
) -> FastAPI:
    """Create the business API and its single prewarmed Worker pool."""

    if maximum_workers < 1:
        raise ValueError("maximum_workers must be at least 1")
    if default_timeout_ms < 1:
        raise ValueError("default_timeout_ms must be at least 1")

    @asynccontextmanager
    async def lifespan(application: FastAPI) -> AsyncIterator[None]:
        pool = EdgeSandboxPool(
            workers=maximum_workers,
            timeout_ms=default_timeout_ms,
            close_worker_after_network_requests=False,
            default_options=build_evaluation_runtime_options(
                timeout_ms=default_timeout_ms
            ),
            one_shot_workers=True,
            prewarm=True,
        )
        application.state.sandbox_pool = pool
        try:
            yield
        finally:
            pool.close()

    application = FastAPI(
        title="Local Business Edge Worker API",
        version="1.0.0",
        lifespan=lifespan,
    )

    @application.get("/health", response_model=HealthResponse)
    async def health(request: Request) -> HealthResponse:
        pool: EdgeSandboxPool = request.app.state.sandbox_pool
        return HealthResponse(
            status="ok",
            maximum_workers=pool.maximum_workers,
            live_workers=pool.live_worker_count,
            worker_process_ids=list(pool.worker_process_ids),
            trace_enabled=False,
        )

    @application.post("/v1/run", response_model=BusinessResponse)
    async def run(
        request: Request,
        job: BusinessRequest,
    ) -> BusinessResponse:
        pool: EdgeSandboxPool = request.app.state.sandbox_pool
        try:
            result = await run_business_job(pool, job)
        except ValueError as exc:
            raise HTTPException(
                status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
                detail=str(exc),
            ) from exc
        except BusinessFailure as exc:
            raise HTTPException(
                status_code=status.HTTP_502_BAD_GATEWAY,
                detail=str(exc),
            ) from exc

        return BusinessResponse(
            worker_id=result.worker_id,
            worker_process_id=result.worker_process_id,
            replacement_worker_process_ids=list(
                result.replacement_worker_process_ids
            ),
            fingerprint=_fingerprint_response(result.fingerprint),
            javascript_value=result.javascript_value,
            sec_headers=dict(result.sec_headers),
            accept_language=result.accept_language,
            tl_status_code=result.tl_status_code,
            tl_response_text=result.tl_response_text,
            x_kpsdk_ct=result.x_kpsdk_ct,
            x_kpsdk_st=result.x_kpsdk_st,
        )

    return application


def serve_local(
    *,
    host: str = "127.0.0.1",
    port: int = 8765,
    maximum_workers: int = DEFAULT_WORKERS,
    default_timeout_ms: int = DEFAULT_TIMEOUT_MS,
) -> None:
    """Start the business service programmatically without a CLI wrapper."""

    import uvicorn

    uvicorn.run(
        create_app(
            maximum_workers=maximum_workers,
            default_timeout_ms=default_timeout_ms,
        ),
        host=host,
        port=port,
        workers=1,
    )


app = create_app()
