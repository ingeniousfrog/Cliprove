"""In-memory login session tracking."""

from __future__ import annotations

import time
import uuid
from dataclasses import dataclass, field
from typing import Any


@dataclass
class AuthLoginSession:
    session_id: str
    platform: str
    status: str = "pending"
    message: str | None = None
    qr_image_base64: str | None = None
    cookies: str | None = None
    created_at: float = field(default_factory=time.time)
    internal: Any = field(default=None, repr=False)

    def to_dict(self) -> dict[str, Any]:
        return {
            "sessionId": self.session_id,
            "platform": self.platform,
            "status": self.status,
            "message": self.message,
            "qrImageBase64": self.qr_image_base64,
            "cookies": self.cookies,
        }


class AuthSessionManager:
    def __init__(self) -> None:
        self._sessions: dict[str, AuthLoginSession] = {}

    def create(self, platform: str) -> AuthLoginSession:
        session = AuthLoginSession(
            session_id=str(uuid.uuid4()),
            platform=platform,
        )
        self._sessions[session.session_id] = session
        self._prune()
        return session

    def get(self, session_id: str) -> AuthLoginSession | None:
        return self._sessions.get(session_id)

    def remove(self, session_id: str) -> None:
        self._sessions.pop(session_id, None)

    def _prune(self, max_sessions: int = 50) -> None:
        if len(self._sessions) <= max_sessions:
            return
        terminal = [
            session_id
            for session_id, session in self._sessions.items()
            if session.status in {"completed", "failed", "expired"}
        ]
        for session_id in terminal[: len(self._sessions) - max_sessions]:
            self._sessions.pop(session_id, None)


auth_session_manager = AuthSessionManager()
