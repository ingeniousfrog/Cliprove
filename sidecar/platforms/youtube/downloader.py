"""YouTube download via yt-dlp."""

from __future__ import annotations

import asyncio
import json
import urllib.request
from pathlib import Path
from typing import Any

import yt_dlp

from platforms.cover_url import normalize_cover_url
from platforms.errors import ffmpeg_unavailable
from platforms.ffmpeg_resolve import resolve_ffmpeg_path


def _format_selector(quality_id: str | None) -> str:
    if not quality_id or quality_id in {"best", "highest"}:
        return "bestvideo*+bestaudio/best"
    if quality_id.endswith("p") and quality_id[:-1].isdigit():
        height = quality_id[:-1]
        return f"bestvideo[height<={height}]+bestaudio/best[height<={height}]"
    if quality_id.isdigit():
        return f"bestvideo[height<={quality_id}]+bestaudio/best[height<={quality_id}]"
    return f"{quality_id}+bestaudio/best"


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


def _fetch_thumbnail(url: str, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    request = urllib.request.Request(
        url,
        headers={"User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"},
    )
    with urllib.request.urlopen(request, timeout=60) as response:
        dest.write_bytes(response.read())


def _download_video_sync(
    *,
    canonical_url: str,
    output_dir: str,
    asset_ids: list[str],
    proxy: str = "",
    ffmpeg_path: str = "ffmpeg",
    quality_id: str | None = None,
) -> dict[str, Any]:
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)

    save_video = "video" in asset_ids
    save_subtitles = "subtitle" in asset_ids
    save_metadata = "metadata" in asset_ids
    save_cover = "cover" in asset_ids

    ydl_opts: dict[str, Any] = {
        "quiet": True,
        "no_warnings": True,
        "noplaylist": True,
        "outtmpl": str(output_path / "%(title)s [%(id)s].%(ext)s"),
        "format": _format_selector(quality_id),
        "merge_output_format": "mp4",
        "writesubtitles": save_subtitles,
        "writeautomaticsub": save_subtitles,
        "subtitleslangs": ["zh-Hans", "zh-CN", "zh", "en"],
        "writethumbnail": save_cover and save_video,
        "embedthumbnail": False,
        "skip_download": not save_video,
        "socket_timeout": 20,
        "retries": 2,
        "extractor_retries": 2,
    }
    if proxy:
        ydl_opts["proxy"] = proxy

    resolved_ffmpeg = resolve_ffmpeg_path(ffmpeg_path)
    if resolved_ffmpeg:
        ydl_opts["ffmpeg_location"] = resolved_ffmpeg
    elif save_video:
        raise ffmpeg_unavailable(
            "未找到 FFmpeg。macOS 可执行 brew install ffmpeg，或在设置中指定路径"
        )

    media_paths: list[str] = []
    subtitle_paths: list[str] = []
    cover_path: str | None = None
    metadata_path: str | None = None

    with yt_dlp.YoutubeDL(ydl_opts) as ydl:
        info = ydl.extract_info(canonical_url, download=save_video)
        if info is None:
            raise RuntimeError("无法解析 YouTube 视频信息")

        if save_cover and not save_video:
            thumb_url = _best_thumbnail(info)
            if not thumb_url:
                raise RuntimeError("无法获取封面地址")
            cover_file = output_path / "cover.jpg"
            _fetch_thumbnail(thumb_url, cover_file)
            cover_path = str(cover_file)

        if save_metadata:
            metadata_path = str(output_path / "metadata.json")
            Path(metadata_path).write_text(
                json.dumps(info, ensure_ascii=False, indent=2, default=str),
                encoding="utf-8",
            )

    for path in sorted(output_path.iterdir()):
        if not path.is_file():
            continue
        suffix = path.suffix.lower()
        name = path.name.lower()
        if suffix in {".mp4", ".mkv", ".webm", ".mov"}:
            media_paths.append(str(path))
        elif suffix in {".vtt", ".srt", ".ass"} or ".subtitle" in name:
            subtitle_paths.append(str(path))
        elif suffix in {".jpg", ".jpeg", ".png", ".webp"} and cover_path is None:
            cover_path = str(path)

    if save_cover and cover_path is None:
        for path in output_path.iterdir():
            if path.suffix.lower() in {".jpg", ".jpeg", ".png", ".webp"}:
                cover_path = str(path)
                break

    file_size = sum(
        path.stat().st_size for path in output_path.iterdir() if path.is_file()
    )

    if save_video and not media_paths:
        raise RuntimeError("视频下载失败，请检查网络、FFmpeg 配置或视频可用性")

    if save_cover and cover_path is None:
        raise RuntimeError("封面下载失败，请检查网络")

    return {
        "outputDir": str(output_path),
        "mediaPaths": media_paths,
        "coverPath": cover_path,
        "metadataPath": metadata_path,
        "subtitlePaths": subtitle_paths,
        "fileSize": file_size,
    }


async def download_video(
    *,
    canonical_url: str,
    output_dir: str,
    asset_ids: list[str],
    proxy: str = "",
    ffmpeg_path: str = "ffmpeg",
    quality_id: str | None = None,
) -> dict[str, Any]:
    return await asyncio.to_thread(
        _download_video_sync,
        canonical_url=canonical_url,
        output_dir=output_dir,
        asset_ids=asset_ids,
        proxy=proxy,
        ffmpeg_path=ffmpeg_path,
        quality_id=quality_id,
    )
