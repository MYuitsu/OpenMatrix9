import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
HELPER = ROOT / "skills/openmatrix9-workflow/scripts/reference_index.py"


class ReferenceIndexTests(unittest.TestCase):
    def call(self, *args, root=ROOT, expected=0):
        result = subprocess.run(
            [sys.executable, str(HELPER), "--project-root", str(root), *args],
            capture_output=True, text=True, encoding="utf-8",
        )
        self.assertEqual(result.returncode, expected, result.stderr)
        self.assertTrue(result.stdout.strip(), "CLI must return a JSON result, including errors")
        return json.loads(result.stdout)

    def test_exact_feature_resolves_source_page_and_existing_spec(self):
        data = self.call("feature", "OM9-GEM-001")
        self.assertEqual(data["feature"]["command"], "gvLoader")
        self.assertEqual(data["feature"]["source_page_pdf"], 176)
        self.assertTrue(Path(data["spec_file"]).is_file())

    def test_search_keeps_subd_distinct_from_tsplines(self):
        data = self.call("search", "Clay Edit", "--domain", "14-subd")
        self.assertEqual([x["id"] for x in data["features"]], ["OM9-SUBD-001"])

    def test_ui_route_does_not_require_full_ghidra_export(self):
        data = self.call("route", "ui-menu")
        paths = [x["path"] for x in data["read_now"]]
        self.assertTrue(any(x.endswith("om9-main-001-main-menu.md") for x in paths))
        self.assertFalse(any(x.endswith("Matrix90.exe.c") for x in paths))
        self.assertTrue(all(x["exists"] for x in data["read_now"]))

    def test_each_stage_handoff_resolves_a_real_companion_skill(self):
        routes = json.loads((ROOT / "skills/openmatrix9-workflow/references/stages.json").read_text(encoding="utf-8"))
        for stage in routes["stages"]:
            with self.subTest(stage=stage):
                data = self.call("route", stage)
                self.assertTrue((ROOT / "skills" / data["skill"] / "SKILL.md").is_file())

    def test_unknown_feature_is_a_clear_error(self):
        data = self.call("feature", "OM9-UNKNOWN-999", expected=2)
        self.assertIn("OM9-UNKNOWN-999", data["error"])

    def test_missing_catalog_does_not_fallback_to_another_project(self):
        with tempfile.TemporaryDirectory() as temporary:
            data = self.call("feature", "OM9-GEM-001", root=Path(temporary), expected=2)
            self.assertIn("FEATURES.json", data["error"])

    def test_status_distinguishes_plan_and_implementation(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            plan = root / "docs/superpowers/plans/2026-10-03-matrix9-menu.md"
            plan.parent.mkdir(parents=True)
            plan.write_text("Plan only", encoding="utf-8")
            data = self.call("status", root=root)
            self.assertTrue(data["observations"]["menu_plan_exists"])
            self.assertFalse(data["observations"]["rust_menu_exists"])
            self.assertIsNone(data["ledger"])

    def test_feature_path_cannot_escape_spec_package(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            spec = root / "ref/matrix9/OpenMatrix9_Codex_Spec_v1"
            spec.mkdir(parents=True)
            (spec / "FEATURES.json").write_text(json.dumps([
                {"id": "OM9-TEST-001", "spec_path": "../../outside.md"}
            ]), encoding="utf-8")
            data = self.call("feature", "OM9-TEST-001", root=root, expected=2)
            self.assertIn("outside", data["error"])


if __name__ == "__main__":
    unittest.main()
