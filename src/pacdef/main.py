from __future__ import annotations

import io
import logging
import sys
from importlib import metadata
from os import environ
from typing import Callable, Optional

from .action import Action
from .args import Arguments
from .aur_helper import AURHelper
from .cmd import run
from .config import Config
from .constants import EXIT_ERROR, EXIT_INTERRUPT, EXIT_SUCCESS, NOTHING_TO_DO
from .db import DB
from .group import Group
from .package import Package
from .path import file_exists
from .review import Reviewer
from .user_input import get_user_confirmation


def main():
    _setup_logger()
    args = Arguments()
    config = Config()
    helper = AURHelper.from_config(config)
    db = DB()
    pacdef = Pacdef(args=args, config=config, aur_helper=helper, db=db)
    pacdef.run_action_from_arg()


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


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        sys.exit(EXIT_INTERRUPT)


def _show_version() -> None:
    """Print version information to STDOUT.

    The value of `VERSION` is set during compile time by the PKGBUILD using `build()`.
    """
    print(f"pacdef, version: {metadata.version('pacdef')}")


class Pacdef:
    """Class representing the main routines of pacdef."""

    def __init__(
        self,
        args: Optional[Arguments] = None,
        config: Optional[Config] = None,
        aur_helper: Optional[AURHelper] = None,
        db: Optional[DB] = None,
        groups: Optional[list[Group]] = None,
    ):
        """Save the provided arguments as attributes, or use defaults when none are provided."""
        self._conf = config or Config()
        self._args = args or Arguments()
        self._aur_helper = aur_helper or AURHelper(self._conf.aur_helper)
        self._groups: list[Group] = groups or self._read_groups()
        self._db: DB = db or DB()
        self._sanity_check()

    @property
    def _action_map(self) -> dict[Action, Callable[[], None]]:
        """Return a dict matching all actions to their corresponding Pacdef methods."""
        return {
            Action.clean: self._remove_unmanaged_packages,
            Action.edit: self._edit_group_file,
            Action.groups: self._list_groups,
            Action.import_: self._import_groups,
            Action.new: self._new_group,
            Action.remove: self._remove_group,
            Action.review: self._review,
            Action.search: self._search_package,
            Action.show: self._show_group,
            Action.sync: self._install_packages_from_groups,
            Action.unmanaged: self._show_unmanaged_packages,
            Action.version: _show_version,
        }

    def _edit_group_file(self) -> None:
        logging.info("editing group files")
        groups = self._get_groups_matching_arguments()
        paths = [str(group.path) for group in groups]
        run([str(self._conf.editor), *paths], check=True)

    def _new_group(self) -> None:
        if self._args.groups is None:
            logging.error("Cannot create new group. No name supplied.")
            exit(EXIT_ERROR)

        # check if we can create all groups before we actually create them
        for group in self._args.groups:
            if group in [g.name for g in self._groups]:
                logging.error(f"Cannot create new group '{group}', it already exists.")
                exit(EXIT_ERROR)

        for group in self._args.groups:
            Group.new_file(group, self._conf.groups_path)

        if self._args.edit_new:
            self._groups = self._read_groups()
            self._edit_group_file()

    def run_action_from_arg(self) -> None:
        """Get the function from the provided action arg, execute the function."""
        if self._args.action is not None:
            self._action_map[self._args.action]()

    def _review(self) -> None:
        reviewer = Reviewer(
            self._groups, self._get_unmanaged_packages(), self._aur_helper
        )
        reviewer.ask_user_for_actions()
        reviewer.print_strategy()
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
        try:
            get_user_confirmation()
        except io.UnsupportedOperation:
            pass
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
            if file_exists(link_target):
                logging.warning(f"{path.name} already exists, skipping")
            else:
                link_target.symlink_to(path.absolute())

    def _remove_group(self) -> None:
        """Remove the provided groups from the pacdef groups directory.

        More than one group can be provided. This method is atomic: If not all groups are found, none are removed.
        """
        found_groups = self._get_groups_matching_arguments()
        for group in found_groups:
            group.remove()

    def _get_groups_matching_arguments(self) -> list[Group]:
        found_groups = []
        if self._args.groups is None:
            raise ValueError("no group supplied")
        for name in self._args.groups:
            try:
                found_groups.append(self._find_group_by_name(name))
            except FileNotFoundError as err:
                logging.error(err)
                exit(EXIT_ERROR)
        return found_groups

    def _find_group_by_name(self, name: str) -> Group:
        logging.info(f"Searching for group '{name}'")
        for group in self._groups:
            if group == name:
                logging.info(f"found group under {group.path}")
                return group
        raise FileNotFoundError(f"Did not find the group '{name}'.")

    def _search_package(self):
        """Show imported group which contains `_args.package`.

        The package name may be a regex. Only one package may be provided in the args.
        Exits with `EXIT_ERROR` if the package cannot be found.
        """
        if self._args.package is None:
            logging.error("no search string provided")
            sys.exit(EXIT_ERROR)

        matches = [
            (group, package)
            for group in self._groups
            for package in group
            if package.matches_regex(self._args.package)
        ]

        for group, package in matches:
            print(f"{group.name}: {package}")

        if matches:
            sys.exit(EXIT_SUCCESS)
        sys.exit(EXIT_ERROR)

    def _show_group(self) -> None:
        """Show all packages required by an imported group.

        More than one group may be provided, which prints the contents of all groups in order.
        """
        found_groups = self._get_groups_matching_arguments()
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
        try:
            get_user_confirmation()
        except io.UnsupportedOperation:
            pass
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

    def _sanity_check(self) -> None:
        if self._conf.warn_symlinks:
            for group in self._groups:
                if not group.path.is_symlink():
                    logging.warning(f"group '{group.name}' is not a symlink")
