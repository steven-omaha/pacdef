#!/usr/bin/python

"""
Declarative package manager for Arch Linux.

https://github.com/steven-omaha/pacdef
"""

from __future__ import annotations

import argparse
import configparser
import logging
import os
import subprocess
import sys
from enum import Enum
from os import environ
from pathlib import Path
from typing import Any, Callable

# does not exist on github workflow
try:
    import pyalpm  # type: ignore
except ModuleNotFoundError:
    logging.warning("pyalpm not found")
    pyalpm = None


EXIT_SUCCESS = 0
EXIT_ERROR = 1
EXIT_INTERRUPT = 130

COMMENT = "#"
PARU = Path("/usr/bin/paru")
VERSION = "unknown"

NOTHING_TO_DO = "nothing to do"


def _main():
    _setup_logger()
    args = Arguments()
    config = Config()
    helper = AURHelper.from_config(config)
    db = DB()
    pacdef = Pacdef(args=args, config=config, aur_helper=helper, db=db)
    pacdef.run_action_from_arg()


def _get_user_confirmation() -> None:
    """Ask the user if he wants to continue. Exits if the answer is not `y` or of length zero."""
    user_input = input("Continue? [Y/n] ").lower()
    if user_input == "" or user_input == "y":
        return
    else:
        sys.exit(EXIT_SUCCESS)


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
        else:
            self.action = None
            self.files = None
            self.groups = None
            self.package = None

    @staticmethod
    def _parse_package(args: argparse.Namespace) -> Package | None:
        if not hasattr(args, "package"):
            return None
        return Package(args.package)

    @staticmethod
    def _setup_parser() -> argparse.ArgumentParser:
        parser = argparse.ArgumentParser(
            description="a declarative manager of Arch packages"
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
        parser_search.add_argument("package", help="the package to search for")
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
            if not _file_exists(f):
                logging.error(
                    f"Cannot handle '{f}'. "
                    f"Check that it exists and if it is a file."
                )
                sys.exit(EXIT_ERROR)
        return files

    @staticmethod
    def _parse_action(args: argparse.Namespace) -> Action:
        for _, action in Action.__members__.items():
            if action.value == args.action:
                return action
        else:
            logging.error("Did not understand what you want me to do")
            sys.exit(EXIT_ERROR)

    @staticmethod
    def _parse_groups(args: argparse.Namespace) -> list[str] | None:
        if not hasattr(args, "group"):
            return None
        return args.group


def _dir_exists(path: Path) -> bool:
    return path.exists() and path.is_dir()


def _file_exists(path: Path) -> bool:
    return path.exists() and path.is_file()


class Action(Enum):
    """Enum of actions that can be provided as first argument to `pacdef`."""

    clean = "clean"
    edit = "edit"
    groups = "groups"
    import_ = "import"
    remove = "remove"
    review = "review"
    search = "search"
    show = "show"
    sync = "sync"
    unmanaged = "unmanaged"
    version = "version"


class Config:
    """Class reading and holding the runtime configuration."""

    def __init__(
        self,
        groups_path: Path = None,
        aur_helper: Path = None,
        config_file_path: Path = None,
        editor: Path = None,
    ):
        """Instantiate using the provided values. If these are None, use the config file / defaults."""
        config_base_dir = self._get_xdg_config_home()
        pacdef_path = config_base_dir.joinpath("pacdef")
        config_file_path = config_file_path or pacdef_path.joinpath("pacdef.conf")

        self._config = self._read_config_file(config_file_path)
        self.groups_path: Path = groups_path or pacdef_path.joinpath("groups")
        logging.info(f"{self.groups_path=}")
        self._create_config_files_and_dirs(config_file_path, pacdef_path)

        self.aur_helper: Path = aur_helper or self._get_aur_helper()
        self._editor: Path | None = editor or self._get_editor()
        logging.info(f"{self.aur_helper=}")
        self._sanity_check()

    def _create_config_files_and_dirs(self, config_file_path, pacdef_path):
        """Create config files and dirs. Will be executed on first-time run of pacdef."""
        if not _dir_exists(pacdef_path):
            pacdef_path.mkdir(parents=True)
        if not _dir_exists(self.groups_path):
            self.groups_path.mkdir()
        if not _file_exists(config_file_path):
            config_file_path.touch()

    @property
    def editor(self) -> Path:
        """Get the editor. Error out if none is found."""
        if self._editor is None:
            msg = (
                "I do not know which editor to use.\n"
                "  Either set the environment variables EDITOR or VISUAL, or set\n"
                "  editor in the [misc] section in pacdef.conf."
            )
            logging.error(msg)
            sys.exit(EXIT_ERROR)
        return self._editor

    @staticmethod
    def _get_xdg_config_home() -> Path:
        try:
            config_base_dir = Path(environ["XDG_CONFIG_HOME"])
        except KeyError:
            config_base_dir = Path.home() / ".config"
        logging.debug(f"{config_base_dir=}")
        return config_base_dir

    @staticmethod
    def _read_config_file(config_file: Path) -> configparser.ConfigParser:
        config = configparser.ConfigParser()
        try:
            config.read(config_file)
        except configparser.ParsingError as e:
            logging.error(f"Could not parse the config: {e}")
        return config

    def _get_value_from_conf(
        self, section: str, key: str, warn_missing: bool = False
    ) -> str | None:
        try:
            result = self._config[section][key]
        except KeyError:
            if warn_missing:
                logging.warning(f"{key} in section [{section}] not set")
            result = None
        return result

    def _get_editor(self) -> Path | None:
        editor = self._get_value_from_conf("misc", "editor")
        if editor is not None:
            return Path(editor)
        editor = self._get_value_from_env_variables(["EDITOR", "VISUAL"])
        if editor is not None:
            return Path(editor)
        return None

    @classmethod
    def _get_value_from_env_variables(
        cls, variables: list[str], warn_missing: bool = False
    ) -> str | None:
        """Return the value of the first existing environment variable.

        For a list of environment variables, check in the provided order that they exist.
        Return the value of the first existing environment variable.

        :param variables: list of environment variables to read
        :param warn_missing: print a warning if none of the elements in `variables` are found
        :return: value of environment variable or None
        """
        for var in variables:
            result = cls._get_value_from_env(var, warn_missing)
            if result is not None:
                return result
        else:
            return None

    @staticmethod
    def _get_value_from_env(variable: str, warn_missing: bool = False) -> str | None:
        """Get the value of a single environment variable.

        :param variable: environment variable to read
        :param warn_missing: print a warning if the variable is not set
        :return: value of environment variable or None
        """
        try:
            result = os.environ[variable]
        except KeyError:
            if warn_missing:
                logging.warning(f"Environment variable {variable} not set.")
            return None
        logging.info(f"{variable} is set to {result}.")
        return result

    def _get_aur_helper(self) -> Path:
        aur_helper = self._get_value_from_conf("misc", "aur_helper", True)
        if aur_helper is None:
            logging.warning(f"No AUR helper set. Defaulting to {PARU}")
            return PARU
        else:
            return Path(aur_helper)

    def _sanity_check(self):
        number_group_files = len([self.groups_path.iterdir()])
        if number_group_files == 0:
            logging.warning("pacdef does not know any groups. Import one.")


class AURHelper:
    """Abstraction of AUR helpers that act as pacman wrappers."""

    class _Switches:
        """CLI switches for AUR helpers that wrap pacman."""

        install = ["--sync", "--refresh", "--needed"]
        remove = ["--remove", "--recursive"]
        installed_package_info = ["--query", "--info"]
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
        if not _file_exists(path):
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

        :param config: a instance of Config
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

    def __getitem__(self, item):
        """Get the package at index `item`."""
        return self.packages[item]

    def __len__(self):
        """Get number of packages."""
        return len(self.packages)

    def __eq__(self, other: object):
        """Compare with other groups or strings."""
        if isinstance(other, Group):
            return self.name == other.name
        elif isinstance(other, str):
            return self.name == other
        else:
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
        lines = text.split("\n")[:-1]  # last line is empty
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
        if len(package_name) >= 0:
            return Package(package_name)
        else:
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

        def check_not_symlink():
            if not group.is_symlink() and group.is_file():
                logging.warning(f"found group {group}, but it is not a symlink")

        check_dir()
        check_broken_symlink()
        check_not_symlink()

    def append(self, package: Package):
        """Add package to group, by memorizing it and appending it to the group file."""
        self.packages.append(package)
        self.packages.sort()
        with open(self.path, "a") as fd:
            fd.write(f"{package}\n")


class Pacdef:
    """Class representing the main routines of pacdef."""

    def __init__(
        self,
        args: Arguments = None,
        config: Config = None,
        aur_helper: AURHelper = None,
        db: DB = None,
    ):
        """Save the provided arguments as attributes, or use defaults when none are provided."""
        self._conf = config or Config()
        self._args = args or Arguments()
        self._aur_helper = aur_helper or AURHelper(self._conf.aur_helper)
        self._groups: list[Group] = self._read_groups()
        self._db: DB = db or DB()

    # noinspection PyPep8Naming
    def _get_action_map(self) -> dict[Action, Callable[[], None]]:
        """Return a dict matching all actions to their corresponding Pacdef methods."""
        ACTION_MAP = {
            Action.clean: self._remove_unmanaged_packages,
            Action.edit: self._edit_group_file,
            Action.groups: self._list_groups,
            Action.import_: self._import_groups,
            Action.remove: self._remove_group,
            Action.review: self._review,
            Action.search: self._search_package,
            Action.show: self._show_group,
            Action.sync: self._install_packages_from_groups,
            Action.unmanaged: self._show_unmanaged_packages,
            Action.version: _show_version,
        }
        return ACTION_MAP

    def _edit_group_file(self) -> None:
        logging.info("editing group files")
        groups = self._get_groups_matching_arguments()
        paths = [str(group.path) for group in groups]
        CommandRunner.run([str(self._conf.editor), *paths], check=True)

    def run_action_from_arg(self) -> None:
        """Get the function from the provided action arg, execute the function."""
        if self._args.action is not None:
            self._get_action_map()[self._args.action]()

    def _review(self) -> None:
        reviewer = Reviewer(
            self._groups, self._get_unmanaged_packages(), self._aur_helper
        )
        reviewer.ask_user_for_actions()
        reviewer.run_actions()

    def _remove_unmanaged_packages(self) -> None:
        """Remove packages not managed by pacdef.

        Fetches unmanaged packages, then asks the user to confirm removing the packages. Then removes them using
        the AUR helper.
        """
        unmanaged_packages = self._get_unmanaged_packages()
        if len(unmanaged_packages) == 0:
            print(NOTHING_TO_DO)
            sys.exit(EXIT_SUCCESS)
        print("Would remove the following packages and their dependencies:")
        for package in unmanaged_packages:
            print(f"  {package}")
        _get_user_confirmation()
        self._aur_helper.remove(unmanaged_packages)

    def _list_groups(self):
        """Print names of the imported groups to STDOUT."""
        groups = self._get_group_names()
        for group in groups:
            print(group)

    def _import_groups(self) -> None:
        if self._args.files is None:
            return
        for path in self._args.files:
            link_target = self._conf.groups_path / path.name
            if _file_exists(link_target):
                logging.warning(f"{path.name} already exists, skipping")
            else:
                link_target.symlink_to(path.absolute())

    def _remove_group(self) -> None:
        """Remove the provided groups from the pacdef groups directory.

        More than one group can be provided. This method is atomic: If not all groups are found, none are removed.
        """
        try:
            found_groups = self._get_groups_matching_arguments()
        except FileNotFoundError as e:
            logging.error(e)
            sys.exit(EXIT_ERROR)
        for group in found_groups:
            group.remove()

    def _get_groups_matching_arguments(self) -> list[Group]:
        found_groups = []
        if self._args.groups is None:
            raise ValueError("no group supplied")
        for name in self._args.groups:
            found_groups.append(self._find_group_by_name(name))
        return found_groups

    def _find_group_by_name(self, name: str) -> Group:
        logging.info(f"Searching for group '{name}'")
        for group in self._groups:
            if group == name:
                logging.info(f"found group under {group.path}")
                return group
        else:
            raise FileNotFoundError(f"Did not find the group '{name}'.")

    def _search_package(self):
        """Show imported group with contains `_args.package`.

        Only one package may be provided in the args. Exits with `EXIT_ERROR` if the package cannot be found.
        """
        for group in self._groups:
            if self._args.package in group:
                print(group.name)
                sys.exit(EXIT_SUCCESS)
        else:
            sys.exit(EXIT_ERROR)

    def _show_group(self) -> None:
        """Show all packages required by an imported group.

        More than one group may be provided, which prints the contents of all groups in order.
        """
        try:
            found_groups = self._get_groups_matching_arguments()
        except FileNotFoundError as e:
            logging.error(e)
            sys.exit(EXIT_ERROR)
        for group in found_groups:
            print(group.content)

    def _install_packages_from_groups(self) -> None:
        """Install all packages from the imported package groups."""
        to_install = self._calculate_packages_to_install()
        if len(to_install) == 0:
            print(NOTHING_TO_DO)
            sys.exit(EXIT_SUCCESS)
        print("Would install the following packages:")
        for package in to_install:
            print(f"  {package}")
        _get_user_confirmation()
        self._aur_helper.install(to_install)

    def _show_unmanaged_packages(self) -> None:
        """Print unmanaged packages to STDOUT."""
        unmanaged_packages = self._get_unmanaged_packages()
        for package in unmanaged_packages:
            print(package)

    def _calculate_packages_to_install(self) -> list[Package]:
        """Determine which packages must be installed to satisfy the dependencies in the group files.

        :return: list of packages that will be installed
        """
        managed_packages = set(self._get_managed_packages())
        logging.debug(f"{managed_packages=}")
        installed_packages = set(self._db.get_all_installed_packages())
        logging.debug(f"{installed_packages=}")
        pacdef_only = list(managed_packages - installed_packages)
        pacdef_only.sort()
        logging.debug(f"{pacdef_only=}")
        return pacdef_only

    def _get_unmanaged_packages(self) -> list[Package]:
        """Get explicitly installed packages which are not in the imported pacdef groups.

        :return: list of unmanaged packages
        """
        managed_packages = set(self._get_managed_packages())
        logging.debug(f"{managed_packages=}")
        explicitly_installed_packages = set(
            self._db.get_explicitly_installed_packages()
        )
        logging.debug(f"{explicitly_installed_packages=}")
        unmanaged_packages = list(explicitly_installed_packages - managed_packages)
        unmanaged_packages.sort()
        logging.debug(f"{unmanaged_packages=}")
        return unmanaged_packages

    def _get_managed_packages(self) -> list[Package]:
        """Get all packaged that are known to pacdef (i.e. are located in imported group files).

        :return: list of packages
        """
        packages = []
        for group in self._groups:
            packages.extend(group.packages)
        if len(packages) == 0:
            logging.warning("pacdef does not know any packages.")
        return packages

    def _get_group_names(self) -> list[str]:
        """Get list of the names of all imported groups (= list of filenames in the pacdef group directory).

        :return: list of imported group names
        """
        groups = [group.name for group in self._groups]
        return groups

    def _read_groups(self) -> list[Group]:
        """Read all imported groups (= list of files in the pacdef group directory).

        :return: list of imported groups
        """
        paths = [group for group in self._conf.groups_path.iterdir()]
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


class Reviewer:
    """Handles logic related to `pacdef review`."""

    def __init__(
        self,
        groups: list[Group],
        unmanaged_packages: list[Package],
        aur_helper: AURHelper,
    ):
        """Initialize Reviewer with data from Pacdef."""
        logging.info("Reviewer initialization")
        self._groups = groups
        self._unmanaged_packages = unmanaged_packages
        self._current_package_index = 0
        self._actions: list[Review] = []
        self._aur_helper = aur_helper

        logging.debug("groups")
        logging.debug([f"{group.name}" for group in groups])
        logging.debug("unmanaged_packages")
        logging.debug([f"{package}" for package in unmanaged_packages])
        logging.debug(f"{aur_helper=}")

    def ask_user_for_actions(self) -> None:
        """Populate self._actions with Reviews."""
        self._print_unmanaged_packages()
        while self._current_package_index < len(self._unmanaged_packages):
            print(self._current_package)
            self._actions.append(self._get_action_from_user_input_for_current_package())
            logging.info("proceeding with next package")
            self._current_package_index += 1
        logging.info("all packages processed")

    def _print_unmanaged_packages(self) -> None:
        if self._unmanaged_packages:
            print("Unmanaged packages:")
            for package in self._unmanaged_packages:
                print(f"  {package}")
            print()

    @property
    def _current_package(self) -> Package:
        logging.debug(f"{self._current_package_index=}")
        current_package = self._unmanaged_packages[self._current_package_index]
        logging.debug(f"{current_package=}")
        return current_package

    def _get_action_from_user_input_for_current_package(self) -> Review:
        action = self._get_user_input(
            "assign to (g)roup, (d)elete, (s)kip, (i)nfo, (a)s dependency? ",
            self._parse_input_action,
        )
        group = None
        if action == ReviewAction.assign_to_group:
            self._print_enumerated_groups()
            group = self._get_user_input("Group or (c)ancel? ", self._parse_input_group)
            if group is None:
                return self._get_action_from_user_input_for_current_package()
        elif action == ReviewAction.info:
            self._aur_helper.print_info(self._current_package)
            return self._get_action_from_user_input_for_current_package()
        print()
        return Review(action, self._current_package, group)

    @staticmethod
    def _get_user_input(
        prompt: str,
        validator: Callable[[str | None], Any],
        *,
        default: str | None = None,
    ) -> Any:
        user_input: str | None
        result: str | None
        user_input, result = "", ""
        while not user_input:
            user_input = input(prompt).lower() or default
            logging.info(f"{user_input=}")
            try:
                result = validator(user_input)
            except ValueError:
                logging.info("parsing: ValueError, resetting user_input")
                user_input = ""
            except KeyboardInterrupt:
                sys.exit(EXIT_INTERRUPT)
        return result

    def _parse_input_action(self, user_input: str | None) -> ReviewAction:
        if user_input is None:
            raise ValueError("Cannot provide ReviewAction identified by `None`.")
        return self._get_action_map()[user_input]

    # noinspection PyPep8Naming
    @staticmethod
    def _get_action_map() -> dict[str, ReviewAction]:
        ACTION_MAP = {
            "g": ReviewAction.assign_to_group,
            "d": ReviewAction.delete,
            "s": ReviewAction.skip,
            "i": ReviewAction.info,
            "a": ReviewAction.as_dependency,
        }
        return ACTION_MAP

    def _print_enumerated_groups(self):
        width = len(str(len(self._groups)))
        for i, group in enumerate(self._groups):
            print(f"{str(i).rjust(width)}: {group.name}")

    def _parse_input_group(self, user_input: str | None) -> Group | None:
        if user_input is None:
            raise ValueError("Cannot find Group identified by `None`.")
        if user_input == "c":
            logging.info("Group selection: cancel")
            return None
        try:
            logging.info("Group selection")
            return self._groups[int(user_input)]
        except TypeError as e:  # No input
            logging.info("no input")
            raise ValueError(e)

    def run_actions(self) -> None:
        """Run actions from self._actions."""
        self._print_strategy()

        if not (self._to_assign or self._to_delete or self._as_dependency):
            print(NOTHING_TO_DO)
            sys.exit(EXIT_SUCCESS)

        if not self._get_user_input(
            "Confirm? [y, N] ", lambda from_user: from_user == "y", default="n"
        ):
            logging.info("No user confirmation")
            sys.exit(EXIT_SUCCESS)
        logging.info("user confirmation, running actions in order")
        self._delete(self._to_delete)
        self._assign(self._to_assign)
        self._make_dependency(self._as_dependency)

    @property
    def _to_delete(self) -> list[Review]:
        return self._get_reviews_by_action(ReviewAction.delete)

    @property
    def _to_assign(self) -> list[Review]:
        return self._get_reviews_by_action(ReviewAction.assign_to_group)

    @property
    def _as_dependency(self) -> list[Review]:
        return self._get_reviews_by_action(ReviewAction.as_dependency)

    def _get_reviews_by_action(self, action: ReviewAction) -> list[Review]:
        return [review for review in self._actions if review.action == action]

    def _print_strategy(self) -> None:
        logging.debug(f"{self._actions=}")
        if self._to_delete:
            self._print_to_delete(self._to_delete)
        if self._to_assign:
            self._print_to_assign(self._to_assign)
        if self._as_dependency:
            self._print_as_dependency(self._as_dependency)

    @staticmethod
    def _print_to_assign(to_assign):
        print("Will assign packages as follows:")
        for review in to_assign:
            logging.debug(review)
            print(f"  {review.package} -> {review.group.name}")
        print()

    @staticmethod
    def _print_to_delete(to_delete):
        print("Will delete the following packages:")
        for review in to_delete:
            logging.debug(review)
            print(f"  {review.package}")
        print()

    def _delete(self, to_delete: list[Review]) -> None:
        logging.info("_delete")
        packages = [review.package for review in to_delete]
        if packages:
            logging.debug(f"{packages=}")
            self._aur_helper.remove(packages)

    @staticmethod
    def _assign(to_assign: list[Review]):
        logging.info("_assign")
        for review in to_assign:
            logging.debug(review)
            if review.group is not None:
                review.group.append(review.package)

    @staticmethod
    def _print_as_dependency(as_dependency: list[Review]) -> None:
        print("Will mark the following packages as installed as dependency:")
        for review in as_dependency:
            logging.debug(review)
            print(f"  {review.package}")
        print()

    def _make_dependency(self, as_dependency: list[Review]) -> None:
        logging.info("_make_dependency")
        packages = [review.package for review in as_dependency]
        if packages:
            logging.debug(f"{packages=}")
            self._aur_helper.as_dependency(packages)


class ReviewAction(Enum):
    """Possible actions for `pacdef review`."""

    assign_to_group = "assign to group"
    delete = "delete"
    skip = "skip"
    info = "info"
    as_dependency = "as dependency"


class Review:
    """Holds results of review for a single package."""

    def __init__(self, action: ReviewAction, package: Package, group: Group | None):
        """Initialize Review with results of review."""
        self._action = action
        self._package = package
        self._group = group

    def __repr__(self):
        """Representation for debugging purposes."""
        return f"Review: {self._action}, {self._package}, {self._group}"

    @property
    def action(self) -> ReviewAction:
        """Get the ReviewAction of the instance."""
        return self._action

    @property
    def group(self) -> Group | None:
        """Get the group of the instance."""
        return self._group

    @property
    def package(self) -> Package:
        """Get the package of the instance."""
        return self._package


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

    def __eq__(self, other: object):
        """Check if equal to other package by comparing the name only."""
        if isinstance(other, Package):
            return self.name == other.name
        elif isinstance(other, str):
            return self.name == other
        else:
            raise ValueError("Must be compared with Package or string.")

    def __hash__(self):
        return hash(self.name)

    def __lt__(self, other):
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
                raise
        else:
            repo = None
            name = package_string
        return name, repo


class CommandRunner:
    """Abstract calls of external commands to provide logging."""

    @staticmethod
    def run(command: list[str], *args, **kwargs):
        """Run a command using subprocess.run."""
        logging.info(f"Executing command with subprocess.run: {command, args, kwargs}")
        try:
            subprocess.run(command, *args, **kwargs)
        except subprocess.CalledProcessError as e:
            logging.error(e)
            sys.exit(EXIT_ERROR)

    @staticmethod
    def call(command: list[str], *args, **kwargs) -> None:
        """Run a command using subprocess.call."""
        logging.info(f"Executing command with subprocess.call: {command, args, kwargs}")
        try:
            subprocess.call(command, *args, **kwargs)
        except FileNotFoundError:
            logging.error(f'Could not execute the program "{command[0]}".')
            sys.exit(EXIT_ERROR)


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
        instances = [
            Package(pkg.name)
            for pkg in self._db.pkgcache
            if pkg.reason == pyalpm.PKG_REASON_EXPLICIT
        ]
        return instances

    def get_all_installed_packages(self) -> list[Package]:
        """Get all installed packages. Equivalent to `pacman -Qq`."""
        return [Package(pkg.name) for pkg in self._db.pkgcache]

    @classmethod
    def _get_db_path(cls) -> Path:
        # the config may contain flags, which cannot be parsed by configparser
        lines = cls._get_lines_from_config(Path("/etc/pacman.conf"))
        for line in lines:
            if not cls._line_is_comment(line) and cls._DB_PATH_KEY in line:
                return cls._get_path_from_line(line)
        else:
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


def _setup_logger() -> None:
    """Set up the logger.

    When the log level is below WARNING (i.e. INFO or DEBUG), the line number of the logging statement is printed as
    well.
    """
    try:
        level_name: str = environ["LOGLEVEL"]
    except KeyError:
        level_name = "WARNING"

    level: int = logging.getLevelName(level_name.upper())
    if level < logging.WARNING:
        logging.basicConfig(format="%(levelname)s:%(lineno)d: %(message)s", level=level)
    else:
        logging.basicConfig(format="%(levelname)s: %(message)s", level=level)


def _show_version() -> None:
    """Print version information to STDOUT.

    The value of `VERSION` is set during compile time by the PKGBUILD using `build()`.
    """
    print(f"pacdef, version: {VERSION}")


if __name__ == "__main__":
    try:
        _main()
    except KeyboardInterrupt:
        sys.exit(EXIT_INTERRUPT)
