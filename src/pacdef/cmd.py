# pyright: reportUnknownArgumentType=none, reportUnknownParameterType=none

from __future__ import annotations

import logging
import subprocess
import sys

from pacdef.constants import EXIT_ERROR


class CommandRunner:
    """Abstract calls of external commands to provide logging."""

    @staticmethod
    def run(command: list[str], *args, **kwargs):
        """Run a command using subprocess.run."""
        logging.info(f"Executing command with subprocess.run: {command, args, kwargs}")
        try:
            subprocess.run(command, *args, **kwargs).check_returncode()
        except subprocess.CalledProcessError as e:
            logging.error(e)
            sys.exit(EXIT_ERROR)

    @staticmethod
    def call(command: list[str], *args, **kwargs) -> None:
        """Run a command using subprocess.call."""
        logging.info(f"Executing command with subprocess.call: {command, args, kwargs}")
        try:
            assert subprocess.call(command, *args, **kwargs)
        except (FileNotFoundError, AssertionError):
            logging.error(f'Could not successfully execute the program "{command[0]}".')
            sys.exit(EXIT_ERROR)
