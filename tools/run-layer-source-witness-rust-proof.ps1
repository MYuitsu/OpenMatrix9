$ErrorActionPreference='Stop'
$manifest='C:/Users/nguye/.codex/worktrees/layer-session-handoff/FreeCAD-src/Mod/OpenMatrix9/rust/Cargo.toml'
$proof='H:/FreeCAD-src/build/om9-layer-edits/docs/validation/layer-session'
& rtk proxy cargo fmt --manifest-path $manifest
if($LASTEXITCODE -ne 0){throw 'Rust format failed'}
& rtk proxy cargo test --offline --locked --manifest-path $manifest *> (Join-Path $proof 'layer-source-witness-rust-full.log')
if($LASTEXITCODE -ne 0){throw 'Rust full suite failed'}
& rtk proxy cargo fmt --manifest-path $manifest --check *> (Join-Path $proof 'layer-source-witness-rust-fmt.log')
if($LASTEXITCODE -ne 0){throw 'Rust format check failed'}
& rtk proxy cargo clippy --offline --locked --manifest-path $manifest --all-targets -- -D warnings *> (Join-Path $proof 'layer-source-witness-rust-clippy.log')
if($LASTEXITCODE -ne 0){throw 'Rust strict Clippy failed'}
Write-Output 'Rust full suite/fmt/strict Clippy PASS; native provenance and full Matrix handoff remain separate.'
