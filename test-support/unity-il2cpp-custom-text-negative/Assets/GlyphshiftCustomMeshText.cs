using System;
using System.Collections.Generic;
using UnityEngine;

[DisallowMultipleComponent]
public sealed class GlyphshiftCustomMeshText : MonoBehaviour
{
    public const string StaticMarker = "CUSTOM MESH NEGATIVE";
    public const string DynamicPrefix = "TICK ";

    private const int GlyphWidth = 5;
    private const int GlyphHeight = 7;
    private const int CellWidth = 6;
    private const int CellHeight = 8;
    private const int AtlasColumns = 8;
    private const float QuadWidth = 0.42f;
    private const float QuadHeight = 0.60f;
    private const float Advance = 0.50f;
    private const float LineAdvance = 0.86f;
    private const string GlyphOrder = " ACEGHIKMNOSTUV0123456789";

    private static readonly Dictionary<char, byte[]> GlyphRows =
        new Dictionary<char, byte[]>
        {
            { 'A', Rows("01110", "10001", "10001", "11111", "10001", "10001", "10001") },
            { 'C', Rows("01111", "10000", "10000", "10000", "10000", "10000", "01111") },
            { 'E', Rows("11111", "10000", "10000", "11110", "10000", "10000", "11111") },
            { 'G', Rows("01110", "10001", "10000", "10111", "10001", "10001", "01110") },
            { 'H', Rows("10001", "10001", "10001", "11111", "10001", "10001", "10001") },
            { 'I', Rows("11111", "00100", "00100", "00100", "00100", "00100", "11111") },
            { 'K', Rows("10001", "10010", "10100", "11000", "10100", "10010", "10001") },
            { 'M', Rows("10001", "11011", "10101", "10101", "10001", "10001", "10001") },
            { 'N', Rows("10001", "11001", "10101", "10011", "10001", "10001", "10001") },
            { 'O', Rows("01110", "10001", "10001", "10001", "10001", "10001", "01110") },
            { 'S', Rows("01111", "10000", "10000", "01110", "00001", "00001", "11110") },
            { 'T', Rows("11111", "00100", "00100", "00100", "00100", "00100", "00100") },
            { 'U', Rows("10001", "10001", "10001", "10001", "10001", "10001", "01110") },
            { 'V', Rows("10001", "10001", "10001", "10001", "10001", "01010", "00100") },
            { '0', Rows("01110", "10001", "10011", "10101", "11001", "10001", "01110") },
            { '1', Rows("00100", "01100", "00100", "00100", "00100", "00100", "01110") },
            { '2', Rows("01110", "10001", "00001", "00010", "00100", "01000", "11111") },
            { '3', Rows("11110", "00001", "00001", "01110", "00001", "00001", "11110") },
            { '4', Rows("00010", "00110", "01010", "10010", "11111", "00010", "00010") },
            { '5', Rows("11111", "10000", "10000", "11110", "00001", "00001", "11110") },
            { '6', Rows("01110", "10000", "10000", "11110", "10001", "10001", "01110") },
            { '7', Rows("11111", "00001", "00010", "00100", "01000", "01000", "01000") },
            { '8', Rows("01110", "10001", "10001", "01110", "10001", "10001", "01110") },
            { '9', Rows("01110", "10001", "10001", "01111", "00001", "00001", "01110") },
        };

    [SerializeField]
    private Shader bitmapShader;

    private Mesh ownedMesh;
    private Material ownedMaterial;
    private Texture2D ownedAtlas;
    private int displayedTick = -1;

    public void Configure(Shader shader)
    {
        bitmapShader = shader;
    }

    private void Awake()
    {
        if (bitmapShader == null)
        {
            enabled = false;
            throw new InvalidOperationException("The deterministic bitmap shader is missing.");
        }

        ownedAtlas = CreateAtlas();
        ownedMaterial = new Material(bitmapShader)
        {
            name = "Glyphshift deterministic bitmap material",
            mainTexture = ownedAtlas,
            color = new Color(0.35f, 0.95f, 0.85f, 1.0f),
        };
        ownedMesh = new Mesh
        {
            name = "Glyphshift deterministic custom text mesh",
        };

        MeshFilter meshFilter = gameObject.AddComponent<MeshFilter>();
        meshFilter.sharedMesh = ownedMesh;
        MeshRenderer meshRenderer = gameObject.AddComponent<MeshRenderer>();
        meshRenderer.sharedMaterial = ownedMaterial;
        meshRenderer.sortingOrder = 10;

        UpdateMarker(0);
    }

    private void Update()
    {
        int nextTick = Mathf.FloorToInt(Time.unscaledTime) % 10000;
        if (nextTick != displayedTick)
        {
            UpdateMarker(nextTick);
        }

        if (Input.GetKeyDown(KeyCode.Escape))
        {
            Application.Quit();
        }
    }

    private void OnDestroy()
    {
        DestroyOwned(ownedMesh);
        DestroyOwned(ownedMaterial);
        DestroyOwned(ownedAtlas);
    }

    private void UpdateMarker(int tick)
    {
        displayedTick = tick;
        string text = StaticMarker + "\n" + DynamicPrefix + tick.ToString("D4");
        RebuildMesh(text);
    }

    private void RebuildMesh(string text)
    {
        string[] lines = text.Split('\n');
        List<Vector3> vertices = new List<Vector3>();
        List<Vector2> uvs = new List<Vector2>();
        List<int> triangles = new List<int>();

        int atlasRows = Mathf.CeilToInt((float)GlyphOrder.Length / AtlasColumns);
        float atlasWidth = AtlasColumns * CellWidth;
        float atlasHeight = atlasRows * CellHeight;

        for (int lineIndex = 0; lineIndex < lines.Length; lineIndex++)
        {
            string line = lines[lineIndex];
            float cursorX = -line.Length * Advance * 0.5f;
            float top = -lineIndex * LineAdvance + QuadHeight * 0.5f;
            float bottom = top - QuadHeight;

            foreach (char rawCharacter in line)
            {
                char character = char.ToUpperInvariant(rawCharacter);
                int glyphIndex = GlyphOrder.IndexOf(character);
                if (glyphIndex < 0)
                {
                    glyphIndex = 0;
                }

                if (glyphIndex != 0)
                {
                    int vertexStart = vertices.Count;
                    float left = cursorX;
                    float right = cursorX + QuadWidth;
                    vertices.Add(new Vector3(left, bottom, 0));
                    vertices.Add(new Vector3(left, top, 0));
                    vertices.Add(new Vector3(right, top, 0));
                    vertices.Add(new Vector3(right, bottom, 0));

                    int atlasColumn = glyphIndex % AtlasColumns;
                    int atlasRow = glyphIndex / AtlasColumns;
                    float u0 = atlasColumn * CellWidth / atlasWidth;
                    float v0 = atlasRow * CellHeight / atlasHeight;
                    float u1 = (atlasColumn * CellWidth + GlyphWidth) / atlasWidth;
                    float v1 = (atlasRow * CellHeight + GlyphHeight) / atlasHeight;
                    uvs.Add(new Vector2(u0, v0));
                    uvs.Add(new Vector2(u0, v1));
                    uvs.Add(new Vector2(u1, v1));
                    uvs.Add(new Vector2(u1, v0));

                    triangles.Add(vertexStart);
                    triangles.Add(vertexStart + 1);
                    triangles.Add(vertexStart + 2);
                    triangles.Add(vertexStart);
                    triangles.Add(vertexStart + 2);
                    triangles.Add(vertexStart + 3);
                }

                cursorX += Advance;
            }
        }

        ownedMesh.Clear();
        ownedMesh.SetVertices(vertices);
        ownedMesh.SetUVs(0, uvs);
        ownedMesh.SetTriangles(triangles, 0, true);
        ownedMesh.RecalculateBounds();
    }

    private static Texture2D CreateAtlas()
    {
        int atlasRows = Mathf.CeilToInt((float)GlyphOrder.Length / AtlasColumns);
        int atlasWidth = AtlasColumns * CellWidth;
        int atlasHeight = atlasRows * CellHeight;
        Texture2D atlas = new Texture2D(atlasWidth, atlasHeight, TextureFormat.RGBA32, false)
        {
            name = "Glyphshift deterministic 5x7 bitmap atlas",
            filterMode = FilterMode.Point,
            wrapMode = TextureWrapMode.Clamp,
        };

        Color32[] pixels = new Color32[atlasWidth * atlasHeight];
        Color32 foreground = new Color32(255, 255, 255, 255);
        for (int glyphIndex = 1; glyphIndex < GlyphOrder.Length; glyphIndex++)
        {
            char character = GlyphOrder[glyphIndex];
            byte[] rows = GlyphRows[character];
            int cellX = glyphIndex % AtlasColumns * CellWidth;
            int cellY = glyphIndex / AtlasColumns * CellHeight;
            for (int y = 0; y < GlyphHeight; y++)
            {
                for (int x = 0; x < GlyphWidth; x++)
                {
                    int mask = 1 << (GlyphWidth - 1 - x);
                    if ((rows[y] & mask) == 0)
                    {
                        continue;
                    }

                    int pixelY = cellY + GlyphHeight - 1 - y;
                    pixels[pixelY * atlasWidth + cellX + x] = foreground;
                }
            }
        }

        atlas.SetPixels32(pixels);
        atlas.Apply(false, true);
        return atlas;
    }

    private static byte[] Rows(params string[] rows)
    {
        if (rows.Length != GlyphHeight)
        {
            throw new ArgumentException("A glyph must contain seven rows.", nameof(rows));
        }

        byte[] result = new byte[GlyphHeight];
        for (int y = 0; y < rows.Length; y++)
        {
            if (rows[y].Length != GlyphWidth)
            {
                throw new ArgumentException("A glyph row must contain five pixels.", nameof(rows));
            }

            byte bits = 0;
            for (int x = 0; x < rows[y].Length; x++)
            {
                bits <<= 1;
                if (rows[y][x] == '1')
                {
                    bits |= 1;
                }
            }

            result[y] = bits;
        }

        return result;
    }

    private static void DestroyOwned(UnityEngine.Object value)
    {
        if (value != null)
        {
            Destroy(value);
        }
    }
}
