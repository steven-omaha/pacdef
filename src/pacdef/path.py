from pathlib import Path


def file_exists(path: Path) -> bool:
    """Check whether a path exists and is an actual file."""
    return path.exists() and path.is_file()


def dir_exists(path: Path) -> bool:
    """Check whether a path exists and is an actual folder."""
    return path.exists() and path.is_dir()
