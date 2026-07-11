"""Map raw Douyin aweme payloads to Cliprove DTOs."""

from __future__ import annotations

from typing import Any

from .bootstrap import ensure_engine_path

ensure_engine_path()

from core.downloader_base import BaseDownloader  # noqa: E402


def _first_url(source: Any) -> str | None:
    urls = BaseDownloader._extract_urls(source)
    return urls[0] if urls else None


def _media_type(aweme: dict[str, Any]) -> str:
    aweme_type = aweme.get("aweme_type")
    images = aweme.get("images") or []
    image_post = aweme.get("image_post_info") or {}
    if aweme_type in (2, 68) or images or image_post:
        return "image_post"
    return "video"


def aweme_to_media_item(
    aweme: dict[str, Any],
    *,
    search_keyword: str | None = None,
) -> dict[str, Any]:
    author = aweme.get("author") or {}
    aweme_id = str(aweme.get("aweme_id") or "")
    media_type = _media_type(aweme)
    video = aweme.get("video") or {}
    cover_source = video.get("cover") or video.get("origin_cover")
    if not _first_url(cover_source):
        images = aweme.get("images") or []
        if images:
            cover_source = images[0].get("url_list") or images[0]

    return {
        "platform": "douyin",
        "platformItemId": aweme_id,
        "originalUrl": f"https://www.douyin.com/video/{aweme_id}",
        "canonicalUrl": f"https://www.douyin.com/video/{aweme_id}",
        "title": (aweme.get("desc") or f"Douyin {aweme_id}").strip() or aweme_id,
        "description": aweme.get("desc"),
        "author": {
            "id": str(author.get("sec_uid") or author.get("uid") or "unknown"),
            "name": str(author.get("nickname") or "未知作者"),
            "avatarUrl": _first_url(author.get("avatar_thumb")),
        },
        "publishedAt": int(aweme.get("create_time") or 0) * 1000 or None,
        "mediaType": media_type,
        "durationSec": int((video.get("duration") or 0) // 1000)
        if video.get("duration")
        else None,
        "coverUrl": _first_url(cover_source),
        "searchKeyword": search_keyword,
    }


def aweme_to_parsed_media(aweme: dict[str, Any], original_url: str) -> dict[str, Any]:
    item = aweme_to_media_item(aweme)
    item["originalUrl"] = original_url
    media_type = item["mediaType"]

    assets: list[dict[str, Any]] = [
        {
            "id": "video",
            "kind": "video",
            "label": "视频",
            "selected": media_type == "video",
        },
        {
            "id": "cover",
            "kind": "cover",
            "label": "封面",
            "selected": True,
        },
        {
            "id": "audio",
            "kind": "audio",
            "label": "音频",
            "selected": False,
        },
        {
            "id": "metadata",
            "kind": "metadata",
            "label": "元数据",
            "selected": True,
        },
    ]

    if media_type == "image_post":
        image_count = len(aweme.get("images") or [])
        assets.insert(
            0,
            {
                "id": "images",
                "kind": "image",
                "label": f"图集 ({image_count or '?'})",
                "selected": True,
            },
        )

    return {
        "item": item,
        "assets": assets,
        "qualities": [
            {"id": "highest", "label": "最高画质", "height": 1080},
        ],
    }
