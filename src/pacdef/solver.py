import logging

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
