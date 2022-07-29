from __future__ import annotations

from pathlib import Path

from src.pacdef.constants import PARU

PACMAN = Path("/usr/bin/pacman")
PACMAN_EXISTS = PACMAN.exists()
PARU_EXISTS = PARU.exists()
REASON_NOT_ARCH = "pacman not found. That's not an Arch installation."
REASON_PARU_MISSING = "paru not found"
DEVNULL = Path("/dev/null")
