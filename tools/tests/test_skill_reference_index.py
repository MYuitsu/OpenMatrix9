import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
HELPER = ROOT / "skills/openmatrix9-workflow/scripts/reference_index.py"
spec = importlib.util.spec_from_file_location("reference_index", HELPER)
router = importlib.util.module_from_spec(spec)
spec.loader.exec_module(router)


class ReferenceIndexTests(unittest.TestCase):
    def call(self, *args, root=ROOT, expected=0):
        result = subprocess.run([sys.executable, str(HELPER), "--project-root", str(root), *args], capture_output=True, text=True, encoding="utf-8")
        self.assertEqual(result.returncode, expected, result.stdout + result.stderr)
        return json.loads(result.stdout)

    def record(self, root, relative, identifier):
        path = root / "OM9_Milestones" / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps({"id": identifier, "title": "Test", "level": "L2", "milestone": "M02"}), encoding="utf-8")

    def test_exact_feature_resolves_actual_m02_selection_contract(self):
        data = self.call("feature", "OM9-L2-01-07")
        self.assertEqual(data["feature"]["domain"], "M02")
        self.assertEqual(Path(data["spec_file"]).name, "L2.json")
        self.assertIn("M02 - Viewport, Gumball", data["spec_file"])
        self.assertTrue(all(x["exists"] for x in data["read_next"]))

    def test_search_filters_milestone_and_preserves_ids(self):
        data = self.call("search", "Subobject Face Edge Vertex", "--domain", "M02")
        self.assertEqual(data["features"][0]["id"], "OM9-L3-01-07-003")
        self.assertTrue(all(x["domain"] == "M02" for x in data["features"]))

    def test_all_required_stage_documents_exist(self):
        routes = json.loads(router.ROUTES.read_text(encoding="utf-8"))
        for stage in routes["stages"]:
            with self.subTest(stage=stage):
                data = self.call("route", stage)
                self.assertTrue((ROOT / "skills" / data["skill"] / "SKILL.md").is_file())
                self.assertTrue(all(x["exists"] for x in data["read_now"]))
                self.assertFalse(any(x["path"].endswith("Matrix90.exe.c") for x in data["read_now"]))

    def test_unknown_id_is_a_clear_error(self):
        data = self.call("feature", "OM9-UNKNOWN-999", expected=2)
        self.assertIn("OM9-UNKNOWN-999", data["error"])

    def test_missing_package_does_not_fallback_to_another_project(self):
        with tempfile.TemporaryDirectory() as temporary:
            data = self.call("feature", "OM9-L2-01-07", root=Path(temporary), expected=2)
            self.assertIn("OM9_Milestones", data["error"])

    def test_status_does_not_treat_plan_as_implementation(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            plan = root / "docs/superpowers/plans/2026-10-03-matrix9-menu.md"
            plan.parent.mkdir(parents=True)
            plan.write_text("Plan only", encoding="utf-8")
            data = self.call("status", root=root)
            self.assertTrue(data["observations"]["menu_plan_exists"])
            self.assertFalse(data["observations"]["rust_menu_exists"])
            self.assertIsNone(data["ledger"])

    def test_path_cannot_escape_milestone_package(self):
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaisesRegex(ValueError, "outside package"):
                router.spec_file(Path(temporary), {"id": "test", "spec_path": "../../outside.md"})

    def test_duplicate_ids_fail_instead_of_picking_arbitrary_record(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.record(root, "M01/A/L2.json", "OM9-L2-01-07")
            self.record(root, "M02/B/L2.json", "OM9-L2-01-07")
            data = self.call("feature", "OM9-L2-01-07", root=root, expected=2)
            self.assertIn("Duplicate milestone ID", data["error"])

    def test_reference_json_is_not_an_executable_feature_record(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.record(root, "M02/A/L2.json", "OM9-L2-01-07")
            self.record(root, "M02/A/reference/source/l4.json", "OM9-L2-01-07")
            self.assertEqual(len(router.catalog(root)), 1)


if __name__ == "__main__":
    unittest.main()
