"""PyInstaller runtime hook: point OpenSSL at the bundled certifi CA store.

Homebrew/macOS Python freezes often bake OpenSSL's default CA path to a
machine-local location (e.g. /opt/homebrew/etc/openssl@3/cert.pem). Inside the
onefile sidecar that path is useless, so HTTPS (Bilibili login, etc.) fails with
CERTIFICATE_VERIFY_FAILED even though certifi/cacert.pem is packaged.
"""

from __future__ import annotations

import os


def _configure() -> None:
    try:
        import certifi
    except Exception:
        return

    cafile = certifi.where()
    if not cafile or not os.path.isfile(cafile):
        return

    # Always override: a stale SSL_CERT_FILE pointing at a missing host path
    # would otherwise keep breaking verification in the frozen binary.
    os.environ["SSL_CERT_FILE"] = cafile
    os.environ["REQUESTS_CA_BUNDLE"] = cafile
    os.environ["CURL_CA_BUNDLE"] = cafile


_configure()
