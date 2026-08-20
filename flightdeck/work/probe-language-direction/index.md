# 探针语言方向

Status: Finished

## Goal

让新建探针的临时词典明确配置源语言和目标语言，与 Glyphshift 支持任意语言方向的产品承诺一致；删除
没有真实检测能力支撑的“源语言自动识别”表述，并确保创建合同把两项语言原样保存到 Dictionary。

## Current

临时 Dictionary 创建器现在常显并要求源语言与目标语言，默认 `en-US → zh-CN`；中英文帮助文案分别
说明目标软件当前界面语言与要写入译文的语言，不再声称自动识别。前端状态、TypeScript DTO 与 Rust
request 同时携带 `sourceLocale`/`targetLocale`，后端拒绝 `auto`、空值与非法 locale，并原样写入
Dictionary Metadata。Desktop API 已提升到 v32；复用已有 Dictionary 与 Probe Run 持久模型不变。

同步 Release Review 已用 9-Adapter Runtime 启动，真实 WebView 验证两个字段、默认值和文案均正确；
Review 窗口停在新建探针 Modal，供用户直接检查。

## Next

None

## Progress

- 已用用户截图确认缺陷位于统一 Probe creation Modal 的 Temporary Dictionary 分支。
- 已定位现有 Playwright seam；当前测试明确断言源语言字段数量为 0，可直接转成精确红灯回归。
- Playwright 红灯精确复现两次：中文创建与取消重置均找不到“源语言”。
- 创建请求测试验证 `ja-JP → ko-KR` 原样进入 IPC；取消再打开恢复 `en-US → zh-CN`。
- Desktop Shell 77/77、Probe Playwright 17/17、前端生产构建和定向视觉用例通过；机械检测 0 finding。
- 1440×900、960×640 视觉检查无溢出；同步 Release Runtime 9 个 Adapter 校验通过，真实 WebView
  Playwright 1/1 通过。

## References

- [探针创建来源统一](../probe-creation-sources/index.md)
- [领域语言](../../../CONTEXT.md)
