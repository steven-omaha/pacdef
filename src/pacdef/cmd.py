# pyright: reportUnknownArgumentType=none, reportUnknownParameterType=none

from __future__ import annotations

import logging
import subprocess
import sys

from .constants import EXIT_ERROR


def run(command: list[str], *args, **kwargs):
    """Run a command using subprocess.run and provide logging."""
    logging.info(f"Executing command with subprocess.run: {command, args, kwargs}")
    try:
        subprocess.run(command, *args, **kwargs).check_returncode()
    except subprocess.CalledProcessError as e:
        logging.error(e)
        sys.exit(EXIT_ERROR)
