"""Download Douyin assets into Cliprove directory layout."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .bootstrap import ensure_engine_path
from .constants import DOUYIN_USER_AGENT

ensure_engine_path()

from auth import CookieManager  # noqa: E402
from config import ConfigLoader  # noqa: E402
from control import RateLimiter, RetryHandler  # noqa: E402
from core.api_client import DouyinAPIClient  # noqa: E402
from core.video_downloader import VideoDownloader  # noqa: E402
from storage import FileManager  # noqa: E402
from utils.cookie_utils import parse_cookie_header  # noqa: E402


class CliproveDouyinDownloader(VideoDownloader):
    """Reuse engine download logic but force a flat Cliprove output directory."""

    def __init__(
        self,
        output_dir: Path,
        asset_ids: set[str],
        *args: Any,
        **kwargs: Any,
    ):
        super().__init__(*args, **kwargs)
        self._cliprove_output_dir = output_dir
        self._asset_ids = asset_ids

    def _build_aweme_file_context(
        self,
        aweme_data: dict[str, Any],
        author_name: str,
        mode: str | None = None,
        *,
        collection_dir: str | None = None,
    ) -> dict[str, Any] | None:
        metadata = self._aweme_file_metadata(aweme_data)
        if metadata is None:
            return None
        self._cliprove_output_dir.mkdir(parents=True, exist_ok=True)
        return {
            **metadata,
            "file_stem": "media",
            "save_dir": self._cliprove_output_dir,
        }

    async def _download_aweme_assets(
        self,
        aweme_data: dict[str, Any],
        author_name: str,
        mode: str | None = None,
        *,
        db_batch: list[dict[str, Any]] | None = None,
        collection_dir: str | None = None,
    ) -> bool:
        original_cover = self.config.config.get("cover")
        original_music = self.config.config.get("music")
        self.config.config["cover"] = "cover" in self._asset_ids
        self.config.config["music"] = "audio" in self._asset_ids
        try:
            ok = await super()._download_aweme_assets(
                aweme_data,
                author_name,
                mode,
                db_batch=db_batch,
                collection_dir=collection_dir,
            )
        finally:
            self.config.config["cover"] = original_cover
            self.config.config["music"] = original_music

        if not ok:
            return False

        if "metadata" in self._asset_ids:
            metadata_path = self._cliprove_output_dir / "metadata.json"
            metadata_path.write_text(
                json.dumps(aweme_data, ensure_ascii=False, indent=2),
                encoding="utf-8",
            )

        self._normalize_output_names()
        return True

    def _normalize_output_names(self) -> None:
        mapping = {
            "media.mp4": "video.mp4",
            "media_cover.jpg": "cover.jpg",
            "media_music.mp3": "audio.mp3",
        }
        for src, dest in mapping.items():
            source = self._cliprove_output_dir / src
            target = self._cliprove_output_dir / dest
            if source.exists() and source != target:
                if target.exists():
                    target.unlink()
                source.rename(target)

        for path in sorted(self._cliprove_output_dir.glob("media_*.jpg")):
            index = path.stem.split("_")[-1]
            if index.isdigit():
                target = self._cliprove_output_dir / f"image_{index}.jpg"
                if path != target:
                    path.rename(target)


def cookies_dict(cookie_header: str) -> dict[str, str]:
    if not cookie_header.strip():
        return {}
    return parse_cookie_header(cookie_header)


async def download_aweme(
    *,
    aweme_id: str,
    output_dir: str,
    asset_ids: list[str],
    cookies: str,
    proxy: str = "",
) -> dict[str, Any]:
    output_path = Path(output_dir)
    asset_set = set(asset_ids or ["video", "cover", "metadata"])
    cookie_map = cookies_dict(cookies)

    config = ConfigLoader()
    config.config["database"] = False
    config.config["cover"] = "cover" in asset_set
    config.config["music"] = "audio" in asset_set
    config.config["path"] = str(output_path.parent)
    if proxy:
        config.config["proxy"] = proxy

    cookie_manager = CookieManager()
    if cookie_map:
        cookie_manager.set_cookies(cookie_map)

    async with DouyinAPIClient(
        cookie_map,
        proxy=proxy or None,
        user_agent=DOUYIN_USER_AGENT,
    ) as api_client:
        aweme = await api_client.get_video_detail(aweme_id)
        if not aweme:
            raise RuntimeError(f"无法获取作品详情: {aweme_id}")

        downloader = CliproveDouyinDownloader(
            output_path,
            asset_set,
            config,
            api_client,
            FileManager(config.get("path")),
            cookie_manager,
            database=None,
            rate_limiter=RateLimiter(),
            retry_handler=RetryHandler(
                max_retries=int(config.get("retry_times", 3) or 3)
            ),
        )

        author = aweme.get("author") or {}
        success = await downloader._download_aweme_assets(
            aweme,
            str(author.get("nickname") or "unknown"),
        )
        if not success:
            raise RuntimeError("下载失败，请检查 Cookie 或网络连接")

        media_paths = [
            str(path)
            for path in sorted(output_path.iterdir())
            if path.is_file() and path.name != "metadata.json"
        ]
        cover_path = output_path / "cover.jpg"
        metadata_path = output_path / "metadata.json"

        return {
            "outputDir": str(output_path),
            "mediaPaths": media_paths,
            "coverPath": str(cover_path) if cover_path.exists() else None,
            "metadataPath": str(metadata_path) if metadata_path.exists() else None,
            "subtitlePaths": [],
            "fileSize": sum(path.stat().st_size for path in output_path.iterdir() if path.is_file()),
        }
