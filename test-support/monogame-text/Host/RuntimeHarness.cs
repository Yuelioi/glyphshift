using System.Runtime.InteropServices;
using System.Text;

// Calls the shipping Runtime's public ABI. Translation and observation are owned
// by its Kernel/Capture, not a replacement callback in the synthetic window.
unsafe sealed class RuntimeHarness
{
    [StructLayout(LayoutKind.Sequential)] struct Command { public uint Size; public byte* Json; public uint Length; }
    [StructLayout(LayoutKind.Sequential)] struct Query { public uint Size; public byte* Output; public uint Capacity; public uint Length; }
    [UnmanagedFunctionPointer(CallingConvention.Winapi)] delegate uint CommandFn(Command* command);
    [UnmanagedFunctionPointer(CallingConvention.Winapi)] delegate uint QueryFn(Query* query);
    [UnmanagedFunctionPointer(CallingConvention.Winapi)] delegate uint StopFn(nint ignored);
    readonly nint library;
    readonly string deployments;
    int generation = 1;
    public RuntimeHarness(string libraryPath, string deployments)
    {
        library = NativeLibrary.Load(libraryPath); this.deployments = deployments;
    }
    T Export<T>(string name) where T : Delegate => Marshal.GetDelegateForFunctionPointer<T>(NativeLibrary.GetExport(library, name));
    int Call(string name, string json)
    {
        byte[] data = Encoding.UTF8.GetBytes(json);
        fixed (byte* pointer = data)
        {
            var command = new Command { Size = (uint)sizeof(Command), Json = pointer, Length = (uint)data.Length };
            return (int)Export<CommandFn>(name)(&command);
        }
    }
    public int Activate() => Call("glyphshift_runtime_activate_v1", File.ReadAllText(Path.Combine(deployments, $"deployment-{generation}.json")));
    public int Deactivate() => (int)Export<StopFn>("glyphshift_runtime_deactivate_v1")(0);
    public object SetGeneration(int value)
    {
        int status = Call("glyphshift_runtime_update_v1", File.ReadAllText(Path.Combine(deployments, $"publication-{value}.json")));
        if (status != 0) throw new InvalidOperationException("Runtime update failed: " + status);
        generation = value; return new { generation };
    }
    public string Read(string name)
    {
        var output = new byte[256 * 1024];
        fixed (byte* pointer = output)
        {
            var query = new Query { Size = (uint)sizeof(Query), Output = pointer, Capacity = (uint)output.Length };
            uint status = Export<QueryFn>(name)(&query);
            if (status != 0) throw new InvalidOperationException("Runtime query failed: " + status);
            return Encoding.UTF8.GetString(output, 0, checked((int)query.Length));
        }
    }
}
