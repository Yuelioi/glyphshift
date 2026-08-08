# Unity IL2CPP custom-text negative harness

This deterministic fixture renders two readable lines with a runtime-generated 5x7 bitmap atlas
and a `MeshRenderer`. It keeps the Unity UI and TextMesh Pro types in the IL2CPP player, but creates
no standard text component. A runtime guard fails the player if any TMP or uGUI Text object exists.

The tracked directory is source-only. The build wrapper creates a disposable Unity project and
all build evidence below the repository-local test area.

```powershell
$env:GLYPHSHIFT_UNITY_EDITOR = '<authorized-unity-editor>'
pwsh -File scripts/build-unity-il2cpp-negative-harness.ps1
```

Use `-ValidateOnly` when the matching editor and Windows IL2CPP module are unavailable.
