# Cargo CfgMate (Rust 配置助手)

<div align="center">
  <img src="./show/icon.png" width="128" height="128" />
  <h1>Cargo CfgMate</h1>
  <p>
    一个现代化的、跨平台的图形界面工具，用于轻松管理 Cargo 配置 (<code>~/.cargo/config.toml</code>)。
  </p>
  <p>
    <b>🚀 开源 | ⚡ 高效 | 🎨 美观</b>
  </p>
</div>


---

## ✨ 核心功能

- **📦 镜像源管理**: 一键切换 crates.io 国内镜像源（中科大、清华、字节跳动、阿里云等），从此告别下载卡顿。
- **⚡ 编译与工具链**:
  - 智能检测并安装 `sccache` 编译缓存工具。
  - 一键管理 Rustup Targets（交叉编译目标）。
  - 快速配置 `build.jobs` 数以优化 CPU 使用。
- **🔗 链接器优化**:
  - 快速切换 `lld-link` 或 `mold` 以提速 2-5 倍。
  - 常用 `rustflags`（如静态链接 CRT、去除符号表）一键开关，无需记忆复杂参数。
  - 支持自定义链接器路径及校验。
- **🔨 环境变量**: 图形化管理 `[env]` 变量，支持 `force` 和 `relative` 选项。
- **🌐 网络代理**: 内置 Clash、V2Ray 等常用代理端口预设，亦可自定义 HTTP/HTTPS 代理。

## 📸 界面预览

![](./show/1.png)
![](./show/2.png)
![](./show/3.png)
![](./show/4.png)
![](./show/5.png)
![](./show/6.png)
![](./show/7.png)
![](./show/8.png)
- **Dashboard**: 清晰的侧边栏导航。
- **工具链检测**: 自动检测环境缺失工具并提供一键安装。
- **暗黑模式**: 精心设计的深色主题，专业且护眼。

## 🛠️ 安装与运行

### 方式一：下载发布版 (Windows)

1. 前往 [Releases](https://github.com/yourusername/cargo-cfgmate/releases) 页面下载最新版安装包或可执行文件。
2. 双击运行即可。

### 方式二：从源码编译

确保已安装 Rust 和 Node.js。

```bash
# 1. 克隆仓库
git clone https://github.com/yourusername/cargo-cfgmate.git
cd cargo-cfgmate

# 2. 安装前端依赖
npm install

# 3. 开发模式运行
npm run tauri dev

# 4. 编译 Release 版本
npm run tauri build
# 构建产物位于 src-tauri/target/release/ (或对应 target 子目录)
```

## 📝 配置说明

程序会自动读取和修改用户全局配置文件：

- Windows: `%USERPROFILE%\.cargo\config.toml`
- Linux/macOS: `~/.cargo/config.toml`

> 修改前建议备份您的配置文件。虽然本工具已做好了兼容处理，但安全第一。

## 🤝 贡献

欢迎提交 Issue 或 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 提交 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 
