[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$VcpkgRoot,

    [Parameter(Mandatory)]
    [string]$TessdataRoot,

    [string]$OutputRoot,

    [switch]$Force
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$pinnedVcpkgCommit = 'eaca4a577b6b678c6e10252754b6988a61746c19'
$pinnedTessdataCommit = '87416418657359cb625c412a48b6e1d6d41c29bd'
$expectedModelHashes = [ordered]@{
    'chi_sim.traineddata' = 'a5fcb6f0db1e1d6d8522f39db4e848f05984669172e584e8d76b6b3141e1f730'
    'eng.traineddata' = '7d4322bd2a7749724879683fc3912cb542f19906c83bcc1a52132556427170b2'
}

$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$localTestRoot = [System.IO.Path]::GetFullPath((Join-Path $repoRoot 'local-test'))
if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $localTestRoot 'build\ocr-runtime-support'
}

function Resolve-ExistingDirectory([string]$Candidate, [string]$Purpose) {
    if (-not (Test-Path -LiteralPath $Candidate -PathType Container)) {
        throw "$Purpose must be an existing directory."
    }
    return (Resolve-Path -LiteralPath $Candidate).Path
}

function Assert-LocalTestPath([string]$Candidate, [string]$Purpose) {
    $prefix = $localTestRoot.TrimEnd('\') + '\'
    if (-not $Candidate.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "$Purpose must stay below the repository local-test root."
    }
}

function Invoke-GitText([string]$Root, [string[]]$Arguments, [string]$Purpose) {
    $output = @(& git -C $Root @Arguments 2>&1)
    if ($LASTEXITCODE -ne 0) {
        throw "$Purpose failed."
    }
    return (($output -join "`n").Trim())
}

function Assert-PinnedCheckout([string]$Root, [string]$ExpectedCommit, [string]$Purpose) {
    $head = Invoke-GitText $Root @('rev-parse', 'HEAD') "$Purpose revision check"
    if ($head -ne $ExpectedCommit) {
        throw "$Purpose must be checked out at the pinned revision."
    }
    $trackedChanges = Invoke-GitText $Root @('status', '--porcelain', '--untracked-files=no') `
        "$Purpose worktree check"
    if (-not [string]::IsNullOrWhiteSpace($trackedChanges)) {
        throw "$Purpose contains tracked modifications."
    }
}

function Read-NormalizedText([string]$Path) {
    return [System.IO.File]::ReadAllText($Path, [System.Text.Encoding]::UTF8).Replace("`r`n", "`n")
}

function Write-Utf8Text([string]$Path, [string]$Text) {
    [System.IO.File]::WriteAllText(
        $Path,
        $Text,
        [System.Text.UTF8Encoding]::new($false)
    )
}

function Replace-RequiredText(
    [string]$Text,
    [string]$OldValue,
    [string]$NewValue,
    [string]$Purpose
) {
    if (-not $Text.Contains($OldValue)) {
        throw "Pinned port layout changed while applying: $Purpose"
    }
    return $Text.Replace($OldValue, $NewValue)
}

function Assert-PortVersion([string]$Path, [string]$ExpectedVersion, [string]$Purpose) {
    $manifest = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    if ($manifest.version -ne $ExpectedVersion) {
        throw "$Purpose version no longer matches the reviewed OCR closure."
    }
}

if (-not $IsWindows) {
    throw 'OCR runtime support can only be built on Windows.'
}

$VcpkgRoot = Resolve-ExistingDirectory $VcpkgRoot 'vcpkg root'
$TessdataRoot = Resolve-ExistingDirectory $TessdataRoot 'tessdata_fast root'
$OutputRoot = [System.IO.Path]::GetFullPath($OutputRoot)
$workRoot = "$OutputRoot.work"
$stagingRoot = "$OutputRoot.staging"

Assert-LocalTestPath $VcpkgRoot 'vcpkg root'
Assert-LocalTestPath $TessdataRoot 'tessdata_fast root'
Assert-LocalTestPath $OutputRoot 'OCR support output'
Assert-LocalTestPath $workRoot 'OCR support work directory'
Assert-LocalTestPath $stagingRoot 'OCR support staging directory'
Assert-PinnedCheckout $VcpkgRoot $pinnedVcpkgCommit 'vcpkg'
Assert-PinnedCheckout $TessdataRoot $pinnedTessdataCommit 'tessdata_fast'

$vcpkgExecutable = Join-Path $VcpkgRoot 'vcpkg.exe'
if (-not (Test-Path -LiteralPath $vcpkgExecutable -PathType Leaf)) {
    throw 'The pinned vcpkg checkout has not been bootstrapped.'
}

$portRoot = Join-Path $VcpkgRoot 'ports'
Assert-PortVersion (Join-Path $portRoot 'tesseract\vcpkg.json') '5.5.2' 'Tesseract port'
Assert-PortVersion (Join-Path $portRoot 'leptonica\vcpkg.json') '1.87.0' 'Leptonica port'
Assert-PortVersion (Join-Path $portRoot 'libpng\vcpkg.json') '1.6.58' 'libpng port'
Assert-PortVersion (Join-Path $portRoot 'zlib\vcpkg.json') '1.3.2' 'zlib port'

foreach ($model in $expectedModelHashes.GetEnumerator()) {
    $modelPath = Join-Path $TessdataRoot $model.Key
    if (-not (Test-Path -LiteralPath $modelPath -PathType Leaf)) {
        throw "Missing pinned tessdata_fast model: $($model.Key)"
    }
    $actualHash = (Get-FileHash -LiteralPath $modelPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualHash -ne $model.Value) {
        throw "Pinned tessdata_fast model hash mismatch: $($model.Key)"
    }
}
$tessdataLicense = Join-Path $TessdataRoot 'LICENSE'
if (-not (Test-Path -LiteralPath $tessdataLicense -PathType Leaf)) {
    throw 'The pinned tessdata_fast checkout must contain LICENSE.'
}

if ((Test-Path -LiteralPath $OutputRoot) -and -not $Force) {
    throw 'OCR support output already exists. Pass -Force to replace this local-only output.'
}
foreach ($temporaryRoot in @($workRoot, $stagingRoot)) {
    if (Test-Path -LiteralPath $temporaryRoot) {
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}

$overlayRoot = Join-Path $workRoot 'overlay'
$installRoot = Join-Path $workRoot 'installed'
New-Item -ItemType Directory -Path $overlayRoot -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $portRoot 'leptonica') -Destination $overlayRoot -Recurse
Copy-Item -LiteralPath (Join-Path $portRoot 'tesseract') -Destination $overlayRoot -Recurse

$leptonicaManifestPath = Join-Path $overlayRoot 'leptonica\vcpkg.json'
$leptonicaManifest = Get-Content -LiteralPath $leptonicaManifestPath -Raw | ConvertFrom-Json
$leptonicaManifest.dependencies = @(
    'libpng',
    [ordered]@{ name = 'vcpkg-cmake'; host = $true },
    [ordered]@{ name = 'vcpkg-cmake-config'; host = $true }
)
Write-Utf8Text $leptonicaManifestPath (($leptonicaManifest | ConvertTo-Json -Depth 10) + "`n")

$leptonicaPortPath = Join-Path $overlayRoot 'leptonica\portfile.cmake'
$leptonicaPort = Read-NormalizedText $leptonicaPortPath
$leptonicaOriginalOptions = @'
        -DCMAKE_REQUIRE_FIND_PACKAGE_GIF=TRUE
        -DCMAKE_REQUIRE_FIND_PACKAGE_JPEG=TRUE
        -DCMAKE_REQUIRE_FIND_PACKAGE_PNG=TRUE
        -DCMAKE_REQUIRE_FIND_PACKAGE_TIFF=TRUE
        -DCMAKE_REQUIRE_FIND_PACKAGE_ZLIB=TRUE
'@
$leptonicaSlimOptions = @'
        -DENABLE_GIF=OFF
        -DENABLE_JPEG=OFF
        -DENABLE_OPENJPEG=OFF
        -DCMAKE_REQUIRE_FIND_PACKAGE_PNG=TRUE
        -DENABLE_TIFF=OFF
        -DENABLE_WEBP=OFF
        -DCMAKE_REQUIRE_FIND_PACKAGE_ZLIB=TRUE
'@
$leptonicaPort = Replace-RequiredText $leptonicaPort $leptonicaOriginalOptions `
    $leptonicaSlimOptions 'Leptonica image dependency closure'
Write-Utf8Text $leptonicaPortPath $leptonicaPort

$tesseractManifestPath = Join-Path $overlayRoot 'tesseract\vcpkg.json'
$tesseractManifest = Get-Content -LiteralPath $tesseractManifestPath -Raw | ConvertFrom-Json
$tesseractManifest.dependencies = @($tesseractManifest.dependencies | Where-Object {
    $dependencyName = if ($_ -is [string]) { $_ } else { $_.name }
    $dependencyName -notin @('curl', 'libarchive')
})
Write-Utf8Text $tesseractManifestPath (($tesseractManifest | ConvertTo-Json -Depth 10) + "`n")

$tesseractPortPath = Join-Path $overlayRoot 'tesseract\portfile.cmake'
$tesseractPort = Read-NormalizedText $tesseractPortPath
$tesseractPort = Replace-RequiredText $tesseractPort `
    '        -DCMAKE_REQUIRE_FIND_PACKAGE_LibArchive=ON' `
    '        -DCMAKE_DISABLE_FIND_PACKAGE_LibArchive=ON' `
    'Tesseract archive dependency'
$tesseractPort = Replace-RequiredText $tesseractPort `
    '        -DCMAKE_REQUIRE_FIND_PACKAGE_CURL=ON' `
    '        -DCMAKE_DISABLE_FIND_PACKAGE_CURL=ON' `
    'Tesseract network dependency'
$tesseractPort = Replace-RequiredText $tesseractPort `
    '        -DCMAKE_DISABLE_FIND_PACKAGE_OpenCL=ON' `
    "        -DCMAKE_DISABLE_FIND_PACKAGE_OpenCL=ON`n        -DGRAPHICS_DISABLED=ON`n        -DCMAKE_CXX_FLAGS=/DTESSERACT_DISABLE_DEBUG_FONTS" `
    'Tesseract production-only build options'
$tesseractConfigRewrite = @'
vcpkg_replace_string("${CURRENT_PACKAGES_DIR}/share/tesseract/TesseractConfig.cmake"
    "find_dependency(Leptonica)"
[[
find_dependency(CURL)
find_dependency(Leptonica)
find_dependency(LibArchive)
if(ANDROID)
    find_dependency(CpuFeaturesNdkCompat CONFIG)
endif()
]]
)

'@
$tesseractPort = Replace-RequiredText $tesseractPort $tesseractConfigRewrite '' `
    'Tesseract package dependency metadata'
Write-Utf8Text $tesseractPortPath $tesseractPort

New-Item -ItemType Directory -Path $installRoot -Force | Out-Null
Write-Output 'Building the pinned slim OCR native closure...'
& $vcpkgExecutable install 'tesseract:x64-windows' `
    "--overlay-ports=$overlayRoot" `
    "--x-install-root=$installRoot" `
    '--clean-after-build'
if ($LASTEXITCODE -ne 0) {
    throw 'The slim OCR native closure did not build.'
}

$runtimeBin = Join-Path $installRoot 'x64-windows\bin'
$requiredDllNames = @(
    'leptonica-1.87.0.dll',
    'libpng16.dll',
    'tesseract55.dll',
    'z.dll'
)
$actualDllNames = @(Get-ChildItem -LiteralPath $runtimeBin -Filter '*.dll' -File |
    Sort-Object Name | Select-Object -ExpandProperty Name)
if (($actualDllNames -join "`n") -ne (($requiredDllNames | Sort-Object) -join "`n")) {
    throw 'The built OCR DLL closure differs from the reviewed four-library allowlist.'
}

New-Item -ItemType Directory -Path $stagingRoot -Force | Out-Null
foreach ($dllName in $requiredDllNames) {
    Copy-Item -LiteralPath (Join-Path $runtimeBin $dllName) -Destination $stagingRoot
}
foreach ($modelName in $expectedModelHashes.Keys) {
    Copy-Item -LiteralPath (Join-Path $TessdataRoot $modelName) -Destination $stagingRoot
}

$shareRoot = Join-Path $installRoot 'x64-windows\share'
$licenseSources = [ordered]@{
    'tesseract-LICENSE.txt' = Join-Path $shareRoot 'tesseract\copyright'
    'leptonica-LICENSE.txt' = Join-Path $shareRoot 'leptonica\copyright'
    'libpng-LICENSE.txt' = Join-Path $shareRoot 'libpng\copyright'
    'zlib-LICENSE.txt' = Join-Path $shareRoot 'zlib\copyright'
    'tessdata-fast-LICENSE.txt' = $tessdataLicense
}
foreach ($license in $licenseSources.GetEnumerator()) {
    if (-not (Test-Path -LiteralPath $license.Value -PathType Leaf)) {
        throw "Missing OCR dependency license: $($license.Key)"
    }
    Copy-Item -LiteralPath $license.Value -Destination (Join-Path $stagingRoot $license.Key)
}

$notice = @"
GlyphShift OCR runtime support notices

This local support set contains:
- Tesseract 5.5.2 (Apache-2.0)
- Leptonica 1.87.0 (see leptonica-LICENSE.txt)
- libpng 1.6.58 (libpng-2.0)
- zlib 1.3.2 (Zlib)
- tessdata_fast chi_sim and eng models at $pinnedTessdataCommit (Apache-2.0)

The reviewed Windows x64 build disables Tesseract network, archive, OpenCL,
ScrollView graphics, and debug-font support. Leptonica retains only PNG and
zlib support because the worker supplies bounded in-memory RGBA frames.
"@
Write-Utf8Text (Join-Path $stagingRoot 'THIRD-PARTY-NOTICES.txt') ($notice.Trim() + "`n")

$artifactEntries = @(Get-ChildItem -LiteralPath $stagingRoot -File | Sort-Object Name |
    ForEach-Object {
        [ordered]@{
            file = $_.Name
            sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
            bytes = $_.Length
        }
    })
$supportManifest = [ordered]@{
    schema = 'glyphshift/ocr-runtime-support/1'
    platform = 'windows-x86_64'
    source_revisions = [ordered]@{
        vcpkg = $pinnedVcpkgCommit
        tessdata_fast = $pinnedTessdataCommit
    }
    components = [ordered]@{
        tesseract = '5.5.2'
        leptonica = '1.87.0'
        libpng = '1.6.58'
        zlib = '1.3.2'
    }
    artifacts = $artifactEntries
}
Write-Utf8Text (Join-Path $stagingRoot 'ocr-support.json') `
    (($supportManifest | ConvertTo-Json -Depth 10) + "`n")

if (Test-Path -LiteralPath $OutputRoot) {
    Remove-Item -LiteralPath $OutputRoot -Recurse -Force
}
Move-Item -LiteralPath $stagingRoot -Destination $OutputRoot

$outputFiles = @(Get-ChildItem -LiteralPath $OutputRoot -File)
$outputBytes = ($outputFiles | Measure-Object Length -Sum).Sum
Write-Output "OCR runtime support ready: $($outputFiles.Count) files, $outputBytes bytes."
Write-Output 'The default Runtime Bundle build will use this verified OCR support directory.'
