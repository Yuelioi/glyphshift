using System;
using TMPro;
using UnityEngine;
using UguiText = UnityEngine.UI.Text;

[DefaultExecutionOrder(-1000)]
[DisallowMultipleComponent]
public sealed class GlyphshiftStandardUiAbsenceGuard : MonoBehaviour
{
    public const string ContractMarker = "TMP_UGUI_TYPES_PRESENT_LIVE_TEXT_ZERO";

    private const int RecheckFrameInterval = 30;
    private const string TmpTextTypeName = "TMPro.TMP_Text";
    private const string UguiTextTypeName = "UnityEngine.UI.Text";

    private void Awake()
    {
        AssertExpectedTypeIdentities();
        AssertNoStandardTextObjects();
    }

    private void LateUpdate()
    {
        if (Time.frameCount % RecheckFrameInterval == 0)
        {
            AssertNoStandardTextObjects();
        }
    }

    private void AssertNoStandardTextObjects()
    {
        TMP_Text[] tmpObjects = Resources.FindObjectsOfTypeAll<TMP_Text>();
        UguiText[] uguiObjects = Resources.FindObjectsOfTypeAll<UguiText>();
        if (tmpObjects.Length == 0 && uguiObjects.Length == 0)
        {
            return;
        }

        FailContract(
            "expected zero live text objects but found TMP=" + tmpObjects.Length +
            " and uGUI=" + uguiObjects.Length + ".");
    }

    private void AssertExpectedTypeIdentities()
    {
        if (typeof(TMP_Text).FullName == TmpTextTypeName && typeof(UguiText).FullName == UguiTextTypeName)
        {
            return;
        }

        FailContract("the expected TMP/uGUI type identities are unavailable.");
    }

    private void FailContract(string detail)
    {
        enabled = false;
        string message = ContractMarker + ": " + detail;
        Debug.LogError(message);
        Application.Quit(86);
        throw new InvalidOperationException(message);
    }
}
