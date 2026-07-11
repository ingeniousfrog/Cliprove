"""Douyin browser login via Playwright."""

from __future__ import annotations

import asyncio
import time
from typing import Any

from platforms.cookies import cookies_dict_to_header

from .sessions import AuthLoginSession

LOGIN_URL = "https://www.douyin.com/"
LOGIN_TIMEOUT_SEC = 300
POLL_INTERVAL_SEC = 2.0
SUGGESTED_KEYS = {
    "msToken",
    "ttwid",
    "odin_tt",
    "passport_csrf_token",
    "sid_guard",
    "sessionid",
    "sid_tt",
}


def _filter_cookies(cookies: dict[str, str]) -> dict[str, str]:
    picked = {key: value for key, value in cookies.items() if key in SUGGESTED_KEYS and value}
    return picked or cookies


def _cookies_from_storage(storage: dict[str, Any]) -> dict[str, str]:
    cookies: dict[str, str] = {}
    for cookie in storage.get("cookies") or []:
        if not isinstance(cookie, dict):
            continue
        domain = str(cookie.get("domain") or "")
        if not domain.endswith("douyin.com"):
            continue
        name = str(cookie.get("name") or "").strip()
        value = str(cookie.get("value") or "").strip()
        if name and value:
            cookies[name] = value
    return cookies


async def _run_douyin_browser_login(session: AuthLoginSession) -> None:
    try:
        from playwright.async_api import async_playwright  # type: ignore
    except ImportError:
        session.status = "failed"
        session.message = (
            "未安装 Playwright。请在 sidecar 虚拟环境中运行："
            "pip install playwright && playwright install chromium"
        )
        return

    session.status = "pending"
    session.message = "已打开浏览器窗口，请完成抖音登录（可扫码）"

    try:
        async with async_playwright() as playwright:
            browser = await playwright.chromium.launch(headless=False)
            context = await browser.new_context()
            page = await context.new_page()
            try:
                await page.goto(LOGIN_URL, wait_until="domcontentloaded", timeout=120_000)
            except Exception:
                # Navigation can time out while the page keeps loading assets.
                pass

            deadline = time.time() + LOGIN_TIMEOUT_SEC
            while time.time() < deadline:
                if not browser.is_connected():
                    session.status = "failed"
                    session.message = "浏览器窗口已关闭，登录未完成"
                    return

                storage = await context.storage_state()
                cookies = _filter_cookies(_cookies_from_storage(storage))
                if cookies.get("sessionid"):
                    session.cookies = cookies_dict_to_header(cookies)
                    session.status = "completed"
                    session.message = "抖音登录成功"
                    return

                await asyncio.sleep(POLL_INTERVAL_SEC)

            session.status = "failed"
            session.message = "登录超时，请重试"
    except Exception as exc:  # noqa: BLE001
        session.status = "failed"
        session.message = f"抖音登录失败: {exc}"


def start_douyin_browser_login(session: AuthLoginSession) -> None:
    asyncio.create_task(_run_douyin_browser_login(session))
