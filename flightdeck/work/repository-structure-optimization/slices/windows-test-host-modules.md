# Windows test-support Host module

Status: Complete

## Goal

保持第一方 GDI/GDI+/Direct2D/DirectWrite 像素证据与 UIA 合成控件命令协议不变，把 1239 行 Windows-only
测试 Host 按 Rendering acceptance 与 UIA server 两种生命周期拆开。

## Baseline

- `test-support/windows-host/src/lib.rs` 1239 行，全部生产实现包在一个 `cfg(windows)` module 内。
- 5-993 行共享 COM/DIB/字体与四类绘制技术的像素证据；995-1228 行是独立 UIA 标准控件进程服务器。
- crate 只供确定性/授权测试使用；没有 tracked 测试数据或本机路径/PID。

## Plan

- [x] 提取 Rendering acceptance module，保留共享 COM/DIB 和公开绘制函数。
- [x] 提取 UIA synthetic server，保留 line-oriented command/keepalive/privacy 行为。
- [x] Windows module 与 crate 根保持原公开 re-export 集合。
- [x] 运行 package 编译/直接下游、Clippy、格式、架构和 diff。
- [x] Standards/Spec 自审公开 surface、unsafe seam、像素证据和 UIA 命令协议。

## Decisions

- GDI/GDI+/Direct2D/DirectWrite 暂留同一 Rendering module：它们共享 DIB、COM、尺寸与 PixelEvidence。
- UIA server 拥有窗口/控件/命令循环，与像素渲染没有共享状态，是独立深 seam。

## Result

- crate 根从 1239 行降至 7 行；Windows facade 12 行、Rendering 977 行、UIA server 233 行。
- 实现按原源码区间搬迁并只接受 `rustfmt` 的 import/闭包排版调整；公开 item 集合保持一致，没有扩大
  `unsafe` surface 或改变资源释放顺序。
- `glyphshift-windows-host` 的 2 项可见合同和三个直接依赖 package 回归通过；其中本机 Qt 合同仍按原条件
  ignored，没有运行或记录真实软件路径。
- package Clippy、workspace 格式、架构检查和 diff check 通过；新文件未包含本机路径、PID 或证据引用。

## Review

- Standards：Rendering 独占 COM/DIB/像素资源，UIA 独占窗口线程和命令循环；facade 仅负责 Windows 条件编译
  与稳定 re-export，模块边界对应真实生命周期。
- Spec：所有公开绘制入口、`PixelEvidence` 与 UIA server 入口均保留；像素证据、密码控件隐私、keepalive、
  provider block/unblock 和退出协议没有行为改动。
