# Kairomo 开发规则

## 产品与范围

Kairomo 是本地优先的时间与任务管理桌面工具，首版 macOS，预留 Windows。目标时长只是提醒，不自动终止专注。界面维护 zh-CN / zh-TW / en-US / ja-JP，不复制 site 的视觉与业务代码。

## 架构边界

- src/features/{timer,history,settings} 按功能组织 Vue 组件；src/app 只组装应用。
- src/shared/ipc 是前端唯一 Tauri 通信入口，组件不得直接 invoke、访问数据库或重新实现权威计时。
- crates/timer-core 为纯 Rust 计时状态、记录和接口，不依赖 Tauri、SQLite 或平台代码。
- crates/timer-storage 实现核心定义的持久化接口，负责 SQLite 与编号迁移，不操作窗口。
- src-tauri 为桌面包：commands 参数与错误转换，runtime 调度和保存，platform 系统差异。lib.rs 只组装。
- 依赖方向：desktop → core / storage，storage → core；core 不反向依赖适配层。
- 核心状态是唯一事实来源，窗口只读取快照并提交操作。时间按有效区间计算，不按界面刷新次数累加。
- Rust 包边界由 Cargo 检查；前端导入边界由 scripts/check-architecture.mjs 检查。避免无归属的大型 utils/services 文件。

## 主题套件设计

涉及主题设计、主题设置或主题展示开发时，先阅读 [主题套件设计规范](docs/theme-kit.md)，按其中的五个设计区域、固定边界和验收要求工作；咖啡具体实现另见 [展示系列](docs/display-series.md)。不要把咖啡的罐子、堆积或液体视为其他主题的通用要求。

## 前端样式

- 使用 Tailwind CSS 3.4 + PostCSS，配置由本仓库独立维护；不导入 site 的配置、样式或字体资源。
- 布局、间距、颜色、圆角和排版优先写在 Vue 模板的工具类中，不为一次性布局创建包装 CSS 类。
- 颜色通过 tailwind.config.js 映射 src/shared/styles/tokens.css 的语义变量；浅色和深色继续跟随系统。
- 普通字号使用 text-xs、text-sm、text-base 等命名层级，不使用任意像素字号；计时大数字和正号使用集中定义的 text-timer、text-timer-sign。
- 共用按钮、对话框和提示条样式放在 shared/styles/base.css 的 components 层；只有确实无法清晰表达的特殊效果保留少量 CSS。不要把页面样式整段搬成 @apply。
- 动态状态使用完整、可扫描的类名，不拼接类名字符串；图表数值高度继续使用动态 style。
- 图标使用现有 Lucide SVG 组件；菜单栏品牌标识以 src-tauri/icons/tray.svg 为源，生成透明 tray.png。应用安装图标是独立资源。
- 不因迁移样式改变窗口拖动、键盘焦点、计时规则或历史数据；验证 360×280 计时窗口和 520／620 像素记录、设置窗口。

## 数据与运行

- SQLite 位于系统应用数据目录。源码不保存用户记录、数据库、导出文件、日志或签名凭据。
- 数据变更采用 migrations 编号迁移；写入失败不得向用户报告成功。
- 计时、暂停、休息、睡眠、恢复规则见 docs/timing-rules.md。睡眠暂停，唤醒手动恢复；锁屏不自动暂停。
- 启动前检查当前进程和端口 1420，不启停其他项目服务。

## 验证

- npm run check（类型、lint、架构）
- npm run build
- cargo fmt --all -- --check
- cargo test --workspace
- cargo clippy --workspace --all-targets -- -D warnings
- npm run tauri -- build --debug --bundles app（macOS）
- Windows 需 Windows runner 或真机验证，不把 macOS 检查当作跨平台验收。

## 协作与 Git

继承上层 AGENTS.md 和 dev-main / git-rule。按已批准范围开发，新增范围先说明。使用 feature/、fix/、chore/、docs/ 分支。一行中文 Conventional Commit。未获明确仓库提交口令不得 commit/push/PR/merge。版本 0.1.0，更新版本不等于发布。

以下为发给 Agent 的协作口令，不是终端命令；仅授权 `kairomo/` 仓库，不包括外层工作区规则或其他项目：

- `kairomo push`：完成本项目相关验证，提交已确认任务的改动并推送任务分支；不创建 PR、不合并。
- `kairomo pr`：完成验证、提交、推送，创建或更新对应 PR，返回链接后停止；不合并、不切回 `main`。
- `kairomo merge`：完成验证、提交、推送，创建或复用对应 PR，核对检查后 squash merge，切回 `main` 并执行 `git pull origin main`。

批准修改文件（包括回复 `1`）不等于授权提交或上传。完整 merge 口令包含整套前置流程，无须逐步重复确认。普通 push、pr、merge 不包含发布软件、创建标签或上传安装包。排除本机记录、数据库、导出文件、日志、`.dev/`、`target/`、`dist/` 和签名凭据；保留其他任务的改动。
