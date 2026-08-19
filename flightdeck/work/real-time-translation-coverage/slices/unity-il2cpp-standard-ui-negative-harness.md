# Unity IL2CPP Standard UI 同后端负例宿主

Status: Open

## Goal

回答一个窄问题：一份确定性 Windows x64 Unity IL2CPP non-development Player 在持续显示
自研 bitmap atlas + Mesh 文字、但没有 TMP/uGUI live text object 时，未发布的 Standard UI
Adapter 能否在同后端内可靠拒绝。本 Slice 只建立原型前置负例，不实现 Adapter，不进入
Catalog 或 Runtime Bundle。

## Delivery

- [x] 在 `test-support/` 下建立可跟踪的最小 Unity 工程源码；源码与合成资产可提交，Unity 缓存、
  构建物、日志、截图与本机路径只能进入 `local-test/evidence/`。
- [x] 实现运行时生成的 bitmap atlas、`MeshFilter` 与 `MeshRenderer` 固定/动态文字 marker；
  工程保留官方 TMP/uGUI package 与类型，但场景不创建 Canvas、TMP 或 uGUI Text，并由运行时守卫持续
  断言两类 live text object 都为零。
- [x] 用工程自有 Editor build script 固定 `StandaloneWindows64 + IL2CPP + BuildOptions.None`，并在
  构建后检查 x64、`GameAssembly.dll`、`global-metadata.dat` 与无 Mono runtime。
- [ ] 在已安装 Unity 2022.3.5f1 Windows IL2CPP 模块的机器上完成一次构建，实际通过上述产物检查。
- [ ] 连续两次 runtime-only smoke：固定 marker 可见、动态 marker 发生变化、不加载 Glyphshift，
  均由窗口关闭正常退出。
- [ ] 完成负例宿主的边界自审：自绘 marker 真实存在，标准 UI live object 为空，且没有用窗口
  标题、日志或元数据类型存在冒充界面文字。

## Current

连续四次有界公开候选复核仍没有得到官方 Windows x64 IL2CPP Shipping-like 自绘进程级负例。
第四轮的 nofun 已证明 IL2CPP 与 guest bitmap atlas/mesh 可以共存，但它同时实例化 TMP，并且
没有发布包自带的固定 guest 文字触发，因此只能作为混合栈对照，不能作为进程级负例。

为了不再用第五轮无界搜索阻塞实现，样本 gate 现分成两层：本 Slice 的确定性同后端负例通过后，
只允许开始未发布的 IL2CPP observe-only 原型；在新增生产 crate、Catalog 或 Runtime Bundle 项之前，
仍必须补齐一份外部 Shipping-like 自绘负例。

确定性宿主源码与构建包装器现已加入：运行时用自建 5x7 bitmap atlas 和 Mesh 显示固定
`CUSTOM MESH NEGATIVE` 与每秒变化的 `TICK ####`。官方 TMP/uGUI 类型会保留进 IL2CPP metadata，但不
创建标准文字实例；运行时守卫在发现任一 TMP/uGUI Text live object 时退出。静态合同会拒绝标准 UI
渲染路径、缺少类型/守卫、缺失字形、非 Windows x64、非 IL2CPP 或 Development build 配置。当前机器
没有可用的 Unity 2022.3.5f1 Editor 与 Windows IL2CPP 模块，因此尚未生成或运行 Player；这不是负例
gate 已通过。

## Next

- 在获准且具备 Unity 2022.3.5f1 Windows IL2CPP 模块后，通过本机环境变量提供 Editor 路径并运行
  定向构建包装器；只把生成工程、构建物与日志写入本地 evidence。
- 启动生成的 Player 完成两次 runtime-only smoke，记录固定 marker、动态 marker、已加载模块与正常
  窗口关闭；不加载 Glyphshift。
- 构建与 smoke 通过后完成一次边界审核，再决定是否开启未发布 observe-only 原型；不运行全仓测试，
  不附加外部程序。

## Decisions

- 确定性合成负例只解锁 observe-only 原型，不能提升为真实软件支持或生产接入证据。
- 生产 gate 仍要求外部、可重复的 Windows x64 IL2CPP Shipping-like 自绘负例；四轮公开筛选的
  No-Go 不会被改写成“无需真实负例”。
- 拒绝必须依据同后端中没有 TMP/uGUI live text object，不能仅因 metadata 包含或缺少某个类名。
- 混合栈中自绘 marker 漏抓不是进程级负例；只要存在受支持的 live object，Standard UI lane
  就应只承诺覆盖该标准对象，而不是拒绝整个进程。
- 宿主只使用 Unity 公开组件生成自研文字 Mesh；不复制商业资产，不伪装成外部游戏。

## Verification

- `pwsh -NoProfile -File scripts/build-unity-il2cpp-negative-harness.ps1 -ValidateOnly`：通过；静态合同同时
  检查自绘渲染路径、TMP/uGUI package 与 live-object 守卫、固定/动态 marker 字形覆盖、Windows x64
  IL2CPP Release build 配置与源码目录无二进制制品；真实构建还会检查 metadata 保留两类标准 UI 类型。
- PowerShell AST 解析：0 errors；定向尾随空白与本机隐私扫描：0 findings；本地 evidence 输出路径受
  仓库 `target` ignore 规则覆盖。
- 两轮独立只读审核均未发现阻塞项；编辑器版本边界、组件创建漏检与类型身份/metadata 证据强度问题
  均已修复并经复核关闭。
- Unity 2022.3 API 资料已确认当前使用的 IL2CPP compiler configuration、build target 与非开发构建
  设置公开存在；没有可用 Editor/module，故 C# 编译、Player 产物检查和两次 runtime-only smoke 未运行。
- 第四轮公开复核已经独立审核：7 个不重复候选，0 接受 / 7 拒绝；未下载、运行、注入或本地
  重建任何候选。

## Boundaries

- 不修改或运行外部候选；本 Slice 后续只运行 Glyphshift 仓库自有的确定性宿主。
- 不开始 IL2CPP attach，直到宿主的构建形态、可见自绘 marker 和无标准 UI live object 都已证明。
- 不新增生产 crate、Catalog、Runtime Bundle 或用户可见 Adapter 技术项。
