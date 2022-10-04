from __future__ import annotations

import logging
import re

from .constants import EXIT_ERROR


class Package:
    """Class that represents a single package."""

    def __init__(self, package_string: str):
        """Initialize an instance by a package string.

        :param package_string: The string describing the package. May contain a repository prefix followed by a `/`.
                               Examples: `zsh` or `repo/spotify`.
        """
        self.name: str
        self.repo: str | None
        self.name, self.repo = self._split_into_name_and_repo(package_string)
        if len(self.name) == 0:
            raise ValueError("invalid package name, is empty")

    # noinspection PyUnresolvedReferences
    def __eq__(self, other: object):
        """Check if equal to other package by comparing the name only."""
        match other:  # pyright: ignore[reportMatchNotExhaustive]
            case Package():
                return self.name == other.name
            case str():
                return self.name == other
        raise ValueError("Must be compared with Package or string.")

    def __hash__(self):
        return hash(self.name)

    def __lt__(self, other: object):
        if not isinstance(other, Package):
            raise NotImplementedError
        return self.name < other.name

    def __repr__(self):
        """Print `repo/package` if a repo was provided, otherwise print `package`."""
        if self.repo is not None:
            result = f"{self.repo}/{self.name}"
        else:
            result = self.name
        return result

    @staticmethod
    def _split_into_name_and_repo(package_string: str) -> tuple[str, str | None]:
        """Take a string in the form `repository/package`, return package and repository.

        Returns `(package_name, None)` if it does not contain a repository prefix.
        :param package_string: string of a single package, optionally starting with a repository prefix
        :return: package name, repository
        """
        if "/" in package_string:
            try:
                repo, name = package_string.split("/")
            except ValueError:  # too many values to unpack
                logging.error(
                    f"could not split this line into repo and package:\n{package_string}"
                )
                exit(EXIT_ERROR)
        else:
            repo = None
            name = package_string
        # noinspection PyUnboundLocalVariable
        return name, repo

    def matches_regex(self, regex: Package) -> bool:
        """Check whether the package's representation matches a regex."""
        return bool(re.search(repr(regex), repr(self)))
