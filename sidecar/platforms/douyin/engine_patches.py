"""Runtime patches for vendored douyin-downloader without forking upstream."""

from __future__ import annotations

from typing import Any


def apply_douyin_engine_patches() -> None:
    from auth import MsTokenManager
    from core.api_client import DouyinAPIClient
    from utils.xbogus import XBogus

    if getattr(DouyinAPIClient, "_cliprove_patched", False):
        return

    original_init = DouyinAPIClient.__init__
    original_search = DouyinAPIClient.search_aweme

    def patched_init(
        self: DouyinAPIClient,
        cookies: dict[str, str],
        proxy: str | None = None,
        user_agent: str | None = None,
        **kwargs: Any,
    ) -> None:
        original_init(self, cookies, proxy, **kwargs)
        selected = (user_agent or "").strip()
        if not selected:
            return
        self.headers["User-Agent"] = selected
        self._signer = XBogus(selected)
        self._ms_token_manager = MsTokenManager(user_agent=selected)

    async def patched_search_aweme(self: DouyinAPIClient, keyword: str, *args: Any, **kwargs: Any):
        old_referer = self.headers.get("Referer")
        self.headers["Referer"] = f"https://www.douyin.com/search/{keyword}?type=general"
        try:
            return await original_search(self, keyword, *args, **kwargs)
        finally:
            if old_referer is not None:
                self.headers["Referer"] = old_referer

    DouyinAPIClient.__init__ = patched_init  # type: ignore[method-assign]
    DouyinAPIClient.search_aweme = patched_search_aweme  # type: ignore[method-assign]
    DouyinAPIClient._cliprove_patched = True  # type: ignore[attr-defined]
