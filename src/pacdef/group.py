from __future__ import annotations

import logging
import sys
from pathlib import Path

from .constants import COMMENT, EXIT_ERROR
from .package import Package


class Group:
    """Class representing a group file."""

    def __init__(self, packages: list[Package], path: Path):
        """Initialize Group instance. Consider Group.from_file where applicable."""
        self.packages = packages
        self._path: Path = path

    @property
    def name(self) -> str:
        """Return the name of the group."""
        return self._path.name

    @property
    def path(self) -> Path:
        """Return path of the group."""
        return self._path

    def __contains__(self, item: Package):
        """Check if package exists in group."""
        return item in self.packages

    def __getitem__(self, item: int):
        """Get the package at index `item`."""
        return self.packages[item]

    def __len__(self):
        """Get number of packages."""
        return len(self.packages)

    # noinspection PyUnresolvedReferences
    def __eq__(self, other: object):
        """Compare with other groups or strings."""
        match other:
            case Group():
                return self.name == other.name
            case str():
                return self.name == other
            case _:
                raise ValueError("Must be compared with Group or string.")

    @property
    def content(self) -> str:
        """Representation are the newline-separated names of the packages."""
        return "\n".join([package.name for package in self.packages])

    def __repr__(self):
        return f"group: {self.name}"

    @classmethod
    def from_file(cls, path: Path) -> Group:
        """Read a group file, return an instance of Group containing the packages.

        :param path: path to group file
        :return: instance of Group
        """
        cls._sanity_check(path)
        text = path.read_text()
        lines = text.split("\n")
        packages = []
        for line in lines:
            package = cls._get_package_from_line(line)
            if package is not None:
                packages.append(package)
        instance = cls(packages, path)
        return instance

    @staticmethod
    def _get_package_from_line(line: str) -> Package | None:
        """Get package from a line of a group file.

        Ignores everything after a `#` character.

        :param line: a single line of a group file
        :return: instance of Package when string contained a package, otherwise None.
        """
        before_comment = line.split(COMMENT)[0]
        package_name = before_comment.strip()
        if len(package_name) > 0:
            return Package(package_name)
        return None

    def remove(self):
        """Delete the symlink under the group path."""
        logging.info(f"removing group {self.name}")
        self._path.unlink()

    @staticmethod
    def _sanity_check(group: Path) -> None:
        """Sanity check an imported group file.

        Checks for broken symlinks, directories and actual files (instead of symlinks). Prints a warning if a
        check fails.

        :param group: path to a group to be imported
        """

        def check_dir():
            if group.is_dir():
                logging.warning(f"found directory {group} instead of group file")

        def check_broken_symlink():
            if group.is_symlink() and not group.exists():
                logging.warning(f"found group {group}, but it is a broken symlink")

        check_dir()
        check_broken_symlink()

    def append(self, package: Package):
        """Add package to group, by memorizing it and appending it to the group file."""
        self.packages.append(package)
        self.packages.sort()
        with open(self.path, "a") as fd:
            output = f"{package}\n"
            bytes_written = fd.write(output)
            if not len(output) == bytes_written:
                logging.warning(
                    f"It seems we could not write everything. Check the group file {self.path}"
                )

    @classmethod
    def new_file(cls, name: str, path: Path) -> None:
        """Create new group file in `path` with `name`."""
        (path / name).touch()

    @classmethod
    def read_groups_from_dir(cls, groups_path: Path) -> list[Group]:
        """Read all imported groups (= list of files in the pacdef group directory).

        :return: list of imported groups
        """
        paths = [group for group in groups_path.iterdir()]
        paths.sort()
        groups = []
        for path in paths:
            # noinspection PyBroadException
            try:
                groups.append(Group.from_file(path))
            except Exception as e:
                logging.error(f"Could not parse group file {path}.")
                logging.error(e)
                print(sys.exit(EXIT_ERROR))
        logging.debug(f"groups: {[group.name for group in groups]}")
        return groups
