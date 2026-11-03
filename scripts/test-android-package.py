#!/usr/bin/env python3
"""Regression checks for refusing unsafe Android native package inputs (AN02/14/16)."""

from pathlib import Path
import struct
import tempfile
import unittest
import zipfile

from android_binary import elf_identity, inspect_apk_libraries


def arm64_library(import_name="libc.so", alignment=16384):
    """A synthetic ELF fixture only; never a runnable library or APK candidate."""
    raw = bytearray(1024)
    raw[:6] = b"\x7fELF\x02\x01"
    struct.pack_into("<HH", raw, 16, 3, 183)
    struct.pack_into("<Q", raw, 32, 64)
    struct.pack_into("<HHH", raw, 52, 64, 56, 2)
    struct.pack_into("<IIQQQQQQ", raw, 64, 1, 5, 0, 0, 0, len(raw), len(raw), alignment)
    struct.pack_into("<IIQQQQQQ", raw, 120, 2, 4, 256, 256, 256, 64, 64, 8)
    strings = b"\0" + import_name.encode("ascii") + b"\0"
    raw[512:512 + len(strings)] = strings
    for index, pair in enumerate([(5, 512), (10, len(strings)), (1, 1), (0, 0)]):
        struct.pack_into("<qQ", raw, 256 + index * 16, *pair)
    return bytes(raw)


def apk_fixture(path, library, *, abi="arm64-v8a", aligned=True, compressed=False):
    name = f"lib/{abi}/libtbg_client.so"
    entry = zipfile.ZipInfo(name)
    if aligned:
        padding = (-(30 + len(name.encode()) + 4)) % 16384
        entry.extra = struct.pack("<HH", 0xffff, padding) + bytes(padding)
    entry.compress_type = zipfile.ZIP_DEFLATED if compressed else zipfile.ZIP_STORED
    with zipfile.ZipFile(path, "w") as archive:
        archive.writestr(entry, library)


class NativeInputContracts(unittest.TestCase):
    def test_arm64_imports_and_page_alignment_are_reported(self):
        identity = elf_identity(arm64_library())
        self.assertEqual(identity["abi"], "arm64-v8a")
        self.assertEqual(identity["imports"], ["libc.so"])
        self.assertEqual(identity["load_alignment"], [16384])

    def test_wrong_abi_truncation_and_old_page_alignment_are_rejected(self):
        wrong_abi = bytearray(arm64_library())
        struct.pack_into("<H", wrong_abi, 18, 62)
        for raw in [bytes(wrong_abi), arm64_library()[:300], arm64_library(alignment=4096)]:
            with self.subTest(size=len(raw)), self.assertRaises(ValueError):
                elf_identity(raw)

    def test_dynamic_strings_cannot_escape_load_data(self):
        raw = bytearray(arm64_library())
        struct.pack_into("<qQ", raw, 256, 5, 8192)
        with self.assertRaisesRegex(ValueError, "outside LOAD"):
            elf_identity(raw)

    def test_apk_checks_actual_zip_offset_and_library_imports(self):
        with tempfile.TemporaryDirectory(prefix="tbg-apk-fixture-") as temp:
            apk = Path(temp) / "fixture.zip"
            apk_fixture(apk, arm64_library())
            result = inspect_apk_libraries(apk)
            self.assertEqual(result["libtbg_client.so"]["zip_data_offset"], 16384)
            apk_fixture(apk, arm64_library("libc++_shared.so"))
            with self.assertRaisesRegex(ValueError, "unbundled imports"):
                inspect_apk_libraries(apk)

    def test_apk_rejects_compressed_unaligned_and_other_abi_libraries(self):
        cases = [dict(compressed=True), dict(aligned=False), dict(abi="x86_64")]
        with tempfile.TemporaryDirectory(prefix="tbg-apk-fixture-") as temp:
            apk = Path(temp) / "fixture.zip"
            for options in cases:
                with self.subTest(options=options):
                    apk_fixture(apk, arm64_library(), **options)
                    with self.assertRaises(ValueError):
                        inspect_apk_libraries(apk)

    def test_apk_without_game_library_is_not_a_candidate(self):
        with tempfile.TemporaryDirectory(prefix="tbg-apk-fixture-") as temp:
            apk = Path(temp) / "fixture.zip"
            with zipfile.ZipFile(apk, "w") as archive:
                archive.writestr("assets/logo.webp", b"not-a-game")
            with self.assertRaisesRegex(ValueError, "no ARM64 game library"):
                inspect_apk_libraries(apk)


if __name__ == "__main__":
    unittest.main()
