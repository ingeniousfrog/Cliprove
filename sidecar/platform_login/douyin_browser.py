"""Douyin browser login via Playwright."""

from __future__ import annotations

import asyncio
import time
from typing import Any

from platforms.cookies import cookies_dict_to_header
from platforms.douyin.bootstrap import ensure_engine_path

from .sessions import AuthLoginSession

ensure_engine_path()

from tools.cookie_fetcher import filter_cookies, try_extract_ms_token  # noqa: E402
from utils.cookie_utils import sanitize_cookies  # noqa: E402

LOGIN_URL = "https://www.douyin.com/passport/web/login/"
HOME_URL = "https://www.douyin.com/"
LOGIN_TIMEOUT_SEC = 300
POLL_INTERVAL_SEC = 2.5

_BROWSER_ARGS = [
    "--disable-blink-features=AutomationControlled",
    "--no-first-run",
    "--no-default-browser-check",
]

_USER_AGENT = (
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
    "AppleWebKit/537.36 (KHTML, like Gecko) "
    "Chrome/131.0.0.0 Safari/537.36"
)


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
    return sanitize_cookies(cookies)


async def _close_browser(browser: Any) -> None:
    try:
        if browser.is_connected():
            await browser.close()
    except Exception:  # noqa: BLE001
        pass


async def _collect_login_cookies(
    context: Any,
    page: Any,
    observed_cookie_headers: list[str],
    observed_mstokens: list[str],
) -> dict[str, str]:
    try:
        await page.goto(HOME_URL, wait_until="domcontentloaded", timeout=60_000)
    except Exception:  # noqa: BLE001
        pass

    await asyncio.sleep(2.5)

    storage = await context.storage_state()
    cookies = _cookies_from_storage(storage)
    ms_token = await try_extract_ms_token(
        page,
        cookies,
        observed_cookie_headers,
        observed_mstokens,
    )
    if ms_token and not cookies.get("msToken"):
        cookies["msToken"] = ms_token

    return filter_cookies(cookies)


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
    session.message = "正在打开浏览器，请完成抖音登录（可扫码）"

    browser = None
    observed_cookie_headers: list[str] = []
    observed_mstokens: list[str] = []

    try:
        async with async_playwright() as playwright:
            browser = await playwright.chromium.launch(
                headless=False,
                args=_BROWSER_ARGS,
                ignore_default_args=["--enable-automation"],
            )
            context = await browser.new_context(
                user_agent=_USER_AGENT,
                viewport={"width": 1280, "height": 800},
                locale="zh-CN",
            )
            page = await context.new_page()

            def _on_request(request: Any) -> None:
                try:
                    headers = request.headers or {}
                    cookie_header = headers.get("cookie")
                    if cookie_header:
                        observed_cookie_headers.append(cookie_header)
                    url = request.url or ""
                    if "msToken=" in url:
                        from tools.cookie_fetcher import extract_ms_token_from_text

                        token = extract_ms_token_from_text(url)
                        if token:
                            observed_mstokens.append(token)
                except Exception:  # noqa: BLE001
                    return

            page.on("request", _on_request)

            try:
                await page.goto(LOGIN_URL, wait_until="domcontentloaded", timeout=60_000)
            except Exception:  # noqa: BLE001
                pass

            session.message = "已打开登录窗口，请完成抖音登录（可扫码）"

            deadline = time.time() + LOGIN_TIMEOUT_SEC
            while time.time() < deadline:
                if not browser.is_connected():
                    session.status = "failed"
                    session.message = "登录窗口已关闭，登录未完成"
                    return

                storage = await context.storage_state()
                cookies = _cookies_from_storage(storage)
                if cookies.get("sessionid"):
                    session.message = "登录成功，正在收集搜索所需凭证…"
                    cookies = await _collect_login_cookies(
                        context,
                        page,
                        observed_cookie_headers,
                        observed_mstokens,
                    )
                    if not cookies.get("sessionid"):
                        session.status = "failed"
                        session.message = "登录完成但未获取到有效 Cookie，请重试"
                        return

                    session.cookies = cookies_dict_to_header(cookies)
                    session.status = "completed"
                    session.message = "抖音登录成功"
                    await _close_browser(browser)
                    browser = None
                    return

                await asyncio.sleep(POLL_INTERVAL_SEC)

            session.status = "failed"
            session.message = "登录超时，请重试"
    except Exception as exc:  # noqa: BLE001
        session.status = "failed"
        session.message = f"抖音登录失败: {exc}"
    finally:
        if browser is not None:
            await _close_browser(browser)


def start_douyin_browser_login(session: AuthLoginSession) -> None:
    asyncio.create_task(_run_douyin_browser_login(session))
