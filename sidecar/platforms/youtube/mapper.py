"""Map yt-dlp YouTube payloads to Cliprove DTOs."""

from __future__ import annotations

from typing import Any

from platforms.cover_url import normalize_cover_url


def youtube_watch_url(video_id: str) -> str:
    return f"https://www.youtube.com/watch?v={video_id}"


def youtube_embed_url(video_id: str) -> str | None:
    if not video_id:
        return None
    return f"https://www.youtube.com/embed/{video_id}"


def _best_thumbnail(info: dict[str, Any]) -> str | None:
    thumbnails = info.get("thumbnails") or []
    if thumbnails:
        for entry in reversed(thumbnails):
            if isinstance(entry, dict):
                url = entry.get("url")
                if isinstance(url, str) and url:
                    return normalize_cover_url(url)

    thumbnail = info.get("thumbnail")
    if isinstance(thumbnail, str) and thumbnail:
        return normalize_cover_url(thumbnail)
    return None


def _duration_seconds(value: Any) -> int | None:
    if value is None:
        return None
    if isinstance(value, (int, float)):
        return int(value) or None
    text = str(value).strip()
    if text.isdigit():
        return int(text)
    return None


def _published_at_ms(value: Any) -> int | None:
    if isinstance(value, (int, float)) and value > 0:
        return int(value) * 1000
    return None


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
        bitrate = fmt.get("tbr") or 0
        if height not in seen or bitrate > (seen[height].get("_tbr") or 0):
            seen[height] = {
                "id": f"{height}p",
                "label": f"{height}P",
                "height": height,
                "_tbr": bitrate,
            }

    return [
        {"id": item["id"], "label": item["label"], "height": item["height"]}
        for item in sorted(seen.values(), key=lambda entry: entry["height"], reverse=True)
    ]


def info_to_media_item(
    info: dict[str, Any],
    *,
    search_keyword: str | None = None,
) -> dict[str, Any]:
    video_id = str(info.get("id") or info.get("display_id") or "").strip()
    webpage_url = (
        info.get("webpage_url")
        or info.get("original_url")
        or info.get("url")
        or (youtube_watch_url(video_id) if video_id else "")
    )
    if (
        video_id
        and "youtube.com/watch" not in str(webpage_url)
        and "youtu.be/" not in str(webpage_url)
    ):
        webpage_url = youtube_watch_url(video_id)

    channel_name = (
        info.get("channel")
        or info.get("uploader")
        or info.get("creator")
        or "Unknown channel"
    )
    channel_id = (
        info.get("channel_id")
        or info.get("uploader_id")
        or info.get("channel_url")
        or "unknown"
    )

    return {
        "platform": "youtube",
        "platformItemId": video_id,
        "originalUrl": str(webpage_url),
        "canonicalUrl": str(webpage_url),
        "title": str(info.get("title") or video_id or "Untitled YouTube video"),
        "description": info.get("description"),
        "author": {
            "id": str(channel_id),
            "name": str(channel_name),
            "avatarUrl": None,
        },
        "publishedAt": _published_at_ms(info.get("timestamp")),
        "mediaType": "video",
        "durationSec": _duration_seconds(info.get("duration")),
        "coverUrl": _best_thumbnail(info),
        "previewUrl": youtube_embed_url(video_id),
        "searchKeyword": search_keyword,
    }


def info_to_parsed_media(info: dict[str, Any], original_url: str) -> dict[str, Any]:
    item = info_to_media_item(info)
    item["originalUrl"] = original_url

    qualities = _quality_options(info)
    if not qualities:
        qualities = [
            {"id": "best", "label": "最佳画质", "height": 1080},
            {"id": "720p", "label": "720P", "height": 720},
        ]

    return {
        "item": item,
        "assets": [
            {"id": "video", "kind": "video", "label": "视频", "selected": True},
            {"id": "cover", "kind": "cover", "label": "封面", "selected": True},
            {"id": "subtitle", "kind": "subtitle", "label": "字幕", "selected": False},
            {"id": "metadata", "kind": "metadata", "label": "元数据", "selected": True},
        ],
        "qualities": qualities,
    }
