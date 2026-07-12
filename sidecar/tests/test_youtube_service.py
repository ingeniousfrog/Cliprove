from __future__ import annotations

import asyncio
import sys
from pathlib import Path
from typing import Any

SIDECAR_ROOT = Path(__file__).resolve().parents[1]
if str(SIDECAR_ROOT) not in sys.path:
    sys.path.insert(0, str(SIDECAR_ROOT))

from platforms.youtube import service
from platforms.youtube.mapper import info_to_media_item


def test_info_to_media_item_maps_youtube_fields() -> None:
    item = info_to_media_item(
        {
            "id": "abc123XYZ00",
            "title": "A useful video",
            "webpage_url": "https://www.youtube.com/watch?v=abc123XYZ00",
            "channel_id": "UC123",
            "channel": "Example Channel",
            "duration": 125,
            "timestamp": 1_704_067_200,
            "thumbnails": [
                {"url": "https://img.youtube.com/vi/abc123XYZ00/default.jpg"},
                {"url": "https://img.youtube.com/vi/abc123XYZ00/hqdefault.jpg"},
            ],
        },
        search_keyword="cliprove",
    )

    assert item["platform"] == "youtube"
    assert item["platformItemId"] == "abc123XYZ00"
    assert item["canonicalUrl"] == "https://www.youtube.com/watch?v=abc123XYZ00"
    assert item["author"]["name"] == "Example Channel"
    assert item["durationSec"] == 125
    assert item["publishedAt"] == 1_704_067_200_000
    assert item["coverUrl"] == "https://img.youtube.com/vi/abc123XYZ00/hqdefault.jpg"
    assert item["previewUrl"] == "https://www.youtube.com/embed/abc123XYZ00"
    assert item["searchKeyword"] == "cliprove"


def test_search_uses_yt_dlp_search_and_cursor_slicing() -> None:
    async def run() -> None:
        seen_urls: list[str] = []

        class FakeYdl:
            def __init__(self, opts: dict[str, Any]):
                assert opts["extract_flat"] == "in_playlist"
                assert opts["skip_download"] is True

            def __enter__(self) -> "FakeYdl":
                return self

            def __exit__(self, *_args: Any) -> None:
                return None

            def extract_info(self, url: str, *, download: bool) -> dict[str, Any]:
                assert download is False
                seen_urls.append(url)
                return {
                    "entries": [
                        {
                            "id": f"video-{index}",
                            "title": f"Result {index}",
                            "url": f"https://www.youtube.com/watch?v=video-{index}",
                            "channel": "Channel",
                        }
                        for index in range(5)
                    ]
                }

        original_youtube_dl = service.yt_dlp.YoutubeDL
        service.yt_dlp.YoutubeDL = FakeYdl
        try:
            page = await service.YouTubeService().search(
                "lofi focus",
                cursor="2",
                page_size=2,
                filters={"sort": "date"},
            )
        finally:
            service.yt_dlp.YoutubeDL = original_youtube_dl

        assert seen_urls == ["ytsearchdate5:lofi focus"]
        assert [item["platformItemId"] for item in page["items"]] == [
            "video-2",
            "video-3",
        ]
        assert page["cursor"] == "4"
        assert page["hasMore"] is True
        assert page["supportedFilters"] == ["sort"]

    asyncio.run(run())


def test_download_delegates_to_youtube_downloader() -> None:
    async def run() -> None:
        calls: list[dict[str, Any]] = []

        async def fake_download_video(**kwargs: Any) -> dict[str, Any]:
            calls.append(kwargs)
            return {
                "mediaPaths": ["/tmp/video.mp4"],
                "coverPath": "/tmp/cover.jpg",
                "metadataPath": "/tmp/metadata.json",
                "subtitlePaths": [],
                "fileSize": 12,
            }

        original_download_video = service.download_video
        service.download_video = fake_download_video
        try:
            result = await service.YouTubeService().download(
                canonical_url="https://www.youtube.com/watch?v=abc123XYZ00",
                output_dir="/tmp/cliprove",
                asset_ids=["video", "cover", "metadata"],
                proxy="http://127.0.0.1:7890",
                ffmpeg_path="/opt/homebrew/bin/ffmpeg",
                quality_id="720p",
            )
        finally:
            service.download_video = original_download_video

        assert result["mediaPaths"] == ["/tmp/video.mp4"]
        assert calls == [
            {
                "canonical_url": "https://www.youtube.com/watch?v=abc123XYZ00",
                "output_dir": "/tmp/cliprove",
                "asset_ids": ["video", "cover", "metadata"],
                "proxy": "http://127.0.0.1:7890",
                "ffmpeg_path": "/opt/homebrew/bin/ffmpeg",
                "quality_id": "720p",
            }
        ]

    asyncio.run(run())


def test_parse_maps_youtube_region_restriction() -> None:
    async def run() -> None:
        class FakeYdl:
            def __init__(self, _opts: dict[str, Any]):
                pass

            def __enter__(self) -> "FakeYdl":
                return self

            def __exit__(self, *_args: Any) -> None:
                return None

            def extract_info(self, _url: str, *, download: bool) -> dict[str, Any]:
                assert download is False
                raise RuntimeError(
                    "ERROR: [youtube] MtNR2zYLuPA: The uploader has not made "
                    "this video available in your country"
                )

        original_youtube_dl = service.yt_dlp.YoutubeDL
        service.yt_dlp.YoutubeDL = FakeYdl
        try:
            try:
                await service.YouTubeService().parse(
                    "https://www.youtube.com/watch?v=MtNR2zYLuPA"
                )
            except RuntimeError as exc:
                assert str(exc).startswith("CLIPROVE_REGION_RESTRICTED:")
            else:
                raise AssertionError("expected region restriction error")
        finally:
            service.yt_dlp.YoutubeDL = original_youtube_dl

    asyncio.run(run())
