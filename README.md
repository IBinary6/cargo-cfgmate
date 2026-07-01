# Cargo CfgMate

[![Release](https://img.shields.io/github/v/release/IBinary6/cargo-cfgmate?sort=semver)](https://github.com/IBinary6/cargo-cfgmate/releases/latest)
[![Platform](https://img.shields.io/badge/platform-Windows-blue)](#安装)
[![License](https://img.shields.io/github/license/IBinary6/cargo-cfgmate)](LICENSE)

Cargo CfgMate 是一个用于管理 Rust Cargo 配置的桌面工具。它提供图形界面来编辑
`~/.cargo/config.toml`，并集中管理 Cargo 镜像源、rustup 下载源、编译参数、链接器、
环境变量、网络代理、命令别名和缓存清理。

当前正式发布版支持 Windows x64 / Win32。项目基于 Tauri，后续可以扩展到 macOS 和
Linux，但这些平台尚未作为正式 Release 验证。

## 安装

### Windows 一行安装

在 PowerShell 中运行：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/IBinary6/cargo-cfgmate/master/install.ps1 | iex"
```

安装脚本会自动识别 `windows-x64` / `windows-win32`，下载最新 Release，安装到：

```text
%LOCALAPPDATA%\Programs\CargoCfgMate
```

脚本会创建开始菜单和桌面快捷方式，并默认启动程序。程序本身需要管理员权限，启动时
Windows 会弹出 UAC 确认。

### 手动下载

也可以从 [Releases](https://github.com/IBinary6/cargo-cfgmate/releases/latest) 下载：

- `cargo-cfgmate-<version>-for-windows-x64.zip`
- `cargo-cfgmate-<version>-for-windows-win32.zip`

解压后运行 `cargo-cfgmate.exe`。

## 写入边界

Cargo CfgMate 会读取用户的 Cargo 配置文件：

```text
%USERPROFILE%\.cargo\config.toml
```

为避免意外修改用户环境，当前行为约束如下：

- 启动和刷新只读取配置，不会创建或修改 `config.toml`。
- 只有用户点击“保存配置”时，才会写入 Cargo 配置。
- 如果用户点击保存时配置文件或父目录不存在，程序会创建并写入。
- rustup 下载源、缓存清理、target 安装、sccache 安装等操作只会在用户主动点击对应按钮后执行。
- 对已有配置文件执行保存时，程序会先询问是否创建备份。

## 功能

- Cargo 镜像源：切换 crates.io 国内镜像源，支持自定义 source。
- rustup 下载源：配置 `RUSTUP_DIST_SERVER` 和 `RUSTUP_UPDATE_ROOT`。
- 编译优化：配置 `build.jobs`、`rustc-wrapper`、profile 参数和常用 rustflags。
- 工具链管理：查看并安装 Rust targets。
- 链接器配置：为不同 target 配置 linker、runner 和 rustflags。
- 环境变量：编辑 Cargo `[env]` 配置，支持 `force` 和 `relative`。
- 网络设置：配置 HTTP/HTTPS 代理、offline 模式等。
- 备份恢复：管理 Cargo 配置备份、导入和导出配置。
- 缓存工具：查看并清理 Cargo registry/git 缓存。

## 界面预览

![源与镜像](show/1.png)
![编译优化](show/2.png)
![工具链](show/3.png)
![链接器](show/4.png)
![环境变量](show/5.png)
![网络设置](show/6.png)
![备份恢复](show/7.png)
![常用工具](show/8.png)

## 从源码构建

需要安装 Rust、Node.js 和 Windows 构建工具。

```bash
git clone https://github.com/IBinary6/cargo-cfgmate.git
cd cargo-cfgmate
npm install
npm run tauri dev
```

构建 Release：

```bash
npm run tauri build
```

生成便携 exe（不生成安装器）：

```bash
npm run tauri build -- --target x86_64-pc-windows-msvc --no-bundle
```

## 已知限制

- 当前正式 Release 只覆盖 Windows。
- 程序未做代码签名，Windows 可能提示“未知发布者”。
- Tauri 需要 Microsoft Edge WebView2 Runtime；现代 Windows 通常已预装。

## 许可证

本项目使用 [MIT License](LICENSE)。
