import configparser
import subprocess
from os import environ
from pathlib import Path
from unittest import mock

import builtins
import pytest

import pacdef

PACMAN = Path('/usr/bin/pacman')
PACMAN_EXISTS = PACMAN.exists()
REASON_NOT_ARCH = 'pacman not found. That\'s not an Arch installation.'


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


@pytest.mark.skipif(not PACMAN_EXISTS, reason=REASON_NOT_ARCH)
def test_get_all_installed_packages_arch():
    instance = pacdef.AURHelper(PACMAN)  # pacman is good enough for the test case
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


class TestAURHelper:
    @staticmethod
    @pytest.mark.skipif(not PACMAN_EXISTS, reason=REASON_NOT_ARCH)
    def test___init__():
        name = Path(PACMAN.name)  # pacman is good enough for the test case
        instance = pacdef.AURHelper(name)
        assert instance._path == PACMAN

        with pytest.raises(FileNotFoundError):
            pacdef.AURHelper(Path('does not exist'))

    def test__execute(self):
        def check_valid(command_run: list[str]):
            assert command_run[0] == str(instance._path)
            assert command_run[1:] == command_given

        command_given: list[str] = ['some', 'command']
        instance = object.__new__(pacdef.AURHelper)
        instance._path = pacdef.PARU
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


class TestPacdef:
    def _test_basic_printing_function(self, test_method: str, patched_method: str, capsys):
        instance = self._get_test_instance()
        method = instance.__getattribute__(test_method)
        with mock.patch.object(instance, patched_method, lambda: None):
            with pytest.raises(TypeError):
                method()

        with mock.patch.object(instance, patched_method, lambda: []):
            method()
        out, err = capsys.readouterr()
        assert len(out) == 0
        assert len(err) == 0

        packages = ['base']
        with mock.patch.object(instance, patched_method, lambda: packages):
            method()
        out, err = capsys.readouterr()
        for package in packages:
            assert package in out
        assert len(err) == 0

        packages = ['base', 'python']
        with mock.patch.object(instance, patched_method, lambda: packages):
            method()
        out, err = capsys.readouterr()
        for package in packages:
            assert package in out
        assert len(err) == 0

    @staticmethod
    def _get_test_instance() -> pacdef.Pacdef:
        instance = object.__new__(pacdef.Pacdef)
        conf = object.__new__(pacdef.Config)
        aur_helper = object.__new__(pacdef.AURHelper)
        conf.aur_helper = aur_helper
        instance._conf = conf
        return instance

    @classmethod
    def test_remove_unmanaged_packages_none(cls):
        instance = cls._get_test_instance()
        with mock.patch.object(instance, '_get_unmanaged_packages', lambda: []):
            with pytest.raises(SystemExit):
                instance.remove_unmanaged_packages()

    @classmethod
    @pytest.mark.parametrize('packages', [
        [
            ['neovim'],
            ['neovim', 'python'],
        ]
    ])
    def test_remove_unmanaged_packages_for_packages(cls, packages):
        def check_valid(_, args: list[str]) -> None:
            for arg in args:
                assert arg in packages

        instance = cls._get_test_instance()
        with mock.patch.object(instance._conf.aur_helper, 'remove', check_valid):
            with mock.patch.object(instance, '_get_unmanaged_packages', lambda: packages):
                with mock.patch.object(pacdef, 'get_user_confirmation', lambda: None):
                    instance.remove_unmanaged_packages()

    def test_show_groups(self, capsys):
        self._test_basic_printing_function('show_groups', '_get_group_names', capsys)

    def test_import_groups(self):
        pass  # TODO

    def test_remove_group(self):
        pass  # TODO

    def test_search_package(self):
        pass  # TODO

    def test_show_group(self):
        pass  # TODO

    @classmethod
    def test_install_packages_from_groups_none(cls):
        instance = cls._get_test_instance()
        with mock.patch.object(instance, '_calculate_packages_to_install', lambda: []):
            with pytest.raises(SystemExit):
                instance.install_packages_from_groups()

    @classmethod
    @pytest.mark.parametrize(
        'packages',
        [
            ['neovim'],
            ['neovim', 'python'],
            ['neovim', 'repo/python'],
        ]
    )
    def test_install_packages_from_groups_for_packages(cls, packages):
        def check_valid(_, args: list[str]) -> None:
            for arg in args:
                assert arg in packages

        instance = cls._get_test_instance()
        with mock.patch.object(instance._conf.aur_helper, 'install', check_valid):
            with mock.patch.object(instance, '_calculate_packages_to_install', lambda: packages):
                with mock.patch.object(pacdef, 'get_user_confirmation', lambda: None):
                    instance.install_packages_from_groups()

    def test_show_unmanaged_packages(self, capsys):
        self._test_basic_printing_function('show_unmanaged_packages', '_get_unmanaged_packages', capsys)

    @classmethod
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
    def test__calculate_packages_to_install(cls, pacdef_packages, installed_packages, expected_result):
        instance = cls._get_test_instance()
        with mock.patch.object(instance, '_get_managed_packages', lambda: pacdef_packages):
            with mock.patch.object(instance._conf.aur_helper, 'get_all_installed_packages', lambda: installed_packages):
                result = instance._calculate_packages_to_install()
                assert result == expected_result

    @classmethod
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
    def test_get_unmanaged_packages(cls, pacdef_packages, installed_packages, expected_result):
        instance = cls._get_test_instance()
        with mock.patch.object(instance, '_get_managed_packages', lambda: pacdef_packages):
            with mock.patch.object(instance._conf.aur_helper, 'get_explicitly_installed_packages',
                                   lambda: installed_packages):
                result = instance._get_unmanaged_packages()
                assert result == expected_result

    def test__get_managed_packages(self):
        pass  # TODO

    def test__get_group_names(self):
        pass  # TODO

    def test__get_groups(self):
        pass  # TODO


def test_get_packages_from_group():
    pass  # TODO


def test_get_package_from_line():
    pass  # TODO


def test_remove_repo_prefix_from_packages():
    pass  # TODO


def test_remove_repo_prefix_from_package():
    pass  # TODO


@pytest.mark.parametrize(
    'pacdef_packages, system_packages, pacdef_only, system_only',
    [
        (['base'], [], ['base'], []),
        ([], ['base'], [], ['base']),
        ([], [], [], []),
        (['base'], ['base'], [], []),
        (['repo/base'], ['base'], [], []),
        (['repo/base'], [], ['base'], []),
    ]
)
def test_calculate_package_diff_keep_prefix_no(pacdef_packages, system_packages, pacdef_only, system_only):
    system_result, pacdef_result = pacdef.calculate_package_diff(system_packages, pacdef_packages, keep_prefix=False)
    assert system_result == system_only
    assert pacdef_result == pacdef_only


@pytest.mark.parametrize(
    'pacdef_packages, system_packages, pacdef_only, system_only',
    [
        (['base'], [], ['base'], []),
        ([], ['base'], [], ['base']),
        ([], [], [], []),
        (['base'], ['base'], [], []),
        (['repo/base'], ['base'], [], []),
        (['repo/base'], [], ['repo/base'], []),
    ]
)
def test_calculate_package_diff_keep_prefix_yes(pacdef_packages, system_packages, pacdef_only, system_only):
    system_result, pacdef_result = pacdef.calculate_package_diff(system_packages, pacdef_packages, keep_prefix=True)
    assert system_result == system_only
    assert pacdef_result == pacdef_only
