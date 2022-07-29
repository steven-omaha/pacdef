from __future__ import annotations

import logging
import sys
import termios
import tty
from typing import Any, Callable

from .constants import EXIT_INTERRUPT, EXIT_SUCCESS, INTERRUPT_ASCII_CODE


class UserInput:
    """Handles all user input related features."""

    @classmethod
    def get_user_confirmation(cls) -> None:
        """Ask the user if he wants to continue. Exits if the answer is not `y` or of length zero.

        Wraps `get_user_input` internally.
        """

        def check_wants_to_continue(input_: str | None) -> bool:
            if input_ is None:
                return False
            if input_.lower().strip() in ["", "y"]:
                return True
            return False

        want_to_continue = cls.get_user_input(
            "Continue? [Y/n] ",
            check_wants_to_continue,
            single_character=True,
        )
        if not want_to_continue:
            sys.exit(EXIT_SUCCESS)

    @classmethod
    def get_user_input(
        cls,
        prompt: str,
        validator: Callable[[str | None], Any],
        *,
        default: str | None = None,
        single_character: bool = False,
    ) -> Any:
        """Show the prompt to the user and parse the input.

        Exits on Ctrl-C.

        :param prompt: The prompt to show to the user. Should end with a space.
        :param validator: A function to parse / validate the user input.
        :param default: Default value if no input was received.
        :param single_character: If True, read only a single character, and return immediately (do not wait for newline)
        :return:
        """
        user_input: str | None
        result: Any
        user_input, result = "", ""
        while not user_input:
            print(prompt, end="", flush=True)
            if single_character:
                user_input = cls._read_character()
            else:
                user_input = cls._read_line()
            user_input = user_input or default
            logging.info(f"{user_input=}")
            try:
                result = validator(user_input)
            except ValueError:
                logging.info("parsing: ValueError, resetting user_input")
                user_input = ""
            except KeyboardInterrupt:
                sys.exit(EXIT_INTERRUPT)
        return result

    @staticmethod
    def _read_line() -> str:
        user_input = input()
        return user_input

    @classmethod
    def _read_character(cls) -> str:
        """Tweak the STDIN buffer to return immediately after a single char has been read."""
        fd = sys.stdin.fileno()
        old_settings = termios.tcgetattr(fd)
        try:
            tty.setraw(sys.stdin.fileno())
            ch = sys.stdin.read(1)
        finally:
            termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
        if ch == INTERRUPT_ASCII_CODE:
            print()
            sys.exit(EXIT_INTERRUPT)
        print(ch)  # in raw mode, user input is not echoed automatically
        return ch
