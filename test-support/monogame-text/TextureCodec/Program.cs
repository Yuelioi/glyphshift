using Glyphshift.MonoGame;

int passed = 0;
void Check(byte[] block, string format, byte[] expected, int width = 4, int height = 4)
{
    var rgba = TexturePixels.DecodeDxt(block, width, height, format);
    if (rgba.Length != width * height * 4) throw new Exception("Decoded extent mismatch");
    for (int i = 0; i < width * height; i++) if (!rgba.AsSpan(i * 4, 4).SequenceEqual(expected)) throw new Exception("RGBA palette mismatch: " + format);
    passed++;
}
Check(new byte[]{255,255,0,0,170,170,170,170}, "Dxt1", new byte[]{170,170,170,255});
Check(new byte[]{0,0,255,255,255,255,255,255}, "Dxt1", new byte[]{0,0,0,0});
Check(new byte[]{136,136,136,136,136,136,136,136,0,248,0,0,0,0,0,0}, "Dxt3", new byte[]{255,0,0,136});
Check(new byte[]{136,136,136,136,136,136,136,136,0,248,0,0,0,0,0,0}, "Dxt3", new byte[]{255,0,0,136}, 3, 2);
Check(new byte[]{0,255,255,255,255,255,255,255,31,0,0,0,0,0,0,0}, "Dxt5", new byte[]{0,0,255,255});
var bc3 = new byte[16]; bc3[0]=255; bc3[8]=31;
ulong bits=0; for(int i=0;i<16;i++) bits |= 2UL << (3*i);
for(int i=0;i<6;i++) bc3[2+i]=(byte)(bits>>(8*i));
Check(bc3,"Dxt5",new byte[]{0,0,255,218});
try { TexturePixels.DecodeDxt(new byte[1],4,4,"Dxt3"); throw new Exception("Malformed blocks accepted"); } catch(ArgumentException) {passed++;}
Console.WriteLine($"Texture codec: {passed} cases passed");
