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
from pathlib import Path

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
