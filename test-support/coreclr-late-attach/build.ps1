[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$output = Join-Path $repoRoot 'local-test/coreclr-late-attach'
& (Join-Path $repoRoot 'scripts/build-monogame-native.ps1') -OutputRoot $output -Fixture
foreach ($project in @('SyntheticHost','Runner')) {
    & dotnet build (Join-Path $PSScriptRoot "$project/$project.csproj") -c Release --artifacts-path (Join-Path $output 'managed') --nologo
    if ($LASTEXITCODE -ne 0) { throw "Managed fixture build failed: $project" }
}
