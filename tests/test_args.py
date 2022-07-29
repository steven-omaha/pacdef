from __future__ import annotations

import argparse
from pathlib import Path

import pytest

from src.pacdef.args import Arguments


class TestArguments:
    def test__parse_files(self, tmpdir):
        files = [Path(tmpdir) / "group"]
        args = argparse.Namespace()
        setattr(args, "file", files[0].name)
        with pytest.raises(SystemExit):
            Arguments._parse_files(args)
