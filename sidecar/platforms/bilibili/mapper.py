"""Map yt-dlp / bilibili-api payloads to Cliprove DTOs."""

from __future__ import annotations

from typing import Any

from platforms.cover_url import normalize_cover_url


def _best_thumbnail(info: dict[str, Any]) -> str | None:
    thumbs = info.get("thumbnails") or []
    if thumbs:
        url = thumbs[-1].get("url")
        return normalize_cover_url(url if isinstance(url, str) else None)
    return normalize_cover_url(info.get("thumbnail") if isinstance(info.get("thumbnail"), str) else None)


def _quality_options(info: dict[str, Any]) -> list[dict[str, Any]]:
    seen: dict[int, dict[str, Any]] = {}
    for fmt in info.get("formats") or []:
        if not isinstance(fmt, dict):
            continue
        if fmt.get("vcodec") in (None, "none"):
            continue
        height = fmt.get("height")
        if not height:
            continue
        height = int(height)
        fmt_id = str(fmt.get("format_id") or height)
        label = f"{height}P"
        if height not in seen or (fmt.get("tbr") or 0) > (seen[height].get("_tbr") or 0):
            seen[height] = {
                "id": fmt_id,
                "label": label,
                "height": height,
                "_tbr": fmt.get("tbr") or 0,
            }
    return [
        {"id": item["id"], "label": item["label"], "height": item["height"]}
        for item in sorted(seen.values(), key=lambda x: x["height"], reverse=True)
    ]


def _part_assets(info: dict[str, Any]) -> list[dict[str, Any]]:
    entries = info.get("entries") or []
    if not entries:
        return []

    assets: list[dict[str, Any]] = []
    for index, entry in enumerate(entries, start=1):
        if not isinstance(entry, dict):
            continue
        part_id = str(entry.get("id") or index)
        title = entry.get("title") or f"P{index}"
        assets.append(
            {
                "id": f"part-{part_id}",
                "kind": "video",
                "label": f"P{index}: {title}",
                "url": entry.get("webpage_url") or entry.get("original_url"),
                "selected": True,
            }
        )
    return assets


def info_to_media_item(
    info: dict[str, Any],
    *,
    search_keyword: str | None = None,
) -> dict[str, Any]:
    bvid = str(info.get("id") or info.get("display_id") or "")
    webpage_url = info.get("webpage_url") or info.get("original_url") or ""
    if not webpage_url and bvid:
        webpage_url = f"https://www.bilibili.com/video/{bvid}"

    entries = [entry for entry in (info.get("entries") or []) if isinstance(entry, dict)]
    media_type = "multipart" if len(entries) > 1 else "video"

    return {
        "platform": "bilibili",
        "platformItemId": bvid,
        "originalUrl": webpage_url,
        "canonicalUrl": webpage_url,
        "title": str(info.get("title") or bvid),
        "description": info.get("description"),
        "author": {
            "id": str(info.get("uploader_id") or info.get("channel_id") or "unknown"),
            "name": str(info.get("uploader") or info.get("channel") or "未知 UP 主"),
            "avatarUrl": None,
        },
        "publishedAt": int(info.get("timestamp") or 0) * 1000 or None,
        "mediaType": media_type,
        "durationSec": int(info.get("duration") or 0) or None,
        "coverUrl": _best_thumbnail(info),
        "searchKeyword": search_keyword,
    }


def info_to_parsed_media(info: dict[str, Any], original_url: str) -> dict[str, Any]:
    item = info_to_media_item(info)
    item["originalUrl"] = original_url

    assets: list[dict[str, Any]] = _part_assets(info)
    if not assets:
        assets = [
            {"id": "video", "kind": "video", "label": "视频", "selected": True},
        ]

    assets.extend(
        [
            {"id": "cover", "kind": "cover", "label": "封面", "selected": True},
            {"id": "subtitle", "kind": "subtitle", "label": "字幕", "selected": False},
            {"id": "metadata", "kind": "metadata", "label": "元数据", "selected": True},
        ]
    )

    qualities = _quality_options(info)
    if not qualities:
        qualities = [
            {"id": "best", "label": "最佳画质", "height": 1080},
            {"id": "720p", "label": "720P", "height": 720},
        ]

    return {
        "item": item,
        "assets": assets,
        "qualities": qualities,
    }


def _parse_duration(value: Any) -> int | None:
    if value is None:
        return None
    if isinstance(value, (int, float)):
        return int(value)
    text = str(value).strip()
    if not text:
        return None
    if text.isdigit():
        return int(text)
    if ":" in text:
        parts = text.split(":")
        try:
            if len(parts) == 2:
                return int(parts[0]) * 60 + int(parts[1])
            if len(parts) == 3:
                return int(parts[0]) * 3600 + int(parts[1]) * 60 + int(parts[2])
        except ValueError:
            return None
    return None


def search_result_to_media_item(
    result: dict[str, Any],
    *,
    search_keyword: str,
) -> dict[str, Any] | None:
    if not isinstance(result, dict):
        return None

    bvid = str(result.get("bvid") or "")
    if not bvid:
        return None

    author = result.get("author") or ""
    if isinstance(author, dict):
        author_name = author.get("name") or "未知 UP 主"
        author_id = str(author.get("mid") or "unknown")
    else:
        author_name = str(author or "未知 UP 主")
        author_id = str(result.get("mid") or "unknown")

    duration_sec = _parse_duration(result.get("duration"))
    pic = normalize_cover_url(
        result.get("pic") if isinstance(result.get("pic"), str) else None
    ) or normalize_cover_url(
        result.get("cover") if isinstance(result.get("cover"), str) else None
    )

    return {
        "platform": "bilibili",
        "platformItemId": bvid,
        "originalUrl": f"https://www.bilibili.com/video/{bvid}",
        "canonicalUrl": f"https://www.bilibili.com/video/{bvid}",
        "title": str(result.get("title") or bvid).replace("<em class=\"keyword\">", "").replace("</em>", ""),
        "description": result.get("description"),
        "author": {
            "id": author_id,
            "name": author_name,
            "avatarUrl": None,
        },
        "publishedAt": None,
        "mediaType": "video",
        "durationSec": duration_sec,
        "coverUrl": pic,
        "searchKeyword": search_keyword,
    }
