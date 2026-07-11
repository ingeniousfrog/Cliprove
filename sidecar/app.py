"""Cliprove Python sidecar."""

from __future__ import annotations

import argparse
from typing import Any

try:
    from fastapi import FastAPI, HTTPException
    from pydantic import BaseModel, Field
    import uvicorn
except ImportError as exc:  # pragma: no cover
    raise SystemExit(
        "Missing sidecar dependencies. Run: pip install -r sidecar/requirements.txt"
    ) from exc

from jobs import Job, JobManager
from platforms.douyin.service import douyin_service

APP_VERSION = "0.3.0-phase2"
job_manager = JobManager()

app = FastAPI(title="Cliprove Sidecar", version=APP_VERSION)


class ParseRequest(BaseModel):
    url: str
    cookies: str = ""
    proxy: str = ""


class DownloadRequest(BaseModel):
    platform: str
    platform_item_id: str = Field(alias="platformItemId")
    output_dir: str = Field(alias="outputDir")
    asset_ids: list[str] = Field(default_factory=list, alias="assetIds")
    cookies: str = ""
    proxy: str = ""

    model_config = {"populate_by_name": True}


class AuthRequest(BaseModel):
    platform: str
    cookies: str = ""
    proxy: str = ""


class SearchRequest(BaseModel):
    platform: str
    keyword: str
    cursor: str | None = None
    page_size: int = Field(default=20, alias="pageSize")
    filters: dict[str, str] | None = None
    cookies: str = ""
    proxy: str = ""

    model_config = {"populate_by_name": True}


@app.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok", "version": APP_VERSION}


@app.post("/v1/parse")
async def parse_media(request: ParseRequest) -> dict[str, Any]:
    try:
        if _is_douyin(request.url):
            return await douyin_service.parse(
                request.url,
                cookies=request.cookies,
                proxy=request.proxy,
            )
        raise HTTPException(status_code=400, detail="unsupported_link")
    except HTTPException:
        raise
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc)) from exc
    except Exception as exc:  # noqa: BLE001
        raise HTTPException(status_code=500, detail=str(exc)) from exc


@app.post("/v1/download")
async def start_download(request: DownloadRequest) -> dict[str, Any]:
    if request.platform != "douyin":
        raise HTTPException(status_code=400, detail="unsupported platform")

    async def run(job: Job) -> dict[str, Any]:
        job.stage = "downloading"
        job.progress = 0.2
        result = await douyin_service.download(
            platform_item_id=request.platform_item_id,
            output_dir=request.output_dir,
            asset_ids=request.asset_ids or ["video", "cover", "metadata"],
            cookies=request.cookies,
            proxy=request.proxy,
        )
        job.progress = 0.95
        return result

    job = await job_manager.submit(run)
    job_manager.prune()
    return job.to_dict()


@app.get("/v1/jobs/{job_id}")
def get_job(job_id: str) -> dict[str, Any]:
    job = job_manager.get(job_id)
    if job is None:
        raise HTTPException(status_code=404, detail="job not found")
    return job.to_dict()


@app.post("/v1/search")
async def search_media(request: SearchRequest) -> dict[str, Any]:
    try:
        if request.platform == "douyin":
            return await douyin_service.search(
                request.keyword,
                cursor=request.cursor,
                page_size=request.page_size,
                filters=request.filters,
                cookies=request.cookies,
                proxy=request.proxy,
            )
        raise HTTPException(status_code=400, detail="unsupported platform")
    except HTTPException:
        raise
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc)) from exc
    except Exception as exc:  # noqa: BLE001
        raise HTTPException(status_code=500, detail=str(exc)) from exc


@app.post("/v1/auth/validate")
async def validate_auth(request: AuthRequest) -> dict[str, Any]:
    if request.platform == "douyin":
        return await douyin_service.validate_auth(
            cookies=request.cookies,
            proxy=request.proxy,
        )
    return {
        "platform": request.platform,
        "valid": False,
        "message": "该平台尚未接入真实引擎",
    }


def _is_douyin(url: str) -> bool:
    lowered = url.lower()
    return any(
        token in lowered
        for token in ("douyin.com", "iesdouyin.com", "v.douyin.com")
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="Cliprove Python sidecar")
    parser.add_argument("--port", type=int, default=18765)
    parser.add_argument("--host", default="127.0.0.1")
    args = parser.parse_args()
    uvicorn.run(app, host=args.host, port=args.port, log_level="warning")


if __name__ == "__main__":
    main()
