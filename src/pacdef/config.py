from __future__ import annotations

import configparser
import logging
import os
import sys
from os import environ
from pathlib import Path
from typing import Optional

from .constants import EXIT_ERROR, PARU
from .path import dir_exists, file_exists


class Config:
    """Class reading and holding the runtime configuration."""

    def __init__(
        self,
        *,
        groups_path: Optional[Path] = None,
        aur_helper: Optional[Path] = None,
        aur_helper_rm_args: Optional[list[str]] = None,
        config_file_path: Optional[Path] = None,
        editor: Optional[Path] = None,
        warn_symlinks: bool = True,
    ):
        """Instantiate using the provided values. If these are None, use the config file / defaults."""
        config_base_dir = _get_xdg_config_home()
        pacdef_path = config_base_dir.joinpath("pacdef")
        config_file_path = config_file_path or pacdef_path.joinpath("pacdef.conf")

        self._config = _read_config_file(config_file_path)
        self.groups_path: Path = groups_path or pacdef_path.joinpath("groups")
        logging.info(f"{self.groups_path=}")
        self._create_config_files_and_dirs(config_file_path, pacdef_path)

        self.aur_helper: Path = aur_helper or self._get_aur_helper()
        self.aur_helper_rm_args: list[str] = aur_helper_rm_args or self._get_rm_args()
        self._editor: Path | None = editor or self._get_editor()
        self._warn_symlinks: bool = warn_symlinks and self._get_warn_symlinks()
        logging.info(f"{self.aur_helper=}")
        self._sanity_check()

    def _create_config_files_and_dirs(self, config_file_path: Path, pacdef_path: Path):
        """Create config files and dirs. Will be executed on first-time run of pacdef."""
        if not dir_exists(pacdef_path):
            pacdef_path.mkdir(parents=True)
        if not dir_exists(self.groups_path):
            self.groups_path.mkdir()
        if not file_exists(config_file_path):
            config_file_path.touch()

    @property
    def editor(self) -> Path:
        """Get the editor. Error out if none is found."""
        if self._editor is None:
            msg = (
                "I do not know which editor to use.\n"
                "  Either set the environment variables EDITOR or VISUAL, or set\n"
                "  editor in the [misc] section in pacdef.conf."
            )
            logging.error(msg)
            sys.exit(EXIT_ERROR)
        return self._editor

    @property
    def warn_symlinks(self) -> bool:
        """Get value of warn_symlinks from config, default: true."""
        return self._warn_symlinks

    def _get_value_from_conf(
        self, section: str, key: str, *, warn_missing: bool = False
    ) -> str | None:
        try:
            result = self._config[section][key]
        except KeyError:
            if warn_missing:
                logging.warning(f"{key} in section [{section}] not set")
            result = None
        return result

    def _get_editor(self) -> Path | None:
        editor = self._get_value_from_conf("misc", "editor")
        if editor is not None:
            return Path(editor)
        editor = self._get_value_from_env_variables(["EDITOR", "VISUAL"])
        if editor is not None:
            return Path(editor)
        return None

    def _get_rm_args(self) -> list[str] | None:
        args = self._get_value_from_conf("misc", "aur_rm_args")
        if args is not None:
            return args.split()
        else:
            return None

    @classmethod
    def _get_value_from_env_variables(
        cls, variables: list[str], warn_missing: bool = False
    ) -> str | None:
        """Return the value of the first existing environment variable.

        For a list of environment variables, check in the provided order that they exist.
        Return the value of the first existing environment variable.

        :param variables: list of environment variables to read
        :param warn_missing: print a warning if none of the elements in `variables` are found
        :return: value of environment variable or None
        """
        for var in variables:
            result = _get_value_from_env(var, warn_missing)
            if result is not None:
                return result
        return None

    def _get_aur_helper(self) -> Path:
        aur_helper = self._get_value_from_conf("misc", "aur_helper", warn_missing=True)
        if aur_helper is None:
            logging.warning(f"No AUR helper set. Defaulting to {PARU}")
            return PARU
        return Path(aur_helper)

    def _sanity_check(self):
        number_group_files = len([self.groups_path.iterdir()])
        if number_group_files == 0:
            logging.warning("pacdef does not know any groups. Import or create one.")

    def _get_bool_from_conf(self, section: str, key: str, *, default: bool) -> bool:
        value = self._get_value_from_conf(section, key)
        # fmt: off
        value_result_map = {
            None: default,
            "false": False,
            "true": True
        }
        # fmt: on
        try:
            return value_result_map[value]
        except KeyError:
            msg = (
                f"invalid value in config: [{section}] has {key}={value}\n"
                f"possible values: true, false (default: {default}) "
            )
            raise ValueError(msg)

    def _get_warn_symlinks(self) -> bool:
        section = "misc"
        key = "warn_not_symlink"
        try:
            return self._get_bool_from_conf(section, key, default=True)
        except ValueError as err:
            logging.error(err)
            exit(EXIT_ERROR)


def _read_config_file(config_file: Path) -> configparser.ConfigParser:
    config = configparser.ConfigParser()
    try:
        _ = config.read(config_file)
    except configparser.ParsingError as e:
        logging.error(f"Could not parse the config: {e}")
    return config


def _get_value_from_env(variable: str, warn_missing: bool = False) -> str | None:
    """Get the value of a single environment variable.

    :param variable: environment variable to read
    :param warn_missing: print a warning if the variable is not set
    :return: value of environment variable or None
    """
    try:
        result = os.environ[variable]
    except KeyError:
        if warn_missing:
            logging.warning(f"Environment variable {variable} not set.")
        return None
    logging.info(f"{variable} is set to {result}.")
    return result


def _get_xdg_config_home() -> Path:
    try:
        config_base_dir = Path(environ["XDG_CONFIG_HOME"])
    except KeyError:
        config_base_dir = Path.home() / ".config"
    logging.debug(f"{config_base_dir=}")
    return config_base_dir
