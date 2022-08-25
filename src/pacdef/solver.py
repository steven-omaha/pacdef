import logging
from typing import Optional

from .args import Arguments
from .group import Group
from .package import Package


def calc_unmanaged_packages(
    managed_packages: list[Package], explicitly_installed_packages: list[Package]
) -> list[Package]:
    """Get explicitly installed packages which are not in the imported pacdef groups.

    :return: list of unmanaged packages
    """
    logging.debug(f"{managed_packages=}")
    logging.debug(f"{explicitly_installed_packages=}")

    set_explicit = set(explicitly_installed_packages)
    set_managed = set(managed_packages)

    unmanaged_packages = list(set_explicit - set_managed)
    unmanaged_packages.sort()

    logging.debug(f"{unmanaged_packages=}")

    return unmanaged_packages


def calc_packages_to_install(
    managed_packages: list[Package], installed_packages: list[Package]
) -> list[Package]:
    """Determine which packages must be installed to satisfy the dependencies in the group files.

    :return: list of packages that will be installed
    """
    logging.debug(f"{managed_packages=}")
    logging.debug(f"{installed_packages=}")

    pacdef_only = list(set(managed_packages) - set(installed_packages))
    pacdef_only.sort()

    logging.debug(f"{pacdef_only=}")

    return pacdef_only


def get_groups_matching_arguments(
    args: Arguments, groups: list[Group]
) -> Optional[list[Group]]:
    found_groups = []
    if args.groups is None:
        raise ValueError("no group supplied")
    for name in args.groups:
        try:
            found_groups.append(find_group_by_name(name, groups))
        except FileNotFoundError as err:
            logging.error(err)
            return None
    return found_groups


def find_group_by_name(name: str, groups: list[Group]) -> Group:
    logging.info(f"Searching for group '{name}'")
    for group in groups:
        if group == name:
            logging.info(f"found group under {group.path}")
            return group
    raise FileNotFoundError(f"Did not find the group '{name}'.")


def get_managed_packages(groups: list[Group]) -> list[Package]:
    """Get all packaged that are known to pacdef (i.e. are located in imported group files).

    :return: list of packages
    """
    packages = []
    for group in groups:
        packages.extend(group.packages)
    if len(packages) == 0:
        logging.warning("pacdef does not know any packages.")
    return packages
