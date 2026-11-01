#!/usr/bin/env python3
"""Check native executable identity across locales before compiling the client."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import threading
import tomllib
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("package_native", ROOT / "scripts/package-native.py")
packager = importlib.util.module_from_spec(spec)
spec.loader.exec_module(packager)
SOURCE = "a" * 40
VERSION = "0.1.2-rc.1"


def command(**changes):
    metadata = {"source_commit": SOURCE, "version": VERSION, "profile": "release", "label": "Tupi ·"}
    metadata.update(changes)
    identity = f"Tupi Battle Grounds {VERSION} · protocol 31 · source {SOURCE}"
    # Mimic Rust's raw UTF-8 writes, regardless of the child Python locale.
    program = (
        f"import sys; output = {json.dumps(metadata, ensure_ascii=False)!r} "
        f"if sys.argv[-1] == '--build-info' else {identity!r}; "
        "sys.stdout.buffer.write(output.encode('utf-8'))"
    )
    return [sys.executable, "-c", program]


class NativePackage(unittest.TestCase):
    def test_windows_legacy_locale_reproduces_the_failure_and_utf8_identity_passes(self):
        executable = command()
        # CPython 3.11 uses this locale default for text=True on the Windows runner.
        with patch("subprocess._text_encoding", return_value="cp1252"):
            previous = subprocess.check_output([*executable, "--version"], text=True)
            self.assertNotIn(f" {VERSION} ·", previous)
            value = packager.release_identity(executable, VERSION, SOURCE)
        self.assertEqual(value["label"], "Tupi ·")
        self.assertEqual(value["source_commit"], SOURCE)

    def test_stale_source_wrong_version_and_debug_binary_are_still_rejected(self):
        for changes in [{"source_commit": "b" * 40}, {"version": "0.0.0"}, {"profile": "debug"}]:
            with self.subTest(changes=changes), self.assertRaisesRegex(SystemExit, "mismatch"):
                packager.release_identity(command(**changes), VERSION, SOURCE)

    def test_conflicting_human_identity_and_non_utf8_output_are_rejected(self):
        with patch.object(packager.subprocess, "check_output", side_effect=[
            json.dumps({"source_commit": SOURCE, "version": VERSION, "profile": "release"}).encode("utf-8"),
            b"another executable",
        ]), self.assertRaisesRegex(SystemExit, "disagrees"):
            packager.release_identity(["unused"], VERSION, SOURCE)
        metadata = json.dumps({"source_commit": SOURCE, "version": VERSION, "profile": "release"})
        for flag in ["--build-info", "--version"]:
            program = (
                f"import sys; output = bytes([255]) if sys.argv[-1] == {flag!r} "
                f"else {metadata.encode('utf-8')!r}; sys.stdout.buffer.write(output)"
            )
            with self.subTest(flag=flag), patch.object(threading, "excepthook") as reader_error:
                with self.assertRaises(UnicodeDecodeError):
                    packager.release_identity([sys.executable, "-c", program], VERSION, SOURCE)
                reader_error.assert_not_called()

    def test_identity_probe_is_bounded_and_utf8_even_when_calling_through_a_runner(self):
        metadata = {"source_commit": SOURCE, "version": VERSION, "profile": "release"}
        identity = f"Tupi Battle Grounds {VERSION} · source {SOURCE}"
        with patch.object(packager.subprocess, "check_output", side_effect=[
            json.dumps(metadata).encode("utf-8"), identity.encode("utf-8"),
        ]) as call:
            packager.release_identity(["runner", "client.exe"], VERSION, SOURCE)
        for args in call.call_args_list:
            self.assertEqual(args.args[0][:2], ["runner", "client.exe"])
            self.assertEqual(args.kwargs, {"timeout": 30})


class WindowsWorkflow(unittest.TestCase):
    def setUp(self):
        spec = importlib.util.spec_from_file_location("ci_gate", ROOT / "scripts/ci-required-checks.py")
        gate = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(gate)
        self.jobs = gate.workflow_jobs((ROOT / ".github/workflows/native-release.yml").read_text(encoding="utf-8"))
        self.windows = self.jobs["windows"]

    def test_release_and_regressions_share_the_reviewed_optimized_configuration(self):
        config = tomllib.loads((ROOT / ".github/cargo-windows-release.toml").read_text(encoding="utf-8"))
        self.assertEqual(config, {"profile": {"release": {"package": {"tbg-client": {"codegen-units": 16}}}}})
        cargo = [line.strip() for line in self.windows.splitlines() if line.strip().startswith("cargo ")]
        self.assertEqual(len(cargo), 3)
        for line in cargo:
            for required in ["--config .github/cargo-windows-release.toml", "--locked", "--release",
                             "-p tbg-client --bin tbg-client", "--target x86_64-pc-windows-msvc",
                             "--features ui-acceptance"]:
                self.assertIn(required, line)
        self.assertIn("--no-run", cargo[0])
        self.assertIn("platform:: -- --test-threads=1", cargo[1])
        self.assertIn(" build ", cargo[2])

    def test_completed_dependency_cache_is_not_a_release_or_validation_bypass(self):
        before = self.windows.index("'CACHE_ON_FAILURE=false'")
        compiled = self.windows.index("--no-run")
        armed = self.windows.index("'CACHE_ON_FAILURE=true'")
        self.assertLess(self.windows.index("- uses: Swatinem/rust-cache@"), before)
        self.assertLess(before, compiled)
        self.assertLess(compiled, armed)
        self.assertLess(self.windows.index("test-windows-installer.ps1"), compiled)
        for required in ["check-windows-package.ps1", "scripts/package-native.py --platform windows",
                         "scripts/package-windows-installer.ps1", "RUSTFLAGS: -C target-feature=+crt-static"]:
            self.assertIn(required, self.windows)
        self.assertNotIn("cache-workspace-crates: true", self.windows)
        self.assertNotIn("CARGO_BUILD_JOBS: 2", self.windows)
        self.assertIn("needs: [validate, linux, windows, promotion-source]", self.jobs["publish"])

    def test_inno_uses_the_reviewed_distribution_and_engine_version_not_pe_placeholders(self):
        spec = json.loads((ROOT / "deploy/native/windows/inno-toolchain.json").read_text(encoding="utf-8"))
        self.assertEqual(spec, {
            "version": "6.7.1",
            "url": "https://github.com/jrsoftware/issrc/releases/download/is-6_7_1/innosetup-6.7.1.exe",
            "sha256": "4d11e8050b6185e0d49bd9e8cc661a7a59f44959a621d31d11033124c4e8a7b0",
        })
        installer = (ROOT / "deploy/native/windows/tbg.iss").read_text(encoding="utf-8")
        self.assertIn("#if PREPROCVER != 0x06070100", installer)
        setup = (ROOT / "scripts/setup-windows-inno.ps1").read_text(encoding="utf-8")
        self.assertLess(setup.index('if ($hash -ne $spec.sha256)'), setup.index('Start-Process -FilePath $setup'))
        self.assertIn('/CURRENTUSER /PORTABLE=1', setup)
        self.assertLess(self.windows.index('scripts/setup-windows-inno.ps1'), self.windows.index('scripts/windows-env.ps1'))
        packager = (ROOT / "scripts/package-windows-installer.ps1").read_text(encoding="utf-8")
        self.assertNotIn('VersionInfo.ProductVersion', packager)
        self.assertIn('$env:TBG_INNO_COMPILER', packager)


if __name__ == "__main__":
    unittest.main()
