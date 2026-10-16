#!/usr/bin/env python3
"""A TAKEN ISSUE THAT HAS GONE QUIET IS FREED: the rule of `CONTRIBUTING` - "say you take it, and it is yours for
two weeks" - kept by a run once a day (`.github/workflows/stale-claims.yml`).

An open issue with an assignee is looked at from the last sign of work: the assignment itself or a comment by a
person (a bot's comment is no sign of work). After 12 quiet days the bot asks whether the work goes on; once the
issue is 14 days quiet and the question is at least 2 days old, the assignment is lifted and the issue said free.
An issue with an open pull request referring to it is never touched: the work is visibly going on. A maintainer
assigned to an issue is not freed either: the rule is for the claims of contributors.

    tools/stale_claims.py              act on the repository of GH_REPO through `gh`
    tools/stale_claims.py --dry-run    say what would be done, change nothing
    tools/stale_claims.py --self-test  the decision on cases written out here
"""
import argparse
import dataclasses
import datetime
import json
import os
import subprocess
import sys

WARN_AFTER = datetime.timedelta(days=12)
FREE_AFTER = datetime.timedelta(days=14)
# the warning stands at least this long before the claim is lifted, also when the run missed the twelfth day
NOTICE = datetime.timedelta(days=2)

WARNED = "<!-- stale-claim-warning -->"
FREED = "<!-- stale-claim-released -->"

WARNING_TEXT = (
    WARNED + "\n"
    "This issue has been assigned for 12 days without news here. Are you still working on it? "
    "A comment is enough to keep it. Without one, the issue becomes free for anybody to take in 2 days."
)
FREED_TEXT = (
    FREED + "\n"
    "No news for 14 days, so the assignment is lifted and the issue is free to take again. "
    "{who}, if you are still working on it, say so here and it will be yours again."
)

# the people whose assignment is their own business: a maintainer may keep an issue as long as needed
MAINTAINING = {"admin", "maintain", "write"}


@dataclasses.dataclass
class Claim:
    """What is known of a taken issue: when it was taken, the comments of people, the warnings of the bot, and
    whether an open pull request refers to it."""
    assigned_at: datetime.datetime
    human_comments: list
    warnings: list
    open_pull_request: bool


def decide(claim, now):
    """`keep`, `warn` or `free`."""
    if claim.open_pull_request:
        return "keep"
    last = max([claim.assigned_at] + claim.human_comments)
    quiet = now - last
    warned = [w for w in claim.warnings if w > last]
    if warned and quiet >= FREE_AFTER and now - max(warned) >= NOTICE:
        return "free"
    if not warned and quiet >= WARN_AFTER:
        return "warn"
    return "keep"


def gh(*args, payload=None):
    out = subprocess.run(["gh", *args], input=json.dumps(payload) if payload else None, capture_output=True, text=True)
    if out.returncode != 0:
        raise RuntimeError(f"gh {' '.join(args)}: {out.stderr.strip()}")
    return json.loads(out.stdout) if out.stdout.strip() else None


def pages(path):
    """Every item of a listing, all pages joined."""
    got = gh("api", "--paginate", "--slurp", path)
    return [item for page in got for item in page]


def when(text):
    return datetime.datetime.fromisoformat(text.replace("Z", "+00:00"))


def claim_of(repo, number):
    """The claim of issue `number`, read from its timeline."""
    assigned, people, warnings, open_pr = [], [], [], False
    for e in pages(f"repos/{repo}/issues/{number}/timeline?per_page=100"):
        kind = e.get("event")
        if kind == "assigned":
            assigned.append(when(e["created_at"]))
        elif kind == "commented":
            body = e.get("body") or ""
            if (e.get("user") or {}).get("type") == "Bot":
                if WARNED in body:
                    warnings.append(when(e["created_at"]))
            else:
                people.append(when(e["created_at"]))
        elif kind == "cross-referenced":
            source = (e.get("source") or {}).get("issue") or {}
            if source.get("pull_request") and source.get("state") == "open":
                open_pr = True
    if not assigned:
        return None
    return Claim(max(assigned), people, warnings, open_pr)


def maintains(repo, login):
    try:
        return gh("api", f"repos/{repo}/collaborators/{login}/permission")["permission"] in MAINTAINING
    except RuntimeError:
        # a person who is not a collaborator answers with an error: a contributor
        return False


def run(repo, dry):
    now = datetime.datetime.now(datetime.timezone.utc)
    for issue in pages(f"repos/{repo}/issues?state=open&assignee=*&per_page=100"):
        if issue.get("pull_request"):
            continue
        number = issue["number"]
        logins = [a["login"] for a in issue.get("assignees") or []]
        contributors = [login for login in logins if not maintains(repo, login)]
        if not contributors:
            continue
        claim = claim_of(repo, number)
        if claim is None:
            continue
        verdict = decide(claim, now)
        print(f"#{number} {contributors}: {verdict}")
        if dry or verdict == "keep":
            continue
        if verdict == "warn":
            gh("api", "-X", "POST", f"repos/{repo}/issues/{number}/comments", "--input", "-", payload={"body": WARNING_TEXT})
        else:
            gh("api", "-X", "DELETE", f"repos/{repo}/issues/{number}/assignees", "--input", "-", payload={"assignees": contributors})
            who = ", ".join("@" + login for login in contributors)
            gh("api", "-X", "POST", f"repos/{repo}/issues/{number}/comments", "--input", "-", payload={"body": FREED_TEXT.format(who=who)})


def self_test():
    day = datetime.timedelta(days=1)
    t0 = datetime.datetime(2026, 10, 1, tzinfo=datetime.timezone.utc)
    cases = [
        ("taken yesterday", Claim(t0, [], [], False), t0 + day, "keep"),
        ("quiet for 12 days", Claim(t0, [], [], False), t0 + 12 * day, "warn"),
        ("quiet for 12 days, warned already", Claim(t0, [], [t0 + 12 * day], False), t0 + 13 * day, "keep"),
        ("quiet for 14 days after a warning", Claim(t0, [], [t0 + 12 * day], False), t0 + 14 * day, "free"),
        ("a comment after the warning", Claim(t0, [t0 + 13 * day], [t0 + 12 * day], False), t0 + 14 * day, "keep"),
        ("the comment was 12 days ago", Claim(t0, [t0 + 13 * day], [t0 + 12 * day], False), t0 + 25 * day, "warn"),
        ("a missed run: quiet for 20 days, never warned", Claim(t0, [], [], False), t0 + 20 * day, "warn"),
        ("warned only a day ago", Claim(t0, [], [t0 + 19 * day], False), t0 + 20 * day, "keep"),
        ("an open pull request", Claim(t0, [], [t0 + 12 * day], True), t0 + 30 * day, "keep"),
        ("taken again after being freed", Claim(t0 + 15 * day, [], [t0 + 12 * day], False), t0 + 16 * day, "keep"),
    ]
    wrong = [f"{name}: {decide(c, now)}, not {want}" for name, c, now, want in cases if decide(c, now) != want]
    for line in wrong:
        print(line)
    print(f"{len(cases) - len(wrong)} of {len(cases)} cases decided right")
    return 1 if wrong else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    repo = os.environ.get("GH_REPO")
    if not repo:
        print("GH_REPO is not set (owner/name)", file=sys.stderr)
        return 2
    run(repo, args.dry_run)
    return 0


if __name__ == "__main__":
    sys.exit(main())
