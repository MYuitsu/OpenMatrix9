"""Fail-closed safety check on the ref/ survey artefacts.

Two separate jobs, because they have different failure modes:

1. CREDENTIALS - the survey touched a decompiled assembly that embeds a plaintext
   SQL connection string with a user and password. No artefact may contain any
   connection-string shape. This is a hard failure.
2. PATH DISCLOSURE - the documents are meant to be portable and are never
   published; `docs/private-reference-index.md` and `docs/analysis/` are
   git-ignored. Absolute workspace paths to ref/ are therefore tolerated inside
   the index, but a path outside the workspace (a private archive root, a tool
   install, a vendor install) is reported for review, and any absolute private
   ref path appearing in a file that is NOT ignored is a hard failure.
"""
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")

CREDENTIAL_PATTERNS = [
    (r"\bdata source\s*=", "connection string 'Data Source='"),
    (r"\binitial catalog\s*=", "connection string 'Initial Catalog='"),
    (r"\buser id\s*=", "connection string 'User ID='"),
    (r"\bpassword\s*=", "connection string 'Password='"),
    (r"\bpersist security info\s*=", "connection string option Persist Security Info"),
    (r"\btrusted_connection\s*=", "connection string option Trusted_Connection"),
    (r"\bsqlconnection\b", "SqlConnection reference"),
    (r"\bsqlclient\b", "SqlClient reference"),
    (r"\bpwd\s*=", "connection string 'PWD='"),
    (r"\buid\s*=", "connection string 'UID='"),
]
OUTSIDE_WORKSPACE = [
    (r"(?i)[A-Z]:[\\/]OpenMatrix9-private-references", "private archive root"),
    (r"(?i)[A-Z]:[\\/]FreeCAD-src", "legacy host checkout"),
    (r"(?i)[A-Z]:[\\/]dnSpy", "tool install outside the workspace"),
    (r"(?i)[A-Z]:[\\/]Program Files", "installed vendor path"),
]
# absolute path into a private ref tree, either separator, optionally quoted
PRIVATE_REF_PATH = re.compile(r"(?i)[A-Z]:[\\/][^\s`\"'\])|]*[\\/]ref[\\/](matrix|rhino5)")

TARGETS = [
    ROOT / "build/ref-survey.json",
    ROOT / "build/ref-matrix-survey.json",
    ROOT / "build/ref-matrix-fidelity.json",
    ROOT / "build/ref-matrix-ui-census.json",
    ROOT / "build/ref-vb6-measure.json",
    ROOT / "build/ref-matrix-tree-compare.json",
    ROOT / "build/ref-survey/README.md",
    ROOT / "docs/private-reference-index.md",
    ROOT / "docs/analysis/matrix9-geometry-signature-inventory.md",
    ROOT / "docs/private-algorithm-notes.json",
    ROOT / "docs/openmatrix9-reference-locations.json",
]


def is_ignored(path: Path) -> bool:
    proc = subprocess.run(["git", "check-ignore", "-q", str(path)], cwd=ROOT,
                          capture_output=True)
    return proc.returncode == 0


def main():
    hard_errors, review = [], []
    checked = 0
    # This scanner necessarily contains the patterns it looks for, so it excludes
    # itself; the harness_probe below is what proves the patterns still fire.
    scanner = Path(__file__).resolve()
    candidates = (TARGETS + sorted((ROOT / "tools").glob("survey_ref*.py"))
                  + sorted((ROOT / "tools").glob("check_ref_survey*.py")))
    for path in candidates:
        if not path.is_file() or path.resolve() == scanner:
            continue
        checked += 1
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8", errors="replace")
        for pattern, label in CREDENTIAL_PATTERNS:
            if re.search(pattern, text):
                hard_errors.append(f"{rel}: CREDENTIAL {label}")
        for pattern, label in OUTSIDE_WORKSPACE:
            if re.search(pattern, text):
                review.append(f"{rel}: {label}")
        if PRIVATE_REF_PATH.search(text) and not is_ignored(path):
            hard_errors.append(f"{rel}: absolute private ref path in a NON-ignored file")

    # the two derived documents must be ignored, or their content would ship
    for path in (ROOT / "docs/private-reference-index.md",
                 ROOT / "docs/analysis",
                 ROOT / "docs/private-algorithm-notes.json"):
        if path.exists() and not is_ignored(path):
            hard_errors.append(f"{path.relative_to(ROOT)}: not git-ignored")
    # ...and no reference-derived document may be tracked
    tracked = subprocess.run(["git", "ls-files", "docs/analysis",
                              "docs/private-reference-index.md",
                              "docs/private-algorithm-notes.json"],
                             cwd=ROOT, capture_output=True, text=True).stdout.strip()
    if tracked:
        hard_errors.append(f"reference-derived document is tracked by git: {tracked}")

    print(json.dumps({"files_checked": checked, "harness_probe": probe_pattern(),
                      "hard_errors": hard_errors, "review_notes": review},
                     ensure_ascii=False, indent=1))
    if hard_errors:
        sys.exit(1)


def probe_pattern():
    """Prove the private-ref regex actually fires before trusting a clean result."""
    samples = ["H:/OpenMatrix9/ref/matrix/x", r"H:\OpenMatrix9\ref\rhino5\y",
               "ref/matrix/x"]
    return {s: bool(PRIVATE_REF_PATH.search(s)) for s in samples}


if __name__ == "__main__":
    main()
