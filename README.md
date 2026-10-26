# Kairomo

**Time for what matters.**

Kairomo 是本地优先的时间管理桌面工具。名字源自希腊神话 Kairos「恰当的时机」，目前实现专注、休息和真实时长记录。首版面向 macOS；Windows 构建流程已准备，待 Windows 验证。

- 到达目标时长只提醒，不自动结束专注。50 分钟后再学 12 分钟，保存为同一段 62 分钟。
- 手动开始休息、继续专注或结束本轮；暂停时间不计入有效时长。
- 菜单栏时间入口，点击展开极简计时浮窗；记录与设置在独立窗口打开。收起后继续计时，退出时暂停保存。
- SQLite 本机存储；异常恢复、记录修正、删除和 CSV 导出。
- 中文、英文界面；不需要账户，不连接云服务。

## 日常启动（macOS 本地版本）

已经构建的应用位于本仓库 `target/debug/bundle/macos/Kairomo.app`。在访达进入该目录，双击 `Kairomo.app` 即可启动；也可以在仓库根目录执行：

```bash
open target/debug/bundle/macos/Kairomo.app
```

启动后，从顶部菜单栏的图标或时间打开计时小窗。日常使用不需要运行开发服务或保持终端打开。当前为本地调试应用，尚未提供正式安装器或自动更新。

更新构建后，从菜单栏右键菜单选择“退出 Kairomo / Quit”，再打开上述应用，即可加载新版；不需要重启 Mac。仅收起小窗不会退出应用。正常退出会暂停并保存当前计时，重新打开后手动继续；已保存的历史记录保留。

若应用包尚不存在，先按下方开发环境要求准备依赖，再在仓库根目录执行 `bash scripts/dev.sh build`。`npm run build` 只构建前端，不生成桌面应用包。

## 开发

依赖 Node.js 22.13+、Rust stable（含 rustfmt / clippy）。macOS 需要 Xcode 工具；Windows 需要 MSVC C++ 构建工具与 WebView2。遵循 [Tauri 官方前置要求](https://v2.tauri.app/start/prerequisites/)。

```bash
npm ci
npm run tauri -- dev
```

仓库内也提供 `bash scripts/dev.sh start|stop|restart|status|logs|list|build|help`。脚本仅管理本项目。启动开发模式前先退出日常应用，避免重复实例。退出开发模式使用脚本 stop；日常应用退出使用菜单栏的 Quit。`status` 只检查脚本管理的开发进程，不代表日常应用是否正在运行。当前工作区外层脚本尚未接入 Kairomo，请进入本仓库后运行这些命令。

```bash
npm run check
npm run format:check
npm run build
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm run tauri -- build --debug --bundles app
```

最后一条为 macOS 本地调试应用构建，不是签名发行版。Windows 可用 `npm run tauri -- build --debug --no-bundle` 检查应用构建。正式安装包、签名、公证和发布另行配置，仓库没有启用自动发布。

## 结构与规范

- [项目规则](AGENTS.md)
- [架构与包边界](docs/architecture.md)
- [计时与恢复规则](docs/timing-rules.md)
- [开发与平台验证](docs/development.md)

仓库为一个产品的独立 Git 仓库；三个 Rust 包由 Cargo Workspace 统一管理，前端按功能模块拆分。它不引用 Jacory Space 中其他产品的运行时或源码。

版本 0.1.0。当前未指定开源许可证；连接 GitHub 仓库不等于授权发布。
