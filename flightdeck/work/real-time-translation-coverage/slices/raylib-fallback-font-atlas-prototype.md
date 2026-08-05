# raylib fallback Font/atlas 原型

Status: Complete

## Goal

回答一个窄问题：当目标 raylib `Font` 的 atlas 不含中文字形时，能否在不修改应用字符串或原 Font 的
前提下，于公开绘制调用期间临时使用自有 fallback Font，并安全完成 Dictionary 两代更新、停用恢复和
GPU context 关闭前销毁。答案必须同时来自可操作状态机与外部真实软件的动态构建。

## Prototype question

状态由 GPU context epoch、渲染线程、Dictionary generation、所需 codepoint 集合、当前/待回收 atlas、
启用状态和关闭阶段构成。原型要验证：新 atlas 先创建再原子切换、旧 atlas 只在无本帧引用时回收；停用
立即透传原字符串和原 Font；关闭只能在同一渲染线程、context 仍有效时清空资源。任何线程或生命周期
条件不满足时都必须 fail open。

## Delivery

- [x] 建立一个本机 throwaway 终端原型，以纯状态机暴露 connect、draw、publish、disable、enable、
  close 与 context recreate 转移，并能逐步显示完整状态和非法动作原因。
- [x] 用单一命令运行预设场景，覆盖首代 atlas、第二代新增字形、停用透传、重新启用与 context 关闭；
  不为该 throwaway shell 增加生产依赖或测试。
- [x] 固定 Musializer 源码版本并使用项目自有 hot-reload Windows 配置，确认目标 EXE 导入、进程加载及
  raylib 公开导出；静态或不健康运行立即 No-Go。
- [x] 在真实 `DrawTextEx` 调用中验证原文命中、首代中文、第二代新增字形、停用恢复和关闭前资源释放，
  保留可见像素与诊断证据；不扩张到私有地址或软件品牌分支。
- [x] 两层结果均为 Go，已单独打开生产 Adapter Slice。

## Current

原型完成。状态机 23 个转移保持不变量；真实动态目标完成首代、第二代、停用和重新启用的完整像素验收，
关闭前也释放最后一个 fallback atlas。默认 `LoadFontEx` 行式 atlas 对较宽 CJK glyph 集合会错误置零
rectangle，因此最终原型使用公开 `LoadFontData` + `GenImageFontAtlas(packMethod=1)`，并逐一校验 bitmap、
rectangle 边界和重叠。48px 栅格配合 mipmap 与双线性过滤在目标 64px 绘制中清晰可读。

## Next

- 转入 [raylib `DrawTextEx` 生产 Adapter](raylib-draw-text-adapter.md)，原型壳与临时 Hook 不进入生产树。

## Result

- 动态 ABI、实际模块、公开导出与 `DrawTextEx` 命中通过。
- 首代 5 个中文 glyph、第二代 7 个新增集合、停用原文、重新启用译文均由像素确认。
- 第二代切换后的旧 atlas 与停用后的活动 atlas 均在 `EndDrawing` 完成后释放；重新启用后的最后一个
  atlas 在 `CloseWindow` 销毁 context 前由同一渲染线程释放。
- 初始截图一帧错位、字体缺字和默认 atlas packing 失败都被像素或公开结构校验发现并修正，没有把
  日志命中误报成可见成功。

## Decision

**Go。** raylib 5.5 动态公开 ABI 可以形成带中文 fallback Font 的实时 `TextReplace` seam；进入独立
生产 Adapter，但只承诺动态 `raylib.dll`、公开 `DrawTextEx` 和可验证字体/atlas，静态链接、复制实现、
`DrawTextCodepoint(s)` 直调与已缓存纹理继续不在覆盖范围。

## Boundaries

- 所有原型代码、外部源码、真实字体、可执行文件、日志、截图和运行描述只放在仓库忽略的本机测试目录。
- `Font` 按值 ABI 必须与实际 raylib 版本和架构精确匹配；不识别时安全放行。
- 不修改目标持有的字符串、Font 或业务状态；替换缓冲和 fallback Font 只服务当前同步绘制。
- 不使用 EXE 名称、品牌、私有地址或版本签名作为生产选择条件。
- 本 Slice 只回答可行性，不新增生产 crate、Catalog、Bundle 或用户可选项。

## References

- [SDL3_ttf 之后的动态 C 文字入口复核](../references/post-sdl3-next-dynamic-c-seam-review.md)
