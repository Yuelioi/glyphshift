[CmdletBinding()]
param(
    [switch]$ArchiveUia
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\..\..\..\..')).Path
if (-not $ArchiveUia) {
    throw 'UIA is archive-only. Pass -ArchiveUia only after explicit user authorization.'
}

& cargo build --manifest-path (Join-Path $repoRoot 'Cargo.toml') `
    -p glyphshift-windows-runtime-target
if ($LASTEXITCODE -ne 0) {
    throw "UIA archive fixture build failed with exit code $LASTEXITCODE"
}

& cargo test `
    --manifest-path (Join-Path $PSScriptRoot '..\uia\Cargo.toml') `
    archive_uia_ `
    -- `
    --ignored
if ($LASTEXITCODE -ne 0) {
    throw "UIA archive contract failed with exit code $LASTEXITCODE"
}

& cargo test `
    --manifest-path (Join-Path $PSScriptRoot 'Cargo.toml') `
    archive_uia_ `
    -- `
    --ignored
if ($LASTEXITCODE -ne 0) {
    throw "UIA worker archive contract failed with exit code $LASTEXITCODE"
}
