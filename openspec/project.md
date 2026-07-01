# Project Context

## Purpose
Cargo CfgMate 是基于 Tauri 2 + React + TypeScript 的桌面工具，用于图形化管理 Rust Cargo 配置（`~/.cargo/config.toml`）。核心目标是降低配置门槛、提升下载与构建效率，并确保不破坏用户已有配置。

## Tech Stack
- 前端：React 19、TypeScript、Vite 7、Tailwind CSS
- 桌面与通信：Tauri 2、Tauri API v2（`@tauri-apps/api/core` 的 `invoke`）
- 后端：Rust、Serde、TOML

## Project Conventions

### Code Style
- TypeScript/React 使用 2 空格缩进、双引号、分号
- 组件/文件 PascalCase，工具函数 camelCase
- 使用 `@/` 作为 `src/` 别名，路径分隔符统一使用 `/`
- `src/types.ts` 与 `src-tauri/src/config.rs` 保持结构一致

### Architecture Patterns
- 前端通过 `invoke` 调用 Rust `#[tauri::command]`，统一返回 `Result<T, String>`
- 配置读写集中在 `src-tauri/src/config.rs`，使用 Serde + TOML
- 使用 `#[serde(flatten)]` 保留未知字段，避免覆盖用户自定义配置
- 保存前调用 `cleanEmptyValues` 清理空值

### Testing Strategy
- 暂无专用测试框架
- 主要验证：`npx tsc --noEmit` + `npm run tauri dev`
- 若添加测试：前端 `*.test.tsx`，后端 `#[cfg(test)]` + `cargo test`

### Git Workflow
- 提交信息使用短的 conventional 风格（如 `feat: ...`）
- PR 需包含摘要、测试步骤、UI 截图
- 涉及 `~/.cargo/config.toml` 或 Tauri command 的变更需明确说明

## Domain Context
- 管理 Cargo 配置文件（`~/.cargo/config.toml`）
- 支持镜像源切换、编译参数优化、工具链/目标安装、链接器配置、环境变量与网络代理

## Important Constraints
- 不能破坏用户已有 Cargo 配置，必须保留未知字段
- 前端类型必须与 Rust 结构保持同步
- 需处理跨平台路径与系统命令差异

## External Dependencies
- Cargo / Rustup 环境
- crates.io 或镜像源 registry
- Tauri 运行时与系统命令可用性
