"""Bilibili platform service for Cliprove sidecar."""

from __future__ import annotations

import asyncio
import re
from typing import Any

import yt_dlp
from bilibili_api import search, video
from bilibili_api.search import OrderVideo, SearchObjectType

from platforms.cookies import cookie_header_to_dict, normalize_bilibili_url, write_netscape_cookie_file
from platforms.errors import map_exception

from .downloader import download_video
from .mapper import bilibili_preview_url, info_to_parsed_media, search_result_to_media_item


def _is_bilibili_url(url: str) -> bool:
    lowered = url.lower()
    return (
        "bilibili.com" in lowered
        or "b23.tv" in lowered
        or bool(re.fullmatch(r"bv[\w]+", url.strip(), flags=re.IGNORECASE))
        or bool(re.fullmatch(r"av\d+", url.strip(), flags=re.IGNORECASE))
    )


def _ydl_base_opts(cookies: str = "", proxy: str = "") -> dict[str, Any]:
    opts: dict[str, Any] = {
        "quiet": True,
        "no_warnings": True,
        "noplaylist": False,
        "extract_flat": False,
    }
    cookie_file = write_netscape_cookie_file(cookies)
    if cookie_file:
        opts["cookiefile"] = str(cookie_file)
    if proxy:
        opts["proxy"] = proxy
    return opts


def _parse_filter_order(filters: dict[str, Any] | None) -> OrderVideo:
    mapping = {
        "total": OrderVideo.TOTALRANK,
        "totalrank": OrderVideo.TOTALRANK,
        "click": OrderVideo.CLICK,
        "pubdate": OrderVideo.PUBDATE,
        "dm": OrderVideo.DM,
        "stow": OrderVideo.STOW,
    }
    if not filters:
        return OrderVideo.TOTALRANK
    raw = str(filters.get("sort") or "total")
    return mapping.get(raw.lower(), OrderVideo.TOTALRANK)


async def _resolve_preview_url(bvid: str) -> str | None:
    if not bvid:
        return None
    try:
        item = video.Video(bvid=bvid)
        info = await item.get_info()
        pages = info.get("pages") if isinstance(info, dict) else None
        first_page = pages[0] if isinstance(pages, list) and pages else {}
        cid = first_page.get("cid") if isinstance(first_page, dict) else None
        aid = info.get("aid") if isinstance(info, dict) else None
        return bilibili_preview_url(bvid, aid=aid, cid=cid)
    except Exception:  # noqa: BLE001
        return bilibili_preview_url(bvid)


class BilibiliService:
    async def parse(self, url: str, cookies: str = "", proxy: str = "") -> dict[str, Any]:
        if not _is_bilibili_url(url):
            raise ValueError("不是有效的 Bilibili 链接")

        normalized = normalize_bilibili_url(url)
        opts = _ydl_base_opts(cookies, proxy)

        def extract() -> dict[str, Any]:
            try:
                with yt_dlp.YoutubeDL(opts) as ydl:
                    info = ydl.extract_info(normalized, download=False)
                    if not info:
                        raise ValueError("无法解析 Bilibili 链接")
                    return info
            except Exception as exc:  # noqa: BLE001
                raise map_exception(exc) from exc

        info = await asyncio.to_thread(extract)
        parsed = info_to_parsed_media(info, url.strip())
        bvid = parsed["item"].get("platformItemId")
        if isinstance(bvid, str):
            parsed["item"]["previewUrl"] = await _resolve_preview_url(bvid)
        return parsed

    async def search(
        self,
        keyword: str,
        *,
        cursor: str | None = None,
        page_size: int = 20,
        filters: dict[str, Any] | None = None,
        cookies: str = "",
        proxy: str = "",
    ) -> dict[str, Any]:
        page = int(cursor) if cursor else 1
        page_size = max(1, min(page_size, 50))
        order = _parse_filter_order(filters)

        result = await search.search_by_type(
            keyword,
            SearchObjectType.VIDEO,
            page=page,
            order_type=order,
            page_size=page_size,
        )

        raw_items = result.get("result") or []
        items = [
            mapped
            for item in raw_items
            if (mapped := search_result_to_media_item(item, search_keyword=keyword))
        ]

        num_pages = int(result.get("numPages") or page)
        has_more = page < num_pages

        if not items and page == 1:
            raise ValueError("未找到相关结果，请尝试更换关键词或调整筛选条件")

        return {
            "items": items,
            "cursor": str(page + 1) if has_more else None,
            "hasMore": has_more,
            "supportedFilters": ["sort", "media_type"],
        }

    async def preview_url(self, bvid: str) -> str | None:
        return await _resolve_preview_url(bvid)

    async def download(
        self,
        *,
        canonical_url: str,
        output_dir: str,
        asset_ids: list[str],
        cookies: str = "",
        proxy: str = "",
        ffmpeg_path: str = "ffmpeg",
        quality_id: str | None = None,
    ) -> dict[str, Any]:
        return await download_video(
            canonical_url=canonical_url,
            output_dir=output_dir,
            asset_ids=asset_ids,
            cookies=cookies,
            ffmpeg_path=ffmpeg_path,
            quality_id=quality_id,
        )

    async def validate_auth(self, cookies: str = "", proxy: str = "") -> dict[str, Any]:
        cookie_map = cookie_header_to_dict(cookies)
        if not cookie_map.get("SESSDATA"):
            return {
                "platform": "bilibili",
                "valid": False,
                "message": "未配置 SESSDATA Cookie",
            }

        try:
            from bilibili_api import Credential, user

            credential = Credential(
                sessdata=cookie_map.get("SESSDATA"),
                bili_jct=cookie_map.get("bili_jct"),
                buvid3=cookie_map.get("buvid3"),
            )
            info = await user.get_self_info(credential)
            name = info.get("name") or info.get("uname") or "已登录用户"
            return {
                "platform": "bilibili",
                "valid": True,
                "message": f"Cookie 有效（{name}）",
            }
        except Exception as exc:  # noqa: BLE001
            return {
                "platform": "bilibili",
                "valid": False,
                "message": f"Cookie 验证失败: {exc}",
            }


bilibili_service = BilibiliService()
