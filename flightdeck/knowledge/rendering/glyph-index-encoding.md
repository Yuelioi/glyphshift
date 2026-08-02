# ⚠ AE 界面文字是字形索引,不是字符（GDI 层）

## 现象

hook `ExtTextOutW` 抓到 AE 菜单文字是 `,ORK` `+JOZ` `)USVUYOZOUT` 这种,不是 `File`/`Edit`/`Composition`。

## 真因

AE 调 `ExtTextOutW` 时 **options 带 `ETO_GLYPH_INDEX` (0x0010)**:`lpString` 里是**字形索引(glyph ID)**,不是 Unicode 字符。字体是标准 `Microsoft YaHei UI`(charset=1),不是私有字体。

菜单字体里 glyph_index ≈ char_code − 26(ASCII 段),所以 `,`(0x2C=44)+26=`F`(70),整串 +26 即还原成可读英文。**这是字体 cmap 的字形排布巧合,不是加密**;不同字体偏移不同(对话框那套字体偏移≠26 → 不能硬套 +26,见下)。

词间空格在原串里是 `0x03`;`.`(省略号点)是 `0x14`(20),+26=46=`.`。

## 解法（菜单已验证可行）

1. 解码:针对当前字体调用 `GetGlyphIndices`，由候选字符集生成 glyph→char 反查表；其中 `0x03` 特判为空格。同时也试原串直接查，兼容少数传入真字符的调用。早期观测到的 `+26` 只适用于特定菜单字体，不是生产算法。
2. 命中后画中文,**关键**:回写时 **清掉 `ETO_GLYPH_INDEX` 标志**(`options & !0x10`)。否则 GDI 把中文 Unicode 码位当字形索引画 → 乱码方块。**这个坑排查了好几轮**(先误判成 charset 问题,清字体没用,最后定位到 glyph-index 标志)。
3. 字体:用 `GetCurrentObject(hdc,OBJ_FONT)`+`GetObjectW` 取原字号,重建一支 `lfCharSet=DEFAULT_CHARSET` 的微软雅黑画中文(别继承原 LOGFONT 的 charset)。

## 通用化（对话框/其它字体）

不同字体 glyph→char 偏移不同,不能写死 +26。当前实现按字体 face 名称缓存反查表：用 `GetGlyphIndices` 跑候选字符集(ASCII 0x20–0x7E)建立 **glyph→char 反查表**，拿到字形索引串后用反表还原明文再查字典。

## 注意

- chunk/字形数:回写中文 lpDx(每字符间距数组)对不上,传 null 让 GDI 自己排版。
- 菜单栏只在**菜单激活态**走 GDI 重绘;退出激活态 AE 用自绘路径重画回英文 → 截图要在激活态抓(按 Alt 激活后立即抓屏)。
