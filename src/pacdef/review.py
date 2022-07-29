from __future__ import annotations

import logging
import sys
from enum import Enum, auto

from .aur_helper import AURHelper
from .constants import EXIT_ABORT, EXIT_SUCCESS, NOTHING_TO_DO
from .group import Group
from .package import Package
from .user_input import get_user_input


class _ReviewAction(Enum):
    """Possible actions for `pacdef review`."""

    assign_to_group = "assign to group"
    delete = "delete"
    skip = "skip"
    info = "info"
    as_dependency = "as dependency"


class Review:
    """Holds results of review for a single package."""

    def __init__(self, action: _ReviewAction, package: Package, group: Group | None):
        """Initialize Review with results of review."""
        self._action = action
        self._package = package
        self._group = group

    def __repr__(self):
        """Representation for debugging purposes."""
        return f"Review: {self._action}, {self._package}, {self._group}"

    @property
    def action(self) -> _ReviewAction:
        """Get the _ReviewAction of the instance."""
        return self._action

    @property
    def group(self) -> Group | None:
        """Get the group of the instance."""
        return self._group

    @property
    def package(self) -> Package:
        """Get the package of the instance."""
        return self._package


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
        # noinspection SpellCheckingInspection
        action = get_user_input(
            "assign to (g)roup, (d)elete, (s)kip, (i)nfo, (a)s dependency, (q)uit? ",
            self._parse_input_action,
            single_character=True,
        )
        group = None
        if action == _ReviewAction.assign_to_group:
            self._print_enumerated_groups()
            # noinspection SpellCheckingInspection
            group = get_user_input("Group or (c)ancel? ", self._parse_input_group)
            if group is None:
                return self._get_action_from_user_input_for_current_package()
        elif action == _ReviewAction.info:
            self._aur_helper.print_info(self._current_package)
            return self._get_action_from_user_input_for_current_package()
        print()
        return Review(action, self._current_package, group)

    def _parse_input_action(self, user_input: str | None) -> _ReviewAction:
        if user_input is None:
            raise ValueError("Cannot provide _ReviewAction identified by `None`.")
        if user_input == "q":
            sys.exit(EXIT_SUCCESS)
        try:
            action = self._get_action_map()[user_input]
        except KeyError:
            raise ValueError("Invalid user input.")
        return action

    # noinspection PyPep8Naming
    @staticmethod
    def _get_action_map() -> dict[str, _ReviewAction]:
        ACTION_MAP = {
            "g": _ReviewAction.assign_to_group,
            "d": _ReviewAction.delete,
            "s": _ReviewAction.skip,
            "i": _ReviewAction.info,
            "a": _ReviewAction.as_dependency,
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

        def check_wants_to_continue(from_user: str | None) -> _Intention:
            if from_user is None:
                return _Intention.unknown
            from_user = from_user.lower().strip()
            if from_user == "n":
                return _Intention.abort
            if from_user == "y":
                return _Intention.confirm
            return _Intention.unknown

        if not (self._to_assign or self._to_delete or self._as_dependency):
            print(NOTHING_TO_DO)
            sys.exit(EXIT_SUCCESS)

        user_input = get_user_input(
            "Confirm? [y, n] ",
            check_wants_to_continue,
            single_character=True,
        )
        match user_input:
            case _Intention.confirm:
                logging.info("user confirmation, running actions in order")
                self._delete(self._to_delete)
                self._assign(self._to_assign)
                self._make_dependency(self._as_dependency)
            case _Intention.abort:
                logging.info("user wants to abort, exiting")
                sys.exit(EXIT_ABORT)
            case _Intention.unknown:
                logging.info("reply not within allowed selection")
                self.run_actions()
            case _:
                raise ValueError("should not happen")

    @property
    def _to_delete(self) -> list[Review]:
        return self._get_reviews_by_action(_ReviewAction.delete)

    @property
    def _to_assign(self) -> list[Review]:
        return self._get_reviews_by_action(_ReviewAction.assign_to_group)

    @property
    def _as_dependency(self) -> list[Review]:
        return self._get_reviews_by_action(_ReviewAction.as_dependency)

    def _get_reviews_by_action(self, action: _ReviewAction) -> list[Review]:
        return [review for review in self._actions if review.action == action]

    def print_strategy(self) -> None:
        """Print the actions that will be executed, based on the reviews that have been executed."""
        logging.debug(f"{self._actions=}")
        if self._to_delete:
            self._print_to_delete(self._to_delete)
        if self._to_assign:
            self._print_to_assign(self._to_assign)
        if self._as_dependency:
            self._print_as_dependency(self._as_dependency)

    @staticmethod
    def _print_to_assign(to_assign: list[Review]) -> None:
        print("Will assign packages as follows:")
        for review in to_assign:
            logging.debug(review)
            assert review.group is not None
            print(f"  {review.package} -> {review.group.name}")
        print()

    @staticmethod
    def _print_to_delete(to_delete: list[Review]) -> None:
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


# noinspection PyArgumentList
class _Intention(Enum):
    abort = auto()
    confirm = auto()
    unknown = auto()
