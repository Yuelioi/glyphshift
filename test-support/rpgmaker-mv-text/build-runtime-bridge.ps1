[CmdletBinding()]
param([Parameter(Mandatory)][string]$OutputRoot, [switch]$ManagedBootstrap)
$ErrorActionPreference = 'Stop'
$output = [IO.Path]::GetFullPath($OutputRoot)
$deps = Join-Path $output 'node-headers'
New-Item -ItemType Directory -Force $deps | Out-Null
$headers = @(
    @('node_api.h', 'FD20B9CE0F418FE77F1000538A609FF91460C76AF0804EDF900228EBA2663466'),
    @('node_api_types.h', '675F199CEDED237B87DB899A3306F23FDD2FDDA2A37CD45A9ABB5193F2CE2982')
)
foreach ($header in $headers) {
    $destination = Join-Path $deps $header[0]
    if (-not (Test-Path $destination)) {
        Invoke-WebRequest ('https://raw.githubusercontent.com/nodejs/node/v9.7.1/src/' + $header[0]) -OutFile $destination
    }
    if ((Get-FileHash $destination).Hash -ne $header[1]) { throw "Header hash mismatch: $($header[0])" }
}
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { throw 'Visual Studio C++ tools are required.' }
$vcvars = Join-Path $vs 'VC/Auxiliary/Build/vcvars32.bat'
$source = Join-Path $PSScriptRoot 'runtime-bridge.cpp'
$sessionSource = Join-Path $PSScriptRoot 'native-session.cpp'
$controlSource = Join-Path $PSScriptRoot 'remote-control.cpp'
$engineSource = Join-Path $PSScriptRoot 'engine-bootstrap.cpp'
$adapterBytes = [IO.File]::ReadAllBytes((Join-Path $PSScriptRoot 'adapter.cjs'))
$initializer = ($adapterBytes | ForEach-Object { [string]$_ }) -join ','
[IO.File]::WriteAllText((Join-Path $deps 'engine_adapter.h'), "#pragma once`nstatic const char embedded_adapter[] = {$initializer,0};`n")
$bootstrapDefine = if ($ManagedBootstrap) { '/DGLYPHSHIFT_MV_MANAGED_BOOTSTRAP' } else { '' }
$driverSource = Join-Path $PSScriptRoot 'remote-driver.cpp'
$archive = Join-Path $output 'minhook-1.3.4.zip'
if (-not (Test-Path $archive)) {
    Invoke-WebRequest 'https://codeload.github.com/TsudaKageyu/minhook/zip/refs/tags/v1.3.4' -OutFile $archive
}
if ((Get-FileHash $archive).Hash -ne '172708123DAA0C98D20D3A980B16A50BE14AF243DC95DEE6F79C24193AD010E4') { throw 'MinHook archive hash mismatch' }
Expand-Archive -LiteralPath $archive -DestinationPath $output -Force
$minhook = Join-Path $output 'minhook-1.3.4'
$hookSources = @('buffer.c', 'hook.c', 'trampoline.c', 'hde/hde32.c') | ForEach-Object { '"' + (Join-Path $minhook "src/$_") + '"' }
$hookInclude = Join-Path $minhook 'include'
$dll = Join-Path $output 'glyphshift_mv_fixture.dll'
$build = Join-Path $output 'compile.cmd'
@('@echo off', "call `"$vcvars`" >nul", "cl /nologo /c /TC /W3 /O2 $($hookSources -join ' ')", 'if errorlevel 1 exit /b 1',
    "cl /nologo /std:c++17 /EHsc /W4 /WX /LD /O2 $bootstrapDefine /I`"$deps`" /I`"$hookInclude`" `"$source`" `"$sessionSource`" `"$controlSource`" `"$engineSource`" buffer.obj hook.obj trampoline.obj hde32.obj /link /OUT:`"$dll`"",
    'if errorlevel 1 exit /b 1', "cl /nologo /std:c++17 /EHsc /W4 /WX /O2 `"$driverSource`" /Fe:glyphshift_mv_remote_driver.exe") | Set-Content $build
Push-Location $output
try { & $build; if ($LASTEXITCODE -ne 0) { throw 'MV Runtime bridge build failed.' } }
finally { Pop-Location }
