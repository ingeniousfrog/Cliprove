"""Structured error prefixes consumed by the Rust task layer."""

from __future__ import annotations


def cliprove_error(code: str, message: str) -> RuntimeError:
    normalized = code.upper().replace("-", "_")
    return RuntimeError(f"CLIPROVE_{normalized}: {message}")


def ffmpeg_unavailable(message: str) -> RuntimeError:
    return cliprove_error("FFMPEG_UNAVAILABLE", message)


def auth_required(message: str) -> RuntimeError:
    return cliprove_error("AUTH_REQUIRED", message)


def auth_expired(message: str) -> RuntimeError:
    return cliprove_error("AUTH_EXPIRED", message)


def verification_required(message: str) -> RuntimeError:
    return cliprove_error("VERIFICATION_REQUIRED", message)


def map_exception(exc: Exception) -> RuntimeError:
    message = str(exc).strip() or exc.__class__.__name__
    lowered = message.lower()
    if "ffmpeg" in lowered:
        return ffmpeg_unavailable(message)
    if any(
        token in lowered
        for token in (
            "sessdata",
            "login",
            "sign in",
            "登录",
            "private video",
            "members only",
            "会员",
            "cookie",
        )
    ):
        if any(token in lowered for token in ("expired", "过期", "invalid", "失效")):
            return auth_expired(message)
        return auth_required(message)
    if any(token in lowered for token in ("captcha", "verify", "验证", "风控")):
        return verification_required(message)
    if isinstance(exc, RuntimeError):
        return exc
    return RuntimeError(message)
