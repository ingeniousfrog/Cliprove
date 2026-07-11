"""Cookie helpers shared by platform modules."""

from __future__ import annotations

import re
import tempfile
from pathlib import Path


def cookie_header_to_dict(cookie_header: str) -> dict[str, str]:
    cookies: dict[str, str] = {}
    for chunk in (cookie_header or "").split(";"):
        chunk = chunk.strip()
        if not chunk or "=" not in chunk:
            continue
        key, value = chunk.split("=", 1)
        cookies[key.strip()] = value.strip()
    return cookies


def write_netscape_cookie_file(cookie_header: str) -> Path | None:
    cookies = cookie_header_to_dict(cookie_header)
    if not cookies:
        return None

    temp = tempfile.NamedTemporaryFile(
        mode="w",
        suffix=".txt",
        delete=False,
        encoding="utf-8",
    )
    temp.write("# Netscape HTTP Cookie File\n")
    for key, value in cookies.items():
        temp.write(
            f".bilibili.com\tTRUE\t/\tFALSE\t0\t{key}\t{value}\n"
        )
    temp.close()
    return Path(temp.name)


def normalize_bilibili_url(url: str) -> str:
    text = url.strip()
    if re.fullmatch(r"BV[\w]+", text, flags=re.IGNORECASE):
        return f"https://www.bilibili.com/video/{text.upper()}"
    if re.fullmatch(r"av\d+", text, flags=re.IGNORECASE):
        return f"https://www.bilibili.com/video/{text.lower()}"
    if text.startswith("http://") or text.startswith("https://"):
        return text
    return f"https://{text}"
