import configparser
import subprocess
from os import environ
from pathlib import Path
from unittest import mock

import builtins
import pytest

import pacdef


def test_dir_exists(tmpdir):
    tmpdir = Path(tmpdir)
    tmpfile = tmpdir.joinpath('tmpfile')
    tmpfile.touch()
    assert not pacdef.dir_exists(tmpfile)
    tmpfile.unlink()
    assert not pacdef.dir_exists(tmpfile)
    assert pacdef.dir_exists(tmpdir)


def test_file_exists(tmpdir):
    tmpfile = Path(tmpdir).joinpath('tmpfile')
    tmpfile.touch()
    assert pacdef.file_exists(tmpfile)
    tmpfile.unlink()
    assert not pacdef.file_exists(tmpfile)
    tmpfile.mkdir()
    assert not pacdef.file_exists(tmpfile)


class TestConfig:
    @staticmethod
    def test__get_xdg_config_home(tmpdir, monkeypatch):
        monkeypatch.delenv('XDG_CONFIG_HOME', raising=False)
        result = pacdef.Config._get_xdg_config_home()
        assert result == Path(f'{environ["HOME"]}/.config')

        monkeypatch.setenv('XDG_CONFIG_HOME', str(tmpdir))
        result = pacdef.Config._get_xdg_config_home()
        assert result == Path(tmpdir)

    @staticmethod
    def test__get_aur_helper(tmpdir):
        with mock.patch.object(pacdef, 'file_exists', lambda x: x == Path('/usr/bin/paru')):
            tmpfile = Path(tmpdir).joinpath('tmp.conf')

            helper = pacdef.Config._get_aur_helper(tmpfile)
            assert helper._path == pacdef.PARU

            with open(tmpfile, 'w') as fd:
                fd.write('some strange content')
            helper = pacdef.Config._get_aur_helper(tmpfile)
            assert helper._path == pacdef.PARU

            with open(tmpfile, 'w') as fd:
                fd.write('[misc]\nsomething')
            helper = pacdef.Config._get_aur_helper(tmpfile)
            assert helper._path == pacdef.PARU

            with open(tmpfile, 'w') as fd:
                fd.write('[misc]\naur_helper=something')
            with pytest.raises(FileNotFoundError):
                pacdef.Config._get_aur_helper(tmpfile)

            with open(tmpfile, 'w') as fd:
                fd.write('[misc]\naur___hELPer=paru')
            helper = pacdef.Config._get_aur_helper(tmpfile)
            assert helper._path == pacdef.PARU

            with open(tmpfile, 'w') as fd:
                fd.write('[misc]\naur_helper=paru')
            helper = pacdef.Config._get_aur_helper(tmpfile)
            assert helper._path == pacdef.PARU

            with open(tmpfile, 'w') as fd:
                fd.write('[misc]\naur_helper=/usr/bin/paru')
            helper = pacdef.Config._get_aur_helper(tmpfile)
            assert helper._path == pacdef.PARU

    @staticmethod
    def test__write_config_stub(tmpdir):
        tmpfile = Path('/a')
        with pytest.raises(PermissionError):
            pacdef.Config._write_config_stub(tmpfile)

        tmpfile = Path(tmpdir).joinpath('pacdef.conf')
        pacdef.Config._write_config_stub(tmpfile)
        config = configparser.ConfigParser()
        config.read(tmpfile)
        assert config['misc']['aur_helper'] == 'paru'

    @staticmethod
    def test___init__(tmpdir, monkeypatch):
        monkeypatch.setenv('XDG_CONFIG_HOME', str(tmpdir))
        groups = Path(tmpdir).joinpath('pacdef/groups')
        conf_file = Path(tmpdir).joinpath('pacdef/pacdef.conf')

        with mock.patch.object(pacdef, 'file_exists', lambda x: x == Path('/usr/bin/paru')):
            config = pacdef.Config()
        aur_helper = Path('/usr/bin/paru')

        assert config.groups_path == groups
        assert config.aur_helper._path == aur_helper
        assert conf_file.is_file()


@pytest.mark.parametrize('user_input', ['Y', 'y'])
def test_get_user_confirmation_continue(user_input):
    with mock.patch.object(builtins, 'input', lambda _: user_input):
        assert pacdef.get_user_confirmation() is None


@pytest.mark.parametrize('user_input', ['', 'n', 'N', 'asd#!|^l;"f'])
def test_get_user_confirmation_exit(user_input):
    with mock.patch.object(builtins, 'input', lambda _: user_input):
        with pytest.raises(SystemExit):
            pacdef.get_user_confirmation()


@pytest.mark.parametrize(
    'pacdef_packages, installed_packages, expected_result',
    [
        (['base'], [], ['base']),
        ([], ['base'], []),
        ([], [], []),
        (['base'], ['base'], []),
        (['repo/base'], [], ['repo/base']),
        (['repo/base'], ['base'], []),
    ]
)
def test_calculate_packages_to_install(pacdef_packages, installed_packages, expected_result):
    with mock.patch.object(pacdef, 'get_packages_from_pacdef', lambda _: pacdef_packages):
        aur_helper = object.__new__(pacdef.AURHelper)
        conf = object.__new__(pacdef.Config)
        conf.aur_helper = aur_helper
        with mock.patch.object(aur_helper, 'get_all_installed_packages', lambda: installed_packages):
            result = pacdef.calculate_packages_to_install(conf)
            assert result == expected_result


@pytest.mark.parametrize(
    'pacdef_packages, installed_packages, expected_result',
    [
        (['base'], [], []),
        ([], ['base'], ['base']),
        ([], [], []),
        (['base'], ['base'], []),
        (['repo/base'], [], []),
        (['repo/base'], ['base'], []),
    ]
)
def test_get_unmanaged_packages(pacdef_packages, installed_packages, expected_result):
    with mock.patch.object(pacdef, 'get_packages_from_pacdef', lambda _: pacdef_packages):
        aur_helper = object.__new__(pacdef.AURHelper)
        conf = object.__new__(pacdef.Config)
        conf.aur_helper = aur_helper
        with mock.patch.object(aur_helper, 'get_explicitly_installed_packages', lambda: installed_packages):
            result = pacdef.get_unmanaged_packages(conf)
            assert result == expected_result


@pytest.mark.skipif(not Path('/usr/bin/pacman').exists(), reason='pacman not found. That\'s not an Arch installation.')
def test_get_all_installed_packages_arch():
    instance = pacdef.AURHelper(pacdef.PARU)
    result = instance.get_all_installed_packages()
    assert type(result) == list
    assert len(result) > 0
    for item in result:
        assert type(item) == str
        assert len(item) > 0


def test_get_path_from_group_name(tmpdir):
    conf = pacdef.Config.__new__(pacdef.Config)
    conf.groups_path = Path(tmpdir)
    exists = Path(conf.groups_path.joinpath('exists'))
    exists.touch()
    result = pacdef.get_path_from_group_name(conf, exists.name)
    assert result == exists

    with pytest.raises(FileNotFoundError):
        pacdef.get_path_from_group_name(conf, "does not exist")

    symlink = conf.groups_path.joinpath('symlink')
    symlink.symlink_to(exists)
    result = pacdef.get_path_from_group_name(conf, symlink.name)
    assert result == symlink

    exists.unlink()
    result = pacdef.get_path_from_group_name(conf, symlink.name)
    assert result == symlink


def test_install_packages_from_groups_none():
    with mock.patch.object(pacdef, 'calculate_packages_to_install', lambda _: []):
        with pytest.raises(SystemExit):
            # noinspection PyTypeChecker
            pacdef.install_packages_from_groups(None)


@pytest.mark.parametrize(
    'packages',
    [
        ['neovim'],
        ['neovim', 'python'],
        ['neovim', 'repo/python'],
    ]
)
def test_install_packages_from_groups_for_packages(packages):
    def check_valid(_, args: list[str]) -> None:
        for arg in args:
            assert arg in packages

    with mock.patch.object(pacdef.AURHelper, 'install', check_valid):
        aur_helper = pacdef.AURHelper(pacdef.PARU)
        conf = pacdef.Config.__new__(pacdef.Config)
        conf.aur_helper = aur_helper
        with mock.patch.object(pacdef, 'calculate_packages_to_install', lambda _: packages):
            with mock.patch.object(pacdef, 'get_user_confirmation', lambda: None):
                pacdef.install_packages_from_groups(conf)


def test_remove_unmanaged_packages_none():
    conf = pacdef.Config.__new__(pacdef.Config)
    with mock.patch.object(pacdef, 'get_unmanaged_packages', lambda _: []):
        with pytest.raises(SystemExit):
            pacdef.remove_unmanaged_packages(conf)


@pytest.mark.parametrize('packages', [
    [
        ['neovim'],
        ['neovim', 'python'],
    ]
])
def test_remove_unmanaged_packages_for_packages(packages):
    def check_valid(_, args: list[str]) -> None:
        for arg in args:
            assert arg in packages

    with mock.patch.object(pacdef.AURHelper, 'remove', check_valid):
        aur_helper = pacdef.AURHelper(pacdef.PARU)
        conf = pacdef.Config.__new__(pacdef.Config)
        conf.aur_helper = aur_helper
        with mock.patch.object(pacdef, 'get_unmanaged_packages', lambda _: packages):
            with mock.patch.object(pacdef, 'get_user_confirmation', lambda: None):
                pacdef.remove_unmanaged_packages(conf)


class TestAURHelper:
    @staticmethod
    def test___init__():
        name = Path(pacdef.PARU.name)
        instance = pacdef.AURHelper(name)
        assert instance._path == pacdef.PARU

        with pytest.raises(FileNotFoundError):
            pacdef.AURHelper(Path('does not exist'))

    def test__execute(self):
        def check_valid(command_run: list[str]):
            assert command_run[0] == str(instance._path)
            assert command_run[1:] == command_given

        command_given: list[str] = ['some', 'command']
        instance = pacdef.AURHelper(pacdef.PARU)
        with mock.patch.object(subprocess, 'call', check_valid):
            instance._execute(command_given)

        instance = object.__new__(pacdef.AURHelper)
        instance._path = Path('does not exist')
        with pytest.raises(SystemExit):
            instance._execute(command_given)

    @staticmethod
    def check_switches_valid(command: list[str], switches: list[str]):
        for switch in switches:
            assert switch in command

    @staticmethod
    def check_switches_before_packages(command: list[str], switches: list[str]):
        switch_positions: dict[str, int] = {}
        for switch in switches:
            for position, arg in enumerate(command):
                if arg == switch:
                    switch_positions[switch] = position
                    break
            else:
                raise AssertionError
        assert max(switch_positions.values()) == len(switches) - 1

    @staticmethod
    def check_packages_present(command: list[str], packages: list[str]):
        for package in packages:
            assert package in command

    @classmethod
    @pytest.mark.parametrize(
        'packages',
        [
            ['neovim'],
            ['neovim', 'python'],
            ['neovim', 'repo/python'],
        ]
    )
    def test_install(cls, packages):
        def check_valid(command):
            cls.check_switches_valid(command, pacdef.AURHelper._Switches.install.value)
            cls.check_switches_before_packages(command, pacdef.AURHelper._Switches.install.value)
            cls.check_packages_present(command, packages)

        with mock.patch.object(pacdef.AURHelper, '_execute', check_valid):
            instance = pacdef.AURHelper(pacdef.PARU)
            instance.install(packages)

    @classmethod
    @pytest.mark.parametrize(
        'packages',
        [
            ['neovim'],
            ['neovim', 'python'],
        ]
    )
    def test_remove(cls, packages):
        def check_valid(command):
            cls.check_switches_valid(command, pacdef.AURHelper._Switches.remove.value)
            cls.check_switches_before_packages(command, pacdef.AURHelper._Switches.remove.value)
            cls.check_packages_present(command, packages)

        with mock.patch.object(pacdef.AURHelper, '_execute', check_valid):
            instance = pacdef.AURHelper(pacdef.PARU)
            instance.remove(packages)
