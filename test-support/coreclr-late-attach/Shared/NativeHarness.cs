using System.Runtime.InteropServices;

unsafe sealed class NativeHarness
{
    [StructLayout(LayoutKind.Sequential)] struct FixedUtf8 { public ushort Length; public fixed byte Bytes[96]; }
    [StructLayout(LayoutKind.Sequential)] struct Descriptor
    {
        public uint Size; public FixedUtf8 Id;
        public ushort Major, Minor, Patch, AbiMajor, AbiMinor;
        public uint ApplyModel, Placement; public ulong Features; public uint Platforms, Architectures;
    }
    [StructLayout(LayoutKind.Sequential)] struct Api
    {
        public uint Size; public Descriptor Descriptor;
        public nint Negotiate, Activate, Deactivate, Refresh;
    }
    [StructLayout(LayoutKind.Sequential)] struct Host { public uint Size; public nint Context, Decide, Characters; }
    [StructLayout(LayoutKind.Sequential)] struct Negotiation { public int Status; public ulong Active; }
    [StructLayout(LayoutKind.Sequential)] struct Decision { public int Status; public ulong Generation; public uint Bits, TextLength, FontLength; }
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] delegate Api Entry();
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] delegate Negotiation ActivateFn(ref Host host, ulong requested, ulong granted);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] delegate Negotiation NegotiateFn(ulong requested, ulong granted);
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] delegate int CommandFn();
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)] delegate Decision DecideFn(nint context, char* source, uint length, char* output, uint capacity, char* font, uint fontCapacity);
    readonly nint library;
    readonly Api api;
    readonly DecideFn decide;
    readonly CommandFn status, revert;
    readonly ManualResetEventSlim entered = new(false), release = new(false);
    bool block;
    int generation = 1;
    readonly string sourceText, firstText, secondText;
    readonly Func<string, string>? renderer;
    public long Calls;
    public NativeHarness(string path, string sourceText = "Source text", string firstText = "第一代译文", string secondText = "第二代译文", Func<string, string>? renderer = null)
    {
        this.sourceText = sourceText; this.firstText = firstText; this.secondText = secondText; this.renderer = renderer;
        library = NativeLibrary.Load(path);
        api = Marshal.GetDelegateForFunctionPointer<Entry>(NativeLibrary.GetExport(library, "glyphshift_adapter_entry_v1"))();
        if (sizeof(Descriptor) != 136 || sizeof(Api) != (IntPtr.Size == 8 ? 176 : 160) || api.Size != sizeof(Api) || api.Descriptor.Size != sizeof(Descriptor)
            || sizeof(Decision) != 32 || sizeof(Host) != (IntPtr.Size == 8 ? 32 : 16)) throw new InvalidOperationException("Native ABI layout mismatch");
        var negotiate = Marshal.GetDelegateForFunctionPointer<NegotiateFn>(api.Negotiate);
        if (negotiate(3, 1).Status != 2 || negotiate(4, 7).Status != 1) throw new InvalidOperationException("Authorization negotiation failed");
        decide = DecideText;
        status = Export("glyphshift_fixture_status");
        revert = Export("glyphshift_fixture_revert");
    }
    CommandFn Export(string name) => Marshal.GetDelegateForFunctionPointer<CommandFn>(NativeLibrary.GetExport(library, name));
    public int Activate(ulong features = 3)
    {
        var host = new Host { Size = (uint)sizeof(Host), Context = 1, Decide = Marshal.GetFunctionPointerForDelegate(decide) };
        return Marshal.GetDelegateForFunctionPointer<ActivateFn>(api.Activate)(ref host, features, features).Status;
    }
    public int Deactivate() => Marshal.GetDelegateForFunctionPointer<CommandFn>(api.Deactivate)();
    public int Status() => status();
    public int Revert() => revert();
    public object SetGeneration(int value) { Volatile.Write(ref generation, value); return new { generation = value }; }
    public object CheckDrain()
    {
        entered.Reset(); release.Reset(); Volatile.Write(ref block, true);
        var rendering = Task.Run(() => renderer!(sourceText));
        Task<int>? stopping = null;
        try
        {
            if (!entered.Wait(TimeSpan.FromSeconds(5))) throw new InvalidOperationException("Callback did not enter");
            using var stopEntered = new ManualResetEventSlim(false);
            stopping = Task.Run(() => { stopEntered.Set(); return Deactivate(); });
            if (!stopEntered.Wait(TimeSpan.FromSeconds(5))) throw new InvalidOperationException("Stop task did not start");
            bool waited = !stopping.Wait(TimeSpan.FromMilliseconds(100));
            release.Set();
            if (!Task.WaitAll(new Task[] { rendering, stopping }, TimeSpan.FromSeconds(5))) throw new InvalidOperationException("Drain did not finish");
            return new { waited, status = stopping.Result, inFlight = rendering.Result, restored = renderer!(sourceText) };
        }
        finally
        {
            Volatile.Write(ref block, false); release.Set();
            rendering.Wait(TimeSpan.FromSeconds(5)); stopping?.Wait(TimeSpan.FromSeconds(5));
        }
    }
    Decision DecideText(nint context, char* source, uint length, char* output, uint capacity, char* font, uint fontCapacity)
    {
        try
        {
            Interlocked.Increment(ref Calls);
            if (Volatile.Read(ref block)) { entered.Set(); if (!release.Wait(TimeSpan.FromSeconds(5))) return new Decision { Status = 5 }; }
            string original = new(source, 0, checked((int)length));
            int current = Volatile.Read(ref generation);
            if (current == -1) return new Decision { Status = 5 };
            if (current == -2) return new Decision { Bits = 1, TextLength = capacity + 1 };
            if (current == -3) { output[0] = 'x'; output[1] = '\0'; return new Decision { Bits = 1, TextLength = 2 }; }
            string? translation = original == sourceText ? (current == 1 ? firstText : secondText) : null;
            if (translation == null) return new Decision { Generation = (ulong)current };
            if (translation.Length > capacity) return new Decision { Status = 5 };
            translation.AsSpan().CopyTo(new Span<char>(output, (int)capacity));
            return new Decision { Generation = (ulong)current, Bits = 1, TextLength = (uint)translation.Length };
        }
        catch { return new Decision { Status = 5 }; }
    }
    // CLR and injected IL can still call this DLL. Never NativeLibrary.Free here.
}
