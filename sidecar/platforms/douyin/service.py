"""Douyin platform service for Cliprove sidecar."""

from __future__ import annotations

from typing import Any

from .bootstrap import ensure_engine_path

ensure_engine_path()

from core.api_client import DouyinAPIClient, LoginRequiredError  # noqa: E402
from core.url_parser import URLParser  # noqa: E402
from utils.validators import is_short_url, normalize_short_url, parse_url_type  # noqa: E402

from .downloader import cookies_dict, download_aweme
from .mapper import aweme_to_parsed_media


class DouyinService:
    async def parse(self, url: str, cookies: str = "", proxy: str = "") -> dict[str, Any]:
        cookie_map = cookies_dict(cookies)
        resolved_url = url.strip()

        async with DouyinAPIClient(cookie_map, proxy=proxy or None) as api_client:
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

        async with DouyinAPIClient(cookie_map, proxy=proxy or None) as api_client:
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

            if info:
                nickname = (info.get("nickname") or info.get("name") or "已登录用户")
                return {
                    "platform": "douyin",
                    "valid": True,
                    "message": f"Cookie 有效（{nickname}）",
                }

            return {
                "platform": "douyin",
                "valid": True,
                "message": "Cookie 格式有效（未能读取账号昵称）",
            }


douyin_service = DouyinService()
