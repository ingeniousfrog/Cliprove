"""YouTube platform service for Cliprove sidecar."""

from __future__ import annotations

import asyncio
from typing import Any

import yt_dlp

from platforms.errors import map_exception

from .downloader import download_video
from .mapper import info_to_media_item, info_to_parsed_media

YOUTUBE_PARSE_TIMEOUT_SECONDS = 25
YOUTUBE_SEARCH_TIMEOUT_SECONDS = 30
YOUTUBE_SEARCH_MAX_RESULTS = 100


def _is_youtube_url(url: str) -> bool:
    lowered = url.lower()
    return any(
        token in lowered
        for token in ("youtube.com", "youtu.be", "youtube-nocookie.com")
    )


def _ydl_base_opts(proxy: str = "") -> dict[str, Any]:
    opts: dict[str, Any] = {
        "quiet": True,
        "no_warnings": True,
        "noplaylist": True,
        "socket_timeout": 15,
        "retries": 1,
        "extractor_retries": 1,
    }
    if proxy:
        opts["proxy"] = proxy
    return opts


def _search_prefix(filters: dict[str, Any] | None) -> str:
    raw = str((filters or {}).get("sort") or "relevance").lower()
    if raw in {"date", "latest", "upload_date"}:
        return "ytsearchdate"
    return "ytsearch"


class YouTubeService:
    async def parse(self, url: str, proxy: str = "") -> dict[str, Any]:
        if not _is_youtube_url(url):
            raise ValueError("不是有效的 YouTube 链接")

        opts = _ydl_base_opts(proxy)

        def extract() -> dict[str, Any]:
            try:
                with yt_dlp.YoutubeDL(opts) as ydl:
                    info = ydl.extract_info(url.strip(), download=False)
                    if not info:
                        raise ValueError("无法解析 YouTube 链接")
                    return info
            except Exception as exc:  # noqa: BLE001
                raise map_exception(exc) from exc

        try:
            info = await asyncio.wait_for(
                asyncio.to_thread(extract),
                timeout=YOUTUBE_PARSE_TIMEOUT_SECONDS,
            )
        except asyncio.TimeoutError as exc:
            raise ValueError("YouTube 链接解析超时，请稍后重试") from exc

        return info_to_parsed_media(info, url.strip())

    async def search(
        self,
        keyword: str,
        *,
        cursor: str | None = None,
        page_size: int = 20,
        filters: dict[str, Any] | None = None,
        proxy: str = "",
    ) -> dict[str, Any]:
        trimmed = keyword.strip()
        if not trimmed:
            raise ValueError("请输入搜索关键词")

        offset = max(0, int(cursor) if cursor else 0)
        page_size = max(1, min(page_size, 50))
        fetch_limit = min(offset + page_size + 1, YOUTUBE_SEARCH_MAX_RESULTS)
        search_url = f"{_search_prefix(filters)}{fetch_limit}:{trimmed}"
        opts = {
            **_ydl_base_opts(proxy),
            "extract_flat": "in_playlist",
            "skip_download": True,
        }

        def extract() -> dict[str, Any]:
            try:
                with yt_dlp.YoutubeDL(opts) as ydl:
                    info = ydl.extract_info(search_url, download=False)
                    if not info:
                        raise ValueError("无法搜索 YouTube 内容")
                    return info
            except Exception as exc:  # noqa: BLE001
                raise map_exception(exc) from exc

        try:
            info = await asyncio.wait_for(
                asyncio.to_thread(extract),
                timeout=YOUTUBE_SEARCH_TIMEOUT_SECONDS,
            )
        except asyncio.TimeoutError as exc:
            raise ValueError("YouTube 搜索超时，请稍后重试") from exc

        entries = [
            entry
            for entry in (info.get("entries") or [])
            if isinstance(entry, dict) and entry.get("id")
        ]
        page_entries = entries[offset : offset + page_size]
        items = [
            info_to_media_item(entry, search_keyword=trimmed)
            for entry in page_entries
        ]
        has_more = len(entries) > offset + page_size

        if not items and offset == 0:
            raise ValueError("未找到相关结果，请尝试更换关键词")

        next_offset = offset + page_size
        return {
            "items": items,
            "cursor": str(next_offset) if has_more else None,
            "hasMore": has_more,
            "supportedFilters": ["sort"],
        }

    async def download(
        self,
        *,
        canonical_url: str,
        output_dir: str,
        asset_ids: list[str],
        proxy: str = "",
        ffmpeg_path: str = "ffmpeg",
        quality_id: str | None = None,
    ) -> dict[str, Any]:
        return await download_video(
            canonical_url=canonical_url,
            output_dir=output_dir,
            asset_ids=asset_ids,
            proxy=proxy,
            ffmpeg_path=ffmpeg_path,
            quality_id=quality_id,
        )

    async def validate_auth(self, proxy: str = "") -> dict[str, Any]:
        return {
            "platform": "youtube",
            "valid": True,
            "message": "YouTube 公开视频通常无需登录",
        }


youtube_service = YouTubeService()
