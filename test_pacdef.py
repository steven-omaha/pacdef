from __future__ import annotations

import argparse
import builtins
import logging
import subprocess
from os import environ
from pathlib import Path
from unittest import mock

import pytest

import pacdef

PACMAN = Path("/usr/bin/pacman")
PACMAN_EXISTS = PACMAN.exists()
PARU_EXISTS = pacdef.PARU.exists()
REASON_NOT_ARCH = "pacman not found. That's not an Arch installation."
REASON_PARU_MISSING = "paru not found"
NULL = Path("/dev/null")


def test_dir_exists(tmpdir):
    tmpdir = Path(tmpdir)
    tmpfile = tmpdir.joinpath("tmpfile")
    tmpfile.touch()
    assert not pacdef._dir_exists(tmpfile)
    tmpfile.unlink()
    assert not pacdef._dir_exists(tmpfile)
    assert pacdef._dir_exists(tmpdir)


def test_file_exists(tmpdir):
    tmpfile = Path(tmpdir).joinpath("tmpfile")
    tmpfile.touch()
    assert pacdef._file_exists(tmpfile)
    tmpfile.unlink()
    assert not pacdef._file_exists(tmpfile)
    tmpfile.mkdir()
    assert not pacdef._file_exists(tmpfile)


class TestConfig:
    @staticmethod
    def test__get_xdg_config_home(tmpdir, monkeypatch):
        monkeypatch.delenv("XDG_CONFIG_HOME", raising=False)
        result = pacdef.Config._get_xdg_config_home()
        assert result == Path(f'{environ["HOME"]}/.config')

        monkeypatch.setenv("XDG_CONFIG_HOME", str(tmpdir))
        result = pacdef.Config._get_xdg_config_home()
        assert result == Path(tmpdir)

    @staticmethod
    def test__get_aur_helper(tmpdir):
        with mock.patch.object(
            pacdef, "_file_exists", lambda x: x == Path("/usr/bin/paru")
        ):
            tmpfile = Path(tmpdir).joinpath("tmp.conf")

            conf = pacdef.Config(config_file_path=tmpfile)
            assert conf.aur_helper == pacdef.PARU

            with open(tmpfile, "w") as fd:
                fd.write("some strange content")
            conf = pacdef.Config(config_file_path=tmpfile)
            assert conf.aur_helper == pacdef.PARU

            with open(tmpfile, "w") as fd:
                fd.write("[misc]\nsomething")
            conf = pacdef.Config(config_file_path=tmpfile)
            assert conf.aur_helper == pacdef.PARU

            something = "something"
            with open(tmpfile, "w") as fd:
                fd.write(f"[misc]\naur_helper={something}")
            conf = pacdef.Config(config_file_path=tmpfile)
            assert conf.aur_helper == Path(something)

            with open(tmpfile, "w") as fd:
                fd.write("[misc]\naur___hELPer=paru")
            conf = pacdef.Config(config_file_path=tmpfile)
            assert conf.aur_helper == pacdef.PARU

            with open(tmpfile, "w") as fd:
                fd.write("[misc]\naur_helper=paru")
            conf = pacdef.Config(config_file_path=tmpfile)
            assert conf.aur_helper.name == pacdef.PARU.name

            with open(tmpfile, "w") as fd:
                fd.write("[misc]\naur_helper=/usr/bin/paru")
            conf = pacdef.Config(config_file_path=tmpfile)
            assert conf.aur_helper == pacdef.PARU

    @staticmethod
    def test___init__(tmpdir, monkeypatch):
        monkeypatch.setenv("XDG_CONFIG_HOME", str(tmpdir))
        groups = Path(tmpdir).joinpath("pacdef/groups")
        conf_file = Path(tmpdir).joinpath("pacdef/pacdef.conf")

        with mock.patch.object(
            pacdef, "_file_exists", lambda x: x == Path("/usr/bin/paru")
        ):
            config = pacdef.Config()
        aur_helper = Path("/usr/bin/paru")

        assert config.groups_path == groups
        assert config.aur_helper == aur_helper
        assert conf_file.is_file()


@pytest.mark.parametrize("user_input", ["Y", "y", ""])
def test_get_user_confirmation_continue(user_input):
    with mock.patch.object(builtins, "input", lambda _: user_input):
        assert pacdef._get_user_confirmation() is None


@pytest.mark.parametrize("user_input", ["n", "N", 'asd#!|^l;"f'])
def test_get_user_confirmation_exit(user_input):
    with mock.patch.object(builtins, "input", lambda _: user_input):
        with pytest.raises(SystemExit):
            pacdef._get_user_confirmation()


class TestAURHelper:
    @staticmethod
    @pytest.mark.skipif(not PACMAN_EXISTS, reason=REASON_NOT_ARCH)
    def test___init__():
        name = Path(PACMAN.name)  # pacman is good enough for the test case
        instance = pacdef.AURHelper(name)
        assert instance._path == PACMAN

        with pytest.raises(SystemExit):
            pacdef.AURHelper(Path("does not exist"))

    def test__execute(self, tmpdir):
        def check_valid(command_run: list[str]):
            assert command_run[0] == str(instance._path)
            assert command_run[1:] == command_given

        command_given: list[str] = ["some", "command"]
        instance = self.get_dummy_aur_helper(Path(tmpdir))
        with mock.patch.object(subprocess, "call", check_valid):
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

    @pytest.mark.parametrize(
        "packages",
        [
            [],
            ["neovim"],
            ["neovim", "python"],
            ["neovim", "repo/python"],
        ],
    )
    def test_install(self, tmpdir, packages):
        def check_valid(_, command):
            self.check_switches_valid(command, pacdef.AURHelper._Switches.install)
            self.check_switches_before_packages(
                command, pacdef.AURHelper._Switches.install
            )
            self.check_packages_present(command, packages)

        instance = self.get_dummy_aur_helper(Path(tmpdir))
        with mock.patch.object(pacdef.AURHelper, "_execute", check_valid):
            instance.install(packages)

    @staticmethod
    def get_dummy_aur_helper(tmpdir: Path) -> pacdef.AURHelper:
        path = tmpdir / "aur_helper"
        path.touch()
        return pacdef.AURHelper(path)

    @pytest.mark.parametrize(
        "packages",
        [
            [],
            ["neovim"],
            ["neovim", "python"],
        ],
    )
    def test_remove(self, tmpdir, packages):
        def check_valid(_, command):
            self.check_switches_valid(command, pacdef.AURHelper._Switches.remove)
            self.check_switches_before_packages(
                command, pacdef.AURHelper._Switches.remove
            )
            self.check_packages_present(command, packages)

        instance = self.get_dummy_aur_helper(Path(tmpdir))
        with mock.patch.object(pacdef.AURHelper, "_execute", check_valid):
            instance.remove(packages)


class TestPacdef:
    def _test_basic_printing_function(
        self, test_method: str, patched_method: str, capsys, tmpdir: Path
    ):
        instance = self._get_instance(tmpdir)
        method = instance.__getattribute__(test_method)
        with mock.patch.object(instance, patched_method, lambda: None):
            with pytest.raises(TypeError):
                method()

        with mock.patch.object(instance, patched_method, lambda: []):
            method()
        out, err = capsys.readouterr()
        assert len(out) == 0
        assert len(err) == 0

        packages = ["base"]
        with mock.patch.object(instance, patched_method, lambda: packages):
            method()
        out, err = capsys.readouterr()
        for package in packages:
            assert package in out
        assert len(err) == 0

        packages = ["base", "python"]
        with mock.patch.object(instance, patched_method, lambda: packages):
            method()
        out, err = capsys.readouterr()
        for package in packages:
            assert package in out
        assert len(err) == 0

    @staticmethod
    def _get_instance(tmpdir: Path | str) -> pacdef.Pacdef:
        tmpdir = Path(tmpdir)
        aur_helper = TestAURHelper.get_dummy_aur_helper(tmpdir)
        conf = pacdef.Config(
            aur_helper=NULL, groups_path=tmpdir, config_file_path=NULL, editor=NULL
        )
        args = pacdef.Arguments(process_args=False)
        instance = pacdef.Pacdef(args=args, aur_helper=aur_helper, config=conf)
        return instance

    def test_remove_unmanaged_packages_none(self, tmpdir):
        instance = self._get_instance(tmpdir)
        with mock.patch.object(instance, "_get_unmanaged_packages", lambda: []):
            with pytest.raises(SystemExit):
                instance._remove_unmanaged_packages()

    @pytest.mark.parametrize(
        "packages",
        [
            [
                ["neovim"],
                ["neovim", "python"],
            ]
        ],
    )
    def test_remove_unmanaged_packages_for_packages(self, packages, tmpdir):
        def check_valid(args: list[str]) -> None:
            for arg in args:
                assert arg in packages

        instance = self._get_instance(tmpdir)
        with mock.patch.object(instance._aur_helper, "remove", check_valid):
            with mock.patch.object(
                instance, "_get_unmanaged_packages", lambda: packages
            ):
                with mock.patch.object(pacdef, "_get_user_confirmation", lambda: None):
                    instance._remove_unmanaged_packages()

    def test_list_groups(self, capsys, tmpdir):
        self._test_basic_printing_function(
            "_list_groups", "_get_group_names", capsys, Path(tmpdir)
        )

    def test_import_groups(self, caplog, tmpdir):
        def test_existing():
            caplog.clear()
            count_before = len(list(groupdir.iterdir()))
            instance._import_groups()
            assert len(caplog.records) == 0
            count_after = len(list(groupdir.iterdir()))
            count_after_expected = count_before + len(instance._args.files)
            assert count_after == count_after_expected

        def test_already_imported():
            caplog.clear()
            count_before = len(list(groupdir.iterdir()))
            instance._import_groups()
            assert len(caplog.records) == 2
            for package, record in zip(instance._args.files, caplog.records):
                assert str(package.name) in record.message
            count_after = len(list(groupdir.iterdir()))
            assert count_after == count_before

        def get_instance(new_group_files: list[Path], tmpdir) -> pacdef.Pacdef:
            conf = pacdef.Config(groups_path=groupdir)
            aur_helper = TestAURHelper.get_dummy_aur_helper(tmpdir)
            args = pacdef.Arguments(process_args=False)
            args.files = new_group_files
            instance = pacdef.Pacdef(args=args, aur_helper=aur_helper, config=conf)
            return instance

        tmpdir = Path(tmpdir)
        groupdir = tmpdir.joinpath("groups")
        workdir = tmpdir.joinpath("work")
        groupdir.mkdir()
        workdir.mkdir()
        caplog.set_level(logging.WARNING)

        new_group_files = [workdir.joinpath(f"new_group_{x}") for x in range(3)]

        new_group_files[0].touch()
        instance = get_instance([new_group_files[0]], tmpdir)
        test_existing()

        for f in new_group_files:
            f.touch()
        instance._args.files = new_group_files[1:]
        test_existing()

        test_already_imported()

    def test_install_packages_from_groups_none(self, tmpdir):
        instance = self._get_instance(tmpdir)
        with mock.patch.object(instance, "_calculate_packages_to_install", lambda: []):
            with pytest.raises(SystemExit):
                instance._install_packages_from_groups()

    @pytest.mark.parametrize(
        "packages",
        [
            ["neovim"],
            ["neovim", "python"],
            ["neovim", "repo/python"],
        ],
    )
    def test_install_packages_from_groups_for_packages(self, packages, tmpdir):
        def check_valid(args: list[str]) -> None:
            for arg in args:
                assert arg in packages

        instance = self._get_instance(tmpdir)
        with mock.patch.object(instance._aur_helper, "install", check_valid):
            with mock.patch.object(
                instance, "_calculate_packages_to_install", lambda: packages
            ):
                with mock.patch.object(pacdef, "_get_user_confirmation", lambda: None):
                    instance._install_packages_from_groups()

    def test_show_unmanaged_packages(self, capsys, tmpdir):
        self._test_basic_printing_function(
            "_show_unmanaged_packages", "_get_unmanaged_packages", capsys, Path(tmpdir)
        )

    @pytest.mark.parametrize(
        "pacdef_packages, installed_packages, expected_result",
        [
            (["base"], [], ["base"]),
            ([], ["base"], []),
            ([], [], []),
            (["base"], ["base"], []),
            (["repo/base"], [], ["repo/base"]),
            (["repo/base"], ["base"], []),
            (["b", "a", "d", "c"], ["a", "b"], ["c", "d"]),
        ],
        ids=[
            "only pacdef",
            "only system",
            "nothing",
            "both equal",
            "only pacdef with repo prefix",
            "system with repo prefix",
            "alphabetical ordering",
        ],
    )
    def test__calculate_packages_to_install(
        self, pacdef_packages, installed_packages, expected_result, tmpdir
    ):
        instance = self._get_instance(tmpdir)
        pp = [pacdef.Package(item) for item in pacdef_packages]
        ip = [pacdef.Package(item) for item in installed_packages]
        er = [pacdef.Package(item) for item in expected_result]
        with mock.patch.object(instance, "_get_managed_packages", lambda: pp):
            with mock.patch.object(
                instance._db,
                "get_all_installed_packages",
                lambda: ip,
            ):
                result = instance._calculate_packages_to_install()
                assert result == er

    @pytest.mark.parametrize(
        "pacdef_packages, installed_packages, expected_result",
        [
            (["base"], [], []),
            ([], ["base"], ["base"]),
            ([], [], []),
            (["base"], ["base"], []),
            (["repo/base"], [], []),
            (["repo/base"], ["base"], []),
            (["a", "b"], ["b", "a", "d", "c"], ["c", "d"]),
        ],
        ids=[
            "only pacdef",
            "only system",
            "nothing",
            "both equal",
            "only pacdef with repo prefix",
            "system with repo prefix",
            "alphabetical ordering",
        ],
    )
    def test_get_unmanaged_packages(
        self, pacdef_packages, installed_packages, expected_result, tmpdir
    ):
        instance = self._get_instance(tmpdir)
        pp = [pacdef.Package(item) for item in pacdef_packages]
        ip = [pacdef.Package(item) for item in installed_packages]
        er = [pacdef.Package(item) for item in expected_result]
        with mock.patch.object(instance, "_get_managed_packages", lambda: pp):
            with mock.patch.object(
                instance._db,
                "get_explicitly_installed_packages",
                lambda: ip,
            ):
                result = instance._get_unmanaged_packages()
                assert result == er


class TestArguments:
    def test__parse_files(self, tmpdir):
        files = [Path(tmpdir) / "group"]
        args = argparse.Namespace()
        setattr(args, "file", files[0].name)
        with pytest.raises(SystemExit):
            pacdef.Arguments._parse_files(args)


class TestGroup:
    def test_append_to_empty_group(self, tmpdir):
        tmpfile = Path(tmpdir) / "group"
        package = pacdef.Package("package")
        group = pacdef.Group(packages=[], path=tmpfile)
        group.append(package)
        assert group.packages[-1] == package
        with open(tmpfile) as fd:
            content = fd.read()
        assert content == f"{package.name}\n"

    def test_append_to_nonempty_group(self, tmpdir):
        tmpfile = Path(tmpdir) / "group"
        package1 = pacdef.Package("package1")
        with open(tmpfile, "w") as fd:
            fd.write(f"{package1.name}\n")
        group = pacdef.Group(packages=[package1], path=tmpfile)

        package2 = pacdef.Package("package2")
        group.append(package2)
        assert group.packages == [package1, package2]
        with open(tmpfile) as fd:
            content = fd.read()
        assert content == f"{package1.name}\n{package2.name}\n"


class TestDB:
    @pytest.mark.skipif(pacdef.pyalpm is None, reason=REASON_NOT_ARCH)
    def test_get_explicitly_installed_packages_arch(self):
        instance = pacdef.DB()
        result = instance.get_explicitly_installed_packages()
        assert type(result) == list
        assert len(result) > 0
        for item in result:
            assert isinstance(item, pacdef.Package)
            assert len(item.name) > 0

    @pytest.mark.skipif(pacdef.pyalpm is None, reason=REASON_NOT_ARCH)
    def test_get_all_installed_packages_arch(self):
        instance = pacdef.DB()
        result = instance.get_all_installed_packages()
        assert type(result) == list
        assert len(result) > 0
        for item in result:
            assert isinstance(item, pacdef.Package)
            assert len(item.name) > 0
