from __future__ import annotations

import logging
import sys
from pathlib import Path

from pacdef.cmd import CommandRunner
from pacdef.config import Config
from pacdef.constants import EXIT_ERROR
from pacdef.package import Package
from pacdef.path import file_exists


class AURHelper:
    """Abstraction of AUR helpers that act as pacman wrappers."""

    class _Switches:
        """CLI switches for AUR helpers that wrap pacman."""

        install = ["--sync", "--refresh", "--needed"]
        remove = ["--remove", "--recursive"]
        installed_package_info = ["--query", "--info"]
        # noinspection SpellCheckingInspection
        as_dependency = ["--database", "--asdeps"]

    def __repr__(self):
        """Representation for debugging purposes."""
        return str(self._path)

    def __init__(self, path: Path):
        """Initialize AURHelper.

        If the AUR helper is not found, and error is raised.
        :param path: path to the AUR helper to use (example: `/usr/bin/paru`).
        """
        if not path.is_absolute():
            path = Path("/usr/bin").joinpath(path)
        if not file_exists(path):
            logging.error(f"AUR helper {path} not found.")
            sys.exit(EXIT_ERROR)
        self._path = path
        logging.info(f"AUR helper: {self._path}")

    def _execute(self, command: list[str]) -> None:
        """Execute an AUR helper command without checking the output.

        :param command: the command to execute, list of strings.
        """
        try:
            CommandRunner.call([str(self._path)] + command)
        except FileNotFoundError:
            logging.error(f'Could not start the AUR helper "{self._path}".')
            sys.exit(EXIT_ERROR)

    def install(self, packages: list[Package]) -> None:
        """Install packages in the system.

        :param packages: list of packages to be installed.
        """
        packages_str = [str(p) for p in packages]
        command: list[str] = self._Switches.install + packages_str
        self._execute(command)

    def remove(self, packages: list[Package]) -> None:
        """Remove the packages from the system.

        :param packages: list of packages to be removed.
        """
        packages_str = [str(p) for p in packages]
        command: list[str] = self._Switches.remove + packages_str
        self._execute(command)

    @classmethod
    def from_config(cls, config: Config) -> AURHelper:
        """Create an AUR helper instance using `config.aur_helper`.

        :param config: an instance of Config
        :return: an instance of AURHelper
        """
        return cls(path=config.aur_helper)

    def print_info(self, package: Package) -> None:
        """Print info for an installed package."""
        self._execute(self._Switches.installed_package_info + [str(package)])

    def as_dependency(self, packages: list[Package]) -> None:
        """Mark packages as "installed as dependency"."""
        self._execute(
            self._Switches.as_dependency + [str(package) for package in packages]
        )
