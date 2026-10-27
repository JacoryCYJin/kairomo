# Kairomo

**Time for what matters.**

Kairomo 是本地优先的时间管理桌面工具。名字源自希腊神话 Kairos「恰当的时机」，目前实现专注、休息和真实时长记录。首版面向 macOS；Windows 构建流程已准备，待 Windows 验证。

- 专注默认手动结束：到点后可以继续计时，50 分钟后再学 12 分钟保存为同一段 62 分钟；也可选择到点结束并切到休息待开始。休息到点固定结束，所有阶段切换后都需手动点击开始。
- 手动开始休息、继续专注或结束本轮；暂停时间不计入有效时长。
- 菜单栏时间入口，点击展开极简计时浮窗；记录与设置在独立窗口打开。收起后继续计时，退出时暂停保存。
- SQLite 本机存储；异常恢复、记录修正、删除和 CSV 导出。
- 中文、英文界面；无需账户，专注数据仅保存在本机；检查更新只读取公开版本信息。

## 日常启动（macOS Beta）

当前本地版本为 **0.1.0-beta.2**。安装到“应用程序”后，双击 **Kairomo Beta** 即可使用；也可以执行：

```bash
open '/Applications/Kairomo Beta.app'
```

从顶部菜单栏打开计时小窗，不需要保持终端或开发服务运行。设置页显示版本与“检查更新”；发现新版后，明确选择“下载并安装”才会安装。安装前暂停并保存当前计时、备份数据，重开后手动继续。

Beta 沿用已有日常数据目录，版本升级不创建新数据库。Kairomo Dev 使用独立身份和数据库，不读写日常记录。仅收起小窗不会退出；退出使用菜单栏右键菜单的 Quit。

`npm run build:beta` 生成本地 app、DMG、签名更新包及发布清单；不安装、不上传、不创建 GitHub Release。Beta 安装包与签名更新清单通过 GitHub prerelease 发布。构建、签名与更新边界见 [Beta 打包与更新](docs/releases.md)。

## 开发

依赖 Node.js 22.13+、Rust stable（含 rustfmt / clippy）。macOS 需要 Xcode 工具；Windows 需要 MSVC C++ 构建工具与 WebView2。遵循 [Tauri 官方前置要求](https://v2.tauri.app/start/prerequisites/)。

```bash
npm ci
npm run tauri -- dev
```

仓库内也提供 `bash scripts/dev.sh start|stop|restart|status|logs|list|build|beta|help`。脚本仅管理本项目。开发模式显示为 Kairomo Dev，使用独立数据，可与日常 Beta 并存。退出开发模式使用脚本 stop；日常应用退出使用菜单栏的 Quit。`status` 只检查脚本管理的开发进程，不代表日常应用是否正在运行。当前工作区外层脚本尚未接入 Kairomo，请进入本仓库后运行这些命令。

```bash
npm run check
npm run format:check
npm run build
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm run tauri -- build --debug --bundles app
```

最后一条生成 Kairomo Dev 调试应用。Beta 打包使用 `npm run build:beta`。Windows 可用 `npm run tauri -- build --debug --no-bundle` 检查应用构建。macOS 本地 Beta 使用 ad-hoc 签名；对外分发所需的 Developer ID 与公证另行配置。仓库没有启用自动发布。

## 结构与规范

- [项目规则](AGENTS.md)
- [架构与包边界](docs/architecture.md)
- [计时与恢复规则](docs/timing-rules.md)
- [开发与平台验证](docs/development.md)
- [Beta 打包与更新](docs/releases.md)

仓库为一个产品的独立 Git 仓库；三个 Rust 包由 Cargo Workspace 统一管理，前端按功能模块拆分。它不引用 Jacory Space 中其他产品的运行时或源码。

版本 0.1.0-beta.2。当前未指定开源许可证；连接 GitHub 仓库不等于授权发布。
