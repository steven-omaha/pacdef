#!/usr/bin/python

from __future__ import annotations

import argparse
import configparser
import logging
import subprocess
import sys
from enum import Enum
from os import environ
from pathlib import Path
from typing import Optional

COMMENT = '#'
PARU = Path('/usr/bin/paru')
VERSION = 'unknown'


def main():
    setup_logger()
    conf = Config()
    args = parse_args()

    if args.action == Actions.clean.value:
        remove_unmanaged_packages(conf)
    elif args.action == Actions.groups.value:
        show_groups(conf)
    elif args.action == Actions.import_.value:
        import_groups(conf, args)
    elif args.action == Actions.remove.value:
        remove_group(conf, args)
    elif args.action == Actions.search.value:
        search_package(conf, args)
    elif args.action == Actions.show.value:
        show_group(conf, args)
    elif args.action == Actions.sync.value:
        install_packages_from_groups(conf)
    elif args.action == Actions.unmanaged.value:
        show_unmanaged_packages(conf)
    elif args.action == Actions.version.value:
        show_version()
    else:
        print('Did not understand what you want me to do')
        sys.exit(1)


def aur_helper_execute(aur_helper: Path, command: list[str]) -> None:
    try:
        subprocess.call([str(aur_helper)] + command)
    except FileNotFoundError:
        print(f'Could not start the AUR helper "{aur_helper}".')
        sys.exit(1)


def get_packages_from_pacdef(conf: Config) -> list[str]:
    packages = []
    for group in conf.groups_path.iterdir():
        content = get_packages_from_group(group)
        packages.extend(content)
    if len(packages) == 0:
        logging.warning('pacdef does not know any groups. Import one.')
    return packages


def get_packages_from_group(group: Path) -> list[str]:
    with open(group, 'r') as fd:
        lines = fd.readlines()
    packages = []
    for line in lines:
        package = get_package_from_line(line)
        if package is not None:
            packages.append(package)
    return packages


def get_package_from_line(line: str) -> Optional[str]:
    before_comment = line.split(COMMENT)[0]
    package_name = before_comment.strip()
    if len(package_name) >= 0:
        return package_name
    else:
        return None


def show_unmanaged_packages(conf: Config) -> None:
    unmanaged_packages = get_unmanaged_packages(conf)
    for package in unmanaged_packages:
        print(package)


def install_packages_from_groups(conf: Config) -> None:
    pacdef_packages = get_packages_from_pacdef(conf)
    installed_packages = get_all_installed_packages()
    to_install = calculate_package_diff(installed_packages, pacdef_packages)
    if len(to_install) == 0:
        print('nothing to do')
    else:
        aur_helper_execute(conf.aur_helper, ['--sync', '--refresh', '--needed'] + to_install)


def calculate_package_diff(system_packages: list[str], pacdef_packages: list[str]) -> list[str]:
    """
    Using a custom repository that contains a different version of a package that is also present in the standard repos
    requires distinguishing which version we want to install. Adding the repo in front of the package name (like
    `panthera/zsh-theme-powerlevel10k`) is understood by at least some AUR helpers (paru). If the string contains a
    slash, we check if the part after the slash is a known package.

    :param system_packages: list of packages known by the system
    :param pacdef_packages: list of packages known by pacdef, optionally with repository prefix
    :return: list of pacdef_packages with repository prefix that are not in installed_packages
    """
    to_install = []
    for package_string in pacdef_packages:
        if '/' in package_string:
            repo, package = package_string.split('/')
        else:
            package = package_string
        if package not in system_packages:
            to_install.append(package_string)
    return to_install


def get_all_installed_packages() -> list[str]:
    installed_packages_all = subprocess.check_output(['pacman', '-Qq']).decode('utf-8')
    installed_packages = installed_packages_all.split('\n')[:-1]  # last entry is zero-length
    return installed_packages


def get_path_from_group_name(conf: Config, group_name: str) -> Path:
    group = conf.groups_path.joinpath(group_name)
    if not file_exists(group):
        raise FileNotFoundError
    return group


def show_group(conf: Config, args: argparse.Namespace) -> None:
    groups_to_show = args.group
    if len(groups_to_show) == 0:
        print('which group do you want to show?')
        sys.exit(1)
    imported_groups_name = get_groups_name(conf)
    for group_name in groups_to_show:
        if group_name not in imported_groups_name:
            print(f"I don't know the group {group_name}.")
            sys.exit(1)
    for group_name in groups_to_show:
        group = get_path_from_group_name(conf, group_name)
        packages = get_packages_from_group(group)
        for package in packages:
            print(package)


def remove_group(conf: Config, args: argparse.Namespace) -> None:
    groups = args.group
    if len(groups) == 0:
        print('nothing to remove')
    found_groups = []
    for group_name in groups:
        group_file = conf.groups_path.joinpath(group_name)
        if group_file.is_symlink() or file_exists(group_file):
            found_groups.append(group_file)
        else:
            print(f'Did not find the group {group_name}')
            sys.exit(1)
    for path in found_groups:
        path.unlink()


def import_groups(conf: Config, args: argparse.Namespace) -> None:
    files = args.file
    # check if all file-arguments exist before we do anything (be atomic)
    for f in files:
        path = Path(f)
        if not file_exists(path):
            print(f'Cannot import {f}, does not exist')
            sys.exit(1)
    for f in files:
        path = Path(f)
        link_target = conf.groups_path.joinpath(f)
        if file_exists(link_target):
            print(f'{f} already exists, skipping')
        else:
            link_target.symlink_to(path.absolute())


def search_package(conf: Config, args: argparse.Namespace):
    for group in conf.groups_path.iterdir():
        packages = get_packages_from_group(group)
        if args.package in packages:
            print(group.name)
            sys.exit(0)
    else:
        sys.exit(1)


def show_groups(conf: Config) -> None:
    groups = get_groups_name(conf)
    for group in groups:
        print(group)


def get_groups(conf: Config) -> list[Path]:
    groups = [group for group in conf.groups_path.iterdir()]
    groups.sort()
    logging.debug(f'{groups=}')
    return groups


def get_groups_name(conf: Config) -> list[str]:
    groups = [group.name for group in get_groups(conf)]
    logging.info(f'{groups=}')
    return groups


def remove_unmanaged_packages(conf: Config) -> None:
    unmanaged_packages = get_unmanaged_packages(conf)
    if len(unmanaged_packages) == 0:
        print('nothing to do')
        sys.exit(0)
    print('Would remove the following packages and their dependencies:')
    for package in unmanaged_packages:
        print(package)
    get_user_confirmation()
    aur_helper_execute(conf.aur_helper, ['--remove', '--recursive'] + unmanaged_packages)


def get_user_confirmation() -> None:
    user_input = input('Continue? [y/N] ').lower()
    if len(user_input) > 1 or user_input != 'y':
        sys.exit(0)


def get_unmanaged_packages(conf: Config) -> list[str]:
    pacdef_packages = get_packages_from_pacdef(conf)
    explicitly_installed_packages = get_explicitly_installed_packages()
    unmanaged_packages = [p for p in explicitly_installed_packages if p not in pacdef_packages]
    unmanaged_packages.sort()
    return unmanaged_packages


def get_explicitly_installed_packages() -> list[str]:
    installed_packages_explicit = subprocess.check_output(['pacman', '-Qqe']).decode('utf-8')
    installed_packages_explicit = installed_packages_explicit.split('\n')[:-1]  # last entry is zero-length
    return installed_packages_explicit


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description='a declarative manager of Arch packages')
    subparsers = parser.add_subparsers(dest='action', required=True, metavar='<action>')
    subparsers.add_parser(Actions.clean.value, help='uninstall packages not managed by pacdef')
    subparsers.add_parser(Actions.groups.value, help='show names of imported groups')
    parser_import = subparsers.add_parser(Actions.import_.value, help='import a new group file')
    parser_import.add_argument('file', nargs='+', help='a group file')
    parser_remove = subparsers.add_parser(Actions.remove.value, help='remove previously imported group')
    parser_remove.add_argument('group', nargs='+', help='a previously imported group')
    parser_search = subparsers.add_parser(Actions.search.value, help='show the group containing a package')
    parser_search.add_argument('package', help='the package to search for')
    parser_show_group = subparsers.add_parser(Actions.show.value, help='show packages under an imported group')
    parser_show_group.add_argument('group', nargs='+', help='a previously imported group')
    subparsers.add_parser(Actions.sync.value, help='install packages from all imported groups')
    subparsers.add_parser(Actions.unmanaged.value, help='show explicitly installed packages not managed by pacdef')
    subparsers.add_parser(Actions.version.value, help='show version info')
    args = parser.parse_args()
    return args


def dir_exists(path: Path) -> bool:
    return path.exists() and path.is_dir()


def file_exists(path: Path) -> bool:
    return path.exists() and path.is_file()


class Actions(Enum):
    clean = 'clean'
    groups = 'groups'
    import_ = 'import'
    remove = 'remove'
    search = 'search'
    show = 'show'
    sync = 'sync'
    unmanaged = 'unmanaged'
    version = 'version'


class Config:
    aur_helper: Path
    groups_path: Path

    def __init__(self):
        config_base_dir = self._get_xdg_config_home()

        pacdef_path = config_base_dir.joinpath('pacdef')
        config_file = pacdef_path.joinpath('pacdef.conf')
        self.groups_path = pacdef_path.joinpath('groups')

        if not dir_exists(pacdef_path):
            pacdef_path.mkdir(parents=True)
        if not dir_exists(self.groups_path):
            self.groups_path.mkdir()
        if not file_exists(config_file):
            config_file.touch()

        self.aur_helper = self._get_aur_helper(config_file)
        logging.info(f"{self.aur_helper=}")
        logging.info(f"{self.groups_path=}")

    @staticmethod
    def _get_xdg_config_home() -> Path:
        try:
            config_base = Path(environ['XDG_CONFIG_HOME'])
        except KeyError:
            home = Path.home()
            config_base = home.joinpath('.config')
        return config_base

    @classmethod
    def _get_aur_helper(cls, config_file: Path) -> Path:
        config = configparser.ConfigParser()

        try:
            config.read(config_file)
        except configparser.ParsingError as e:
            logging.error(f'Could not parse the config: {e}')

        try:
            aur_helper = Path(config['misc']['aur_helper'])
        except KeyError:
            logging.warning(f'No AUR helper set. Defaulting to {PARU}')
            aur_helper = PARU
            cls._write_config_stub(config_file)

        if not aur_helper.is_absolute():
            aur_helper = Path('/usr/bin').joinpath(aur_helper)

        if not file_exists(aur_helper):
            raise FileNotFoundError(f'{aur_helper} not found.')
        return aur_helper

    @classmethod
    def _write_config_stub(cls, config_file: Path):
        with open(config_file, 'w') as fd:
            fd.write('[misc]\n')
            fd.write('aur_helper = paru\n')


def setup_logger():
    try:
        level_name = environ['LOGLEVEL']
    except KeyError:
        level_name = 'WARNING'

    level: int = logging.getLevelName(level_name)
    if level < logging.WARNING:
        logging.basicConfig(format='%(levelname)s:%(lineno)d: %(message)s', level=level)
    else:
        logging.basicConfig(format='%(levelname)s: %(message)s', level=level)


def show_version():
    print(f'pacdef, version: {VERSION}')


if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        sys.exit(130)
