#!/usr/bin/python

"""
Declarative package manager for Arch Linux.

https://github.com/steven-omaha/pacdef
"""

from __future__ import annotations

import argparse
import configparser
import logging
import subprocess
import sys
from enum import Enum
from os import environ
from pathlib import Path
from typing import Optional, Callable

EXIT_SUCCESS = 0
EXIT_ERROR = 1
EXIT_INTERRUPT = 130

COMMENT = "#"
PARU = Path("/usr/bin/paru")
VERSION = "unknown"


def _main():
    _setup_logger()
    args = Arguments()
    config = Config()
    helper = AURHelper.from_config(config)
    pacdef = Pacdef(args=args, config=config, aur_helper=helper)
    pacdef.run_action_from_arg()


def _get_packages_from_group(group: Path) -> list[Package]:
    """Read a group file, return the list of contained packages.

    :param group: path to group file
    :return: list of packages contained in group
    """
    text = group.read_text()
    lines = text.split("\n")[:-1]  # last line is empty
    packages = []
    for line in lines:
        package = _get_package_from_line(line)
        if package is not None:
            packages.append(package)
    return packages


def _get_package_from_line(line: str) -> Optional[Package]:
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


def _calculate_package_diff(
    system_packages: list[Package], pacdef_packages: list[Package]
) -> tuple[list[Package], list[Package]]:
    """
    Determine difference in packages between system and pacdef.

    :param system_packages: list of packages known by the system
    :param pacdef_packages: list of packages known by pacdef, optionally with repository prefix
    :return: 2-tuple: list of packages exclusively in argument 1, list of packages exclusively in argument 2
    """
    logging.debug("calculate_package_diff")
    logging.debug(f"{system_packages=}")
    logging.debug(f"{pacdef_packages=}")
    system_only = []
    pacdef_only = []
    for package in system_packages:
        if package not in pacdef_packages:
            system_only.append(package)
    for package in pacdef_packages:
        if package not in system_packages:
            pacdef_only.append(package)
    logging.debug(f"{system_only=}")
    logging.debug(f"{pacdef_only=}")
    return system_only, pacdef_only


def _get_path_from_group_name(conf: Config, group_name: str) -> Path:
    """
    Determine the absolute path of a group by checking if the corresponding file exists in the groups directory.

    :param conf: a Config instance
    :param group_name: name of the group to search for
    :return: path of the group
    """
    group = conf.groups_path.joinpath(group_name)
    if not _file_exists(group):
        if group.is_symlink():
            logging.warning(
                f"found group {group.absolute()}, but it is a broken symlink"
            )
        else:
            raise FileNotFoundError
    return group


def _get_user_confirmation() -> None:
    """Ask the user if he wants to continue. Exits if the answer is not `y` or of length zero."""
    user_input = input("Continue? [Y/n] ").lower()
    if user_input != "y" or len(user_input) == 0:
        sys.exit(EXIT_SUCCESS)


class Arguments:
    """Class providing the command line arguments."""

    def __init__(self):
        """Setup the argument parser, parse the args, collect the results in attributes."""
        parser = self._setup_parser()
        args = parser.parse_args()
        self.action: Action = self._parse_action(args)
        self.files: Optional[list[Path]] = self._parse_files(args)
        self.groups: Optional[list[str]] = self._parse_groups(args)
        self.package: Optional[Package] = self._parse_package(args)

    @staticmethod
    def _parse_package(args: argparse.Namespace) -> Optional[Package]:
        if not hasattr(args, "package"):
            return
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
    def _parse_files(args: argparse.Namespace) -> Optional[list[Path]]:
        if not hasattr(args, "file"):
            return
        files = [Path(f) for f in args.file]
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
    def _parse_groups(args: argparse.Namespace) -> Optional[list[str]]:
        if not hasattr(args, "group"):
            return
        return args.group


def _dir_exists(path: Path) -> bool:
    return path.exists() and path.is_dir()


def _file_exists(path: Path) -> bool:
    return path.exists() and path.is_file()


class Action(Enum):
    """Enum of actions that can be provided as first argument to `pacdef`."""

    clean = "clean"
    groups = "groups"
    import_ = "import"
    remove = "remove"
    search = "search"
    show = "show"
    sync = "sync"
    unmanaged = "unmanaged"
    version = "version"


class Config:
    """Class reading and holding the runtime configuration."""

    aur_helper: Path
    groups_path: Path
    _CONFIG_STUB = f"[misc]\naur_helper = {PARU}\n"

    def __init__(self):
        """Instantiate using the config file. If it does not exist, use defaults."""
        config_base_dir = self._get_xdg_config_home()

        pacdef_path = config_base_dir.joinpath("pacdef")
        config_file = pacdef_path.joinpath("pacdef.conf")
        self.groups_path = pacdef_path.joinpath("groups")

        if not _dir_exists(pacdef_path):
            pacdef_path.mkdir(parents=True)
        if not _dir_exists(self.groups_path):
            self.groups_path.mkdir()
        if not _file_exists(config_file):
            config_file.touch()

        self.aur_helper = self._get_aur_helper(config_file)
        logging.info(f"{self.groups_path=}")

    @staticmethod
    def _get_xdg_config_home() -> Path:
        try:
            config_base_dir = Path(environ["XDG_CONFIG_HOME"])
        except KeyError:
            home = Path.home()
            config_base_dir = home.joinpath(".config")
        logging.debug(f"{config_base_dir=}")
        return config_base_dir

    @classmethod
    def _get_aur_helper(cls, config_file: Path) -> Path:
        config = configparser.ConfigParser()

        try:
            config.read(config_file)
        except configparser.ParsingError as e:
            logging.error(f"Could not parse the config: {e}")

        try:
            path = Path(config["misc"]["aur_helper"])
        except KeyError:
            logging.warning(f"No AUR helper set. Defaulting to {PARU}")
            path = PARU
            cls._write_config_stub(config_file)

        return path

    @classmethod
    def _write_config_stub(cls, config_file: Path):
        logging.info(f"Created config stub under {config_file}")
        config_file.write_text(cls._CONFIG_STUB)


class AURHelper:
    """Abstraction of AUR helpers that act as pacman wrappers."""

    class _Switches:
        """CLI switches for AUR helpers that wrap pacman."""

        install = ["--sync", "--refresh", "--needed"]
        remove = ["--remove", "--recursive"]
        installed_packages = ["--query", "--quiet"]
        explicitly_installed_packages = ["--query", "--quiet", "--explicit"]

    def __init__(self, path: Path):
        """Default constructor for AURHelper.

        If the AUR helper is not found, and error is raised.
        :param path: path to the AUR helper to use (example: `/usr/bin/paru`).
        """
        if not path.is_absolute():
            path = Path("/usr/bin").joinpath(path)
        if not _file_exists(path):
            raise FileNotFoundError(f"{path} not found.")
        self._path = path
        logging.info(f"AUR helper: {self._path}")

    def _execute(self, command: list[str]) -> None:
        """Execute an AUR helper command without checking the output.

        :param command: the command to execute, list of strings.
        """
        try:
            subprocess.call([str(self._path)] + command)
        except FileNotFoundError:
            logging.error(f'Could not start the AUR helper "{self._path}".')
            sys.exit(EXIT_ERROR)

    def _get_output(self, query: list[str]) -> list[str]:
        """Forward the query to the AUR helper, return its STDOUT.

        :param query: command arguments as list of strings
        :return: AUR helper output as list of strings
        """
        command = [str(self._path)] + query
        result = subprocess.check_output(command).decode("utf-8")
        result_list = result.split("\n")[:-1]  # last entry is zero-length
        return result_list

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

    def get_all_installed_packages(self) -> list[Package]:
        """Query the AUR helper for all installed packages.

        :return: list of `Package`s that are installed.
        """
        packages: list[str] = self._get_output(self._Switches.installed_packages)
        instances = [Package(p) for p in packages]
        return instances

    def get_explicitly_installed_packages(self) -> list[Package]:
        """Query the AUR helper for all explicitly installed packages.

        :return: list of `Package`s that were explicitly installed.
        """
        packages = self._get_output(self._Switches.explicitly_installed_packages)
        instances = [Package(p) for p in packages]
        return instances

    @classmethod
    def from_config(cls, config: Config) -> AURHelper:
        """Create an AUR helper instance using `config.aur_helper`.

        :param config: a instance of Config
        :return: an instance of AURHelper
        """
        return cls(path=config.aur_helper)


class Pacdef:
    """Class representing the main routines of pacdef."""

    def __init__(
        self,
        args: Arguments = None,
        config: Config = None,
        aur_helper: AURHelper = None,
    ):
        """Save the provided arguments as attributes, or use defaults when none are provided."""
        self._conf = config or Config()
        self._args = args or Arguments()
        self._aur_helper = aur_helper or AURHelper(PARU)

    def _get_action_map(self) -> dict[Action, Callable]:
        """Return a dict matching all actions to their corresponding Pacdef methods."""
        ACTION_MAP = {
            Action.clean: self._remove_unmanaged_packages,
            Action.groups: self._show_groups,
            Action.import_: self._import_groups,
            Action.remove: self._remove_group,
            Action.search: self._search_package,
            Action.show: self._show_group,
            Action.sync: self._install_packages_from_groups,
            Action.unmanaged: self._show_unmanaged_packages,
            Action.version: _show_version,
        }
        return ACTION_MAP

    def run_action_from_arg(self) -> None:
        """Get the function from the provided action arg, execute the function."""
        action_map = self._get_action_map()
        action_fn = action_map[self._args.action]
        action_fn()

    def _remove_unmanaged_packages(self) -> None:
        """Remove packages not managed by pacdef.

        Fetches unmanaged packages, then asks the user to confirm removing the packages. Then removes them using
        the AUR helper.
        """
        unmanaged_packages = self._get_unmanaged_packages()
        if len(unmanaged_packages) == 0:
            print("nothing to do")
            sys.exit(EXIT_SUCCESS)
        print("Would remove the following packages and their dependencies:")
        for package in unmanaged_packages:
            print(package)
        _get_user_confirmation()
        self._aur_helper.remove(unmanaged_packages)

    def _show_groups(self):
        """Print names of the imported groups to STDOUT."""
        groups = self._get_group_names()
        for group in groups:
            print(group)

    def _import_groups(self) -> None:
        # check if all file-arguments exist before we do anything (be atomic)
        for f in self._args.files:
            path = Path(f)
            if not _file_exists(path):
                logging.error(f"Cannot import {f}. Is it an existing file?")
                sys.exit(EXIT_ERROR)
        for f in self._args.files:
            path = Path(f)
            link_target = self._conf.groups_path.joinpath(f.name)
            if _file_exists(link_target):
                logging.warning(f"{f} already exists, skipping")
            else:
                link_target.symlink_to(path.absolute())

    def _remove_group(self) -> None:
        """Remove the provided groups from the pacdef groups directory.

        More than one group can be provided. This method is atomic: If not all groups are found, none are removed.
        """
        found_groups = []
        for group_name in self._args.groups:
            group_file = self._conf.groups_path.joinpath(group_name)
            if group_file.is_symlink() or _file_exists(group_file):
                found_groups.append(group_file)
            else:
                logging.error(f"Did not find the group {group_name}")
                sys.exit(EXIT_ERROR)
        for path in found_groups:
            path.unlink()

    def _search_package(self):
        """Show imported group with contains `_args.package`.

        Only one package may be provided in the args. Exits with `EXIT_ERROR` if the package cannot be found.
        """
        for group in self._conf.groups_path.iterdir():
            packages = _get_packages_from_group(group)
            if self._args.package in packages:
                print(group.name)
                sys.exit(EXIT_SUCCESS)
        else:
            sys.exit(EXIT_ERROR)

    def _show_group(self) -> None:
        """Show all packages required by an imported group.

        More than one group may be provided, which prints the contents of all groups in order.
        """
        groups_to_show = self._args.groups
        imported_groups_name = self._get_group_names()
        for group_name in groups_to_show:
            if group_name not in imported_groups_name:
                logging.error(f"I don't know the group {group_name}.")
                sys.exit(EXIT_ERROR)
        for group_name in groups_to_show:
            group = _get_path_from_group_name(self._conf, group_name)
            packages = _get_packages_from_group(group)
            for package in packages:
                print(package)

    def _install_packages_from_groups(self) -> None:
        """Install all packages from the imported package groups."""
        to_install = self._calculate_packages_to_install()
        if len(to_install) == 0:
            print("nothing to do")
            sys.exit(EXIT_SUCCESS)
        print("Would install the following packages:")
        for package in to_install:
            print(package)
        _get_user_confirmation()
        self._aur_helper.install(to_install)

    def _show_unmanaged_packages(self) -> None:
        """Print unmanaged packages to STDOUT."""
        unmanaged_packages = self._get_unmanaged_packages()
        for package in unmanaged_packages:
            print(package)

    def _calculate_packages_to_install(self) -> list[Package]:
        """
        Determine which packages must be installed to satisfy the dependencies in the group files.

        :return: list of packages that will be installed
        """
        pacdef_packages = self._get_managed_packages()
        installed_packages = self._aur_helper.get_all_installed_packages()
        _, pacdef_only = _calculate_package_diff(installed_packages, pacdef_packages)
        return pacdef_only

    def _get_unmanaged_packages(self) -> list[Package]:
        """Get explicitly installed packages which are not in the imported pacdef groups.

        :return: list of unmanaged packages
        """
        managed_packages = self._get_managed_packages()
        explicitly_installed_packages = (
            self._aur_helper.get_explicitly_installed_packages()
        )
        unmanaged_packages, _ = _calculate_package_diff(
            explicitly_installed_packages, managed_packages
        )
        unmanaged_packages.sort()
        return unmanaged_packages

    def _get_managed_packages(self) -> list[Package]:
        """
        Get all packaged that are known to pacdef (i.e. are located in imported group files).

        :return: list of packages
        """
        packages = []
        for group in self._conf.groups_path.iterdir():
            content = _get_packages_from_group(group)
            packages.extend(content)
        if len(packages) == 0:
            logging.warning("pacdef does not know any groups. Import one.")
        return packages

    def _get_group_names(self) -> list[str]:
        """
        Get list of the names of all imported groups (= list of filenames in the pacdef group directory).

        :return: list of imported group names
        """
        groups = [group.name for group in self._get_groups()]
        logging.info(f"{groups=}")
        return groups

    def _get_groups(self) -> list[Path]:
        """
        Get list of the paths of all imported groups (= list of files in the pacdef group directory).

        :return: list of imported group paths
        """
        groups = [group for group in self._conf.groups_path.iterdir()]
        groups.sort()
        for group in groups:
            self._sanity_check(group)
        logging.debug(f"{groups=}")
        return groups

    def _sanity_check(self, group):
        def check_dir():
            if group.is_dir():
                logging.warning(f"found directory {group} in {self._conf.groups_path}")

        def check_broken_symlink():
            if group.is_symlink() and not group.exists():
                logging.warning(f"found group {group}, but it is a broken symlink")

        def check_not_symlink():
            if not group.is_symlink() and group.is_file():
                logging.warning(f"found group {group}, but it is not a symlink")

        check_dir()
        check_broken_symlink()
        check_not_symlink()


class Package:
    """Class that represents a single package."""

    def __init__(self, package_string: str):
        """Initialize an instance by a package string.

        :param package_string: The string describing the package. May contain a repository prefix followed by a `/`.
                               Examples: `zsh` or `repo/spotify`.
        """
        self.name: str
        self.repo: Optional[str]
        self.name, self.repo = self._split_into_name_and_repo(package_string)

    def __eq__(self, other: Package):
        """Check if equal to other package by comparing the name only."""
        return self.name == other.name

    def __repr__(self):
        """Print `repo/package` if a repo was provided, otherwise print `package`."""
        if self.repo is not None:
            result = f"{self.repo}/{self.name}"
        else:
            result = self.name
        return result

    def __lt__(self, other: Package):
        """Compare position to another package in lexical ordering by comparing the names.

        :param other: another Package
        :return: True if this package comes before the other package in lexical ordering, otherwise False.
        """
        return self.name < other.name

    @staticmethod
    def _split_into_name_and_repo(package_string: str) -> tuple[str, Optional[str]]:
        """
        Take a string in the form `repository/package`, return package and repository.

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
                sys.exit(EXIT_ERROR)
        else:
            repo = None
            name = package_string
        return name, repo


def _setup_logger() -> None:
    """Setup the logger.

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
