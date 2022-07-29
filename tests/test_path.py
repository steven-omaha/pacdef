from __future__ import annotations

from pathlib import Path

from src.pacdef.path import dir_exists, file_exists


def test_dir_exists(tmpdir):
    tmpdir = Path(tmpdir)
    tmpfile = tmpdir.joinpath("tmpfile")
    tmpfile.touch()
    assert not dir_exists(tmpfile)
    tmpfile.unlink()
    assert not dir_exists(tmpfile)
    assert dir_exists(tmpdir)


def test_file_exists(tmpdir):
    tmpfile = Path(tmpdir).joinpath("tmpfile")
    tmpfile.touch()
    assert file_exists(tmpfile)
    tmpfile.unlink()
    assert not file_exists(tmpfile)
    tmpfile.mkdir()
    assert not file_exists(tmpfile)
