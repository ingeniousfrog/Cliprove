"""Cliprove Python sidecar."""

from __future__ import annotations

import argparse
import re
import urllib.request
from typing import Any
from urllib.parse import unquote

try:
    from fastapi import FastAPI, HTTPException
    from fastapi.responses import Response
    from pydantic import BaseModel, Field
    import uvicorn
except ImportError as exc:  # pragma: no cover
    raise SystemExit(
        "Missing sidecar dependencies. Run: pip install -r sidecar/requirements.txt"
    ) from exc

from jobs import Job, JobManager
from platform_login.sessions import auth_session_manager
from platform_login.bilibili_qr import poll_bilibili_qr_login, start_bilibili_qr_login
from platform_login.douyin_browser import start_douyin_browser_login
from platforms.bilibili.service import bilibili_service
from platforms.cover_url import normalize_cover_url, proxy_referer
from platforms.douyin.service import douyin_service

APP_VERSION = "0.5.0-phase5"
job_manager = JobManager()

app = FastAPI(title="Cliprove Sidecar", version=APP_VERSION)


class ParseRequest(BaseModel):
    url: str
    douyin_cookies: str = Field(default="", alias="douyinCookies")
    bilibili_cookies: str = Field(default="", alias="bilibiliCookies")
    proxy: str = ""

    model_config = {"populate_by_name": True}


class DownloadRequest(BaseModel):
    platform: str
    platform_item_id: str = Field(alias="platformItemId")
    output_dir: str = Field(alias="outputDir")
    asset_ids: list[str] = Field(default_factory=list, alias="assetIds")
    canonical_url: str | None = Field(default=None, alias="canonicalUrl")
    quality_id: str | None = Field(default=None, alias="qualityId")
    douyin_cookies: str = Field(default="", alias="douyinCookies")
    bilibili_cookies: str = Field(default="", alias="bilibiliCookies")
    ffmpeg_path: str = Field(default="ffmpeg", alias="ffmpegPath")
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
    douyin_cookies: str = Field(default="", alias="douyinCookies")
    bilibili_cookies: str = Field(default="", alias="bilibiliCookies")
    proxy: str = ""

    model_config = {"populate_by_name": True}


@app.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok", "version": APP_VERSION}


@app.get("/v1/proxy/image")
def proxy_image(url: str, platform: str = "") -> Response:
    normalized = normalize_cover_url(unquote(url))
    if not normalized or not normalized.startswith(("http://", "https://")):
        raise HTTPException(status_code=400, detail="invalid image url")

    request = urllib.request.Request(
        normalized,
        headers={
            "User-Agent": (
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
                "AppleWebKit/537.36 (KHTML, like Gecko) "
                "Chrome/131.0.0.0 Safari/537.36"
            ),
            "Referer": proxy_referer(normalized, platform),
        },
    )
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            body = response.read()
            content_type = response.headers.get("Content-Type", "image/jpeg")
    except Exception as exc:  # noqa: BLE001
        raise HTTPException(status_code=502, detail=f"image fetch failed: {exc}") from exc

    return Response(content=body, media_type=content_type.split(";")[0])


@app.post("/v1/parse")
async def parse_media(request: ParseRequest) -> dict[str, Any]:
    try:
        if _is_douyin(request.url):
            return await douyin_service.parse(
                request.url,
                cookies=request.douyin_cookies,
                proxy=request.proxy,
            )
        if _is_bilibili(request.url):
            return await bilibili_service.parse(
                request.url,
                cookies=request.bilibili_cookies,
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
    if request.platform not in {"douyin", "bilibili"}:
        raise HTTPException(status_code=400, detail="unsupported platform")

    async def run(job: Job) -> dict[str, Any]:
        job.stage = "downloading"
        job.progress = 0.2
        if request.platform == "douyin":
            result = await douyin_service.download(
                platform_item_id=request.platform_item_id,
                output_dir=request.output_dir,
                asset_ids=request.asset_ids or ["video", "cover", "metadata"],
                cookies=request.douyin_cookies,
                proxy=request.proxy,
            )
        else:
            canonical = request.canonical_url or (
                f"https://www.bilibili.com/video/{request.platform_item_id}"
            )
            result = await bilibili_service.download(
                canonical_url=canonical,
                output_dir=request.output_dir,
                asset_ids=request.asset_ids or ["video", "cover", "metadata"],
                cookies=request.bilibili_cookies,
                proxy=request.proxy,
                ffmpeg_path=request.ffmpeg_path,
                quality_id=request.quality_id,
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
                cookies=request.douyin_cookies,
                proxy=request.proxy,
            )
        if request.platform == "bilibili":
            return await bilibili_service.search(
                request.keyword,
                cursor=request.cursor,
                page_size=request.page_size,
                filters=request.filters,
                cookies=request.bilibili_cookies,
                proxy=request.proxy,
            )
        raise HTTPException(status_code=400, detail="unsupported platform")
    except HTTPException:
        raise
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc)) from exc
    except Exception as exc:  # noqa: BLE001
        raise HTTPException(status_code=500, detail=str(exc)) from exc


@app.get("/v1/bilibili/preview/{bvid}")
async def resolve_bilibili_preview(bvid: str) -> dict[str, Any]:
    try:
        return {
            "previewUrl": await bilibili_service.preview_url(bvid),
        }
    except Exception as exc:  # noqa: BLE001
        raise HTTPException(status_code=500, detail=str(exc)) from exc


@app.post("/v1/auth/login/start")
async def start_platform_login(request: AuthRequest) -> dict[str, Any]:
    platform = request.platform
    if platform not in {"douyin", "bilibili"}:
        raise HTTPException(status_code=400, detail="unsupported platform")

    session = auth_session_manager.create(platform)
    try:
        if platform == "bilibili":
            await start_bilibili_qr_login(session)
        else:
            session.message = "正在启动浏览器登录窗口…"
            start_douyin_browser_login(session)
    except Exception as exc:  # noqa: BLE001
        session.status = "failed"
        session.message = str(exc)
    return session.to_dict()


@app.get("/v1/auth/login/{session_id}")
async def poll_platform_login(session_id: str) -> dict[str, Any]:
    session = auth_session_manager.get(session_id)
    if session is None:
        raise HTTPException(status_code=404, detail="login session not found")

    if session.platform == "bilibili" and session.status not in {
        "completed",
        "failed",
        "expired",
    }:
        await poll_bilibili_qr_login(session)

    return session.to_dict()


@app.post("/v1/auth/validate")
async def validate_auth(request: AuthRequest) -> dict[str, Any]:
    if request.platform == "douyin":
        return await douyin_service.validate_auth(
            cookies=request.cookies,
            proxy=request.proxy,
        )
    if request.platform == "bilibili":
        return await bilibili_service.validate_auth(
            cookies=request.cookies,
            proxy=request.proxy,
        )
    return {
        "platform": request.platform,
        "valid": False,
        "message": "不支持的平台",
    }


def _is_douyin(url: str) -> bool:
    lowered = url.lower()
    return any(
        token in lowered
        for token in ("douyin.com", "iesdouyin.com", "v.douyin.com")
    )


def _is_bilibili(url: str) -> bool:
    lowered = url.lower()
    return (
        "bilibili.com" in lowered
        or "b23.tv" in lowered
        or bool(re.fullmatch(r"bv[\w]+", url.strip(), flags=re.IGNORECASE))
        or bool(re.fullmatch(r"av\d+", url.strip(), flags=re.IGNORECASE))
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="Cliprove Python sidecar")
    parser.add_argument("--port", type=int, default=18765)
    parser.add_argument("--host", default="127.0.0.1")
    args = parser.parse_args()
    uvicorn.run(app, host=args.host, port=args.port, log_level="warning")


if __name__ == "__main__":
    main()
