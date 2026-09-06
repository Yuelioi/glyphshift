using System.Runtime.InteropServices;

namespace Glyphshift.MonoGame;

// Texture2D.GetData<T> returns format storage, not automatically decoded RGBA.
// BC1/BC2/BC3 font atlases therefore need a bounded CPU decode before cloning.
public static class TexturePixels
{
    public static byte[] DecodeDxt(byte[] data, int width, int height, string format)
    {
        if (width < 1 || height < 1 || (long)width * height > 4 * 1024 * 1024)
            throw new ArgumentOutOfRangeException(nameof(width));
        int blockBytes = format switch { "Dxt1" => 8, "Dxt3" or "Dxt5" => 16, _ => throw new NotSupportedException("Unsupported texture format") };
        int across = (width + 3) / 4, down = (height + 3) / 4;
        if (data.Length != checked(across * down * blockBytes)) throw new ArgumentException("Compressed texture size mismatch");
        var rgba = new byte[checked(width * height * 4)];
        Span<byte> colors = stackalloc byte[16];
        Span<byte> alphas = stackalloc byte[8];
        for (int by = 0; by < down; by++) for (int bx = 0; bx < across; bx++)
        {
            int start = (by * across + bx) * blockBytes;
            int colorStart = start + (format == "Dxt1" ? 0 : 8);
            ushort first = BitConverter.ToUInt16(data, colorStart), second = BitConverter.ToUInt16(data, colorStart + 2);
            Color565(first, colors[..4]); Color565(second, colors.Slice(4, 4));
            if (format != "Dxt1" || first > second)
            {
                for (int c = 0; c < 3; c++) {
                    colors[8 + c] = (byte)((2 * colors[c] + colors[4 + c]) / 3);
                    colors[12 + c] = (byte)((colors[c] + 2 * colors[4 + c]) / 3);
                }
                colors[11] = colors[15] = 255;
            }
            else {
                for (int c = 0; c < 3; c++) colors[8 + c] = (byte)((colors[c] + colors[4 + c]) / 2);
                colors[11] = 255; colors.Slice(12, 4).Clear();
            }
            ulong alphaBits = 0;
            if (format == "Dxt3") alphaBits = BitConverter.ToUInt64(data, start);
            if (format == "Dxt5")
            {
                alphas[0] = data[start]; alphas[1] = data[start + 1];
                if (alphas[0] > alphas[1]) {
                    for (int i = 2; i < 8; i++) alphas[i] = (byte)(((8 - i) * alphas[0] + (i - 1) * alphas[1]) / 7);
                } else {
                    for (int i = 2; i < 6; i++) alphas[i] = (byte)(((6 - i) * alphas[0] + (i - 1) * alphas[1]) / 5);
                    alphas[6] = 0; alphas[7] = 255;
                }
                for (int i = 0; i < 6; i++) alphaBits |= (ulong)data[start + 2 + i] << (i * 8);
            }
            uint indices = BitConverter.ToUInt32(data, colorStart + 4);
            for (int i = 0; i < 16; i++)
            {
                int x = bx * 4 + i % 4, y = by * 4 + i / 4;
                if (x >= width || y >= height) continue;
                int palette = (int)((indices >> (i * 2)) & 3) * 4, destination = (y * width + x) * 4;
                colors.Slice(palette, 4).CopyTo(rgba.AsSpan(destination, 4));
                if (format == "Dxt3") rgba[destination + 3] = (byte)(((alphaBits >> (i * 4)) & 15) * 17);
                if (format == "Dxt5") rgba[destination + 3] = alphas[(int)((alphaBits >> (i * 3)) & 7)];
            }
        }
        return rgba;
    }

    static void Color565(ushort color, Span<byte> result)
    {
        int r = (color >> 11) & 31, g = (color >> 5) & 63, b = color & 31;
        result[0] = (byte)((r << 3) | (r >> 2)); result[1] = (byte)((g << 2) | (g >> 4));
        result[2] = (byte)((b << 3) | (b >> 2)); result[3] = 255;
    }

    internal static Array Read(object texture, Type color)
    {
        var type = texture.GetType();
        int width = (int)type.GetProperty("Width")!.GetValue(texture)!, height = (int)type.GetProperty("Height")!.GetValue(texture)!;
        string format = type.GetProperty("Format")!.GetValue(texture)!.ToString()!;
        var read = type.GetMethods().Single(m => m.Name == "GetData" && m.IsGenericMethodDefinition && m.GetParameters().Length == 1 && m.GetParameters()[0].ParameterType.IsArray);
        var pixels = Array.CreateInstance(color, checked(width * height));
        if (format == "Color") { read.MakeGenericMethod(color).Invoke(texture, new object[] { pixels }); return pixels; }
        int blockBytes = format switch { "Dxt1" => 8, "Dxt3" or "Dxt5" => 16, _ => throw new NotSupportedException("Unsupported font texture storage") };
        var compressed = new byte[checked(((width + 3) / 4) * ((height + 3) / 4) * blockBytes)];
        read.MakeGenericMethod(typeof(byte)).Invoke(texture, new object[] { compressed });
        byte[] rgba = DecodeDxt(compressed, width, height, format);
        if (Marshal.SizeOf(color) != 4) throw new NotSupportedException("Unexpected Color layout");
        var pinned = GCHandle.Alloc(pixels, GCHandleType.Pinned);
        try { Marshal.Copy(rgba, 0, pinned.AddrOfPinnedObject(), rgba.Length); }
        finally { pinned.Free(); }
        return pixels;
    }
}
