using System.Text;
using Microsoft.Diagnostics.NETCore.Client;

if (args.Length != 2 && args.Length != 3) throw new ArgumentException("Expected synthetic host process id, native fixture library, and optional target assembly filename");
var module = args.Length == 3 ? args[2] : "Glyphshift.MonoGame.SyntheticHost.dll";
if (module.Contains('\\') || module.Contains('/') || module.Contains('\0') || module.Length > 240) throw new ArgumentException("Invalid target assembly filename");
new DiagnosticsClient(int.Parse(args[0])).AttachProfiler(TimeSpan.FromSeconds(15),
    new Guid("903d5c72-54b7-4d31-959d-c856f26fa229"), Path.GetFullPath(args[1]), Encoding.UTF8.GetBytes($"glyphshift-monogame-synthetic-v1|module={module}\0"));
Console.WriteLine("Attached synthetic MonoGame profile");
