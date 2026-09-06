# 软件介绍视频：社区 Skill 核查

核查日期：2026-09-06。范围：作者仓库 README、SKILL 与直接关联源码；未安装、未执行第三方脚本，也未在 Windows 渲染验证。

## 结论

针对 Glyphshift，优先考虑 **video-shotcraft + 真实产品截图 + Remotion 渲染**。用户无需录屏：Agent 可准备截图，再由代码完成缩放、转场、光标和字幕。但截图合成的宣传片不能作为运行过程或性能的实录证明。以下为适配判断，而非已完成的本机可用性测试。

| 候选 | 推荐场景 | 输入与能力 | 本项目的主要适配点 |
| --- | --- | --- | --- |
| video-shotcraft | 约半分钟、有真实界面的产品宣传片 | 产品项目/网页或截图、品牌与文案；镜头卡、分镜、声音设计、渲染与终检 | 推荐主流程；网页采集替换成仓库 Playwright，AE 原生窗口需另备真实截图 |
| saas-product-demo-video | 20–45 秒、音乐卡点的 SaaS 短片 | Logo、颜色、字体、文案、音乐；可用截图或示意 UI，配音可选 | 备选；输入问卷和节奏分析完整，但偏音乐驱动，中文字体与 Windows Shell 需适配 |
| video-talkcraft | 以中文讲解为主的软件介绍 | 一份与配音一致的口播稿及成品配音；字幕与动效按人声对齐 | 条件候选；TTS 须另外提供，不是文字直接包办配音的一站式工具 |

## video-shotcraft

作者明确列出 Codex 用法；有镜头配方卡、参考 TSX、完整 Ink Press 模板和在线动态画廊。价值是把“展示什么、如何运镜、音效如何落点、如何审片”编成工作流，超出单纯 Remotion API 提示。支持套模板、自主自由创作及共同创作；Glyphshift 更适合保留自身视觉的自主创作，不必照搬纸墨模板。[README](https://github.com/Vincentwei1021/video-shotcraft/blob/main/README_CN.md)、[SKILL](https://github.com/Vincentwei1021/video-shotcraft/blob/main/SKILL.md)、[动态样片](https://vincentwei1021.github.io/video-shotcraft/)

采集源码依赖 Puppeteer：连接网页、生成 2 倍截图、元素 PNG 和坐标 JSON；它没有提供原生 AE 自动化。适配建议：Glyphshift 网页表面使用仓库 Playwright；AE 翻译前后使用真实且经过隐私检查的截图，在视频里做对比。这样不要求用户录制完整操作过程。[采集脚本](https://github.com/Vincentwei1021/video-shotcraft/blob/main/assets/scripts/capture-template.mjs)

模板依赖 Node 生态的 React、Remotion 与 TypeScript。流程另用 FFmpeg 抽帧；强节奏分析使用 Python/librosa；部分三维组件需要 Three.js 相关依赖。不能因声明支持 Codex 就推定全部 Windows 路径均验证。剪映导出文档明确 macOS 已测、Windows 未验证；第一轮交付应以 MP4 和 Remotion 工程为准。[模板依赖](https://github.com/Vincentwei1021/video-shotcraft/blob/main/template/package.json)、[制作流水线](https://github.com/Vincentwei1021/video-shotcraft/blob/main/references/pipeline.md)、[剪映导出](https://github.com/Vincentwei1021/video-shotcraft/blob/main/references/jianying-export.md)

## saas-product-demo-video

作者说明已用于 Claude Code，其他 Agent 可直接读取 SKILL。Node 18+、Python 3、Remotion 4.x、librosa 是主要依赖；Windows 的 Bash 脚本需 Git Bash 或 WSL。Super Powers 有降级路径；参考视频分析使用 Gemini CLI/API 或手动 AI Studio，不是 MP4 渲染本身的硬依赖。[README](https://github.com/noamdorr/saas-product-demo-video)

优势是品牌素材问卷、脚本结构、节拍识别、光标定位和逐场景渲染。其默认节奏以音乐为主；SKILL 明确旁白主导时应改为帧预算。示意 UI 是允许路线，但 Glyphshift 有现成产品，不宜用它虚构 AE 翻译效果。字体加载示例偏拉丁字集，中文需要单独落实字体覆盖。[SKILL](https://github.com/noamdorr/saas-product-demo-video/blob/main/saas-product-demo-video/SKILL.md)

## video-talkcraft：配音优先时再选

同作者的讲解视频工作流，明确将配音和人物素材视为输入，不提供 TTS 或数字人生成。它用 CPU 语音对齐，驱动字幕与镜头，支持截图与纯动效；更适合“为什么需要它、怎么使用”的讲解，而不是音乐主导短片。默认对齐方案需 Python 包及约 767 MB 模型，存在更轻的替代后端；没有在本任务里验证其 Windows 全链路。[SKILL](https://github.com/Vincentwei1021/video-talkcraft/blob/main/SKILL.md)

## 落地边界

- 推荐首片只证明一个价值：AE 英文效果参数 → Glyphshift 翻译配置/操作 → 真实中文结果。避免把仍未验证的覆盖率或速度写入文案。
- 原始机器截图与生成证据留在本地测试区；只有完成隐私检查且获用户认可的发布素材才进入仓库允许的发布目录。
- 第三方安装命令和工作流只是研究对象，没有执行。正式采用时需把 Bash、Puppeteer 和产物路径约定适配为本仓库规则。
- 项目代码许可、音频素材许可和 Remotion 许可是不同层次；不能仅凭仓库开源许可推断所有素材均无条件使用。[音频许可清单](https://github.com/Vincentwei1021/video-shotcraft/blob/main/assets/audio/ATTRIBUTION.md)
