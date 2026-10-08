# Business-rule skill updates: 2026-10-05

User instruction: finish code and verification, then ask whether changed business behavior should be persisted into skills. Prior explicit authorization for that exact skill update is sufficient. Technical fixes preserving agreed behavior do not trigger this question.

All five OpenMatrix9 SKILL.md files link to the shared policy at `skills/openmatrix9-workflow/references/business-rule-updates.md`. AGENTS.md and the skill guide record the same instruction. No product code changed for this policy update.

Validation:

- Read-only baseline review found the previous skills lacked an explicit post-verification consent rule and prior-authorization exception.
- Read-only forward review passed four scenarios: new business behavior, technical fixes, explicit prior skill authorization, and no reply/refusal. It found no policy gap or extra coding approval gate.
- All five source and five installed skills passed quick_validate with Python UTF-8 mode (`-X utf8`); the validator default Windows encoding cannot read the new Vietnamese text.
- SHA-256 matches for all five source/installed SKILL.md files and the shared policy; links resolve in both locations.
- Eight reference-index tests passed; routing audit reports 607 specs, 16 domains, six stages and no missing required or conditional documents.

Only reusable business-rule persistence needs consent. Technical evidence and pending/refused proposal status may still be recorded. Silence is not consent.
