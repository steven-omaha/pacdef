# type: ignore

from __future__ import annotations

import logging
from pathlib import Path
from unittest import mock

import pytest
from constants import DEVNULL
from test_aur_helper import TestAURHelper

import src.pacdef.main as main
from src.pacdef.args import Arguments
from src.pacdef.config import Config
from src.pacdef.group import Group
from src.pacdef.main import Pacdef
from src.pacdef.package import Package


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
    def _get_instance(tmpdir: Path | str) -> Pacdef:
        tmpdir = Path(tmpdir)
        aur_helper = TestAURHelper.get_dummy_aur_helper(tmpdir)
        conf = Config(
            aur_helper=DEVNULL,
            groups_path=tmpdir,
            config_file_path=DEVNULL,
            editor=DEVNULL,
        )
        args = Arguments(process_args=False)
        instance = Pacdef(args=args, aur_helper=aur_helper, config=conf)
        return instance

    def test_remove_unmanaged_packages_none(self, tmpdir):
        instance = self._get_instance(tmpdir)
        with mock.patch.object(main, "calc_unmanaged_packages", lambda x, y: []):
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
                main, "calc_unmanaged_packages", lambda x, y: packages
            ):
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

        def get_instance(new_group_files: list[Path], tmpdir) -> Pacdef:
            conf = Config(groups_path=groupdir)
            aur_helper = TestAURHelper.get_dummy_aur_helper(tmpdir)
            args = Arguments(process_args=False)
            args.files = new_group_files
            instance = Pacdef(args=args, aur_helper=aur_helper, config=conf)
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
        with mock.patch.object(main, "calc_packages_to_install", lambda x, y: []):
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
                main, "calc_packages_to_install", lambda x, y: packages
            ):
                instance._install_packages_from_groups()

    @pytest.mark.skip(reason="no idea how to fix this atm")
    def test_show_unmanaged_packages(self, capsys, tmpdir):
        self._test_basic_printing_function(
            "_show_unmanaged_packages",
            "main.calc_unmanaged_packages",
            capsys,
            Path(tmpdir),
        )

    def test__new_group(self, tmpdir):
        tmp_path = Path(tmpdir)
        group_name = "a"
        preexisting_group = tmp_path / group_name
        preexisting_group.touch()
        instance = self._get_instance(tmpdir)
        instance._args.groups = [group_name]
        with pytest.raises(SystemExit):
            instance._new_group()

        group_name = "b"
        instance._args.groups = [group_name]
        instance._new_group()

    def test__search_package(self, tmpdir):
        instance = self._get_instance(tmpdir)
        package = Package("abc")
        group = Group([package], DEVNULL)
        arguments = Arguments(process_args=False)
        arguments.package = Package("abc")
        instance._args = arguments
        instance._groups = [group]

        with pytest.raises(SystemExit) as raised:
            instance._search_package()
        assert raised.value.code == 0

        arguments.package = Package("def")

        with pytest.raises(SystemExit) as raised:
            instance._search_package()
        assert raised.value.code == 1

        arguments.package = Package(".*b.*")

        with pytest.raises(SystemExit) as raised:
            instance._search_package()
        assert raised.value.code == 0

        arguments.package = Package("^abc$")

        with pytest.raises(SystemExit) as raised:
            instance._search_package()
        assert raised.value.code == 0
