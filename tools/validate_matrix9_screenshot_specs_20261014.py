"""Validate spec links/catalog/baselines only; never claim UI runtime parity."""
import hashlib
import json
from pathlib import Path
import re
import yaml
from update_matrix9_screenshot_specs_20261014 import FEATURES, REFERENCE, TITLE

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT / "OpenMatrix9_Codex_Spec_v2"


def read(path):
    return json.loads(path.read_text(encoding="utf-8"))


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main():
    features = read(PACKAGE / "FEATURES.json")
    assert features == yaml.safe_load((PACKAGE / "FEATURES.yaml").read_text(encoding="utf-8"))
    assert len(features) == 607 and len({f["id"] for f in features}) == 607
    assert all(f["implementation_status"] == "not_implemented_on_standalone_om9" for f in features)
    lookup = read(PACKAGE / "FEATURE_LOOKUP.json")
    indexed = {f["id"]: f for f in features}
    reference = (PACKAGE / REFERENCE).read_text(encoding="utf-8")
    assert all(f"| S{i} |" in reference for i in range(1, 7))
    assert len(re.findall(r"^\| M9IMG-\d\d \|", reference, flags=re.M)) == 12
    assert len(read(ROOT / "docs/core-requirements.json")["items"]) == 130
    capabilities = read(PACKAGE / "specs/00-rhino-core/CAPABILITY_LOOKUP.json")
    assert len(capabilities["capabilities"]) == 148
    assert capabilities["ui_reference_supplement_path"] == Path(REFERENCE).name
    assert capabilities["ui_reference_fixture_ids"] == [f"M9IMG-{i:02}" for i in range(1,13)]
    paths = {PACKAGE / REFERENCE, PACKAGE / "MATRIX9_UI_CONTRACT.md",
             PACKAGE / "specs/00-rhino-core/UI_DETAILS.md"}
    for fid, (anchor, sources, _, _) in FEATURES.items():
        f = indexed[fid]
        assert f["ui_reference_supplements"] == lookup[fid]["ui_reference_supplements"]
        metadata = next(x for x in f["ui_reference_supplements"] if x["spec_path"] == f"{REFERENCE}#{anchor}")
        assert metadata["source_ids"] == sources
        assert metadata["validation_status"] == "requirements_only_not_runtime_evidence"
        assert f'id="{anchor}"' in reference
        path = PACKAGE / f["spec_path"]
        assert path.read_text(encoding="utf-8").count(TITLE) == 1
        paths.add(path)
    links = 0
    for path in paths:
        text = path.read_text(encoding="utf-8")
        # Check new reference links plus the entire authored analysis document.
        for raw in re.findall(r"\]\(([^)]+)\)", text):
            if path != PACKAGE / REFERENCE and "MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md" not in raw:
                continue
            target, _, fragment = raw.partition("#")
            resolved = (path.parent / target).resolve() if target else path
            assert resolved.is_file(), (path, raw)
            if fragment:
                assert f'id="{fragment}"' in resolved.read_text(encoding="utf-8"), (path, raw)
            links += 1
    baseline = read(PACKAGE / "SOURCE_READ_AUDIT.json")
    for row in baseline["files"]:
        path = ROOT / "OpenMatrix9_Codex_Spec_v1" / row["path"]
        assert path.stat().st_size == row["bytes"] and sha(path) == row["sha256"], row["path"]
    manifest = read(PACKAGE / "MANIFEST.json")
    for row in manifest:
        path = PACKAGE / row["path"]
        assert path.stat().st_size == row["bytes"] and sha(path) == row["sha256"], row["path"]
    ledger = read(PACKAGE / "MIGRATION_LEDGER.json")
    entry = next(x for x in ledger["ui_reference_supplements"] if x["spec_path"] == REFERENCE)
    assert set(entry["affected_feature_ids"]) == set(FEATURES)
    assert entry["images_copied_to_repository"] is False and entry["implementation_status_promoted"] is False
    assert ledger["target_implementation_count"] == 0
    assert len(re.findall(r"^- \[ \] RCORE-00\.M\d\d", (PACKAGE / "specs/00-rhino-core/00-qymcad-foundation-migration.md").read_text(encoding="utf-8"), flags=re.M)) == 12
    print(json.dumps({"features_linked": len(FEATURES), "reference_links_checked": links,
                      "catalog_features": len(features), "core_features": 130, "capabilities": 148,
                      "acceptance_cases": 12, "v1_unchanged": len(baseline["files"]),
                      "v2_manifest": len(manifest), "images_copied": 0, "runtime_completion_claim": False}))


if __name__ == "__main__":
    main()
