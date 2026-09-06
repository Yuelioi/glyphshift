using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text.Json;

// This host knows the Native ABI, as a Runtime host would. It has no profiler or
// managed helper references, and Render is already exercised before activation.
for (int i = 0; i < 100000; i++) SyntheticRenderer.Render("Source text");
Console.WriteLine("READY");
NativeHarness? harness = null;
string? line;
while ((line = Console.ReadLine()) != null)
{
    try
    {
        using var document = JsonDocument.Parse(line);
        var request = document.RootElement;
        string command = request.GetProperty("command").GetString()!;
        object response = command switch
        {
            "load" => Load(request.GetProperty("path").GetString()!),
            "render" => new { text = SyntheticRenderer.Render(request.GetProperty("text").GetString()!), calls = harness?.Calls ?? 0 },
            "generation" => harness!.SetGeneration(request.GetProperty("generation").GetInt32()),
            "status" => new { status = harness!.Status() },
            "deactivate" => new { status = harness!.Deactivate() },
            "activate" => new { status = harness!.Activate() },
            "observe" => new { status = harness!.Activate(1) },
            "drain" => harness!.CheckDrain(),
            "revert" => new { status = harness!.Revert() },
            "gc" => Collect(),
            "exit" => new { exiting = true },
            _ => throw new InvalidOperationException("Unknown command")
        };
        Console.WriteLine(JsonSerializer.Serialize(response));
        if (command == "exit") break;
    }
    catch (Exception ex) { Console.WriteLine(JsonSerializer.Serialize(new { error = ex.ToString() })); }
}
object Load(string path) { harness = new NativeHarness(path, renderer: SyntheticRenderer.Render); return new { status = harness.Activate() }; }
object Collect() { GC.Collect(); GC.WaitForPendingFinalizers(); GC.Collect(); return new { collected = true }; }

public static class SyntheticRenderer
{
    [MethodImpl(MethodImplOptions.NoInlining)]
    public static string Render(string text) => "[" + text + "]";
}
