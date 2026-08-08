# raylib 之后的下一写回 Adapter 选择

Status: Complete

## Goal

在 raylib 生产接入完成后选择下一种能实际增加软件覆盖的 `TextReplace` Adapter。候选必须绑定一个
外部真实目标，并由公开动态入口在每次更新时取得足够完整的原文；UI Automation、OCR 或外部浮层不计入
本切片的实时写回结果。

## Delivery

- [x] 复核当前已否决或暂缓的文字栈，避免重复建设 WPF、WinUI 加载时资源、静态链接框架、缓存纹理或
  需要目标主动开放调试接口的路线。
- [x] 从可重复 Windows 目标中筛出最多三个候选，逐项记录动态 ABI、完整原文、临时写回、代次更新、
  停用恢复、字体与资源生命周期门槛。
- [x] 按门槛决定是否进入本机 Go/No-Go：FLTK 缺外部动态真实目标，SFML 与 Dear ImGui 的当前真实
  候选均为静态/缓存几何路径，因此本轮没有候选进入 Hook，也未立项生产 crate。

## Current

三个候选的一手资料复核已完成。FLTK 的公开 UTF-8 `fl_draw` 是最值得等待真实目标触发的入口，但其
Windows shared build 默认关闭且 C++ DLL 要求编译器匹配，当前没有可重复的外部动态目标；SFML 最终
绘制只提交缓存 vertices，Open Hexagon 又显式静态链接；Dear ImGui 的 Tracy 候选同样静态集成。
本轮以“暂无候选”收口，没有新增 Adapter。UIA 继续只作为查看/采集能力。

## Next

- 等待新的、明确授权且可重复的 Windows 真实目标；先查 PE import、运行模块和公开文字导出，再恢复
  [第一方资料复核](../references/post-raylib-next-adapter-primary-source-review.md)中的 FLTK 或新入口。
- 在固定可见词条命中前不建设原型；UIA、OCR、纹理缓存、静态框架和私有符号不计入候选。

## Boundaries

- 不读取或注入未显式授权的本机进程；真实路径、PID、窗口标题和原始证据只进 `local-test/`。
- 不把成功注入、模块存在、Dictionary 决策或 UIA 观察单独当作实时翻译成功。
- 不采用单软件地址表、私有符号扫描或静态实现复制来制造表面覆盖。

## References

- [raylib `DrawTextEx` 生产 Adapter](raylib-draw-text-adapter.md)
- [raylib 之后的下一 Adapter 第一方资料复核](../references/post-raylib-next-adapter-primary-source-review.md)
- [下一 Adapter 第一方资料评审](../references/next-adapter-primary-source-review.md)
- [SDL3_ttf 之后的动态 C 文字入口复核](../references/post-sdl3-next-dynamic-c-seam-review.md)
