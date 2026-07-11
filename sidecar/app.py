"""Cliprove Python sidecar — Phase 0 health endpoint."""

from __future__ import annotations

import argparse

try:
    from fastapi import FastAPI
    import uvicorn
except ImportError as exc:  # pragma: no cover
    raise SystemExit(
        "Missing sidecar dependencies. Run: pip install -r sidecar/requirements.txt"
    ) from exc

APP_VERSION = "0.1.0-phase0"

app = FastAPI(title="Cliprove Sidecar", version=APP_VERSION)


@app.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok", "version": APP_VERSION}


def main() -> None:
    parser = argparse.ArgumentParser(description="Cliprove Python sidecar")
    parser.add_argument("--port", type=int, default=18765)
    parser.add_argument("--host", default="127.0.0.1")
    args = parser.parse_args()
    uvicorn.run(app, host=args.host, port=args.port, log_level="warning")


if __name__ == "__main__":
    main()
