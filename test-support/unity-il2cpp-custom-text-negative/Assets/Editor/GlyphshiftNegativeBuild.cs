using System;
using System.IO;
using UnityEditor;
using UnityEditor.Build.Reporting;
using UnityEditor.SceneManagement;
using UnityEngine;

public static class GlyphshiftNegativeBuild
{
    private const string OutputEnvironmentVariable = "GLYPHSHIFT_UNITY_NEGATIVE_OUTPUT";
    private const string GeneratedScenePath = "Assets/__GlyphshiftNegative.generated.unity";
    private const string ShaderPath = "Assets/GlyphshiftBitmapText.shader";

    public static void Build()
    {
        if (Application.platform != RuntimePlatform.WindowsEditor)
        {
            throw new InvalidOperationException("The negative harness must be built by a Windows editor.");
        }

        string outputPath = Environment.GetEnvironmentVariable(OutputEnvironmentVariable);
        if (string.IsNullOrWhiteSpace(outputPath) || !Path.IsPathRooted(outputPath))
        {
            throw new InvalidOperationException(OutputEnvironmentVariable + " must be an absolute executable path.");
        }

        if (!outputPath.EndsWith(".exe", StringComparison.OrdinalIgnoreCase))
        {
            throw new InvalidOperationException("The negative harness output must be an executable.");
        }

        Directory.CreateDirectory(Path.GetDirectoryName(outputPath));
        ConfigureShippingPlayer();

        try
        {
            CreateDeterministicScene();
            BuildPlayerOptions options = new BuildPlayerOptions
            {
                scenes = new[] { GeneratedScenePath },
                locationPathName = outputPath,
                target = BuildTarget.StandaloneWindows64,
                targetGroup = BuildTargetGroup.Standalone,
                options = BuildOptions.None,
            };
            BuildReport report = BuildPipeline.BuildPlayer(options);
            if (report.summary.result != BuildResult.Succeeded)
            {
                throw new InvalidOperationException("Unity failed to build the deterministic negative harness.");
            }
        }
        finally
        {
            EditorSceneManager.NewScene(NewSceneSetup.EmptyScene, NewSceneMode.Single);
            AssetDatabase.DeleteAsset(GeneratedScenePath);
        }
    }

    private static void ConfigureShippingPlayer()
    {
        PlayerSettings.companyName = "Glyphshift Test Support";
        PlayerSettings.productName = "Glyphshift Unity IL2CPP Custom Mesh Negative";
        PlayerSettings.runInBackground = false;
        PlayerSettings.resizableWindow = false;
        PlayerSettings.fullScreenMode = FullScreenMode.Windowed;
        PlayerSettings.defaultScreenWidth = 1280;
        PlayerSettings.defaultScreenHeight = 720;
        PlayerSettings.stripEngineCode = true;
        PlayerSettings.SetScriptingBackend(
            BuildTargetGroup.Standalone,
            ScriptingImplementation.IL2CPP);
        PlayerSettings.SetIl2CppCompilerConfiguration(
            BuildTargetGroup.Standalone,
            Il2CppCompilerConfiguration.Release);
        EditorUserBuildSettings.development = false;
        EditorUserBuildSettings.allowDebugging = false;
        EditorUserBuildSettings.connectProfiler = false;
    }

    private static void CreateDeterministicScene()
    {
        EditorSceneManager.NewScene(NewSceneSetup.EmptyScene, NewSceneMode.Single);

        GameObject cameraObject = new GameObject("Main Camera");
        cameraObject.tag = "MainCamera";
        Camera camera = cameraObject.AddComponent<Camera>();
        camera.clearFlags = CameraClearFlags.SolidColor;
        camera.backgroundColor = new Color(0.025f, 0.035f, 0.055f, 1.0f);
        camera.orthographic = true;
        camera.orthographicSize = 3.5f;
        cameraObject.transform.position = new Vector3(0, 0, -10);

        Shader bitmapShader = AssetDatabase.LoadAssetAtPath<Shader>(ShaderPath);
        if (bitmapShader == null)
        {
            throw new InvalidOperationException("The deterministic bitmap shader could not be loaded.");
        }

        GameObject markerObject = new GameObject("Deterministic Custom Mesh Text");
        markerObject.AddComponent<GlyphshiftStandardUiAbsenceGuard>();
        GlyphshiftCustomMeshText marker = markerObject.AddComponent<GlyphshiftCustomMeshText>();
        marker.Configure(bitmapShader);

        if (!EditorSceneManager.SaveScene(
                EditorSceneManager.GetActiveScene(),
                GeneratedScenePath,
                true))
        {
            throw new InvalidOperationException("The deterministic negative scene could not be saved.");
        }

        AssetDatabase.SaveAssets();
    }
}
