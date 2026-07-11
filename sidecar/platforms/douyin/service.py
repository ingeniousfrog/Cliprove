"""Douyin platform service for Cliprove sidecar."""

from __future__ import annotations

from typing import Any

from .bootstrap import ensure_engine_path
from .constants import DOUYIN_USER_AGENT

ensure_engine_path()

from core.api_client import DouyinAPIClient, LoginRequiredError  # noqa: E402
from core.url_parser import URLParser  # noqa: E402
from utils.validators import is_short_url, normalize_short_url, parse_url_type  # noqa: E402

from .downloader import cookies_dict, download_aweme
from .constants import DOUYIN_USER_AGENT
from .mapper import aweme_to_media_item, aweme_to_parsed_media


def _parse_filter_int(filters: dict[str, Any] | None, key: str, mapping: dict[str, int], default: int) -> int:
    if not filters:
        return default
    raw = filters.get(key)
    if raw is None:
        return default
    if isinstance(raw, int):
        return raw
    text = str(raw)
    if text.isdigit():
        return int(text)
    return mapping.get(text, default)


class DouyinService:
    async def parse(self, url: str, cookies: str = "", proxy: str = "") -> dict[str, Any]:
        cookie_map = cookies_dict(cookies)
        resolved_url = url.strip()

        async with DouyinAPIClient(
            cookie_map,
            proxy=proxy or None,
            user_agent=DOUYIN_USER_AGENT,
        ) as api_client:
            if is_short_url(resolved_url):
                resolved = await api_client.resolve_short_url(normalize_short_url(resolved_url))
                if not resolved:
                    raise ValueError("短链解析失败，请检查链接是否有效")
                resolved_url = resolved

            url_type = parse_url_type(resolved_url)
            if url_type not in ("video", "gallery"):
                raise ValueError(f"暂不支持的内容类型: {url_type or 'unknown'}")

            parsed = URLParser.parse(resolved_url)
            if not parsed:
                raise ValueError("无法解析链接")

            aweme_id = parsed.get("aweme_id") or parsed.get("note_id")
            if not aweme_id:
                raise ValueError("未找到作品 ID")

            try:
                aweme = await api_client.get_video_detail(str(aweme_id))
            except LoginRequiredError as exc:
                raise ValueError("需要登录或 Cookie 已失效，请在设置中更新抖音 Cookie") from exc

            if not aweme:
                raise ValueError("作品不存在、已下架或当前 Cookie 无权访问")

            return aweme_to_parsed_media(aweme, url.strip())

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
        cookie_map = cookies_dict(cookies)
        offset = int(cursor) if cursor else 0
        page_size = max(1, min(page_size, 50))

        sort_type = _parse_filter_int(
            filters,
            "sort",
            {"general": 0, "likes": 1, "latest": 2},
            0,
        )
        publish_time = _parse_filter_int(
            filters,
            "publish_time",
            {"all": 0, "day": 1, "week": 7, "half_year": 182},
            0,
        )

        async with DouyinAPIClient(
            cookie_map,
            proxy=proxy or None,
            user_agent=DOUYIN_USER_AGENT,
        ) as api_client:
            try:
                page = await api_client.search_aweme(
                    keyword,
                    offset=offset,
                    count=page_size,
                    sort_type=sort_type,
                    publish_time=publish_time,
                )
            except LoginRequiredError as exc:
                raise ValueError("需要登录或 Cookie 已失效，请在设置中更新抖音 Cookie") from exc

            items = [
                aweme_to_media_item(aweme, search_keyword=keyword)
                for aweme in page.get("items") or []
                if isinstance(aweme, dict)
            ]
            has_more = bool(page.get("has_more"))
            next_cursor = str(page.get("max_cursor") or (offset + len(items)))

            if not items and offset == 0:
                raw = page.get("raw") if isinstance(page.get("raw"), dict) else {}
                status_code = int(page.get("status_code") or raw.get("status_code") or 0)
                status_msg = str(raw.get("status_msg") or "").strip()
                if status_code in (2483,) or "请先登录" in status_msg:
                    raise ValueError("需要登录或 Cookie 已失效，请在设置中更新抖音 Cookie")
                if not cookie_map:
                    raise ValueError("搜索需要配置抖音 Cookie，请先在设置中完成登录")
                detail = status_msg or "平台未返回搜索结果"
                raise ValueError(
                    "搜索未返回结果。"
                    f"{detail}。"
                    "抖音可能要求答题、滑块等平台验证；请在设置中点击「重新登录」，"
                    "在打开的浏览器内完成验证并等待搜索页加载后再重试"
                )

            return {
                "items": items,
                "cursor": next_cursor if has_more and items else None,
                "hasMore": has_more and bool(items),
                "supportedFilters": ["sort", "publish_time"],
            }

    async def download(
        self,
        *,
        platform_item_id: str,
        output_dir: str,
        asset_ids: list[str],
        cookies: str = "",
        proxy: str = "",
    ) -> dict[str, Any]:
        return await download_aweme(
            aweme_id=platform_item_id,
            output_dir=output_dir,
            asset_ids=asset_ids,
            cookies=cookies,
            proxy=proxy,
        )

    async def validate_auth(self, cookies: str = "", proxy: str = "") -> dict[str, Any]:
        cookie_map = cookies_dict(cookies)
        if not cookie_map:
            return {
                "platform": "douyin",
                "valid": False,
                "message": "未配置 Cookie",
            }

        async with DouyinAPIClient(
            cookie_map,
            proxy=proxy or None,
            user_agent=DOUYIN_USER_AGENT,
        ) as api_client:
            try:
                info = await api_client.get_self_info()
            except LoginRequiredError:
                return {
                    "platform": "douyin",
                    "valid": False,
                    "message": "Cookie 已失效，请重新获取",
                }
            except Exception as exc:  # noqa: BLE001
                return {
                    "platform": "douyin",
                    "valid": False,
                    "message": f"验证失败: {exc}",
                }

            nickname = None
            if info:
                nickname = info.get("nickname") or info.get("name") or "已登录用户"

            try:
                probe = await api_client.search_aweme("美食", count=3)
            except LoginRequiredError:
                return {
                    "platform": "douyin",
                    "valid": False,
                    "message": "Cookie 已失效，请重新登录",
                }
            except Exception as exc:  # noqa: BLE001
                return {
                    "platform": "douyin",
                    "valid": False,
                    "message": f"搜索接口验证失败: {exc}",
                }

            items = probe.get("items") or []
            if items:
                label = nickname or "已登录用户"
                return {
                    "platform": "douyin",
                    "valid": True,
                    "message": f"Cookie 有效（{label}），搜索可用",
                }

            raw = probe.get("raw") if isinstance(probe.get("raw"), dict) else {}
            status_msg = str(raw.get("status_msg") or "").strip()
            detail = status_msg or "搜索接口未返回结果"
            label = nickname or "已登录用户"
            return {
                "platform": "douyin",
                "valid": False,
                "message": (
                    f"已登录（{label}），但抖音搜索仍受平台验证限制：{detail}。"
                    "请点击「重新登录」，在浏览器完成答题、滑块等验证，并等待搜索页加载出结果"
                ),
            }


douyin_service = DouyinService()
