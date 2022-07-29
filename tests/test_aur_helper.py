from __future__ import annotations

import subprocess
from pathlib import Path
from unittest import mock

import pytest

from src.pacdef.aur_helper import AURHelper, Switches
from tests.constants import PACMAN, PACMAN_EXISTS, REASON_NOT_ARCH


class TestAURHelper:
    @staticmethod
    @pytest.mark.skipif(not PACMAN_EXISTS, reason=REASON_NOT_ARCH)
    def test___init__():
        name = Path(PACMAN.name)  # pacman is good enough for the test case
        instance = AURHelper(name)
        assert instance._path == PACMAN

        with pytest.raises(SystemExit):
            AURHelper(Path("does not exist"))

    def test__execute(self, tmpdir):
        class Dummy:
            @staticmethod
            def check_returncode():
                return True

        def check_valid(command_run: list[str]):
            assert command_run[0] == str(instance._path)
            assert command_run[1:] == command_given
            return Dummy

        command_given: list[str] = ["some", "command"]
        instance = self.get_dummy_aur_helper(Path(tmpdir))
        with mock.patch.object(subprocess, "run", check_valid):
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
            self.check_switches_valid(command, Switches.install)
            self.check_switches_before_packages(command, Switches.install)
            self.check_packages_present(command, packages)

        instance = self.get_dummy_aur_helper(Path(tmpdir))
        with mock.patch.object(AURHelper, "_execute", check_valid):
            instance.install(packages)

    @staticmethod
    def get_dummy_aur_helper(tmpdir: Path) -> AURHelper:
        path = tmpdir / "aur_helper"
        path.touch()
        return AURHelper(path)

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
            self.check_switches_valid(command, Switches.remove)
            self.check_switches_before_packages(command, Switches.remove)
            self.check_packages_present(command, packages)

        instance = self.get_dummy_aur_helper(Path(tmpdir))
        with mock.patch.object(AURHelper, "_execute", check_valid):
            instance.remove(packages)
