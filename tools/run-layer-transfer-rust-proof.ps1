$ErrorActionPreference='Stop'
$manifest='C:/Users/nguye/.codex/worktrees/layer-session-handoff/FreeCAD-src/Mod/OpenMatrix9/rust/Cargo.toml'
$proof='H:/FreeCAD-src/build/om9-layer-edits/docs/validation/layer-session'
& rtk proxy cargo test --offline --locked --manifest-path $manifest *> (Join-Path $proof 'layer-transfer-rust-full.log')
if($LASTEXITCODE -ne 0){throw 'Rust full suite failed'}
& rtk proxy cargo fmt --manifest-path $manifest --check *> (Join-Path $proof 'layer-transfer-rust-fmt.log')
if($LASTEXITCODE -ne 0){throw 'Rust fmt check failed'}
& rtk proxy cargo clippy --offline --locked --manifest-path $manifest --all-targets -- -D warnings *> (Join-Path $proof 'layer-transfer-rust-clippy.log')
if($LASTEXITCODE -ne 0){throw 'Rust strict Clippy failed'}
Write-Output 'Full Rust suite, fmt and strict Clippy passed; retained/Matrix/full acceptance remains pending.'
