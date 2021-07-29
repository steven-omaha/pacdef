from os import environ
from pathlib import Path
from unittest import mock

import builtins
import pytest

from pacdef import PARU, Config, dir_exists, file_exists, get_user_confirmation


def test_dir_exists(tmpdir):
    tmpdir = Path(tmpdir)
    tmpfile = tmpdir.joinpath('tmpfile')
    tmpfile.touch()
    assert not dir_exists(tmpfile)
    tmpfile.unlink()
    assert not dir_exists(tmpfile)
    assert dir_exists(tmpdir)


def test_file_exists(tmpdir):
    tmpfile = Path(tmpdir).joinpath('tmpfile')
    tmpfile.touch()
    assert file_exists(tmpfile)
    tmpfile.unlink()
    assert not file_exists(tmpfile)
    tmpfile.mkdir()
    assert not file_exists(tmpfile)


class TestConfig:
    @staticmethod
    def test__get_xdg_config_home(tmpdir, monkeypatch):
        monkeypatch.delenv('XDG_CONFIG_HOME', raising=False)
        result = Config._get_xdg_config_home()
        assert result == Path(f'{environ["HOME"]}/.config')

        monkeypatch.setenv('XDG_CONFIG_HOME', str(tmpdir))
        result = Config._get_xdg_config_home()
        assert result == Path(tmpdir)

    @staticmethod
    def test__get_aur_helper(tmpdir):
        tmpfile = Path(tmpdir).joinpath('tmp.conf')

        helper = Config._get_aur_helper(tmpfile)
        assert helper == PARU

        with open(tmpfile, 'w') as fd:
            fd.write('some strange content')
        helper = Config._get_aur_helper(tmpfile)
        assert PARU == helper

        with open(tmpfile, 'w') as fd:
            fd.write('[misc]\nsomething')
        helper = Config._get_aur_helper(tmpfile)
        assert PARU == helper

        with open(tmpfile, 'w') as fd:
            fd.write('[misc]\naur_helper=something')
        with pytest.raises(FileNotFoundError):
            Config._get_aur_helper(tmpfile)

        with open(tmpfile, 'w') as fd:
            fd.write('[misc]\naur___hELPer=paru')
        helper = Config._get_aur_helper(tmpfile)
        assert helper == PARU

        with open(tmpfile, 'w') as fd:
            fd.write('[misc]\naur_helper=paru')
        helper = Config._get_aur_helper(tmpfile)
        assert helper == PARU

        with open(tmpfile, 'w') as fd:
            fd.write('[misc]\naur_helper=/usr/bin/paru')
        helper = Config._get_aur_helper(tmpfile)
        assert helper == PARU

    @staticmethod
    def test___init__(tmpdir, monkeypatch):
        monkeypatch.setenv('XDG_CONFIG_HOME', str(tmpdir))
        groups = Path(tmpdir).joinpath('pacdef/groups')
        conf_file = Path(tmpdir).joinpath('pacdef/pacdef.conf')

        config = Config()
        aur_helper = Path('/usr/bin/paru')

        assert config.groups_path == groups
        assert config.aur_helper == aur_helper
        assert conf_file.is_file()


@pytest.mark.parametrize('user_input', ['Y', 'y'])
def test_get_user_confirmation_continue(user_input):
    with mock.patch.object(builtins, 'input', lambda _: user_input):
        assert get_user_confirmation() is None


@pytest.mark.parametrize('user_input', ['', 'n', 'N', 'asd#!|^l;"f'])
def test_get_user_confirmation_exit(user_input):
    with mock.patch.object(builtins, 'input', lambda _: user_input):
        with pytest.raises(SystemExit):
            get_user_confirmation()
