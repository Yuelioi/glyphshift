# 引擎翻译能力调研汇总

调研日期：2026-08-04

## 结论

同类翻译工具公开展示的引擎覆盖可以作为市场需求参考，但不能直接作为 Glyphshift 的实现清单或支持
证明。公开资料通常只描述“内嵌翻译”“翻译弹窗”“文本翻译”等结果，不提供逐项接入点、版本矩阵、
失败边界和安全模型。

Glyphshift 不以支持对象数量作为目标，而以可验证的技术能力和明确边界定义支持程度。

## 统一分类

公开支持列表经常混合引擎、运行时、构建后端、内容类型和交付方式。Glyphshift 统一按技术路径分类：

| 能力维度 | Glyphshift 中的归属 |
| --- | --- |
| 结构化脚本或项目资源 | 后续独立 Content Adapter / Importer Work |
| 托管运行时 | Engine-aware Extension + Isolated Worker / Adapter |
| AOT 或原生运行时 | 独立能力与版本合同，不与托管后端合并 |
| Web 或脚本运行时 | 优先调查结构化接入点，再决定是否需要注入 |
| 标准可访问性文字 | 现有 observe-only Isolated Worker 路线 |
| 绘制、终端或系统文字 API | 现有低层 Capability Adapter |
| 无结构化通道 | 用户显式选择的 OCR 或外部呈现兜底 |

内容类型和动画中间件不足以单独定义 Adapter；同一引擎的托管与 AOT 构建也不能合并为一个技术能力。

## 公开技术实现的共同特征

公开实现抽样证明了一类通用进程内文字观察方案：独立注入器向授权目标部署文字提取模块，通过进程间
通道接收候选文本，支持自动或手动 Hook 以及多文字流选择，再把选中的原文交给主程序。

这类公开实现通常只能证明 `TextObserve`。在没有译文返回和目标进程写回证据时，不能提升为
`TextReplace`，也不能由通用观察器反推逐引擎内嵌模块的实现。

## 应采纳

### Support Matrix

每项“支持”至少拆成：

- Engine/runtime identity 与具体 build mode；
- 观察来源：脚本、资源、可访问性树、目标进程 API 或外部画面；
- 应用方式：原位替换、语言包、Overlay/弹窗、仅观察或离线改写；
- 已验证版本、真实目标证据和已知限制；
- 失败开放、权限、阻塞与恢复语义。

UI 可以显示用户熟悉的软件或技术名称，但执行能力必须映射到 Extension / Adapter Descriptor，不能在
Core、Dictionary 或 GUI 中增加品牌分支。

### Engine-aware Extension

引擎 Extension 负责识别构建模式、发现目标和组合既有 Adapter / Worker。只有确实不存在通用 Seam 时，
才实现引擎专属能力。首个验证对象必须具有授权真实目标、确定性 Fixture、可验证接入点和可见失败边界。

通用进程内文字提取可以作为独立 observe-only Adapter Pack 候选，但必须使用隔离 Host、有界流选择和
可恢复 Binding；成功提取原文不等于支持实时内嵌替换。

### Content Adapter

项目文件、脚本和资源包翻译具有独立价值，但不属于运行时 Adapter。后续若有明确需求，应建立独立
Content Adapter / Importer Work，负责扫描、导出、回写、备份和版本兼容。

其产出的 `source + translation` 可以与 Dictionary 互转；资源定位、改写证据和处理状态不进入
Dictionary Entry。

## 后续研究

- 优先调查 Web/脚本运行时的结构化观察与回写能力。
- 研究隔离的通用进程内文字观察器、候选流选择和跨启动重匹配，只声明 `TextObserve`。
- 托管运行时和 AOT/原生运行时分别立证，独立记录版本、元数据、权限与崩溃风险。
- 对脚本或资源型目标，先判断需求属于项目文件汉化还是运行时翻译，再路由到不同 Work。
- 没有授权目标和明确接入证据的引擎只保留为需求候选，不创建空 Adapter。

## 不照搬

- 不把支持对象数量作为 KPI。
- 不把内容类型或中间件名称直接作为 Adapter taxonomy。
- 不创建万能 Hook、万能 Renderer 或按可执行文件名判断引擎的 Core 分支。
- 不把文件改写、实时内嵌、弹窗和 observe-only 混成一个“支持”状态。
- 不复制“无延迟”“所有文本”等无边界承诺；每项支持都记录版本、证据和限制。
- 不在产品 Roadmap 中记录竞品名称、付费信息和宣传材料，只保留可复用的技术结论。

## 对当前 Work 的路由

UIA 安全合同与 observe-only Bundle / catalog 集成现已完成；当前 Next 是 Observation Stream Identity。
本次汇总结论仍进入后续 Engine-aware Integration Stage，不抢占当前工作，也不推动尚无真实目标证据
的引擎 Adapter。离线内容翻译确认需求后另开 Content Adapter Work。
