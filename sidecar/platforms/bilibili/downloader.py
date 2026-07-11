"""Bilibili download via yt-dlp."""

from __future__ import annotations

import json
import shutil
from pathlib import Path
from typing import Any

import yt_dlp

from platforms.cookies import write_netscape_cookie_file


def _format_selector(quality_id: str | None) -> str:
    if not quality_id or quality_id in {"best", "highest"}:
        return "bestvideo*+bestaudio/best"
    if quality_id.endswith("p") and quality_id[:-1].isdigit():
        height = quality_id[:-1]
        return f"bestvideo[height<={height}]+bestaudio/best[height<={height}]"
    if quality_id.isdigit():
        return f"bestvideo[height<={quality_id}]+bestaudio/best[height<={quality_id}]"
    return f"{quality_id}+bestaudio/best"


def _selected_parts(asset_ids: list[str]) -> set[str]:
    parts: set[str] = set()
    for asset_id in asset_ids:
        if asset_id.startswith("part-"):
            parts.add(asset_id.removeprefix("part-"))
    return parts


async def download_video(
    *,
    canonical_url: str,
    output_dir: str,
    asset_ids: list[str],
    cookies: str = "",
    ffmpeg_path: str = "ffmpeg",
    quality_id: str | None = None,
) -> dict[str, Any]:
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)

    cookie_file = write_netscape_cookie_file(cookies)
    selected_parts = _selected_parts(asset_ids)
    save_video = "video" in asset_ids or bool(selected_parts)
    save_subtitles = "subtitle" in asset_ids
    save_metadata = "metadata" in asset_ids
    save_cover = "cover" in asset_ids

    ydl_opts: dict[str, Any] = {
        "quiet": True,
        "no_warnings": True,
        "noplaylist": False,
        "outtmpl": str(output_path / "%(title)s [%(id)s].%(ext)s"),
        "format": _format_selector(quality_id),
        "merge_output_format": "mp4",
        "writesubtitles": save_subtitles,
        "writeautomaticsub": save_subtitles,
        "subtitleslangs": ["zh-Hans", "zh-CN", "zh", "en"],
        "writethumbnail": save_cover,
        "embedthumbnail": False,
        "skip_download": not save_video,
    }

    if shutil.which(ffmpeg_path) or Path(ffmpeg_path).exists():
        ydl_opts["ffmpeg_location"] = ffmpeg_path

    if cookie_file:
        ydl_opts["cookiefile"] = str(cookie_file)

    media_paths: list[str] = []
    subtitle_paths: list[str] = []
    cover_path: str | None = None
    metadata_path: str | None = None

    with yt_dlp.YoutubeDL(ydl_opts) as ydl:
        info = ydl.extract_info(canonical_url, download=save_video)
        if info is None:
            raise RuntimeError("无法解析 Bilibili 视频信息")

        entries = [entry for entry in (info.get("entries") or []) if isinstance(entry, dict)]
        if entries and selected_parts:
            filtered = [
                entry
                for entry in entries
                if str(entry.get("id") or "") in selected_parts
            ]
            if not filtered:
                filtered = entries
            for entry in filtered:
                entry_url = entry.get("webpage_url") or entry.get("original_url")
                if entry_url:
                    ydl.download([entry_url])
        elif save_video:
            ydl.download([canonical_url])

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
        if suffix in {".mp4", ".mkv", ".flv", ".webm"}:
            media_paths.append(str(path))
        elif suffix in {".vtt", ".srt", ".ass"} or ".subtitle" in name:
            subtitle_paths.append(str(path))
        elif suffix in {".jpg", ".jpeg", ".png", ".webp"} and cover_path is None:
            cover_path = str(path)

    if save_cover and cover_path is None:
        thumb = output_path / "cover.jpg"
        for path in output_path.iterdir():
            if path.suffix.lower() in {".jpg", ".jpeg", ".png", ".webp"}:
                path.rename(thumb)
                cover_path = str(thumb)
                break

    file_size = sum(
        path.stat().st_size for path in output_path.iterdir() if path.is_file()
    )

    if cookie_file and cookie_file.exists():
        cookie_file.unlink(missing_ok=True)

    if save_video and not media_paths:
        raise RuntimeError("视频下载失败，请检查 Cookie 或 FFmpeg 配置")

    return {
        "outputDir": str(output_path),
        "mediaPaths": media_paths,
        "coverPath": cover_path,
        "metadataPath": metadata_path,
        "subtitlePaths": subtitle_paths,
        "fileSize": file_size,
    }
