#!/usr/bin/env python3
"""Reproduce Windows autocrlf and preserve canonical source, assets and historical bytes."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("native_inputs", ROOT / "scripts/check-native-inputs.py")
inputs = importlib.util.module_from_spec(spec)
spec.loader.exec_module(inputs)


class NativeInputs(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="tbg-native-inputs-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.git("init", "--quiet")
        self.git("config", "core.autocrlf", "true")
        self.files = {
            "Cargo.lock": b"version = 4\n",
            "crates/tbg-core/src/rules.rs": b"// Hashed rules\npub const VALUE: u32 = 1;\n",
            "crates/tbg-client/assets/i18n/en.json": '{"name":"Tupi ·"}\n'.encode("utf-8"),
            "crates/tbg-client/assets/icon.png": b"\x89PNG\r\n\0\x01\x02\xff",
            # A disposable fixture, never the project's read-only archive.
            "archive/historical.txt": b"original\r\nhistorical\r\n",
        }
        (self.root / ".gitattributes").write_bytes((ROOT / ".gitattributes").read_bytes())
        for name, data in self.files.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        self.git("add", ".")

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, stderr=subprocess.PIPE)

    def checkout(self):
        for name in self.files:
            (self.root / name).unlink()
        self.git("checkout-index", "--all", "--force")

    def test_windows_autocrlf_keeps_hashed_text_binary_and_archive_bytes_identical(self):
        self.checkout()
        for name, data in self.files.items():
            self.assertEqual((self.root / name).read_bytes(), data, name)
        self.assertEqual(inputs.problems(self.root), [])

    def test_without_attributes_windows_checkout_reproduces_the_release_failure(self):
        (self.root / ".gitattributes").unlink()
        self.git("update-index", "--force-remove", ".gitattributes")
        self.checkout()
        self.assertIn(b"\r\n", (self.root / "Cargo.lock").read_bytes())
        self.assertTrue(any("Cargo.lock: CRLF" in error for error in inputs.problems(self.root)))

    def test_preflight_rejects_changed_newlines_and_non_utf8_without_touching_files(self):
        path = self.root / "Cargo.lock"
        path.write_bytes(b"version = 4\r\n\xff")
        errors = inputs.problems(self.root)
        self.assertTrue(any("CRLF" in error for error in errors))
        self.assertTrue(any("must be UTF-8" in error for error in errors))
        self.assertEqual(path.read_bytes(), b"version = 4\r\n\xff")


if __name__ == "__main__":
    unittest.main()
