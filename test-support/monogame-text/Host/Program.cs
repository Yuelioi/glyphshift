using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;
using System.Collections.Concurrent;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Reflection;

if (args.Length != 1) throw new ArgumentException("Expected a local evidence directory");
Console.InputEncoding = new UTF8Encoding(false);
Console.OutputEncoding = new UTF8Encoding(false);
using var game = new TextGame(Path.GetFullPath(args[0]));
game.Run();

sealed class TextGame : Game
{
    readonly GraphicsDeviceManager graphics;
    readonly ConcurrentQueue<JsonElement> requests = new();
    readonly string evidence;
    SpriteBatch batch = null!;
    SpriteFont complete = null!, latin = null!, compressedLatin = null!;
    RenderTarget2D surface = null!;
    readonly RasterizerState clipped = new() { ScissorTestEnable = true };
    NativeHarness? adapter;
    RuntimeHarness? runtime;
    JsonElement? drawing;
    object? pendingResponse;
    FrameCapture? pendingFrame;
    int sequence;
    MethodInfo? fallbackReference;
    string originalLatinHash = "";
    public TextGame(string evidence)
    {
        this.evidence = evidence;
        Directory.CreateDirectory(evidence);
        graphics = new GraphicsDeviceManager(this) { PreferredBackBufferWidth = 480, PreferredBackBufferHeight = 180, SynchronizeWithVerticalRetrace = false };
        IsFixedTimeStep = true;
        TargetElapsedTime = TimeSpan.FromMilliseconds(16);
        IsMouseVisible = true;
        Window.Title = "Glyphshift MonoGame synthetic text";
    }
    protected override void LoadContent()
    {
        batch = new SpriteBatch(GraphicsDevice);
        complete = MakeFont(true); latin = MakeFont(false); compressedLatin = MakeFont(false, true);
        originalLatinHash = TextureHash(latin.Texture);
        surface = new RenderTarget2D(GraphicsDevice, 480, 180, false, SurfaceFormat.Color, DepthFormat.None, 0, RenderTargetUsage.PreserveContents);
        foreach (string mode in new[] { "string", "builder", "vector", "builder-vector", "scalar", "builder-scalar" })
            RenderText("AAA", mode, complete, false);
        Console.WriteLine("READY");
        _ = Task.Run(() =>
        {
            string? line;
            while ((line = Console.ReadLine()) != null)
            {
                using var document = JsonDocument.Parse(line);
                requests.Enqueue(document.RootElement.Clone());
            }
            using var exit = JsonDocument.Parse("{\"command\":\"exit\"}");
            requests.Enqueue(exit.RootElement.Clone());
        });
    }
    protected override void Update(GameTime gameTime)
    {
        if (pendingResponse != null) { Console.WriteLine(JsonSerializer.Serialize(pendingResponse)); pendingResponse = null; }
        if (drawing == null && requests.TryDequeue(out var request))
        {
            try
            {
                object? response = null;
                switch (request.GetProperty("command").GetString())
                {
                    case "render": drawing = request; break;
                    case "load":
                        if (Environment.GetEnvironmentVariable("GLYPHSHIFT_TEST_RUNTIME") is { } runtimePath) {
                            runtime = new RuntimeHarness(runtimePath, Environment.GetEnvironmentVariable("GLYPHSHIFT_TEST_DEPLOYMENTS")!);
                            response = new { status = runtime.Activate() };
                        } else { adapter = new NativeHarness(request.GetProperty("path").GetString()!, "AAA", "中文", "中"); response = new { status = adapter.Activate() }; }
                        break;
                    case "status": response = new { status = runtime == null ? adapter!.Status() : 0, calls = adapter?.Calls ?? 0 }; break;
                    case "runtime_observations": response = new { observations = runtime!.Read("glyphshift_runtime_observation_query_v1"), ack = runtime.Read("glyphshift_runtime_activation_query_v1") }; break;
                    case "generation": response = runtime == null ? adapter!.SetGeneration(request.GetProperty("generation").GetInt32()) : runtime.SetGeneration(request.GetProperty("generation").GetInt32()); break;
                    case "deactivate": response = new { status = runtime == null ? adapter!.Deactivate() : runtime.Deactivate() }; break;
                    case "activate": response = new { status = runtime == null ? adapter!.Activate() : runtime.Activate() }; break;
                    case "observe": response = new { status = adapter!.Activate(1) }; break;
                    case "revert": response = new { status = adapter!.Revert() }; break;
                    case "gc": GC.Collect(); GC.WaitForPendingFinalizers(); GC.Collect(); response = new { collected = true }; break;
                    case "exit": Console.WriteLine("{\"exiting\":true}"); Exit(); break;
                    default: throw new InvalidOperationException("Unknown command");
                }
                if (response != null) Console.WriteLine(JsonSerializer.Serialize(response));
            }
            catch (Exception ex) { Console.WriteLine(JsonSerializer.Serialize(new { error = ex.ToString() })); }
        }
        base.Update(gameTime);
    }
    protected override void Draw(GameTime gameTime)
    {
        if (drawing is { } request)
        {
            drawing = null;
            try
            {
                string text = request.GetProperty("text").GetString()!;
                string mode = request.GetProperty("mode").GetString()!;
                bool clip = request.TryGetProperty("clip", out var clipValue) && clipValue.GetBoolean();
                bool missing = request.TryGetProperty("missing", out var missingValue) && missingValue.GetBoolean();
                bool compressed = request.TryGetProperty("compressed", out var compressedValue) && compressedValue.GetBoolean();
                SpriteFont selectedFont = compressed ? compressedLatin : missing ? latin : complete;
                if (request.TryGetProperty("fallbackReference", out var fallback) && fallback.GetBoolean()) {
                    fallbackReference ??= Assembly.Load(File.ReadAllBytes(Environment.GetEnvironmentVariable("GLYPHSHIFT_FALLBACK_REFERENCE")!))
                        .GetType("Glyphshift.MonoGame.FontFallback")!.GetMethod("Resolve")!;
                    fallbackReference.Invoke(null, new object?[] { selectedFont, text, true });
                    fallbackReference.Invoke(null, new object?[] { GraphicsDevice, null, true });
                    selectedFont = (SpriteFont)fallbackReference.Invoke(null, new object?[] { selectedFont, text, true })!;
                }
                var result = RenderText(text, mode, selectedFont, clip);
                string name = $"frame-{++sequence:D3}.png";
                using (var file = File.Create(Path.Combine(evidence, name))) surface.SaveAsPng(file, surface.Width, surface.Height);
                pendingFrame = new FrameCapture(result.hash, result.width, result.height, result.pixels, result.builderUnchanged,
                    adapter?.Calls ?? 0, name, sequence, "", latin.Characters.SequenceEqual(new[] { ' ', 'A', 'B' }) && TextureHash(latin.Texture) == originalLatinHash);
            }
            catch (Exception ex) { GraphicsDevice.SetRenderTarget(null); pendingResponse = new { error = ex.ToString() }; }
        }
        GraphicsDevice.SetRenderTarget(null);
        GraphicsDevice.Clear(Color.Black);
        batch.Begin(samplerState: SamplerState.PointClamp);
        batch.Draw(surface, Vector2.Zero, Color.White);
        batch.End();
        if (pendingFrame != null)
        {
            var backbuffer = new Color[480 * 180];
            GraphicsDevice.GetBackBufferData(backbuffer);
            pendingResponse = pendingFrame with { backbufferHash = PixelHash(backbuffer) };
            pendingFrame = null;
        }
        base.Draw(gameTime);
    }
    (string hash, float width, float height, int pixels, bool builderUnchanged) RenderText(string text, string mode, SpriteFont font, bool clip)
    {
        var builder = new StringBuilder(text);
        var measured = mode.StartsWith("builder", StringComparison.Ordinal) ? font.MeasureString(builder) : font.MeasureString(text);
        GraphicsDevice.SetRenderTarget(surface);
        GraphicsDevice.Clear(new Color(18, 22, 30));
        GraphicsDevice.ScissorRectangle = clip ? new Rectangle(0, 0, 50, 180) : new Rectangle(0, 0, 480, 180);
        batch.Begin(samplerState: SamplerState.PointClamp, rasterizerState: clip ? clipped : RasterizerState.CullNone);
        var position = new Vector2(24, 30);
        // A shadow and foreground exercise repeated draws without mutating the source.
        foreach (var offset in new[] { new Vector2(1, 1), Vector2.Zero })
        {
            var color = offset == Vector2.Zero ? Color.White : Color.Black;
            switch (mode)
            {
                case "string": batch.DrawString(font, text, position + offset, color); break;
                case "builder": batch.DrawString(font, builder, position + offset, color); break;
                case "vector": batch.DrawString(font, text, position + offset, color, 0, Vector2.Zero, new Vector2(3), SpriteEffects.None, 0); break;
                case "builder-vector": batch.DrawString(font, builder, position + offset, color, 0, Vector2.Zero, new Vector2(3), SpriteEffects.None, 0); break;
                case "scalar": batch.DrawString(font, text, position + offset, color, 0, Vector2.Zero, 3f, SpriteEffects.None, 0); break;
                case "builder-scalar": batch.DrawString(font, builder, position + offset, color, 0, Vector2.Zero, 3f, SpriteEffects.None, 0); break;
                default: throw new InvalidOperationException("Unknown draw mode");
            }
        }
        batch.End();
        GraphicsDevice.SetRenderTarget(null);
        var data = new Color[surface.Width * surface.Height];
        surface.GetData(data);
        int pixels = 0;
        for (int i = 0; i < data.Length; i++)
        {
            if (data[i] == Color.White) pixels++;
        }
        return (PixelHash(data), measured.X, measured.Y, pixels, builder.ToString() == text);
    }
    static string PixelHash(Color[] data)
    {
        byte[] bytes = new byte[data.Length * 4];
        for (int i = 0; i < data.Length; i++)
        {
            bytes[i * 4] = data[i].R; bytes[i * 4 + 1] = data[i].G; bytes[i * 4 + 2] = data[i].B; bytes[i * 4 + 3] = data[i].A;
        }
        return Convert.ToHexString(SHA256.HashData(bytes));
    }
    static string TextureHash(Texture2D texture) { var pixels = new Color[texture.Width * texture.Height]; texture.GetData(pixels); return PixelHash(pixels); }
    SpriteFont MakeFont(bool includeChinese, bool compressed = false)
    {
        var glyphs = new SortedDictionary<char, string[]>
        {
            [' '] = new[] { "000", "000", "000", "000", "000", "000", "000" },
            ['A'] = new[] { "01110", "10001", "10001", "11111", "10001", "10001", "10001" },
            ['B'] = new[] { "11110", "10001", "10001", "11110", "10001", "10001", "11110" },
            ['中'] = new[] { "0001000", "0001000", "1111111", "1001001", "1111111", "0001000", "0001000" },
            ['文'] = new[] { "0001000", "1111111", "0100010", "0010100", "0001000", "0010100", "1100011" }
        };
        if (!includeChinese) { glyphs.Remove('中'); glyphs.Remove('文'); }
        int width = glyphs.Sum(glyph => glyph.Value[0].Length + 1);
        width = compressed ? (width + 3) & ~3 : width;
        int textureHeight = compressed ? 8 : 7;
        var texture = new Texture2D(GraphicsDevice, width, textureHeight, false, compressed ? SurfaceFormat.Dxt3 : SurfaceFormat.Color);
        var data = new Color[width * textureHeight];
        var bounds = new List<Rectangle>(); var cropping = new List<Rectangle>(); var chars = new List<char>(); var kerning = new List<Vector3>();
        int x = 0;
        foreach (var (character, rows) in glyphs)
        {
            int glyphWidth = rows[0].Length;
            for (int y = 0; y < 7; y++) for (int column = 0; column < glyphWidth; column++)
                data[y * width + x + column] = rows[y][column] == '1' ? Color.White : Color.Transparent;
            bounds.Add(new Rectangle(x, 0, glyphWidth, 7)); cropping.Add(new Rectangle(0, 0, glyphWidth, 7)); chars.Add(character); kerning.Add(new Vector3(0, glyphWidth, 1));
            x += glyphWidth + 1;
        }
        if (compressed) texture.SetData(EncodeMaskDxt3(data, width, textureHeight)); else texture.SetData(data);
        return new SpriteFont(texture, bounds, cropping, chars, 9, 0, kerning, null);
    }
    static byte[] EncodeMaskDxt3(Color[] pixels, int width, int height)
    {
        var result = new byte[width * height]; int offset = 0;
        for (int y = 0; y < height; y += 4) for (int x = 0; x < width; x += 4)
        {
            ulong alpha = 0; uint indices = 0;
            for (int i = 0; i < 16; i++) {
                bool white = pixels[(y + i / 4) * width + x + i % 4].A != 0;
                if (white) alpha |= 15UL << (i * 4); else indices |= 1U << (i * 2);
            }
            BitConverter.GetBytes(alpha).CopyTo(result, offset);
            result[offset + 8] = 255; result[offset + 9] = 255;
            BitConverter.GetBytes(indices).CopyTo(result, offset + 12); offset += 16;
        }
        return result;
    }
    protected override void UnloadContent()
    {
        adapter?.Deactivate();
        if (runtime != null) runtime.Deactivate();
        complete.Texture.Dispose(); latin.Texture.Dispose(); compressedLatin.Texture.Dispose(); surface.Dispose(); batch.Dispose(); clipped.Dispose();
        base.UnloadContent();
    }
}
sealed record FrameCapture(string hash, float width, float height, int pixels, bool builderUnchanged,
    long calls, string frame, int generationFrame, string backbufferHash, bool originalFontUnchanged);
