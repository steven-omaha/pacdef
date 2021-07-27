from pathlib import Path

from pacdef import dir_exists, file_exists

testpath = Path('/tmp/pacdef_test')


class TestDirExists:
    def test_dir_exists_file(self):
        testpath.touch()
        assert not dir_exists(testpath)
        testpath.unlink()

    def test_dir_exists_nonexistent(self):
        assert not dir_exists(testpath)

    def test_dir_exists_existing(self):
        testpath.mkdir()
        assert dir_exists(testpath)
        testpath.rmdir()


class TestFileExists:
    def test_file_exists_file(self):
        testpath.touch()
        assert file_exists(testpath)
        testpath.unlink()

    def test_file_exists_nonexistent(self):
        assert not file_exists(testpath)

    def test_file_exists_dir(self):
        testpath.mkdir()
        assert not file_exists(testpath)
        testpath.rmdir()
