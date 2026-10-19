# Evidence writer for OM9-L2-01-03 (scratch; run from H:\OpenMatrix9 with the L: target dir).
$ErrorActionPreference = 'Stop'
$env:CARGO_TARGET_DIR = 'L:\om9-target'
$root = 'H:\OpenMatrix9'
$sa = $root   # crates live at the repository root now
$ev = Join-Path $root 'implementation\evidence\M01'
$stamp = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
Set-Location $sa

function Invoke-Cargo {
    param([string[]]$Arguments, [string]$OutFile, [string]$Header)
    $cmd = 'cargo ' + ($Arguments -join ' ')
    $text = & cargo @Arguments 2>&1 | Out-String
    $code = $LASTEXITCODE
    $block = "# $Header`n# command: $cmd`n# cwd: $sa`n# utc: $stamp`n# exit: $code`n`n$text"
    Set-Content -Path $OutFile -Value $block -Encoding utf8
    return $code
}

# Per L3: focused targets (each with an optional test-name filter) and the crate suites.
$focused = [ordered]@{
  'OM9-L3-01-03-001' = @(@('test','-p','om9-core','--test','document_graph','l3_001'), @('test','-p','om9-archive','--test','document_identity_round_trip'))
  'OM9-L3-01-03-002' = @(@('test','-p','om9-core','--test','document_graph','l3_002'))
  'OM9-L3-01-03-003' = @(@('test','-p','om9-core','--test','document_graph','l3_003'))
  'OM9-L3-01-03-004' = @(@('test','-p','om9-core','--test','document_graph','l3_004'))
  'OM9-L3-01-03-005' = @(@('test','-p','om9-core','--test','document_graph','l3_005'))
  'OM9-L3-01-03-006' = @(@('test','-p','om9-core','--test','document_graph','l3_006'))
  'OM9-L3-01-03-007' = @(@('test','-p','om9-core','--test','document_graph','l3_007'))
  'OM9-L3-01-03-008' = @(@('test','-p','om9-core','--test','document_graph','l3_008'), @('test','-p','om9-scene','--test','shared_instances'))
  'OM9-L3-01-03-009' = @(@('test','-p','om9-core','--test','document_graph','l3_009'))
  'OM9-L3-01-03-010' = @(@('test','-p','om9-core','--test','selection_state'), @('test','-p','om9-app','--test','document_contract','l3_010'))
  'OM9-L3-01-03-011' = @(@('test','-p','om9-core','--test','document_changes'), @('test','-p','om9-app','--test','document_contract','l3_011'))
  'OM9-L3-01-03-012' = @(@('test','-p','om9-archive','--test','document_snapshot'), @('test','-p','om9-app','--test','document_contract','l3_012'))
  'OM9-L3-01-03-013' = @(@('test','-p','om9-core','--test','provenance'))
}
$suites = [ordered]@{
  'core'    = @('test','-p','om9-core')
  'archive' = @('test','-p','om9-archive')
  'scene'   = @('test','-p','om9-scene')
  'app'     = @('test','-p','om9-app')
}
$usesSuite = @{
  'OM9-L3-01-03-001' = @('core','archive'); 'OM9-L3-01-03-002' = @('core'); 'OM9-L3-01-03-003' = @('core')
  'OM9-L3-01-03-004' = @('core'); 'OM9-L3-01-03-005' = @('core'); 'OM9-L3-01-03-006' = @('core')
  'OM9-L3-01-03-007' = @('core'); 'OM9-L3-01-03-008' = @('core','scene'); 'OM9-L3-01-03-009' = @('core')
  'OM9-L3-01-03-010' = @('core','app'); 'OM9-L3-01-03-011' = @('core','app')
  'OM9-L3-01-03-012' = @('archive','app'); 'OM9-L3-01-03-013' = @('core')
}

# 1. Suite runs (once each) into L:, then copied into every L3 that needs them.
$suiteFiles = @{}
foreach ($name in $suites.Keys) {
    $file = "L:\evidence-$name.log"
    $code = Invoke-Cargo -Arguments $suites[$name] -OutFile $file -Header "OM9-L2-01-03 regression suite: $name"
    $suiteFiles[$name] = $file
    Write-Host "suite $name exit=$code"
}

# 2. Focused logs per L3 + the regression log from the suite output.
foreach ($l3 in $focused.Keys) {
    $dir = Join-Path $ev $l3
    New-Item -ItemType Directory -Path $dir -Force | Out-Null
    $n = $l3.Substring($l3.Length - 3)
    $target = Join-Path $dir "OM9-L4-01-03-$n-03.target-test.log"
    $parts = @()
    foreach ($args in $focused[$l3]) {
        $text = & cargo @args 2>&1 | Out-String
        $code = $LASTEXITCODE
        $parts += "# command: cargo $($args -join ' ')`n# exit: $code`n`n$text"
    }
    Set-Content -Path $target -Value ("# OM9-L2-01-03 focused verification`n# utc: $stamp`n`n" + ($parts -join "`n")) -Encoding utf8
    $regression = Join-Path $dir "OM9-L4-01-03-$n-05.regression.log"
    $body = ''
    foreach ($name in $usesSuite[$l3]) { $body += (Get-Content $suiteFiles[$name] -Raw) + "`n" }
    Set-Content -Path $regression -Value $body -Encoding utf8
    Write-Host "$l3 written"
}

# 3. L3-008 also carries the L4-04 integration evidence (scene capture/picking).
$sceneSuite = Get-Content $suiteFiles['scene'] -Raw
Set-Content -Path (Join-Path $ev 'OM9-L3-01-03-008\OM9-L4-01-03-008-04.integration.log') -Value $sceneSuite -Encoding utf8

Write-Host 'evidence complete'
