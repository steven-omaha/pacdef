import configparser
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
            assert helper == pacdef.PARU

            with open(tmpfile, 'w') as fd:
                fd.write('some strange content')
            helper = pacdef.Config._get_aur_helper(tmpfile)
            assert pacdef.PARU == helper

            with open(tmpfile, 'w') as fd:
                fd.write('[misc]\nsomething')
            helper = pacdef.Config._get_aur_helper(tmpfile)
            assert pacdef.PARU == helper

            with open(tmpfile, 'w') as fd:
                fd.write('[misc]\naur_helper=something')
            with pytest.raises(FileNotFoundError):
                pacdef.Config._get_aur_helper(tmpfile)

            with open(tmpfile, 'w') as fd:
                fd.write('[misc]\naur___hELPer=paru')
            helper = pacdef.Config._get_aur_helper(tmpfile)
            assert helper == pacdef.PARU

            with open(tmpfile, 'w') as fd:
                fd.write('[misc]\naur_helper=paru')
            helper = pacdef.Config._get_aur_helper(tmpfile)
            assert helper == pacdef.PARU

            with open(tmpfile, 'w') as fd:
                fd.write('[misc]\naur_helper=/usr/bin/paru')
            helper = pacdef.Config._get_aur_helper(tmpfile)
            assert helper == pacdef.PARU

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
        assert config.aur_helper == aur_helper
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
        with mock.patch.object(pacdef, 'get_all_installed_packages', lambda: installed_packages):
            # noinspection PyTypeChecker
            result = pacdef.calculate_packages_to_install(None)
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
        with mock.patch.object(pacdef, 'get_explicitly_installed_packages', lambda: installed_packages):
            # noinspection PyTypeChecker
            result = pacdef.get_unmanaged_packages(None)
            assert result == expected_result


@pytest.mark.skipif(not pacdef.PACMAN.exists(), reason='pacman not found. That\'s not an Arch installation.')
def test_get_all_installed_packages_arch():
    result = pacdef.get_all_installed_packages()
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
    def check_valid(aur_helper: Path, args: list[str]):
        check_aur_helper(aur_helper)
        check_switches_valid(args)
        check_switches_before_packages(args)
        check_packages_present(args)

    def check_aur_helper(aur_helper: Path):
        assert aur_helper == pacdef.PARU

    def check_switches_valid(args: list[str]):
        for switch in switches:
            assert switch in args

    def check_switches_before_packages(args: list[str]):
        switch_positions: dict[str, int] = {}
        for switch in switches:
            for position, arg in enumerate(args):
                if arg == switch:
                    switch_positions[switch] = position
                    break
            else:
                raise AssertionError
        assert max(switch_positions.values()) == len(switches) - 1

    def check_packages_present(args: list[str]):
        for package in packages:
            assert package in args

    switches = ['--sync', '--refresh', '--needed']
    conf = pacdef.Config.__new__(pacdef.Config)
    conf.aur_helper = pacdef.PARU

    with mock.patch.object(pacdef, 'calculate_packages_to_install', lambda _: packages):
        with mock.patch.object(pacdef, 'aur_helper_execute', check_valid):
            pacdef.install_packages_from_groups(conf)
