# OM9 Matrix language in Options

The native OM9 Matrix add-on supplies `Line: length {} mm`. When the add-on's `language` setting is `vi`, its string catalog displays `Đoạn thẳng: dài {} mm`. The packaged default is Vietnamese. The CMD command name remains `Line`.

**Options → General → OM9 Matrix language** reads and writes the same add-on preference as Tools → Add-ons. With the add-on enabled, English or Vietnamese is applied on selection and saved to the user profile. A failed write keeps the preceding preference and translation. Options leaves the selector disabled during a command. This control applies to the labels and messages covered by the add-on's catalog.

The Windows viewport integration opens Options through CMD. It checks a Vietnamese Line summary, rejects a blocked settings-file write, selects English and reads it back from preferences, then checks the English Line summary. The existing Vietnamese and English command-input regression and all 37 add-on host unit tests passed. Tests reused `L:/om9-target`, used Cargo offline and ran with two build jobs.

[Options integration](../../build/standalone-validation/ui-artifacts/2026-11-18-snaps-current-desktop/matrix-language-options.log) · [Existing language regression](../../build/standalone-validation/ui-artifacts/2026-11-18-snaps-current-desktop/matrix-language-existing.log) · [Host unit tests](../../build/standalone-validation/ui-artifacts/2026-11-18-snaps-current-desktop/matrix-language-host.log)
