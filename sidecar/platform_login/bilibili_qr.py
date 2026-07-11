"""Bilibili QR code login."""

from __future__ import annotations

import base64

from bilibili_api import Credential
from bilibili_api.login_v2 import QrCodeLogin, QrCodeLoginEvents

from platforms.cookies import cookies_dict_to_header

from .sessions import AuthLoginSession


def _credential_to_cookie_header(credential: Credential) -> str:
    cookies = credential.get_cookies()
    known = {"SESSDATA", "bili_jct", "buvid3", "buvid4", "sessdata"}
    picked = {
        key: value
        for key, value in cookies.items()
        if value and (key in known or key.lower() in {name.lower() for name in known})
    }
    header = cookies_dict_to_header(picked)
    if not header:
        raise ValueError("登录成功但未获取到有效 Cookie")
    return header


async def start_bilibili_qr_login(session: AuthLoginSession) -> None:
    qr = QrCodeLogin()
    await qr.generate_qrcode()
    picture = qr.get_qrcode_picture()
    session.internal = qr
    session.status = "pending"
    session.message = "请使用 B 站 App 扫描二维码"
    if picture is not None and getattr(picture, "content", None):
        session.qr_image_base64 = base64.b64encode(picture.content).decode("ascii")


async def poll_bilibili_qr_login(session: AuthLoginSession) -> None:
    qr = session.internal
    if not isinstance(qr, QrCodeLogin):
        session.status = "failed"
        session.message = "登录会话无效，请重新开始"
        return

    if session.status in {"completed", "failed", "expired"}:
        return

    try:
        event = await qr.check_state()
    except Exception as exc:  # noqa: BLE001
        session.status = "failed"
        session.message = f"B 站登录失败: {exc}"
        return

    if event == QrCodeLoginEvents.SCAN:
        session.status = "scanned"
        session.message = "已扫码，请在手机上确认登录"
        return

    if event == QrCodeLoginEvents.CONF:
        session.status = "confirmed"
        session.message = "已确认，正在完成登录…"
        return

    if event == QrCodeLoginEvents.TIMEOUT:
        session.status = "expired"
        session.message = "二维码已过期，请重新开始"
        return

    if event == QrCodeLoginEvents.DONE:
        try:
            credential = qr.get_credential()
            session.cookies = _credential_to_cookie_header(credential)
        except Exception as exc:  # noqa: BLE001
            session.status = "failed"
            session.message = f"B 站登录失败: {exc}"
            return
        session.status = "completed"
        session.message = "B 站登录成功"
        session.qr_image_base64 = None
        return

    session.status = "pending"
    session.message = "请使用 B 站 App 扫描二维码"
