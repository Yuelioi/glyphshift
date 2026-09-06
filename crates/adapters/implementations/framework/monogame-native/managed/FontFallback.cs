using System.Collections;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Glyphshift.MonoGame;

// Embedded in the verified native DLL. No compile-time MonoGame reference:
// all font/device types come from the target's already-loaded framework.
public static class FontFallback
{
    const int MaxGlyphs = 512, MaxPixels = 4 * 1024 * 1024, MaxBytes = 32 * 1024 * 1024;
    static readonly ConditionalWeakTable<object, Cache> Fonts = new();
    static readonly object Gate = new();
    static readonly List<Retired> Retirements = new();
    static readonly List<Pending> PendingFonts = new();
    static int allocatedBytes, fontCount;
    sealed class Cache
    {
        public int Thread;
        public object? Current, Device;
        public int Bytes;
        public readonly SortedDictionary<char, Raster> Added = new();
        public readonly HashSet<char> Unsupported = new();
        public readonly HashSet<char> Requested = new();
        public bool Queued;
    }
    sealed record Retired(object Texture, object Device, int Thread, int Bytes);
    sealed record Pending(object Font, object Device, Cache Cache);
    sealed record Raster(int Width, int Height, int Left, int Top, int Advance, byte[] Alpha);

    public static object? Resolve(object? font, string? text, bool drawing)
    {
        try
        {
            if (font is object[] layout && layout.Length == 3 && text != null)
                return Reflow(layout[0], layout[1], (string)layout[2], text);
            if (text == null) { if (font != null) PublishPending(font); return null; }
            if (font == null || text.Length > 65536) return font;
            var fontType = font.GetType();
            if (fontType.FullName != "Microsoft.Xna.Framework.Graphics.SpriteFont") return font;
            var known = ((IEnumerable)Get(font, "Characters")).Cast<char>().ToHashSet();
            var needed = text.Where(c => c != '\r' && c != '\n' && !known.Contains(c)).ToHashSet();
            if (needed.Count == 0) return font;
            if (needed.Count > MaxGlyphs || needed.Any(char.IsSurrogate)) return font;
            if (!Fonts.TryGetValue(font, out var cache))
            {
                if (!drawing) return font;
                if (Interlocked.Increment(ref fontCount) > 32) { Interlocked.Decrement(ref fontCount); return font; }
                cache = Fonts.GetValue(font, _ => new Cache());
            }
            lock (cache)
            {
                if (cache.Current != null && needed.All(cache.Added.ContainsKey)) return cache.Current;
                if (cache.Unsupported.Overlaps(needed) || !drawing) return font;
                int thread = Environment.CurrentManagedThreadId;
                if (cache.Thread != 0 && cache.Thread != thread) return font;
                cache.Thread = thread;
                var texture = Get(font, "Texture");
                var device = Get(texture, "GraphicsDevice");
                cache.Device = device;
                if ((bool)Get(texture, "IsDisposed") || (bool)Get(device, "IsDisposed")) return font;
                int lineHeight = (int)Get(font, "LineSpacing");
                if (lineHeight < 4 || lineHeight > 128 || cache.Added.Count + cache.Requested.Union(needed).Count(c => !cache.Added.ContainsKey(c)) > MaxGlyphs) return font;
                cache.Requested.UnionWith(needed);
                if (!cache.Queued) { cache.Queued = true; lock (Gate) PendingFonts.Add(new Pending(font, device, cache)); }
                // Publish at Present: measure/draw in this frame both keep the
                // original; the next frame sees the same completed fallback.
                return font;
            }
        }
        catch { return font is object[] ? text : font; }
    }

    static string Reflow(object originalFont, object renderedFont, string source, string translation)
    {
        // Explicit translated paragraphs are authoritative. Only reflow a flat
        // translation when the source carries the producer's soft-wrap shape.
        if (translation.Contains('\n') || translation.Contains('\r')) return translation;
        bool soft = false;
        for (int i = 2; i < source.Length; i++) {
            int after = source[i] == '\r' && i + 1 < source.Length && source[i + 1] == '\n' ? i + 2 : source[i] == '\n' ? i + 1 : -1;
            char before = source[i - 2];
            if (after > 0 && after < source.Length && source[i - 1] == ' '
                && ((before < 128 && char.IsLetterOrDigit(before)) || ",.;!?)]'\"".Contains(before))
                && source[after] < 128 && char.IsLetterOrDigit(source[after])) { soft = true; break; }
        }
        if (!soft) return translation;
        var originalGlyphs = ((IEnumerable)Get(originalFont, "Glyphs")).Cast<object>().ToDictionary(g => (char)Field(g, "Character"));
        var renderedGlyphs = ((IEnumerable)Get(renderedFont, "Glyphs")).Cast<object>().ToDictionary(g => (char)Field(g, "Character"));
        static float Width(string text, Dictionary<char, object> glyphs, float spacing) {
            float x = 0; bool first = true;
            foreach (char c in text) {
                if (!glyphs.TryGetValue(c, out var glyph)) return float.PositiveInfinity;
                float left = (float)Field(glyph, "LeftSideBearing");
                x += first ? Math.Max(0, left) : spacing + left;
                x += (float)Field(glyph, "Width") + (float)Field(glyph, "RightSideBearing"); first = false;
            }
            return x;
        }
        float available = source.Replace("\r\n", "\n").Split('\n').Max(line => Width(line, originalGlyphs, (float)Get(originalFont, "Spacing")));
        if (!float.IsFinite(available) || available <= 0) return translation;
        float gap = (float)Get(renderedFont, "Spacing");
        if (Width(translation, renderedGlyphs, gap) <= available) return translation;
        var lines = new List<string>(); string current = "";
        foreach (char character in translation) {
            string next = current + character;
            if (current.Length > 0 && Width(next, renderedGlyphs, gap) > available) {
                int space = current.LastIndexOf(' ');
                if (space > 0) { lines.Add(current[..space]); current = current[(space + 1)..] + character; }
                else { lines.Add(current); current = character.ToString(); }
            } else current = next;
        }
        if (current.Length > 0) lines.Add(current);
        return string.Join("\n", lines);
    }

    static void PublishPending(object device)
    {
        var pending = new List<Pending>();
        lock (Gate)
            for (int i = PendingFonts.Count - 1; i >= 0; i--)
                if (ReferenceEquals(PendingFonts[i].Device, device) && PendingFonts[i].Cache.Thread == Environment.CurrentManagedThreadId)
                { pending.Add(PendingFonts[i]); PendingFonts.RemoveAt(i); }
        foreach (var item in pending)
        {
            lock (item.Cache)
            {
                var cache = item.Cache;
                try
                {
                    var texture = Get(item.Font, "Texture");
                    if ((bool)Get(texture, "IsDisposed") || (bool)Get(device, "IsDisposed")) continue;
                    var added = new SortedDictionary<char, Raster>(cache.Added);
                    bool valid = true;
                    foreach (char character in cache.Requested)
                    {
                        if (added.ContainsKey(character)) continue;
                        var glyph = Rasterize(character, (int)Get(item.Font, "LineSpacing"));
                        if (glyph == null) { cache.Unsupported.Add(character); valid = false; continue; }
                        added.Add(character, glyph);
                    }
                    if (!valid) continue;
                    var (font, bytes) = BuildFont(item.Font, texture, device, added);
                    if (font == null) continue;
                    if (cache.Current != null)
                        lock (Gate) Retirements.Add(new Retired(Get(cache.Current, "Texture"), device, cache.Thread, cache.Bytes));
                    cache.Current = font; cache.Bytes = bytes;
                    cache.Added.Clear(); foreach (var entry in added) cache.Added.Add(entry.Key, entry.Value);
                }
                catch { /* A failed atlas never changes the original font. */ }
                finally { cache.Requested.Clear(); cache.Queued = false; }
            }
        }
        ReleaseRetired(device);
    }

    static object Get(object target, string name) => target.GetType().GetProperty(name)!.GetValue(target)!;
    static object Field(object target, string name) => target.GetType().GetField(name)!.GetValue(target)!;
    static IList ListOf(Type type) => (IList)Activator.CreateInstance(typeof(List<>).MakeGenericType(type))!;
    static void TextureData(object texture, string method, Type color, Array pixels) => texture.GetType().GetMethods()
        .Single(m => m.Name == method && m.IsGenericMethodDefinition && m.GetParameters().Length == 1)
        .MakeGenericMethod(color).Invoke(texture, new object[] { pixels });

    static (object? Font, int Bytes) BuildFont(object source, object sourceTexture, object device, SortedDictionary<char, Raster> added)
    {
        var assembly = source.GetType().Assembly;
        var rectangle = assembly.GetType("Microsoft.Xna.Framework.Rectangle", true)!;
        var vector = assembly.GetType("Microsoft.Xna.Framework.Vector3", true)!;
        var color = assembly.GetType("Microsoft.Xna.Framework.Color", true)!;
        var textureType = sourceTexture.GetType();
        int sourceWidth = (int)Get(sourceTexture, "Width"), sourceHeight = (int)Get(sourceTexture, "Height");
        if (sourceWidth < 1 || sourceHeight < 1 || (long)sourceWidth * sourceHeight > MaxPixels) return (null, 0);
        int width = Math.Max(sourceWidth, 256), x = 0, y = sourceHeight + 1, row = 0;
        var placements = new Dictionary<char, (int X, int Y)>();
        foreach (var (character, raster) in added)
        {
            if (x + raster.Width + 1 > width) { x = 0; y += row + 1; row = 0; }
            placements.Add(character, (x, y)); x += Math.Max(1, raster.Width) + 1; row = Math.Max(row, raster.Height);
        }
        int height = y + row + 1;
        if ((long)width * height > MaxPixels) return (null, 0);
        int bytes = checked(width * height * 4);
        lock (Gate) { if (allocatedBytes + bytes > MaxBytes) return (null, 0); allocatedBytes += bytes; }
        object? atlas = null;
        bool committed = false;
        try
        {
            var sourcePixels = TexturePixels.Read(sourceTexture, color);
            var pixels = Array.CreateInstance(color, width * height);
            for (int i = 0; i < sourceHeight; i++) Array.Copy(sourcePixels, i * sourceWidth, pixels, i * width, sourceWidth);
            var colorCtor = color.GetConstructor(new[] { typeof(byte), typeof(byte), typeof(byte), typeof(byte) })!;
            var colors = Enumerable.Range(0, 256).Select(i => colorCtor.Invoke(new object[] { (byte)i, (byte)i, (byte)i, (byte)i })).ToArray();
            foreach (var (character, raster) in added)
            {
                var position = placements[character];
                for (int r = 0; r < raster.Height; r++) for (int c = 0; c < raster.Width; c++)
                    pixels.SetValue(colors[raster.Alpha[r * raster.Width + c]], (position.Y + r) * width + position.X + c);
            }
            atlas = Activator.CreateInstance(textureType, device, width, height)!;
            TextureData(atlas, "SetData", color, pixels);
            var bounds = ListOf(rectangle); var cropping = ListOf(rectangle); var kernings = ListOf(vector); var characters = new List<char>();
            var oldGlyphs = ((IEnumerable)Get(source, "Glyphs")).Cast<object>().ToDictionary(g => (char)Field(g, "Character"));
            foreach (char character in oldGlyphs.Keys.Concat(added.Keys).Distinct().OrderBy(c => c))
            {
                characters.Add(character);
                if (oldGlyphs.TryGetValue(character, out var glyph))
                {
                    bounds.Add(Field(glyph, "BoundsInTexture")); cropping.Add(Field(glyph, "Cropping"));
                    kernings.Add(Activator.CreateInstance(vector, Field(glyph, "LeftSideBearing"), Field(glyph, "Width"), Field(glyph, "RightSideBearing")));
                }
                else
                {
                    var glyphRaster = added[character]; var at = placements[character];
                    bounds.Add(Activator.CreateInstance(rectangle, at.X, at.Y, Math.Max(1, glyphRaster.Width), Math.Max(1, glyphRaster.Height)));
                    cropping.Add(Activator.CreateInstance(rectangle, 0, glyphRaster.Top, Math.Max(1, glyphRaster.Width), Math.Max(1, glyphRaster.Height)));
                    kernings.Add(Activator.CreateInstance(vector, (float)glyphRaster.Left, (float)glyphRaster.Width,
                        (float)(glyphRaster.Advance - glyphRaster.Left - glyphRaster.Width)));
                }
            }
            var result = Activator.CreateInstance(source.GetType(), atlas, bounds, cropping, characters,
                Get(source, "LineSpacing"), Get(source, "Spacing"), kernings, source.GetType().GetProperty("DefaultCharacter")!.GetValue(source));
            committed = true;
            return (result, bytes);
        }
        finally
        {
            if (!committed) { (atlas as IDisposable)?.Dispose(); lock (Gate) allocatedBytes -= bytes; }
        }
    }

    // Present is a frame boundary. Never dispose textures still queued in a
    // SpriteBatch during a draw callback; keep cleanup on the owning device thread.
    static void ReleaseRetired(object device)
    {
        var dispose = new List<Retired>();
        lock (Gate)
            for (int i = Retirements.Count - 1; i >= 0; i--)
                if (ReferenceEquals(Retirements[i].Device, device) && Retirements[i].Thread == Environment.CurrentManagedThreadId)
                { dispose.Add(Retirements[i]); Retirements.RemoveAt(i); }
        foreach (var item in dispose) { try { ((IDisposable)item.Texture).Dispose(); } finally { lock (Gate) allocatedBytes -= item.Bytes; } }
    }

    static Raster? Rasterize(char character, int height)
    {
        foreach (string family in new[] { "Microsoft YaHei", "Yu Gothic", "Malgun Gothic", "Segoe UI" })
        {
            nint dc = CreateCompatibleDC(0), font = 0, previous = 0;
            if (dc == 0) return null;
            try
            {
                font = CreateFontW(-height, 0, 0, 0, 400, 0, 0, 0, 1, 4, 0, 4, 0, family);
                if (font == 0) continue;
                previous = SelectObject(dc, font);
                var glyphIndex = new ushort[1];
                if (GetGlyphIndicesW(dc, character.ToString(), 1, glyphIndex, 1) == uint.MaxValue || glyphIndex[0] == ushort.MaxValue) continue;
                if (!GetTextMetricsW(dc, out var metrics)) continue;
                var matrix = new Matrix { A = new Fixed { Value = 1 }, D = new Fixed { Value = 1 } };
                uint size = GetGlyphOutlineW(dc, character, 6, out var glyph, 0, null, ref matrix);
                if (size == uint.MaxValue || size > 256 * 256 || glyph.Width > 256 || glyph.Height > 256) continue;
                byte[] raw = new byte[size];
                if (size > 0 && GetGlyphOutlineW(dc, character, 6, out glyph, size, raw, ref matrix) == uint.MaxValue) continue;
                int w = checked((int)glyph.Width), h = checked((int)glyph.Height), stride = (w + 3) & ~3;
                var alpha = new byte[w * h];
                if ((long)stride * h > raw.Length) continue;
                for (int y = 0; y < h; y++) for (int x = 0; x < w; x++) alpha[y * w + x] = (byte)Math.Min(255, (raw[y * stride + x] * 255 + 32) / 64);
                return new Raster(w, h, glyph.X, Math.Max(0, metrics.Ascent - glyph.Y), glyph.AdvanceX, alpha);
            }
            finally { if (previous != 0) SelectObject(dc, previous); if (font != 0) DeleteObject(font); DeleteDC(dc); }
        }
        return null;
    }
    [StructLayout(LayoutKind.Sequential)] struct Fixed { public ushort Fraction; public short Value; }
    [StructLayout(LayoutKind.Sequential)] struct Matrix { public Fixed A, B, C, D; }
    [StructLayout(LayoutKind.Sequential)] struct Glyph { public uint Width, Height; public int X, Y; public short AdvanceX, AdvanceY; }
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)] struct Metrics
    {
        public int Height, Ascent, Descent, Internal, External, AverageWidth, MaxWidth, Weight, Overhang, AspectX, AspectY;
        public char First, Last, Default, Break; public byte Italic, Underlined, StrikeOut, Pitch, CharacterSet;
    }
    [DllImport("gdi32.dll")] static extern nint CreateCompatibleDC(nint dc);
    [DllImport("gdi32.dll")] static extern bool DeleteDC(nint dc);
    [DllImport("gdi32.dll")] static extern nint SelectObject(nint dc, nint value);
    [DllImport("gdi32.dll")] static extern bool DeleteObject(nint value);
    [DllImport("gdi32.dll", CharSet = CharSet.Unicode)] static extern nint CreateFontW(int h, int w, int e, int o, int weight, uint italic, uint underline, uint strike, uint charset, uint output, uint clip, uint quality, uint pitch, string face);
    [DllImport("gdi32.dll", CharSet = CharSet.Unicode)] static extern uint GetGlyphIndicesW(nint dc, string text, int count, [Out] ushort[] indices, uint flags);
    [DllImport("gdi32.dll", CharSet = CharSet.Unicode)] static extern bool GetTextMetricsW(nint dc, out Metrics metrics);
    [DllImport("gdi32.dll")] static extern uint GetGlyphOutlineW(nint dc, uint character, uint format, out Glyph glyph, uint size, [Out] byte[]? bytes, ref Matrix matrix);
}
