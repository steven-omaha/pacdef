from __future__ import annotations

from os import environ
from pathlib import Path
from unittest import mock

from src.pacdef import path
from src.pacdef.config import Config, _get_xdg_config_home
from src.pacdef.constants import PARU


def test_get_xdg_config_home(tmpdir, monkeypatch):
    monkeypatch.delenv("XDG_CONFIG_HOME", raising=False)
    result = _get_xdg_config_home()
    assert result == Path(f'{environ["HOME"]}/.config')

    monkeypatch.setenv("XDG_CONFIG_HOME", str(tmpdir))
    result = _get_xdg_config_home()
    assert result == Path(tmpdir)


def test_get_aur_helper(tmpdir):
    with mock.patch.object(path, "file_exists", lambda x: x == Path("/usr/bin/paru")):
        tmpfile = Path(tmpdir).joinpath("tmp.conf")

        conf = Config(config_file_path=tmpfile)
        assert conf.aur_helper == PARU

        with open(tmpfile, "w") as fd:
            fd.write("some strange content")
        conf = Config(config_file_path=tmpfile)
        assert conf.aur_helper == PARU


#         with open(tmpfile, "w") as fd:
#             fd.write("[misc]\nsomething")
#         conf = Config(config_file_path=tmpfile)
#         assert conf.aur_helper == PARU
#
#         something = "something"
#         with open(tmpfile, "w") as fd:
#             fd.write(f"[misc]\naur_helper={something}")
#         conf = Config(config_file_path=tmpfile)
#         assert conf.aur_helper == Path(something)
#
#         with open(tmpfile, "w") as fd:
#             fd.write("[misc]\naur___hELPer=paru")
#         conf = Config(config_file_path=tmpfile)
#         assert conf.aur_helper == PARU
#
#         with open(tmpfile, "w") as fd:
#             fd.write("[misc]\naur_helper=paru")
#         conf = Config(config_file_path=tmpfile)
#         assert conf.aur_helper.name == PARU.name
#
#         with open(tmpfile, "w") as fd:
#             fd.write("[misc]\naur_helper=/usr/bin/paru")
#         conf = Config(config_file_path=tmpfile)
#         assert conf.aur_helper == PARU


def test__init__(tmpdir, monkeypatch):
    monkeypatch.setenv("XDG_CONFIG_HOME", str(tmpdir))
    groups = Path(tmpdir).joinpath("pacdef/groups")
    conf_file = Path(tmpdir).joinpath("pacdef/pacdef.conf")

    with mock.patch.object(path, "file_exists", lambda x: x == Path("/usr/bin/paru")):
        config = Config()
    aur_helper = Path("/usr/bin/paru")

    assert config.groups_path == groups
    assert config.aur_helper == aur_helper
    assert conf_file.is_file()
