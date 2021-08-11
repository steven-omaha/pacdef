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
    pacdef = Pacdef()
    args = parse_args()

    if args.action == Actions.clean.value:
        pacdef.remove_unmanaged_packages()
    elif args.action == Actions.groups.value:
        pacdef.show_groups()
    elif args.action == Actions.import_.value:
        pacdef.import_groups(args)
    elif args.action == Actions.remove.value:
        pacdef.remove_group(args)
    elif args.action == Actions.search.value:
        pacdef.search_package(args)
    elif args.action == Actions.show.value:
        pacdef.show_group(args)
    elif args.action == Actions.sync.value:
        pacdef.install_packages_from_groups()
    elif args.action == Actions.unmanaged.value:
        pacdef.show_unmanaged_packages()
    elif args.action == Actions.version.value:
        show_version()
    else:
        print('Did not understand what you want me to do')
        sys.exit(1)


def get_packages_from_group(group: Path) -> list[str]:
    try:
        with open(group, 'r') as fd:
            lines = fd.readlines()
    except (IOError, FileNotFoundError):
        logging.error(f'Could not read group file {group.absolute()}')
        sys.exit(1)
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


def remove_repo_prefix_from_packages(pacdef_packages: list[str]) -> list[str]:
    result = []
    for package_string in pacdef_packages:
        package = remove_repo_prefix_from_package(package_string)
        result.append(package)
    return result


def remove_repo_prefix_from_package(package_string: str) -> str:
    """
    Takes a string in the form `repository/package` and returns the package name only. Returns `package_string` if it
    does not contain a repository prefix.
    :param package_string: string of a single package, optionally starting with a repository prefix
    :return: package name
    """
    if '/' in package_string:
        repo, package = package_string.split('/')
    else:
        package = package_string
    return package


def calculate_package_diff(
        system_packages: list[str], pacdef_packages: list[str], keep_prefix: bool = False
) -> tuple[list[str], list[str]]:
    """
    Using a custom repository that contains a different version of a package that is also present in the standard repos
    requires distinguishing which version we want to install. Adding the repo in front of the package name (like
    `panthera/zsh-theme-powerlevel10k`) is understood by at least some AUR helpers (paru). If the string contains a
    slash, we check if the part after the slash is a known package.

    :param system_packages: list of packages known by the system
    :param pacdef_packages: list of packages known by pacdef, optionally with repository prefix
    :param keep_prefix: if a repository prefix exists in a pacdef package, keep it (default: False)
    :return: 2-tuple: list of packages exclusively in argument 1, list of packages exclusively in argument 2
    """
    system_only = []
    pacdef_only = []
    pacdef_packages_without_prefix = remove_repo_prefix_from_packages(pacdef_packages)
    for package in system_packages:
        if package not in pacdef_packages_without_prefix:
            system_only.append(package)
    for package, package_without_prefix in zip(pacdef_packages, pacdef_packages_without_prefix):
        if package_without_prefix not in system_packages:
            if keep_prefix:
                pacdef_only.append(package)
            else:
                pacdef_only.append(package_without_prefix)
    return system_only, pacdef_only


def get_path_from_group_name(conf: Config, group_name: str) -> Path:
    group = conf.groups_path.joinpath(group_name)
    if not file_exists(group):
        if group.is_symlink():
            logging.warning(f'found group {group.absolute()}, but it is a broken symlink')
        else:
            raise FileNotFoundError
    return group


def get_user_confirmation() -> None:
    user_input = input('Continue? [y/N] ').lower()
    if len(user_input) > 1 or user_input != 'y':
        sys.exit(0)


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
    aur_helper: AURHelper
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
            config_base_dir = Path(environ['XDG_CONFIG_HOME'])
        except KeyError:
            home = Path.home()
            config_base_dir = home.joinpath('.config')
        logging.debug(f'{config_base_dir=}')
        return config_base_dir

    @classmethod
    def _get_aur_helper(cls, config_file: Path) -> AURHelper:
        config = configparser.ConfigParser()

        try:
            config.read(config_file)
        except configparser.ParsingError as e:
            logging.error(f'Could not parse the config: {e}')

        try:
            path = Path(config['misc']['aur_helper'])
        except KeyError:
            logging.warning(f'No AUR helper set. Defaulting to {PARU}')
            path = PARU
            cls._write_config_stub(config_file)

        aur_helper = AURHelper(path)
        return aur_helper

    @classmethod
    def _write_config_stub(cls, config_file: Path):
        logging.info(f'Created config stub under {config_file}')
        with open(config_file, 'w') as fd:
            fd.write('[misc]\n')
            fd.write('aur_helper = paru\n')


class AURHelper:
    class _Switches(Enum):
        install = ['--sync', '--refresh', '--needed']
        remove = ['--remove', '--recursive']
        installed_packages = ['--query', '--quiet']
        explicitly_installed_packages = ['--query', '--quiet', '--explicit']

    _path: Path

    def __init__(self, path: Path):
        if not path.is_absolute():
            path = Path('/usr/bin').joinpath(path)
        if not file_exists(path):
            raise FileNotFoundError(f'{path} not found.')
        self._path = path

    def _execute(self, command: list[str]) -> None:
        try:
            subprocess.call([str(self._path)] + command)
        except FileNotFoundError:
            print(f'Could not start the AUR helper "{self._path}".')
            sys.exit(1)

    def _check_output(self, query: list[str]) -> list[str]:
        command = [str(self._path)] + query
        result = subprocess.check_output(command).decode('utf-8')
        result_list = result.split('\n')[:-1]  # last entry is zero-length
        return result_list

    def install(self, packages: list[str]) -> None:
        command: list[str] = self._Switches.install.value + packages
        self._execute(command)

    def remove(self, packages: list[str]) -> None:
        command: list[str] = self._Switches.remove.value + packages
        self._execute(command)

    def get_all_installed_packages(self) -> list[str]:
        return self._check_output(self._Switches.installed_packages.value)

    def get_explicitly_installed_packages(self) -> list[str]:
        return self._check_output(self._Switches.explicitly_installed_packages.value)


class Pacdef:
    _conf: Config

    def __init__(self):
        self._conf = Config()

    def remove_unmanaged_packages(self):
        unmanaged_packages = self._get_unmanaged_packages()
        if len(unmanaged_packages) == 0:
            print('nothing to do')
            sys.exit(0)
        print('Would remove the following packages and their dependencies:')
        for package in unmanaged_packages:
            print(package)
        get_user_confirmation()
        self._conf.aur_helper.remove(unmanaged_packages)

    def show_groups(self):
        groups = self._get_group_names()
        for group in groups:
            print(group)

    def import_groups(self, args: argparse.Namespace) -> None:
        files = args.file
        # check if all file-arguments exist before we do anything (be atomic)
        for f in files:
            path = Path(f)
            if not file_exists(path):
                print(f'Cannot import {f}, does not exist')
                sys.exit(1)
        for f in files:
            path = Path(f)
            link_target = self._conf.groups_path.joinpath(f)
            if file_exists(link_target):
                print(f'{f} already exists, skipping')
            else:
                link_target.symlink_to(path.absolute())

    def remove_group(self, args: argparse.Namespace) -> None:
        groups = args.group
        if len(groups) == 0:
            print('nothing to remove')
        found_groups = []
        for group_name in groups:
            group_file = self._conf.groups_path.joinpath(group_name)
            if group_file.is_symlink() or file_exists(group_file):
                found_groups.append(group_file)
            else:
                print(f'Did not find the group {group_name}')
                sys.exit(1)
        for path in found_groups:
            path.unlink()

    def search_package(self, args: argparse.Namespace):
        for group in self._conf.groups_path.iterdir():
            packages = get_packages_from_group(group)
            if args.package in packages:
                print(group.name)
                sys.exit(0)
        else:
            sys.exit(1)

    def show_group(self, args: argparse.Namespace) -> None:
        groups_to_show = args.group
        if len(groups_to_show) == 0:
            print('which group do you want to show?')
            sys.exit(1)
        imported_groups_name = self._get_group_names()
        for group_name in groups_to_show:
            if group_name not in imported_groups_name:
                print(f"I don't know the group {group_name}.")
                sys.exit(1)
        for group_name in groups_to_show:
            group = get_path_from_group_name(self._conf, group_name)
            packages = get_packages_from_group(group)
            for package in packages:
                print(package)

    def install_packages_from_groups(self) -> None:
        to_install = self._calculate_packages_to_install()
        if len(to_install) == 0:
            print('nothing to do')
            sys.exit(0)
        print('Would install the following packages:')
        for package in to_install:
            print(package)
        get_user_confirmation()
        self._conf.aur_helper.install(to_install)

    def show_unmanaged_packages(self) -> None:
        unmanaged_packages = self._get_unmanaged_packages()
        for package in unmanaged_packages:
            print(package)

    def _calculate_packages_to_install(self) -> list[str]:
        pacdef_packages = self._get_managed_packages()
        installed_packages = self._conf.aur_helper.get_all_installed_packages()
        _, pacdef_only = calculate_package_diff(installed_packages, pacdef_packages, keep_prefix=True)
        return pacdef_only

    def _get_unmanaged_packages(self) -> list[str]:
        managed_packages = self._get_managed_packages()
        explicitly_installed_packages = self._conf.aur_helper.get_explicitly_installed_packages()
        unmanaged_packages, _ = calculate_package_diff(explicitly_installed_packages, managed_packages)
        unmanaged_packages.sort()
        return unmanaged_packages

    def _get_managed_packages(self) -> list[str]:
        packages = []
        for group in self._conf.groups_path.iterdir():
            content = get_packages_from_group(group)
            packages.extend(content)
        if len(packages) == 0:
            logging.warning('pacdef does not know any groups. Import one.')
        return packages

    def _get_group_names(self) -> list[str]:
        groups = [group.name for group in self._get_groups()]
        logging.info(f'{groups=}')
        return groups

    def _get_groups(self) -> list[Path]:
        groups = [group for group in self._conf.groups_path.iterdir()]
        groups.sort()
        for group in groups:
            if group.is_dir():
                logging.warning(f'found directory {group} in {self._conf.groups_path}')
            if group.is_symlink() and not group.exists():
                logging.warning(f'found group {group}, but it is a broken symlink')
            if not group.is_symlink() and group.is_file():
                logging.warning(f'found group {group}, but it is not a symlink')
        logging.debug(f'{groups=}')
        return groups


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
