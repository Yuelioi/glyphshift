# Allegro / Open Surge 真实目标 smoke

Status: Complete

## Goal

用 Open Surge Engine 的 Windows 真实产物验证 Allegro 5 font addon 是否形成可复用的动态绘制时
`TextReplace` seam。必须先证明公开 DLL 边界与实际 TTF 文字命中，再决定是否建立生产 Adapter。

## Delivery

- [x] 取得固定版本的项目发布包和显式动态 Windows 构建，并检查目标 EXE 的 PE imports。
- [x] 运行时确认独立 Allegro font/TTF 模块实际加载，目标文字公开导出可解析；静态或 monolith 立即
  No-Go。
- [x] 按顺序门槛在文字选择前停止：动态组合未形成健康运行目标，不继续尝试 TTF 单行词条，也不扩张
  到 bitmap font、富文本拆段或 `*_ustr`/`*textf` 入口。
- [x] 前两项未形成健康可运行目标，因此按顺序门槛停止，不建立替换原型。
- [x] 根据真实运行结果作 No-Go，不进入 Allegro Adapter 实现。

## Result

- Open Surge v0.6.1.3 正式 Windows 包只有单一 EXE，没有 DLL；PE import 也没有 Allegro 模块，属于
  静态或 monolith 分发，立即排除。
- 使用项目公开构建选项与固定版 Allegro 动态包后，显式动态构建成功；PE import 包含 Allegro core、
  font、TTF 等十个独立 DLL，运行时也实际观察到 core、font 与 TTF 模块，三个计划文字入口均存在公开
  导出。
- 该动态组合无法形成健康目标。调试回溯显示初始化在 `allegro_physfs` 内的 `PHYSFS_openRead` 进入
  无效互斥状态并崩溃；Open Surge 自身与预构建 addon 使用了不共享状态的 PhysFS 实例，这是根据模块
  与回溯作出的诊断。要继续必须定制重建整套 Allegro/PhysFS 依赖，已经偏离可重复真实目标门槛。
- 目标未到达稳定 UI 或文字绘制阶段，因此没有执行 Hook 命中、Dictionary 或可见三态验收，也没有把
  模块与导出成功误报成文字支持。
- 下载、构建、兼容性补丁、PE 输出、调试回溯与运行日志全部只保存在本机私有测试目录。

## Decision

**No-Go。** Open Surge 的用户发布产物不具备动态 Hook seam；源码动态组合又要求项目外的定制依赖
重建才能健康运行。当前没有足够真实采用价值支撑 Allegro 生产 Adapter，不新增 crate、Catalog、
Runtime Bundle 或软件支持声明。如果未来出现原生分发独立 Allegro font DLL 且健康命中公开入口的
外部目标，再以该目标重新打开，而不是保留空置实现。

## Boundaries

- 所有下载、构建物、模块信息、截图、原始日志、运行描述与临时 Hook 只保存在本机私有测试目录。
- 不修改目标业务字符串、字体对象或资源；仅允许在同步绘制调用期间使用独立 UTF-8 缓冲。
- 不做 EXE 名称、版本签名、私有地址或软件品牌分支。
- Open Surge 在 Allegro 之前形成的单行、单色 segment 是本轮最大语义单位，不声称恢复完整富文本消息。

## References

- [SDL3_ttf 之后的动态 C 文字入口复核](../references/post-sdl3-next-dynamic-c-seam-review.md)
