from __future__ import annotations

import pytest

try:
    import pyalpm
except FileNotFoundError:
    pyalpm = None

from constants import REASON_NOT_ARCH

from src.pacdef.db import DB
from src.pacdef.package import Package


class TestDB:
    @pytest.mark.skipif(pyalpm is None, reason=REASON_NOT_ARCH)
    def test_get_explicitly_installed_packages_arch(self):
        instance = DB()
        result = instance.get_explicitly_installed_packages()
        assert type(result) == list
        assert len(result) > 0
        for item in result:
            assert isinstance(item, Package)
            assert len(item.name) > 0

    @pytest.mark.skipif(pyalpm is None, reason=REASON_NOT_ARCH)
    def test_get_all_installed_packages_arch(self):
        instance = DB()
        result = instance.get_all_installed_packages()
        assert type(result) == list
        assert len(result) > 0
        for item in result:
            assert isinstance(item, Package)
            assert len(item.name) > 0
