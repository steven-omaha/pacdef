# type: ignore

import pytest

from src.pacdef.package import Package
from src.pacdef.solver import calc_packages_to_install, calc_unmanaged_packages


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
def test_get_unmanaged_packages(pacdef_packages, installed_packages, expected_result):
    pp = [Package(item) for item in pacdef_packages]
    ip = [Package(item) for item in installed_packages]
    er = [Package(item) for item in expected_result]
    result = calc_unmanaged_packages(pp, ip)
    assert result == er


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
    pacdef_packages, installed_packages, expected_result
):
    pp = [Package(item) for item in pacdef_packages]
    ip = [Package(item) for item in installed_packages]
    er = [Package(item) for item in expected_result]
    result = calc_packages_to_install(pp, ip)
    assert result == er
