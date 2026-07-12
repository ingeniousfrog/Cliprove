from __future__ import annotations

import asyncio
import sys
from pathlib import Path
from typing import Any

SIDECAR_ROOT = Path(__file__).resolve().parents[1]
if str(SIDECAR_ROOT) not in sys.path:
    sys.path.insert(0, str(SIDECAR_ROOT))

from platforms.bilibili import service


def test_ydl_base_opts_include_network_timeouts() -> None:
    opts = service._ydl_base_opts()

    assert opts["socket_timeout"] <= 15
    assert opts["retries"] <= 1
    assert opts["extractor_retries"] <= 1


async def test_parse_uses_extracted_info_without_waiting_for_preview_lookup() -> None:
    class FakeYdl:
        def __init__(self, _opts: dict[str, Any]):
            pass

        def __enter__(self) -> "FakeYdl":
            return self

        def __exit__(self, *_args: Any) -> None:
            return None

        def extract_info(self, _url: str, *, download: bool) -> dict[str, Any]:
            assert download is False
            return {
                "id": "BV1DyfiBNE1q",
                "title": "沟通高手",
                "webpage_url": "https://www.bilibili.com/video/BV1DyfiBNE1q",
                "uploader": "职场指南图书店",
                "duration": 58,
                "thumbnail": "https://i0.hdslb.com/bfs/archive/cover.jpg",
            }

    async def fail_if_called(_bvid: str) -> str | None:
        raise AssertionError("preview lookup should not block parsing")

    original_youtube_dl = service.yt_dlp.YoutubeDL
    original_resolve_preview = service._resolve_preview_url
    service.yt_dlp.YoutubeDL = FakeYdl
    service._resolve_preview_url = fail_if_called
    try:
        parsed = await service.BilibiliService().parse(
            "https://www.bilibili.com/video/BV1DyfiBNE1q"
        )
    finally:
        service.yt_dlp.YoutubeDL = original_youtube_dl
        service._resolve_preview_url = original_resolve_preview

    assert parsed["item"]["platformItemId"] == "BV1DyfiBNE1q"
    assert parsed["item"]["previewUrl"]


async def test_resolve_preview_url_times_out_to_bvid_fallback() -> None:
    class SlowVideo:
        def __init__(self, *, bvid: str):
            self.bvid = bvid

        async def get_info(self) -> dict[str, Any]:
            await asyncio.sleep(0.05)
            return {"aid": 1, "pages": [{"cid": 2}]}

    original_video = service.video.Video
    original_timeout = service.BILIBILI_PREVIEW_TIMEOUT_SECONDS
    service.video.Video = SlowVideo
    service.BILIBILI_PREVIEW_TIMEOUT_SECONDS = 0.001
    try:
        preview_url = await service._resolve_preview_url("BV1DyfiBNE1q")
    finally:
        service.video.Video = original_video
        service.BILIBILI_PREVIEW_TIMEOUT_SECONDS = original_timeout

    assert preview_url == service.bilibili_preview_url("BV1DyfiBNE1q")
