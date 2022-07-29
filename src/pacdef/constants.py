from __future__ import annotations

from enum import Enum
from pathlib import Path

EXIT_SUCCESS = 0
EXIT_ERROR = 1
EXIT_ABORT = 2
EXIT_INTERRUPT = 130
INTERRUPT_ASCII_CODE = "\x03"
COMMENT = "#"
PARU = Path("/usr/bin/paru")
VERSION = "unknown"
NOTHING_TO_DO = "nothing to do"


class Action(Enum):
    """Enum of actions that can be provided as first argument to `pacdef`."""

    clean = "clean"
    edit = "edit"
    groups = "groups"
    import_ = "import"
    new = "new"
    remove = "remove"
    review = "review"
    search = "search"
    show = "show"
    sync = "sync"
    unmanaged = "unmanaged"
    version = "version"
