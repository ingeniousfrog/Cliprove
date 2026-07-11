"""Cover image URL helpers."""

from __future__ import annotations

from urllib.parse import urlparse


def normalize_cover_url(url: str | None) -> str | None:
    if not url or not isinstance(url, str):
        return None
    text = url.strip()
    if not text:
        return None
    if text.startswith("//"):
        return f"https:{text}"
    if text.startswith("http://"):
        return f"https://{text.removeprefix('http://')}"
    if not text.startswith(("http://", "https://")):
        return f"https://{text}"
    return text


def proxy_referer(url: str, platform: str = "") -> str:
    lowered = url.lower()
    if platform == "bilibili" or "hdslb.com" in lowered or "bilibili.com" in lowered:
        return "https://www.bilibili.com/"
    if platform == "douyin" or "douyin" in lowered or "douyinpic.com" in lowered:
        return "https://www.douyin.com/"
    parsed = urlparse(url)
    if parsed.scheme and parsed.netloc:
        return f"{parsed.scheme}://{parsed.netloc}/"
    return "https://www.bilibili.com/"
