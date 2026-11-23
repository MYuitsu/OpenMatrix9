# SPDX-License-Identifier: LGPL-2.1-or-later
"""Exercise dependency closure and payload integrity independently of a developer SDK."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("standalone_package", ROOT / "tools/ci/package_standalone.py")
package = importlib.util.module_from_spec(spec)
spec.loader.exec_module(package)


class StandalonePackageTests(unittest.TestCase):
    def test_upstream_nested_zip_exposes_dlls(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            inner = root / "inner.zip"
            with zipfile.ZipFile(inner, "w") as archive:
                archive.writestr("tbb/bin/tbb12.dll", b"runtime")
            outer = root / "outer.zip"
            with zipfile.ZipFile(outer, "w") as archive:
                archive.write(inner, "3rdparty-vc14-64.zip")
            with package.vendor_archive(outer, root) as archive:
                self.assertEqual(archive.read("tbb/bin/tbb12.dll"), b"runtime")

    def test_transitive_dlls_are_copied_and_cycles_terminate(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            runtime = root / "runtime"
            runtime.mkdir()
            (runtime / "om9.exe").write_bytes(b"app")
            dlls = root / "sdk"
            dlls.mkdir()
            for name in ("TKernel.dll", "tbb12.dll"):
                (dlls / name).write_bytes(name.encode())
            imports = {"om9.exe": ["TKernel.dll"], "TKernel.dll": ["tbb12.dll"], "tbb12.dll": ["TKernel.dll"]}
            package.copy_dependency_closure(runtime, {p.name.lower(): p for p in dlls.iterdir()}, root / "system",
                                            lambda p: imports[p.name])
            self.assertEqual({p.name for p in runtime.iterdir()}, {"om9.exe", "TKernel.dll", "tbb12.dll"})

    def test_app_local_crt_is_bundled_even_if_developer_machine_has_it(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            runtime = root / "runtime"
            runtime.mkdir()
            (runtime / "om9.exe").write_bytes(b"app")
            system = root / "system"
            system.mkdir()
            (system / "VCRUNTIME140.dll").write_bytes(b"machine CRT")
            bundled = root / "VCRUNTIME140.dll"
            bundled.write_bytes(b"redistributable CRT")
            package.copy_dependency_closure(runtime, {"vcruntime140.dll": bundled}, system,
                                            lambda p: ["VCRUNTIME140.dll"] if p.suffix == ".exe" else [])
            self.assertEqual((runtime / "VCRUNTIME140.dll").read_bytes(), b"redistributable CRT")

    def test_os_dlls_and_api_sets_are_not_redistributed(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            runtime = root / "runtime"
            runtime.mkdir()
            (runtime / "om9.exe").write_bytes(b"app")
            system = root / "system"
            system.mkdir()
            (system / "KERNEL32.dll").write_bytes(b"OS")
            package.copy_dependency_closure(runtime, {}, system, lambda _: ["KERNEL32.dll", "api-ms-win-core-synch-l1-2-0.dll"])
            self.assertEqual(len(list(runtime.iterdir())), 1)

    def test_missing_dll_is_an_error_instead_of_a_broken_zip(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "om9.exe").write_bytes(b"app")
            with self.assertRaisesRegex(RuntimeError, "om9.exe -> missing.dll"):
                package.copy_dependency_closure(root, {}, root / "system", lambda _: ["missing.dll"])

    def test_payload_change_after_recording_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "om9.exe").write_bytes(b"original")
            files = {"om9.exe": package.sha(root / "om9.exe")}
            package.verify_manifest(root, files)
            (root / "om9.exe").write_bytes(b"changed")
            with self.assertRaisesRegex(RuntimeError, "payload changed"):
                package.verify_manifest(root, files)


    def test_all_in_one_contains_the_standalone_and_system_addons(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            standalone = root / "runtime/OpenMatrix9"
            standalone.mkdir(parents=True)
            (standalone / "om9.exe").write_bytes(b"app")
            (standalone / "README.txt").write_text("Windows standalone\n", encoding="utf-8")
            files = {path.name: package.sha(path) for path in standalone.iterdir()}
            (standalone / "release-validation.json").write_text(
                json.dumps({"version": "1.2.3", "files": files}), encoding="utf-8"
            )
            (root / "build-paths.json").write_text(
                json.dumps({"sdk": str(root / "sdk"), "crt": str(root / "crt")}),
                encoding="utf-8",
            )
            addons = root / "addons"
            addons.mkdir()
            sums = []
            for addon in ("om9_matrix", "ring_sizes"):
                path = addons / f"{addon}.zip"
                with zipfile.ZipFile(path, "w") as archive:
                    archive.writestr("om9_addon.toml", f'id = "{addon}"\nversion = "1.0.0"\n')
                sums.append(f"{package.sha(path)}  {path.name}")
            sums_path = addons / "SHA256SUMS.txt"
            sums_path.write_text("\n".join(sums) + "\n", encoding="ascii")

            from unittest.mock import patch
            with patch.object(package, "copy_dependency_closure"):
                result = package.bundle_all_in_one(root, environ={"SystemRoot": str(root)})
            with zipfile.ZipFile(result) as archive:
                self.assertEqual(archive.read("OpenMatrix9/om9.exe"), b"app")
                receipt = json.loads(archive.read("OpenMatrix9/release-validation.json"))
                self.assertEqual(receipt["system_addons"], {"om9_matrix": "1.0.0", "ring_sizes": "1.0.0"})
                for addon in receipt["system_addons"]:
                    self.assertIn(f"addons/{addon}/om9_addon.toml", receipt["files"])
            package.verify_manifest(standalone, files)

            sums_path.write_text("0" * 64 + "  om9_matrix.zip\n", encoding="ascii")
            with self.assertRaisesRegex(RuntimeError, "fingerprint mismatch"):
                package.bundle_all_in_one(root, environ={"SystemRoot": str(root)})


if __name__ == "__main__":
    unittest.main()
