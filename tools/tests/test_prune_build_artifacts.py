import importlib.util
import os
from pathlib import Path
import tempfile
import unittest


SPEC = importlib.util.spec_from_file_location("om9_prune", Path(__file__).resolve().parents[1] / "prune_build_artifacts.py")
PRUNE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PRUNE)


class CitedArtifactsTests(unittest.TestCase):
    def test_markdown_code_and_links_protect_old_runs(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            docs = root / "docs"
            docs.mkdir()
            (root / "implementation").mkdir()
            (docs / "evidence.md").write_text(
                "`build/standalone-validation/ui-artifacts/cited-code`\n"
                "[result](build/standalone-validation/ui-artifacts/cited-link/result.json)\n",
                encoding="utf-8",
            )
            runs = root / "runs"
            runs.mkdir()
            for index, name in enumerate(("cited-code", "cited-link", "unreferenced", "latest")):
                path = runs / name
                path.mkdir()
                os.utime(path, (index + 1, index + 1))
            original = PRUNE.ROOT
            try:
                PRUNE.ROOT = root
                protected = PRUNE.cited_runs()
                self.assertEqual(protected, {"cited-code", "cited-link"})
                PRUNE.prune(runs, 1, True, protected)
            finally:
                PRUNE.ROOT = original
            self.assertEqual({path.name for path in runs.iterdir()}, {"cited-code", "cited-link", "latest"})


if __name__ == "__main__":
    unittest.main()
