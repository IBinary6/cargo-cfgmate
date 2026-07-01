# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

Cargo CfgMate 是一个基于 Tauri 2 + React + TypeScript 开发的 Rust Cargo 配置管理工具。提供图形化界面来管理 `~/.cargo/config.toml`，支持下载源镜像切换、编译优化、工具链管理、链接器配置、环境变量和网络设置。

## 技术栈

**前端:**
- React 19 + TypeScript
- Vite 7 (开发服务器和构建工具)
- Tailwind CSS (样式)
- Tauri API v2 (前后端通信)

**后端:**
- Rust (Tauri 2)
- Serde + TOML (配置序列化)
- std::process::Command (系统命令调用)

## 开发命令

```bash
# 安装依赖
npm install

# 开发模式 (启动 Vite dev server + Tauri 窗口)
npm run tauri dev

# 构建前端
npm run build

# 构建 Tauri 应用 (生成安装包)
npm run tauri build

# 前端预览 (仅测试 Vite 构建结果)
npm run preview

# TypeScript 类型检查
npx tsc --noEmit
```

## 核心架构

### 1. 前后端通信架构

采用 Tauri Command 模式，通过 `@tauri-apps/api/core` 的 `invoke` 函数调用 Rust 后端命令：

**Rust 端 (src-tauri/src/lib.rs):**
- 使用 `#[tauri::command]` 宏定义命令
- `tauri::generate_handler![]` 注册所有命令
- 所有命令返回 `Result<T, String>` 用于错误处理

**前端调用示例:**
```typescript
import { invoke } from "@tauri-apps/api/core";

// 获取配置
const config = await invoke<CargoConfig>("get_config");

// 保存配置 (带参数)
await invoke("save_config", { config: newConfig });
```

### 2. 配置管理模块 (src-tauri/src/config.rs)

**核心职责:**
- 定义 `CargoConfig` 及其子结构 (使用 Serde 序列化/反序列化)
- 提供 TOML 配置文件读写功能
- 使用 `#[serde(flatten)]` 和 `HashMap<String, toml::Value>` 保留未知字段

**关键函数:**
- `get_cargo_config_path()`: 跨平台获取 `~/.cargo/config.toml` 路径
- `load_config()`: 读取并解析配置，不存在时返回默认值
- `save_config()`: 序列化并写入配置，自动创建父目录
- `get_current_target()`: 检测当前编译目标平台

**设计亮点:**
- 使用 `#[serde(rename = "replace-with")]` 处理 kebab-case 字段
- `EnvValue` 枚举支持简单字符串或复杂对象
- `#[serde(flatten)]` 保留未知配置段避免丢失数据

### 3. 前端状态管理

**状态分层:**
- **全局状态 (App.tsx):** `config`, `loading`, `saving`, `activeTab`, `toast`
- **Tab 专属状态:** `selectedMirror`, `profileType` 等
- **持久化存储 (src/lib/store.ts):** 使用 `localStorage` 保存 `lastActiveTab`

**配置更新流程:**
1. 用户修改 → 更新本地 `config` state
2. 点击保存 → `cleanEmptyValues()` 清理空值
3. `invoke("save_config")` 写入文件
4. 成功后更新 state 并显示 Toast

### 4. Tab 组件架构 (src/components/tabs/)

**6 个独立 Tab 组件:**
- `RegistryTab`: 下载源镜像切换 (使用 `MIRRORS` 配置)
- `BuildTab`: 编译优化 (jobs、rustc-wrapper、profile 配置)
- `ToolchainTab`: 工具链管理 (检测/安装 rustup targets)
- `LinkerTab`: 链接器配置 (target-specific linker 设置)
- `EnvTab`: 环境变量管理 (支持简单字符串和对象形式)
- `NetworkTab`: 网络设置 (HTTP/HTTPS 代理、offline 模式)

**通用模式:**
```tsx
interface TabProps {
  config: CargoConfig;
  setConfig: (config: CargoConfig) => void;
  showToast?: (msg: string, type?: "success" | "error") => void;
  // Tab 特定 props...
}
```

### 5. 类型系统 (src/types.ts)

**TypeScript 类型与 Rust 结构一一对应:**
- 使用 `Record<string, T>` 对应 Rust 的 `HashMap<String, T>`
- `[key: string]: any` 索引签名对应 `#[serde(flatten)]`
- 导出常量数组 (`TARGET_PLATFORMS`, `LINKER_OPTIONS` 等) 用于下拉菜单

### 6. 系统命令调用模式

**跨平台命令执行 (lib.rs):**
```rust
// Windows 使用 explorer/where, Unix 使用 open/which
#[cfg(target_os = "windows")]
Command::new("explorer").arg(path).spawn()?;

#[cfg(not(target_os = "windows"))]
Command::new("open").arg(path).spawn()?;
```

**异步命令 (install_sccache, install_target):**
- 使用 `async fn` + `Command::output()` 等待完成
- 检查 `output.status.success()` 判断成功/失败
- 返回 `Result<(), String>` 统一错误处理

## 关键实现细节

### 镜像切换逻辑 (src/lib/mirrors.ts + RegistryTab)

1. 用户选择镜像 → 更新 `selectedMirror` state
2. 保存时在 `saveConfig()` 中动态生成 `source` 配置：
   ```typescript
   newSource["crates-io"] = { "replace-with": mirror.replaceWith };
   newSource[mirror.replaceWith] = { registry: mirror.registry };
   ```
3. 选择 "official" 时删除 `crates-io` 配置恢复默认

### 空值清理算法 (App.tsx cleanEmptyValues)

**递归清理策略:**
- 删除 `null`/`undefined`/`""` 值
- 清空数组删除
- 空对象删除
- 避免向 TOML 写入无意义字段

### Profile 配置动态路径

Profile 配置使用动态键 (`profile.dev` / `profile.release`)，通过计算属性更新：
```typescript
setConfig({
  ...config,
  profile: {
    ...config.profile,
    [profileType]: { /* 新配置 */ }
  }
});
```

## 文件路径约定

- 别名 `@/` → `src/` (配置于 vite.config.ts 和 tsconfig.json)
- 使用 `/` 而非 `\` 作为路径分隔符 (兼容 Windows 和 Unix)
- Cargo 配置路径: `~/.cargo/config.toml` (Windows: `%USERPROFILE%\.cargo\config.toml`)

## Tauri 配置要点 (src-tauri/tauri.conf.json)

- **开发 URL:** `http://localhost:1420` (与 Vite 端口一致)
- **构建输出:** `../dist` (相对于 src-tauri 目录)
- **窗口尺寸:** 800x600
- **CSP:** 设为 `null` (允许所有资源加载)

## 注意事项

1. **TOML 序列化:** 使用 `toml::to_string_pretty()` 保持可读性
2. **错误处理:** 所有 Tauri command 必须返回 `Result<T, String>`
3. **类型安全:** 前端 TypeScript 类型与 Rust 结构保持同步
4. **状态清理:** 保存前调用 `cleanEmptyValues()` 避免污染配置文件
5. **平台检测:** 使用 Rust `cfg!()` 宏和编译时条件编译处理跨平台差异
