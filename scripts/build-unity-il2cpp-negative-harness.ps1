param(
    [switch]$ValidateOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Assert-Contract {
    param(
        [Parameter(Mandatory = $true)]
        [bool]$Condition,
        [Parameter(Mandatory = $true)]
        [string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

function Get-PeMachine {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $stream = [System.IO.File]::OpenRead($Path)
    try {
        $reader = [System.IO.BinaryReader]::new($stream)
        $stream.Position = 0x3c
        $peOffset = $reader.ReadInt32()
        $stream.Position = $peOffset
        Assert-Contract ($reader.ReadUInt32() -eq 0x00004550) 'The built executable is not a PE image.'
        return $reader.ReadUInt16()
    }
    finally {
        $stream.Dispose()
    }
}

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$fixtureRoot = Join-Path $repositoryRoot 'test-support\unity-il2cpp-custom-text-negative'
$runtimeSource = Join-Path $fixtureRoot 'Assets\GlyphshiftCustomMeshText.cs'
$absenceGuardSource = Join-Path $fixtureRoot 'Assets\GlyphshiftStandardUiAbsenceGuard.cs'
$buildSource = Join-Path $fixtureRoot 'Assets\Editor\GlyphshiftNegativeBuild.cs'
$shaderSource = Join-Path $fixtureRoot 'Assets\GlyphshiftBitmapText.shader'
$packageManifest = Join-Path $fixtureRoot 'Packages\manifest.json'
$projectVersion = Join-Path $fixtureRoot 'ProjectSettings\ProjectVersion.txt'

$requiredFiles = @(
    $runtimeSource,
    $absenceGuardSource,
    $buildSource,
    $shaderSource,
    $packageManifest,
    $projectVersion
)
foreach ($requiredFile in $requiredFiles) {
    Assert-Contract (Test-Path -LiteralPath $requiredFile -PathType Leaf) 'The Unity negative fixture is incomplete.'
}

$forbiddenExtensions = @('.exe', '.dll', '.pdb', '.mdb', '.dbg', '.zip')
$forbiddenArtifacts = @(
    Get-ChildItem -LiteralPath $fixtureRoot -Recurse -File |
        Where-Object { $forbiddenExtensions -contains $_.Extension.ToLowerInvariant() }
)
Assert-Contract ($forbiddenArtifacts.Count -eq 0) 'Tracked Unity fixture directories must remain source-only.'
$serializedAssets = @(
    Get-ChildItem -LiteralPath (Join-Path $fixtureRoot 'Assets') -Recurse -File |
        Where-Object { $_.Extension.ToLowerInvariant() -in @('.asset', '.prefab', '.unity') }
)
Assert-Contract ($serializedAssets.Count -eq 0) 'The negative scene must be generated from inspected source.'

$runtimeText = Get-Content -Raw -LiteralPath $runtimeSource
$absenceGuardText = Get-Content -Raw -LiteralPath $absenceGuardSource
$buildText = Get-Content -Raw -LiteralPath $buildSource
$manifestText = Get-Content -Raw -LiteralPath $packageManifest
$versionText = Get-Content -Raw -LiteralPath $projectVersion

$forbiddenRuntimePatterns = @(
    'TMPro',
    'TextMeshPro',
    'UnityEngine\.UI',
    '\bCanvas\b',
    '\bRawImage\b',
    '\bOnGUI\b',
    '\bGUI\.'
)
foreach ($pattern in $forbiddenRuntimePatterns) {
    Assert-Contract (-not [regex]::IsMatch($runtimeText, $pattern)) 'The negative runtime introduced a standard UI path.'
}

Assert-Contract ($manifestText -match '"com\.unity\.ugui"\s*:\s*"1\.0\.0"') 'The negative fixture must retain the Unity UI package types.'
Assert-Contract ($manifestText -match '"com\.unity\.textmeshpro"\s*:\s*"3\.0\.6"') 'The negative fixture must retain the TextMesh Pro package types.'
$manifest = $manifestText | ConvertFrom-Json
$dependencyNames = @($manifest.dependencies.PSObject.Properties.Name)
Assert-Contract ($dependencyNames.Count -eq 2) 'The negative fixture must keep a minimal package surface.'

$requiredAbsenceGuardPatterns = @(
    'TMP_UGUI_TYPES_PRESENT_LIVE_TEXT_ZERO',
    'TmpTextTypeName = "TMPro.TMP_Text"',
    'UguiTextTypeName = "UnityEngine.UI.Text"',
    'typeof(TMP_Text).FullName',
    'typeof(UguiText).FullName',
    'Resources.FindObjectsOfTypeAll<TMP_Text>()',
    'Resources.FindObjectsOfTypeAll<UguiText>()',
    'Application.Quit(86)'
)
foreach ($pattern in $requiredAbsenceGuardPatterns) {
    Assert-Contract ($absenceGuardText.Contains($pattern)) 'The negative runtime is missing its live-object absence guard.'
}
Assert-Contract (-not $absenceGuardText.Contains('AddComponent<')) 'The live-object absence guard must not create a standard UI component.'

$allFixtureSource = $runtimeText + "`n" + $absenceGuardText + "`n" + $buildText
$allowedComponentTypes = @(
    'Camera',
    'GlyphshiftCustomMeshText',
    'GlyphshiftStandardUiAbsenceGuard',
    'MeshFilter',
    'MeshRenderer'
)
foreach ($match in [regex]::Matches($allFixtureSource, 'AddComponent\s*<\s*(?<type>[^>\s]+)\s*>')) {
    $componentType = $match.Groups['type'].Value
    Assert-Contract ($allowedComponentTypes -contains $componentType) 'The fixture created a component outside its explicit allowlist.'
}
Assert-Contract (-not [regex]::IsMatch($allFixtureSource, '\.AddComponent\s*\(')) 'The fixture must not use the untyped AddComponent API.'
foreach ($match in [regex]::Matches($allFixtureSource, 'new\s+GameObject\s*\((?<arguments>[^;]*)\)', 'Singleline')) {
    Assert-Contract (-not $match.Groups['arguments'].Value.Contains(',')) 'The fixture must not add constructor-supplied components.'
}
Assert-Contract (-not [regex]::IsMatch($allFixtureSource, '\bInstantiate\s*(?:<|\()')) 'The fixture must not instantiate an uninspected object.'
Assert-Contract (-not [regex]::IsMatch($allFixtureSource, '\bResources\.Load(?:All|Async)?\s*(?:<|\()')) 'The fixture must not load an uninspected object.'

$requiredRuntimePatterns = @(
    'StaticMarker = "CUSTOM MESH NEGATIVE"',
    'DynamicPrefix = "TICK "',
    'new Texture2D',
    'AddComponent<MeshFilter>',
    'AddComponent<MeshRenderer>',
    'SetVertices',
    'SetTriangles'
)
foreach ($pattern in $requiredRuntimePatterns) {
    Assert-Contract ($runtimeText.Contains($pattern)) 'The negative runtime is missing a deterministic mesh-text contract.'
}

$glyphOrderMatch = [regex]::Match(
    $runtimeText,
    'private const string GlyphOrder = "(?<glyphs>[^"]*)";')
$staticMarkerMatch = [regex]::Match(
    $runtimeText,
    'public const string StaticMarker = "(?<marker>[^"]*)";')
$dynamicPrefixMatch = [regex]::Match(
    $runtimeText,
    'public const string DynamicPrefix = "(?<prefix>[^"]*)";')
Assert-Contract $glyphOrderMatch.Success 'The bitmap glyph order could not be inspected.'
Assert-Contract $staticMarkerMatch.Success 'The static marker could not be inspected.'
Assert-Contract $dynamicPrefixMatch.Success 'The dynamic marker prefix could not be inspected.'

$glyphOrder = $glyphOrderMatch.Groups['glyphs'].Value
$definedGlyphs = [System.Collections.Generic.HashSet[char]]::new()
foreach ($match in [regex]::Matches($runtimeText, "\{\s*'(?<glyph>.)',\s*Rows\(")) {
    [void]$definedGlyphs.Add($match.Groups['glyph'].Value[0])
}

$displayedCharacters = @(
    [char[]](
        $staticMarkerMatch.Groups['marker'].Value +
        $dynamicPrefixMatch.Groups['prefix'].Value +
        '0123456789') |
        Sort-Object -Unique
)
foreach ($character in $displayedCharacters) {
    Assert-Contract ($glyphOrder.IndexOf($character) -ge 0) 'A displayed character is missing from the bitmap glyph order.'
    if ($character -ne ' ') {
        Assert-Contract $definedGlyphs.Contains($character) 'A displayed character is missing bitmap row data.'
    }
}

foreach ($character in [char[]]$glyphOrder) {
    if ($character -ne ' ') {
        Assert-Contract $definedGlyphs.Contains($character) 'The bitmap glyph order contains a character without row data.'
    }
}

$requiredBuildPatterns = @(
    'BuildTarget.StandaloneWindows64',
    'ScriptingImplementation.IL2CPP',
    'Il2CppCompilerConfiguration.Release',
    'BuildOptions.None',
    'EditorUserBuildSettings.development = false',
    'AddComponent<GlyphshiftStandardUiAbsenceGuard>',
    'AddComponent<GlyphshiftCustomMeshText>'
)
foreach ($pattern in $requiredBuildPatterns) {
    Assert-Contract ($buildText.Contains($pattern)) 'The negative build script is missing a shipping-like IL2CPP contract.'
}

Assert-Contract ($versionText -match 'm_EditorVersion: 2022\.3\.5f1') 'The negative fixture must stay pinned to Unity 2022.3.5f1.'
$requiredEditorVersionPattern = '^2022\.3\.5(?:f1(?:\b|\s|\()|\.)'
Assert-Contract ('2022.3.5f1' -match $requiredEditorVersionPattern) 'The Unity editor version contract rejected its pinned version.'
Assert-Contract ('2022.3.5.12345' -match $requiredEditorVersionPattern) 'The Unity editor version contract rejected a Windows product version.'
Assert-Contract ('2022.3.50f1' -notmatch $requiredEditorVersionPattern) 'The Unity editor version contract accepted a different patch release.'
Write-Output 'unity_negative_harness_static_contract=passed'

if ($ValidateOnly) {
    return
}

$editorInput = [Environment]::GetEnvironmentVariable('GLYPHSHIFT_UNITY_EDITOR')
Assert-Contract (-not [string]::IsNullOrWhiteSpace($editorInput)) 'GLYPHSHIFT_UNITY_EDITOR is required.'
Assert-Contract (Test-Path -LiteralPath $editorInput -PathType Leaf) 'The configured Unity editor does not exist.'

$editorItem = Get-Item -LiteralPath $editorInput
Assert-Contract ($editorItem.Name -eq 'Unity.exe') 'GLYPHSHIFT_UNITY_EDITOR must identify Unity.exe.'
$editorVersion = $editorItem.VersionInfo.ProductVersion
$editorVersionMatches = $editorVersion -match $requiredEditorVersionPattern
Assert-Contract $editorVersionMatches 'The configured Unity editor must be version 2022.3.5f1.'
$editorDirectory = Split-Path -Parent $editorItem.FullName
$windowsIl2CppModule = Join-Path $editorDirectory 'Data\PlaybackEngines\windowsstandalonesupport\Variations\win64_player_il2cpp'
Assert-Contract (Test-Path -LiteralPath $windowsIl2CppModule -PathType Container) 'The Unity Windows IL2CPP module is required.'

$evidenceRoot = Join-Path $repositoryRoot 'local-test\evidence\unity-il2cpp-negative-harness'
$runId = (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8)
$runRoot = Join-Path $evidenceRoot $runId
$projectRoot = Join-Path $runRoot 'project'
$buildRoot = Join-Path $runRoot 'build'
$outputPath = Join-Path $buildRoot 'GlyphshiftUnityIl2CppNegative.exe'
$createLog = Join-Path $runRoot 'create-project.log'
$buildLog = Join-Path $runRoot 'build-player.log'

New-Item -ItemType Directory -Path $runRoot -Force | Out-Null
& $editorInput -batchmode -nographics -quit -createProject $projectRoot -logFile $createLog
Assert-Contract ($LASTEXITCODE -eq 0) 'Unity failed to create the local negative project.'

$projectAssets = Join-Path $projectRoot 'Assets'
$projectPackages = Join-Path $projectRoot 'Packages'
$projectSettings = Join-Path $projectRoot 'ProjectSettings'
Get-ChildItem -LiteralPath (Join-Path $fixtureRoot 'Assets') -Force |
    Copy-Item -Destination $projectAssets -Recurse -Force
Copy-Item -LiteralPath $packageManifest -Destination (Join-Path $projectPackages 'manifest.json') -Force
Copy-Item -LiteralPath $projectVersion -Destination (Join-Path $projectSettings 'ProjectVersion.txt') -Force

$previousOutput = [Environment]::GetEnvironmentVariable('GLYPHSHIFT_UNITY_NEGATIVE_OUTPUT')
try {
    [Environment]::SetEnvironmentVariable('GLYPHSHIFT_UNITY_NEGATIVE_OUTPUT', $outputPath)
    & $editorInput -batchmode -nographics -quit -projectPath $projectRoot `
        -executeMethod GlyphshiftNegativeBuild.Build -logFile $buildLog
    Assert-Contract ($LASTEXITCODE -eq 0) 'Unity failed to build the local negative player.'
}
finally {
    [Environment]::SetEnvironmentVariable('GLYPHSHIFT_UNITY_NEGATIVE_OUTPUT', $previousOutput)
}

$dataRoot = Join-Path $buildRoot 'GlyphshiftUnityIl2CppNegative_Data'
$metadataPath = Join-Path $dataRoot 'il2cpp_data\Metadata\global-metadata.dat'
Assert-Contract (Test-Path -LiteralPath $outputPath -PathType Leaf) 'The expected player executable is missing.'
Assert-Contract ((Get-PeMachine -Path $outputPath) -eq 0x8664) 'The built player is not x86_64.'
Assert-Contract (Test-Path -LiteralPath (Join-Path $buildRoot 'GameAssembly.dll') -PathType Leaf) 'GameAssembly.dll is missing.'
Assert-Contract (Test-Path -LiteralPath $metadataPath -PathType Leaf) 'IL2CPP metadata is missing.'
Assert-Contract (-not (Test-Path -LiteralPath (Join-Path $buildRoot 'MonoBleedingEdge'))) 'A Mono runtime was emitted unexpectedly.'
$metadataText = [System.Text.Encoding]::UTF8.GetString([System.IO.File]::ReadAllBytes($metadataPath))
Assert-Contract ($metadataText.Contains('TMP_Text')) 'IL2CPP metadata does not retain the TMP text type.'
Assert-Contract ($metadataText.Contains('TMPro.TMP_Text')) 'IL2CPP metadata does not retain the TMP type identity marker.'
Assert-Contract ($metadataText.Contains('UnityEngine.UI.Text')) 'IL2CPP metadata does not retain the uGUI type identity marker.'
Assert-Contract ($metadataText.Contains('TMP_UGUI_TYPES_PRESENT_LIVE_TEXT_ZERO')) 'IL2CPP metadata does not retain the live-object guard.'

Write-Output 'unity_negative_harness_build=passed'
Write-Output ('unity_negative_harness_evidence=local-test/evidence/unity-il2cpp-negative-harness/' + $runId)
