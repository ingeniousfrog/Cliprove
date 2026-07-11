"""Douyin browser login via Playwright."""

from __future__ import annotations

import asyncio
import time
from typing import Any

from platforms.cookies import cookies_dict_to_header
from platforms.douyin.bootstrap import ensure_engine_path
from platforms.douyin.constants import (
    DOUYIN_USER_AGENT,
    SEARCH_WARMUP_KEYWORD,
    SEARCH_WARMUP_URL,
)

from .sessions import AuthLoginSession

ensure_engine_path()

from core.api_client import DouyinAPIClient  # noqa: E402
from tools.cookie_fetcher import (  # noqa: E402
    DEFAULT_AUXILIARY_KEYS,
    DEFAULT_AUXILIARY_PREFIXES,
    SUGGESTED_KEYS,
    extract_ms_token_from_text,
    filter_cookies,
    try_extract_ms_token,
)
from utils.cookie_utils import sanitize_cookies  # noqa: E402

HOME_URL = "https://www.douyin.com/"
LOGIN_TIMEOUT_SEC = 300
POLL_INTERVAL_SEC = 2.5
SEARCH_PROBE_ATTEMPTS = 4

_BROWSER_ARGS = [
    "--disable-blink-features=AutomationControlled",
    "--no-first-run",
    "--no-default-browser-check",
]

_STEALTH_INIT_SCRIPT = """
Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
window.chrome = window.chrome || { runtime: {} };
"""

_LOGIN_BUTTON_SELECTORS = (
    'button:has-text("登录")',
    'p:has-text("登录")',
    'span:has-text("登录")',
    'div:has-text("登录")',
    'a:has-text("登录")',
    '[class*="login-button"]',
    '[id*="login"]',
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


def _merge_cookie_sets(full: dict[str, str], picked: dict[str, str]) -> dict[str, str]:
    merged = dict(picked)
    for key, value in full.items():
        if key in merged:
            continue
        if key in SUGGESTED_KEYS or key in DEFAULT_AUXILIARY_KEYS:
            merged[key] = value
            continue
        if any(key.startswith(prefix) for prefix in DEFAULT_AUXILIARY_PREFIXES):
            merged[key] = value
    return sanitize_cookies(merged)


async def _close_browser(browser: Any) -> None:
    try:
        if browser.is_connected():
            await browser.close()
    except Exception:  # noqa: BLE001
        pass


async def _launch_browser(playwright: Any) -> Any:
    for channel in ("chrome", "msedge", None):
        try:
            kwargs: dict[str, Any] = {
                "headless": False,
                "args": _BROWSER_ARGS,
                "ignore_default_args": ["--enable-automation"],
            }
            if channel:
                kwargs["channel"] = channel
            return await playwright.chromium.launch(**kwargs)
        except Exception:  # noqa: BLE001
            continue
    raise RuntimeError("无法启动浏览器，请安装 Chrome 或运行 playwright install chromium")


async def _open_login_flow(page: Any) -> bool:
    """Open Douyin homepage and try to surface the login UI."""
    await page.goto(HOME_URL, wait_until="domcontentloaded", timeout=60_000)
    await asyncio.sleep(1.5)

    for selector in _LOGIN_BUTTON_SELECTORS:
        try:
            target = page.locator(selector).first
            if await target.is_visible(timeout=1_500):
                await target.click(timeout=3_000)
                await asyncio.sleep(1)
                return True
        except Exception:  # noqa: BLE001
            continue

    return False


async def _warmup_search_credentials(page: Any) -> None:
    try:
        await page.goto(HOME_URL, wait_until="domcontentloaded", timeout=60_000)
        await asyncio.sleep(2)
        await page.goto(SEARCH_WARMUP_URL, wait_until="domcontentloaded", timeout=60_000)
        await asyncio.sleep(3)
        await page.mouse.wheel(0, 900)
        await asyncio.sleep(2)
        await page.mouse.wheel(0, 900)
        await asyncio.sleep(2)
    except Exception:  # noqa: BLE001
        pass


async def _collect_login_cookies(
    context: Any,
    page: Any,
    observed_cookie_headers: list[str],
    observed_mstokens: list[str],
) -> dict[str, str]:
    await _warmup_search_credentials(page)

    storage = await context.storage_state()
    full = _cookies_from_storage(storage)
    cookies = filter_cookies(full)
    cookies = _merge_cookie_sets(full, cookies)

    ms_token = await try_extract_ms_token(
        page,
        cookies,
        observed_cookie_headers,
        observed_mstokens,
    )
    if ms_token and not cookies.get("msToken"):
        cookies["msToken"] = ms_token

    return cookies


async def _probe_search(cookie_map: dict[str, str]) -> bool:
    try:
        async with DouyinAPIClient(
            cookie_map,
            user_agent=DOUYIN_USER_AGENT,
        ) as client:
            result = await client.search_aweme(SEARCH_WARMUP_KEYWORD, count=3)
            return bool(result.get("items"))
    except Exception:  # noqa: BLE001
        return False


async def _finalize_login_cookies(
    session: AuthLoginSession,
    context: Any,
    page: Any,
    observed_cookie_headers: list[str],
    observed_mstokens: list[str],
) -> dict[str, str] | None:
    for attempt in range(SEARCH_PROBE_ATTEMPTS):
        session.message = (
            f"登录成功，正在收集搜索凭证（{attempt + 1}/{SEARCH_PROBE_ATTEMPTS}）…"
            "请勿关闭浏览器"
        )
        cookies = await _collect_login_cookies(
            context,
            page,
            observed_cookie_headers,
            observed_mstokens,
        )
        if not cookies.get("sessionid"):
            return None

        if await _probe_search(cookies):
            return cookies

        await asyncio.sleep(3)

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
    session.message = "正在打开浏览器，请完成抖音登录（可扫码）"

    browser = None
    observed_cookie_headers: list[str] = []
    observed_mstokens: list[str] = []

    try:
        async with async_playwright() as playwright:
            browser = await _launch_browser(playwright)
            context = await browser.new_context(
                user_agent=DOUYIN_USER_AGENT,
                viewport={"width": 1280, "height": 800},
                locale="zh-CN",
            )
            await context.add_init_script(_STEALTH_INIT_SCRIPT)
            page = await context.new_page()

            def _on_request(request: Any) -> None:
                try:
                    headers = request.headers or {}
                    cookie_header = headers.get("cookie")
                    if cookie_header:
                        observed_cookie_headers.append(cookie_header)
                    url = request.url or ""
                    if "msToken=" in url:
                        token = extract_ms_token_from_text(url)
                        if token:
                            observed_mstokens.append(token)
                except Exception:  # noqa: BLE001
                    return

            page.on("request", _on_request)

            try:
                clicked = await _open_login_flow(page)
            except Exception:  # noqa: BLE001
                clicked = False

            if clicked:
                session.message = "已打开登录窗口，请在弹出的登录框中完成扫码"
            else:
                session.message = (
                    "已打开抖音首页，请点击右上角「登录」完成扫码；"
                    "登录后请等待首页加载完成"
                )

            deadline = time.time() + LOGIN_TIMEOUT_SEC
            while time.time() < deadline:
                if not browser.is_connected():
                    session.status = "failed"
                    session.message = "登录窗口已关闭，登录未完成"
                    return

                storage = await context.storage_state()
                cookies = _cookies_from_storage(storage)
                if cookies.get("sessionid"):
                    cookies = await _finalize_login_cookies(
                        session,
                        context,
                        page,
                        observed_cookie_headers,
                        observed_mstokens,
                    )
                    if not cookies or not cookies.get("sessionid"):
                        session.status = "failed"
                        session.message = "登录完成但未获取到有效 Cookie，请重试"
                        return

                    session.cookies = cookies_dict_to_header(cookies)
                    session.status = "completed"
                    if await _probe_search(cookies):
                        session.message = "抖音登录成功，搜索凭证已就绪"
                    else:
                        session.message = (
                            "抖音登录成功，但搜索凭证可能仍受限。"
                            "请稍后在设置中点击「验证登录状态」重试"
                        )
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
