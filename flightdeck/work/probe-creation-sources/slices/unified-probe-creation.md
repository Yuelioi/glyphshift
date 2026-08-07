# 统一 Probe creation request

Status: Complete

## Outcome

Desktop 暴露一个原子 Probe creation command。调用者分别选择 Target Source 与 Dictionary Source；
Desktop 解析或创建实际 Software/Dictionary、计算兼容 Adapter Plan、启动 Probe，并以显式 ownership
决定是否保留临时会话。GUI 只负责表达用户选择，不编排半完成资产。

## Request shape

```text
ProbeCreationRequest
├─ name: optional user override
├─ target
│  ├─ LibrarySoftware { software_id }
│  └─ ActiveProcess { executable_path }
├─ dictionary
│  ├─ LibraryDictionary { dictionary_id }
│  └─ TemporaryDictionary { target_locale }
├─ adapter_ids: optional explicit plan
└─ live_preview_enabled
```

`ActiveProcess` 的路径来自同一轮前台捕获/preflight，不是自由文本绕过授权。缺少显式 Adapter Plan 时，
Desktop 使用当前目标的兼容观察 Adapter；指定计划时仍由 Desktop 复核兼容性。

## Acceptance

- 四种 Target/Dictionary 来源组合均能在单一 command 中成功或完整补偿。
- Library 引用永不被 ownership cleanup 删除；Temporary 资产不靠名称或 ID 推断。
- 临时 Dictionary 使用系统生成名称、`auto` 源语言和请求目标语言。
- 两项均为 Library 引用时返回普通 Probe；其他组合返回可保留/清理的临时 Probe。
- 前端只保留一个新建入口，默认流程不要求手工输入 Software 或 Dictionary 名称。

## Result

- Desktop API `/24` 以 `desktop_create_probe_from_sources` 接收独立 Target/Dictionary 来源，旧的
  quick-test 创建命令和 payload 不保留兼容分支。
- 四种来源组合均由同一命令解析；只有实际新建的 Software/Dictionary 进入 ownership ledger。
- Probe 页只保留一个新建入口；资料库为空时默认当前程序与临时词典，已有资料时默认直接复用。
- 定向 Shell 合同与 Probe Playwright 固定原子创建、临时清理和取消重开表单边界。
