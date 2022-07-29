# pyright: reportUnusedCallResult=none

from __future__ import annotations

import argparse
import logging
import sys
from pathlib import Path

from .constants import EXIT_ERROR, Action
from .package import Package
from .path import file_exists


class Arguments:
    """Class providing the command line arguments."""

    def __init__(self, process_args: bool = True):
        """Set up the argument parser, parse the args, collect the results in attributes."""
        if process_args:
            parser = self._setup_parser()
            args = parser.parse_args()
            self.action: Action | None = self._parse_action(args)
            self.files: list[Path] | None = self._parse_files(args)
            self.groups: list[str] | None = self._parse_groups(args)
            self.package: Package | None = self._parse_package(args)
            self.edit_new: bool = self._parse_edit_new(args)
        else:
            self.action = None
            self.files = None
            self.groups = None
            self.package = None
            self.edit_new = False

    @staticmethod
    def _parse_package(args: argparse.Namespace) -> Package | None:
        if not hasattr(args, "package"):
            return None
        return Package(args.package)

    # noinspection PyTypeChecker
    @staticmethod
    def _setup_parser() -> argparse.ArgumentParser:
        parser = argparse.ArgumentParser(
            description="declarative package manager for Arch Linux"
        )
        subparsers = parser.add_subparsers(
            dest="action", required=True, metavar="<action>"
        )
        subparsers.add_parser(
            Action.clean.value, help="uninstall packages not managed by pacdef"
        )
        parser_edit = subparsers.add_parser(
            Action.edit.value, help="edit one or more existing group files"
        )
        parser_edit.add_argument("group", nargs="+", help="a group file")
        subparsers.add_parser(Action.groups.value, help="show names of imported groups")
        parser_import = subparsers.add_parser(
            Action.import_.value, help="import a new group file"
        )
        parser_import.add_argument("file", nargs="+", help="a group file")
        parser_new = subparsers.add_parser(
            Action.new.value, help="create a new group file"
        )
        parser_new.add_argument("group", nargs="+", help="one or more new groups")
        parser_new.add_argument(
            "-e",
            "--edit",
            action="store_true",
            help="edit the new group file immediately",
        )
        parser_remove = subparsers.add_parser(
            Action.remove.value, help="remove previously imported group"
        )
        parser_remove.add_argument(
            "group", nargs="+", help="a previously imported group"
        )
        subparsers.add_parser(Action.review.value, help="review unmanaged packages")
        parser_search = subparsers.add_parser(
            Action.search.value, help="show the group containing a package"
        )
        parser_search.add_argument(
            "package", help="the package to search for (as string literal, or regex)"
        )
        parser_show_group = subparsers.add_parser(
            Action.show.value, help="show packages under an imported group"
        )
        parser_show_group.add_argument(
            "group", nargs="+", help="a previously imported group"
        )
        subparsers.add_parser(
            Action.sync.value, help="install packages from all imported groups"
        )
        subparsers.add_parser(
            Action.unmanaged.value,
            help="show explicitly installed packages not managed by pacdef",
        )
        subparsers.add_parser(Action.version.value, help="show version info")
        return parser

    @staticmethod
    def _parse_files(args: argparse.Namespace) -> list[Path] | None:
        if not hasattr(args, "file"):
            return None
        files = [Path(f) for f in args.file]
        for f in files:
            if not file_exists(f):
                logging.error(
                    f"Cannot handle '{f}'. "
                    + f"Check that it exists and if it is a file."
                )
                sys.exit(EXIT_ERROR)
        return files

    @staticmethod
    def _parse_action(args: argparse.Namespace) -> Action:
        for _, action in Action.__members__.items():
            if action.value == args.action:
                return action
        logging.error("Did not understand what you want me to do")
        sys.exit(EXIT_ERROR)

    @staticmethod
    def _parse_groups(args: argparse.Namespace) -> list[str] | None:
        if hasattr(args, "group"):
            return args.group
        return None

    @staticmethod
    def _parse_edit_new(args: argparse.Namespace) -> bool:
        if hasattr(args, "edit"):
            return args.edit
        return False
