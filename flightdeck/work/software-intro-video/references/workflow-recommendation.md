# 无需手动录屏的软件介绍视频工作流

核查日期：2026-09-06。范围为官方文档与技能源码核查，未安装候选工具、未实际渲染样片。

## 推荐结论

Glyphshift 首支介绍片建议以真实界面截图为主体，由代码生成镜头运动、标注、字幕和转场。
这不需要用户录屏；若镜头必须展示连续实际操作，再由代理自动采集短段素材。界面重建动画应
忠于产品，AE 翻译效果用真实前后截图，不用生成式视频模型臆造按钮或翻译结果。

按需求选择一个制作引擎：

| 选择 | 能力与适配判断 | 边界 |
| --- | --- | --- |
| HyperFrames 官方 `product-launch-video` | 从产品简介或 URL 进入素材、品牌、脚本分镜、声音、动画、验证与 MP4 导出；适合希望一套流程覆盖全片的用户 | 网页捕获不涵盖 AE 原生窗口；媒体来源和配音提供方需要配置 |
| Remotion 官方 skills + `video-shotcraft` | 官方 skills 负责 React 视频制作规则，社区工作流负责镜头、节奏和音效；适合强调软件界面特写和宣传片观感 | 社区工作流依赖需适配；不能把镜头库等同于自动采集所有桌面软件 |
| `saas-product-demo-video` | 偏短促的音乐卡点产品 demo，可用作分镜和节奏参考 | 更偏 SaaS 场景，Windows 脚本适配成本需评估 |

前三项能力依据分别见 [HyperFrames 工作流源码](https://github.com/heygen-com/hyperframes/blob/main/skills/product-launch-video/SKILL.md)、
[Remotion 官方技能](https://www.remotion.dev/docs/ai/skills)、
[video-shotcraft 官方仓库](https://github.com/Vincentwei1021/video-shotcraft)、
[saas-product-demo-video 官方仓库](https://github.com/noamdorr/saas-product-demo-video)。
上述适配排序是面向本项目的判断，不是性能或成片质量实测排名。

## HyperFrames 核查

- 当前官方入口是 `hyperframes`，按意图路由到 `product-launch-video` 等专门工作流；安装文档
  推荐从 Core Skills 开始，不必安装仓库所有技能。[技能安装与路由](https://hyperframes.heygen.com/guides/skills)
- 产品工作流接受 URL、脚本或简介，并提供无站点素材入口；真实展示类镜头优先使用真实截图，
  允许在截图上叠加需要移动的元素。[工作流源码](https://github.com/heygen-com/hyperframes/blob/main/skills/product-launch-video/SKILL.md)
- 引擎将 HTML/CSS/JS 组合渲染为视频；本地渲染使用 Chromium 与 FFmpeg，另有 Docker 路线。
  官方存在 Windows 渲染 CI，但不能据此承诺全部可选媒体工具在 Windows 上都已验证。
  [渲染文档](https://hyperframes.heygen.com/guides/rendering)、
  [Windows CI](https://github.com/heygen-com/hyperframes/blob/main/.github/workflows/windows-render.yml)
- 默认媒体集成倾向 HeyGen 服务，语音另有本地 Kokoro 路线；离线模式不保证所有素材能力都能
  获取新素材。应在确定旁白语言后先试听，不能把框架开源等同于所有媒体服务无限免费。
  [媒体提供方配置](https://github.com/heygen-com/hyperframes/blob/main/skills/media-use/references/setup-providers.md)
- 框架声明 Apache 2.0；外部音乐、字体、图片、语音服务应各自核对使用条件。
  [框架许可证](https://github.com/heygen-com/hyperframes/blob/main/LICENSE)

## Remotion 与辅助技能

- 官方技能包含 `remotion-best-practices`、`remotion-create`、`remotion-markup`、
  `remotion-studio`、`remotion-render` 和 `remotion-captions` 等，支持 Codex。
  它解决实现、预览和渲染，不应把它本身说成完整产品营销策划。
  [官方技能目录](https://www.remotion.dev/docs/ai/skills)
- Remotion 当前许可允许个人及符合小团队条件的组织免费制作视频；不应默认任意规模公司免费。
  [官方许可](https://github.com/remotion-dev/remotion/blob/main/LICENSE.md)
- OpenAI curated 目录有 `speech`，其技能说明覆盖产品演示旁白，需要配置 API key；它是
  配音辅助，不是视频制作引擎。本轮未安装、未调用。
  [speech 技能源码](https://github.com/openai/skills/blob/main/skills/.curated/speech/SKILL.md)
- 社区镜头技能的细节和 Windows 边界见[社区技能核查](community-skills.md)。

## 建议制作流程（项目方案）

1. 用 README、产品文档和现有设计 token 写 45–60 秒分镜，先把信息压到一个核心价值与三个步骤。
2. 用仓库 Playwright 自动采集 Glyphshift 的真实界面，使用合成演示数据；另采集 AE 同一面板
   翻译前后两个状态。素材只包含该镜头需要的区域。
3. 用所选引擎制作局部放大、平移、光标示意、重点标注与前后对比；真实操作与动画示意不混淆。
4. 第一版可只做字幕和音乐；要旁白则先生成并试听中文音频，再据音频时长排时间轴。
5. 先预览低成本样片，校验字体、字幕阅读时间、画面清晰度、实际产品能力及素材隐私，再输出
   横屏 MP4；竖屏版单独重排镜头，避免直接裁掉关键界面。

建议分镜：0–6 秒英文效果面板提出问题；6–15 秒展示真实翻译前后对比；15–35 秒呈现
“探针收集 → AI 翻译 → 实时预览”；35–48 秒展示词典与工作流复用；48–60 秒收尾到项目入口。
这是一份制作建议，不表示已有视频或所有环节已完成自动化。

## 可查看的样片与安装入口

- [video-shotcraft 镜头样片库](https://vincentwei1021.github.io/video-shotcraft/)
- [HyperFrames 官方站点](https://hyperframes.heygen.com/)
- HyperFrames 官方安装入口：`npx skills add heygen-com/hyperframes`，按文档选择 Core Skills。
- Remotion 官方安装入口：`npx skills add remotion-dev/skills`。
- video-shotcraft 官方安装入口：`npx skills add Vincentwei1021/video-shotcraft`。

安装命令仅记录供选择后使用，本轮没有执行。不要直接把两套引擎的全部技能一起装入产品仓库。
