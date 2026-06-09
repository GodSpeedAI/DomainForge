import io
import os
import tarfile
import tempfile
import zipfile

import pytest

from scripts.ci_tasks import (
    _is_safe_path,
    _validate_tar_member,
    _validate_zip_member,
    unpack_and_verify,
)


class TestSafePath:
    def test_normal_path_is_safe(self):
        assert _is_safe_path("/tmp/dest", "file.txt")

    def test_subdirectory_is_safe(self):
        assert _is_safe_path("/tmp/dest", "sub/file.txt")

    def test_traversal_is_unsafe(self):
        assert not _is_safe_path("/tmp/dest", "../outside.txt")

    def test_deep_traversal_is_unsafe(self):
        assert not _is_safe_path("/tmp/dest", "foo/../../outside.txt")

    def test_absolute_path_is_unsafe(self):
        assert not _is_safe_path("/tmp/dest", "/etc/passwd")


class TestTarValidation:
    def test_safe_member_passes(self):
        member = tarfile.TarInfo(name="safe_file.txt")
        _validate_tar_member("/tmp/dest", member)

    def test_traversal_rejected(self):
        member = tarfile.TarInfo(name="../outside.txt")
        with pytest.raises(ValueError, match="Unsafe path"):
            _validate_tar_member("/tmp/dest", member)

    def test_absolute_path_rejected(self):
        member = tarfile.TarInfo(name="/etc/passwd")
        with pytest.raises(ValueError, match="Unsafe path"):
            _validate_tar_member("/tmp/dest", member)

    def test_symlink_rejected(self):
        member = tarfile.TarInfo(name="link")
        member.type = tarfile.SYMTYPE
        member.linkname = "/etc/passwd"
        with pytest.raises(ValueError, match="Symlink"):
            _validate_tar_member("/tmp/dest", member)

    def test_hardlink_rejected(self):
        member = tarfile.TarInfo(name="link")
        member.type = tarfile.LNKTYPE
        member.linkname = "/etc/passwd"
        with pytest.raises(ValueError, match="Hardlink|Symlink"):
            _validate_tar_member("/tmp/dest", member)

    def test_deep_traversal_rejected(self):
        member = tarfile.TarInfo(name="foo/../../outside.txt")
        with pytest.raises(ValueError, match="Unsafe path"):
            _validate_tar_member("/tmp/dest", member)


class TestZipValidation:
    def test_safe_entry_passes(self):
        _validate_zip_member("/tmp/dest", "safe_file.txt")

    def test_traversal_rejected(self):
        with pytest.raises(ValueError, match="Unsafe path"):
            _validate_zip_member("/tmp/dest", "../outside.txt")

    def test_absolute_path_rejected(self):
        with pytest.raises(ValueError, match="Unsafe path"):
            _validate_zip_member("/tmp/dest", "/etc/passwd")


class TestMaliciousArchiveExtraction:
    def test_malicious_tar_with_traversal_fails(self, tmp_path):
        tar_path = tmp_path / "malicious.tar.gz"
        with tarfile.open(str(tar_path), "w:gz") as tf:
            data = b"malicious content"
            member = tarfile.TarInfo(name="../outside.txt")
            member.size = len(data)
            tf.addfile(member, io.BytesIO(data))

        with pytest.raises(ValueError, match="Unsafe path"):
            unpack_and_verify(str(tar_path), "sea")

    def test_malicious_tar_with_symlink_fails(self, tmp_path):
        tar_path = tmp_path / "symlink.tar.gz"
        with tarfile.open(str(tar_path), "w:gz") as tf:
            member = tarfile.TarInfo(name="evil_link")
            member.type = tarfile.SYMTYPE
            member.linkname = "/etc/passwd"
            tf.addfile(member)

        with pytest.raises(ValueError, match="Symlink"):
            unpack_and_verify(str(tar_path), "sea")

    def test_malicious_tar_with_absolute_path_fails(self, tmp_path):
        tar_path = tmp_path / "absolute.tar.gz"
        with tarfile.open(str(tar_path), "w:gz") as tf:
            data = b"absolute path content"
            member = tarfile.TarInfo(name="/tmp/evil.txt")
            member.size = len(data)
            tf.addfile(member, io.BytesIO(data))

        with pytest.raises(ValueError, match="Unsafe path"):
            unpack_and_verify(str(tar_path), "sea")

    def test_malicious_zip_with_traversal_fails(self, tmp_path):
        zip_path = tmp_path / "malicious.zip"
        with zipfile.ZipFile(str(zip_path), "w") as zf:
            zf.writestr("../outside.txt", "malicious content")

        with pytest.raises(ValueError, match="Unsafe path"):
            unpack_and_verify(str(zip_path), "sea")

    def test_safe_tar_extracts_successfully(self, tmp_path):
        tar_path = tmp_path / "safe.tar.gz"
        with tarfile.open(str(tar_path), "w:gz") as tf:
            data = b"#!/bin/sh\necho v1.0.0"
            member = tarfile.TarInfo(name="sea")
            member.size = len(data)
            member.mode = 0o755
            tf.addfile(member, io.BytesIO(data))

        with patch_verify_cli(tmp_path):
            result = unpack_and_verify(str(tar_path), "sea")
            assert result == 0


def patch_verify_cli(tmp_path):
    import unittest.mock

    return unittest.mock.patch(
        "scripts.ci_tasks.verify_cli_binary", return_value=0
    )
