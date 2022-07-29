from __future__ import annotations

from pathlib import Path

from src.pacdef.group import Group
from src.pacdef.package import Package


class TestGroup:
    def test_append_to_empty_group(self, tmpdir):
        tmpfile = Path(tmpdir) / "group"
        package = Package("package")
        group = Group(packages=[], path=tmpfile)
        group.append(package)
        assert group.packages[-1] == package
        with open(tmpfile) as fd:
            content = fd.read()
        assert content == f"{package.name}\n"

    def test_append_to_nonempty_group(self, tmpdir):
        tmpfile = Path(tmpdir) / "group"
        package1 = Package("package1")
        with open(tmpfile, "w") as fd:
            fd.write(f"{package1.name}\n")
        group = Group(packages=[package1], path=tmpfile)

        package2 = Package("package2")
        group.append(package2)
        assert group.packages == [package1, package2]
        with open(tmpfile) as fd:
            content = fd.read()
        assert content == f"{package1.name}\n{package2.name}\n"

    def test_new_group(self, tmpdir):
        filename = "new_group"
        tmpfile = Path(tmpdir) / filename
        Group.new_file(filename, Path(tmpdir))
        group = Group([], tmpfile)
        assert group.path.exists() and group.path.is_file()
