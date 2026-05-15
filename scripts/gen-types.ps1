$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$cargo = "$env:USERPROFILE\.cargo\bin\cargo.exe"
if (-not (Test-Path $cargo)) { $cargo = "cargo" }

Write-Host "[gen-types] running cargo test export_bindings ..."
& $cargo test -p placebo-shared --features export-types export_bindings -- --nocapture
if ($LASTEXITCODE -ne 0) { Write-Host "[gen-types] cargo test returned non-zero; continuing" }

$src = Join-Path $root "crates/placebo-shared/bindings"
$dst = Join-Path $root "src/types/api"

if (Test-Path $src) {
  Write-Host "[gen-types] copying bindings -> $dst"
  New-Item -ItemType Directory -Force -Path $dst | Out-Null
  Get-ChildItem $dst -Filter *.ts | Remove-Item -Force
  Copy-Item -Recurse -Force "$src\*" $dst
} else {
  Write-Host "[gen-types] no bindings/ yet, nothing to copy (OK on first run)."
}

Write-Host "[gen-types] done"
