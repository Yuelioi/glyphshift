using System.Diagnostics;
using System.Text;
using System.Text.Json;
using System.Security.Cryptography;
using Microsoft.Diagnostics.NETCore.Client;

if (args.Length != 3) throw new ArgumentException("Expected dotnet executable, synthetic host assembly, native fixture library");
var profilerId = new Guid("903d5c72-54b7-4d31-959d-c856f26fa229");
var originalAssemblyHash = SHA256.HashData(File.ReadAllBytes(args[1]));
int assertions = 0;
void Require(bool condition, string description)
{
    if (!condition) throw new InvalidOperationException(description);
    assertions++;
    Console.WriteLine("PASS " + description);
}
Process StartHost()
{
    var start = new ProcessStartInfo(args[0]) { UseShellExecute = false, CreateNoWindow = true, RedirectStandardInput = true, RedirectStandardOutput = true, RedirectStandardError = true };
    start.ArgumentList.Add(args[1]);
    foreach (var key in start.Environment.Keys.ToArray())
        if (key.StartsWith("CORECLR_", StringComparison.OrdinalIgnoreCase) || key.StartsWith("COR_", StringComparison.OrdinalIgnoreCase)
            || key.StartsWith("COMPlus_", StringComparison.OrdinalIgnoreCase) || key.StartsWith("DOTNET_", StringComparison.OrdinalIgnoreCase)) start.Environment.Remove(key);
    return Process.Start(start) ?? throw new InvalidOperationException("Synthetic host launch failed");
}
async Task<JsonElement> Send(Process process, object command)
{
    await process.StandardInput.WriteLineAsync(JsonSerializer.Serialize(command));
    string? line = await process.StandardOutput.ReadLineAsync().WaitAsync(TimeSpan.FromSeconds(15));
    if (line == null) throw new InvalidOperationException("Host exited: " + await process.StandardError.ReadToEndAsync());
    using var document = JsonDocument.Parse(line);
    if (document.RootElement.TryGetProperty("error", out var error)) throw new InvalidOperationException(error.GetString());
    return document.RootElement.Clone();
}
async Task Render(Process process, string source, string expected, string description)
{
    var result = await Send(process, new { command = "render", text = source });
    Require(result.GetProperty("text").GetString() == expected, description);
}
using (var host = StartHost())
{
    try
    {
        Require(await host.StandardOutput.ReadLineAsync().WaitAsync(TimeSpan.FromSeconds(15)) == "READY", "already-running .NET 6 host warmed up");
        await Render(host, "Source text", "[Source text]", "original method before attach");
        Require((await Send(host, new { command = "load", path = Path.GetFullPath(args[2]) })).GetProperty("status").GetInt32() == 0, "Native ABI negotiation and activation");
        new DiagnosticsClient(host.Id).AttachProfiler(TimeSpan.FromSeconds(15), profilerId, Path.GetFullPath(args[2]), Encoding.UTF8.GetBytes("glyphshift-synthetic-only-v1\0"));
        var deadline = Stopwatch.StartNew();
        bool ready = false;
        while (deadline.Elapsed < TimeSpan.FromSeconds(15))
        {
            await Send(host, new { command = "render", text = "Source text" });
            int status = (await Send(host, new { command = "status" })).GetProperty("status").GetInt32();
            if (status < 0) throw new InvalidOperationException($"Profiler failed: 0x{status:X8}");
            if (status == 0) { ready = true; break; }
            await Task.Delay(50);
        }
        Require(ready, "late attach ReJIT completed");
        await Render(host, "Source text", "[第一代译文]", "first generation through real Native ABI callback");
        await Render(host, "Unmapped", "[Unmapped]", "dictionary miss preserves original");
        await Render(host, "", "[]", "empty input preserves original method semantics");
        await Render(host, "a\0b", "[a\0b]", "embedded-NUL input fails open without truncation");
        await Render(host, new string('x', 65537), "[" + new string('x', 65537) + "]", "oversized input fails open");
        Require((await Send(host, new { command = "render", text = (string?)null })).GetProperty("text").GetString() == "[]", "null input retains original behavior");
        foreach (int fault in new[] { -1, -2, -3 })
        {
            await Send(host, new { command = "generation", generation = fault });
            await Render(host, "Source text", "[Source text]", $"invalid decision {fault} preserves original");
        }
        await Send(host, new { command = "generation", generation = 2 });
        await Render(host, "Source text", "[第二代译文]", "same-process second generation");
        await Send(host, new { command = "gc" });
        await Render(host, "Source text", "[第二代译文]", "callback lifetime survives full GC");
        Require((await Send(host, new { command = "observe" })).GetProperty("status").GetInt32() == 0, "observe-only activation");
        await Render(host, "Source text", "[Source text]", "observe-only never substitutes text");
        await Send(host, new { command = "activate" });
        var drain = await Send(host, new { command = "drain" });
        Require(drain.GetProperty("waited").GetBoolean() && drain.GetProperty("status").GetInt32() == 0, "deactivate waits for in-flight callback");
        Require(drain.GetProperty("inFlight").GetString() == "[第二代译文]" && drain.GetProperty("restored").GetString() == "[Source text]", "in-flight call finishes safely and subsequent calls restore");
        Require((await Send(host, new { command = "deactivate" })).GetProperty("status").GetInt32() == 0, "deactivate drains callback registration");
        var stopped = await Send(host, new { command = "render", text = "Source text" });
        long stoppedCalls = stopped.GetProperty("calls").GetInt64();
        Require(stopped.GetProperty("text").GetString() == "[Source text]", "deactivate immediately restores original behavior");
        var repeated = await Send(host, new { command = "render", text = "Source text" });
        Require(repeated.GetProperty("calls").GetInt64() == stoppedCalls, "deactivate stops observation callbacks");
        Require((await Send(host, new { command = "activate" })).GetProperty("status").GetInt32() == 0, "reactivate existing resident component");
        await Render(host, "Source text", "[第二代译文]", "reactivation uses current dictionary");
        Require((await Send(host, new { command = "revert" })).GetProperty("status").GetInt32() == 0, "RequestRevert checks per-method success");
        await Render(host, "Source text", "[Source text]", "reverted method calls original implementation");
        await Send(host, new { command = "activate" });
        await Render(host, "Source text", "[Source text]", "RequestRevert removes interception even with callback reactivated");
        host.Refresh();
        Require(host.Modules.Cast<ProcessModule>().Any(module => module.ModuleName == Path.GetFileName(args[2])), "resident component is explicitly distinguished from unload");
        await Send(host, new { command = "exit" });
        await host.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(15));
        Require(host.ExitCode == 0, "instrumented host exits normally");
        Require(SHA256.HashData(File.ReadAllBytes(args[1])).SequenceEqual(originalAssemblyHash), "target assembly file remains byte-for-byte unchanged");
    }
    finally { if (!host.HasExited) { host.Kill(true); await host.WaitForExitAsync(); } }
}
using (var clean = StartHost())
{
    try
    {
        Require(await clean.StandardOutput.ReadLineAsync().WaitAsync(TimeSpan.FromSeconds(15)) == "READY", "fresh host starts without extension activation");
        await Render(clean, "Source text", "[Source text]", "fresh process retains original behavior");
        clean.Refresh();
        Require(!clean.Modules.Cast<ProcessModule>().Any(module => module.ModuleName == Path.GetFileName(args[2])), "native extension absent from fresh process");
        await Send(clean, new { command = "exit" });
        await clean.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(15));
        Require(clean.ExitCode == 0, "fresh host exits normally");
    }
    finally { if (!clean.HasExited) { clean.Kill(true); await clean.WaitForExitAsync(); } }
}
Console.WriteLine($"Passed {assertions} assertions. Synthetic coverage only; no game was attached.");
