"""Interface for the pacman database."""

from __future__ import annotations

import logging
import sys
from pathlib import Path

try:
    import pyalpm  # pyright: ignore[reportMissingImports]
except ModuleNotFoundError:
    logging.warning("pyalpm not found")
    pyalpm = None

from .constants import COMMENT, EXIT_ERROR
from .package import Package


class DB:
    """Interface for the pacman DB, wraps pyalpm."""

    _ROOT = Path("/")
    _DB_PATH_KEY = "DBPath"
    _DEFAULT_PATH = Path("/var/lib/pacman")

    def __init__(self):
        """Initialize with defaults."""
        # Handle only takes strings, not Paths
        if pyalpm is None:
            return
        handle = pyalpm.Handle(str(self._ROOT), str(self._get_db_path()))
        self._db = handle.get_localdb()

    def get_explicitly_installed_packages(self) -> list[Package]:
        """Get all explicitly installed packages. Equivalent to `pacman -Qqe`."""
        if pyalpm is None:
            logging.error("pyalpm not installed")
            sys.exit(EXIT_ERROR)

        instances = [
            Package(pkg.name)  # pyright: ignore[reportUnknownArgumentType]
            for pkg in self._db.pkgcache
            if pkg.reason == pyalpm.PKG_REASON_EXPLICIT
        ]
        return instances

    def get_all_installed_packages(self) -> list[Package]:
        """Get all installed packages. Equivalent to `pacman -Qq`."""
        return [
            Package(pkg.name)  # pyright: ignore[reportUnknownArgumentType]
            for pkg in self._db.pkgcache
        ]

    @classmethod
    def _get_db_path(cls) -> Path:
        # the config may contain flags, which cannot be parsed by configparser
        lines = cls._get_lines_from_config(Path("/etc/pacman.conf"))
        for line in lines:
            if not cls._line_is_comment(line) and cls._DB_PATH_KEY in line:
                return cls._get_path_from_line(line)
        return cls._DEFAULT_PATH

    @staticmethod
    def _line_is_comment(line: str) -> bool:
        return line.strip().startswith(COMMENT)

    @staticmethod
    def _get_lines_from_config(path: Path) -> list[str]:
        try:
            with open(path) as fd:
                lines = fd.readlines()
        except (FileNotFoundError, IOError):
            logging.error(f"Could not parse {path}.")
            sys.exit(EXIT_ERROR)
        return lines

    @staticmethod
    def _get_path_from_line(line: str) -> Path:
        segments = line.split("=")
        try:
            value = Path(segments[1].strip())
        except IndexError:
            logging.error(f"Could not get the path from this line:\n{line}")
            sys.exit(EXIT_ERROR)
        return value
