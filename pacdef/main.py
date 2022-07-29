import logging
import sys
from os import environ

from pacdef.args import Arguments
from pacdef.aur_helper import AURHelper
from pacdef.config import Config
from pacdef.constants import EXIT_INTERRUPT
from pacdef.db import DB
from pacdef.pacdef import Pacdef


def _setup_logger() -> None:
    """Set up the logger.

    When the log level is below WARNING (i.e. INFO or DEBUG), the line number of the logging statement is printed as
    well.
    """
    try:
        level_name: str = environ["LOGLEVEL"]
    except KeyError:
        level_name = "WARNING"

    level: int = logging.getLevelName(level_name.upper())
    if level < logging.WARNING:
        logging.basicConfig(format="%(levelname)s:%(lineno)d: %(message)s", level=level)
    else:
        logging.basicConfig(format="%(levelname)s: %(message)s", level=level)


def main():
    _setup_logger()
    args = Arguments()
    config = Config()
    helper = AURHelper.from_config(config)
    db = DB()
    pacdef = Pacdef(args=args, config=config, aur_helper=helper, db=db)
    pacdef.run_action_from_arg()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        sys.exit(EXIT_INTERRUPT)
