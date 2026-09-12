"""File-based logging so a user hitting an issue can send back one file.

The desktop GUI runs windowed (no console), so a crash there is otherwise
invisible to us. configure_logging() sends everything to a single rotating
log file under the user's home directory instead -- old activity rotates
into app.log.1/.2/.3 (kept for a bit more context), but the *current*
app.log always has the most recent run, so a user only needs to send that
one file.
"""

import logging
import logging.handlers
import platform
import sys
import threading
from pathlib import Path

import psutil

LOG_DIR = Path.home() / ".accountant_check" / "logs"
LOG_FILE = LOG_DIR / "app.log"

_configured = False


def configure_logging() -> Path:
    """Set up the rotating file handler. Safe to call more than once."""
    global _configured
    if _configured:
        return LOG_FILE

    LOG_DIR.mkdir(parents=True, exist_ok=True)
    handler = logging.handlers.RotatingFileHandler(
        LOG_FILE, maxBytes=1_000_000, backupCount=3, encoding="utf-8"
    )
    handler.setFormatter(logging.Formatter(
        "%(asctime)s %(levelname)s %(name)s: %(message)s"
    ))

    root = logging.getLogger()
    root.setLevel(logging.INFO)
    root.addHandler(handler)

    _configured = True
    logging.getLogger(__name__).info(
        "Logging started (python=%s, platform=%s)",
        sys.version.split()[0], platform.platform(),
    )
    return LOG_FILE


class ResourceHeartbeat:
    """Logs CPU/RAM usage on a background thread every `interval` seconds.

    Runs on its own thread so it keeps reporting even while the main thread
    is stuck in a long synchronous call (e.g. tkinter's mainloop is blocked
    for the whole duration of a button callback) -- if the app appears to
    hang, the log should still show whether resources were tight right
    before it stopped advancing. Use as a context manager around the
    processing call.
    """

    def __init__(self, logger: logging.Logger, interval: float = 5.0):
        self._logger = logger
        self._interval = interval
        self._stop = threading.Event()
        self._thread: threading.Thread | None = None

    def _log_once(self) -> None:
        cpu = psutil.cpu_percent(interval=None)
        mem = psutil.virtual_memory()
        self._logger.info(
            "Resource check: cpu=%.0f%% mem_available=%.0fMB mem_used=%.0f%%",
            cpu, mem.available / (1024 * 1024), mem.percent,
        )

    def _run(self) -> None:
        psutil.cpu_percent(interval=None)  # prime; first reading is meaningless
        while not self._stop.wait(self._interval):
            self._log_once()

    def __enter__(self) -> "ResourceHeartbeat":
        self._log_once()
        self._thread = threading.Thread(target=self._run, daemon=True)
        self._thread.start()
        return self

    def __exit__(self, *exc_info: object) -> None:
        self._stop.set()
        if self._thread is not None:
            self._thread.join(timeout=1)
