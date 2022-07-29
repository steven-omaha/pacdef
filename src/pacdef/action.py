from __future__ import annotations

from enum import Enum


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
