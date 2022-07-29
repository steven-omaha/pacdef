from __future__ import annotations

import logging
from pathlib import Path
from unittest import mock

import pytest

import pacdef
from constants import REASON_NOT_ARCH, DEVNULL


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
            aur_helper=DEVNULL,
            groups_path=tmpdir,
            config_file_path=DEVNULL,
            editor=DEVNULL,
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
                with mock.patch.object(
                    pacdef.UserInput, "get_user_confirmation", lambda: None
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
                with mock.patch.object(
                    pacdef.UserInput, "get_user_confirmation", lambda: None
                ):
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
        package = pacdef.Package("abc")
        group = pacdef.Group([package], DEVNULL)
        arguments = pacdef.Arguments(process_args=False)
        arguments.package = pacdef.Package("abc")
        instance._args = arguments
        instance._groups = [group]

        with pytest.raises(SystemExit) as raised:
            instance._search_package()
        assert raised.value.code == 0

        arguments.package = pacdef.Package("def")

        with pytest.raises(SystemExit) as raised:
            instance._search_package()
        assert raised.value.code == 1

        arguments.package = pacdef.Package(".*b.*")

        with pytest.raises(SystemExit) as raised:
            instance._search_package()
        assert raised.value.code == 0

        arguments.package = pacdef.Package("^abc$")

        with pytest.raises(SystemExit) as raised:
            instance._search_package()
        assert raised.value.code == 0


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

    def test_new_group(self, tmpdir):
        filename = "new_group"
        tmpfile = Path(tmpdir) / filename
        pacdef.Group.new_file(filename, Path(tmpdir))
        group = pacdef.Group([], tmpfile)
        assert group.path.exists() and group.path.is_file()


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
