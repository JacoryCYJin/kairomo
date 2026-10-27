# 开发与验证

## 环境

Node 22.13+（.node-version 为本次验证版本），Rust stable，rustfmt、clippy，以及目标平台 Tauri 前置依赖。使用 npm 与根 Cargo Workspace，不引入其他包管理器。

```bash
rustup component add rustfmt clippy
npm ci
npm run tauri -- dev
```

若 Cargo 已安装但未在 PATH，先执行 `. "$HOME/.cargo/env"`。启动前 `bash scripts/dev.sh status` 检查本项目；默认前端端口 1420，严格拒绝自动换端口。不要启停其他项目。

## 检查

| 命令                                                  | 验证对象                       |
| ----------------------------------------------------- | ------------------------------ |
| npm run check                                         | Vue/TypeScript、lint、模块边界 |
| npm run format:check                                  | 源码与文档格式                 |
| npm run build                                         | 实际前端构建                   |
| cargo fmt --all -- --check                            | Rust 格式                      |
| cargo test --workspace                                | 状态机、存储、协调与导出       |
| cargo clippy --workspace --all-targets -- -D warnings | Rust 静态诊断                  |
| npm run tauri -- build --debug --bundles app          | macOS Kairomo Dev 调试应用     |

CI 为 macOS/Windows 分别检查和构建，不发布、不上传安装包、不创建 Release。Windows 真机的托盘、通知、睡眠恢复仍需单独验收。macOS 的睡眠实测不得通过直接让用户电脑睡眠完成，使用核心测试并由用户在真实日常使用中确认系统链路。

## 本机数据

macOS 日常 Beta：`~/Library/Application Support/com.jacorycyjin.kairomo/kairomo.sqlite3`。开发版：`~/Library/Application Support/com.jacorycyjin.kairomo.dev/kairomo.sqlite3`。两个身份的数据库、设置与窗口偏好独立。Beta 升级保持原应用标识和数据库路径。其他平台通过 Tauri app_data_dir 定位。不要把数据库、CSV 导出、日志、target、dist、签名文件提交到 Git。

浏览器 `npm run dev` 只提供明确标识的界面预览，按钮不可运行计时；只有 Tauri 桌面应用可使用本机状态。开发预览入口及内存测试数据见 [展示系列](display-series.md)，不写用户数据库，生产构建移除该入口。

## 发布边界

0.1.0-beta.1 为首个本地 Beta，使用 `npm run build:beta` 打包；内置签名更新由用户确认安装，本次不发布线上版本。详见 [Beta 打包与更新](releases.md)。正式 macOS 对外分发另做 Developer ID、公证；Windows 另做签名和安装验证。当前不申请系统级权限、不注册自动启动、不配置开机服务。

## 手工验收

- 开始、暂停等待、恢复、提前结束；历史时长排除暂停。
- 咖啡杯和跳过休息都只切换到待开始；等候、退出重开不累计时间，需手动点“开始休息／开始专注”。未开始就再次切换、结束或取消不产生记录和豆子。
- 手动专注到点后仍在 running，主动休息才新增记录；到点结束模式自动保存专注并切到休息待开始。休息在两种模式下均到点结束、切到专注待开始。切换后始终手动开始，关闭通知不改变阶段切换。
- 模式保存、重开恢复和旧数据默认值一致；修改模式不改变已开始的一轮。到点与旧按钮点击同时发生不重复保存、不启动或跳过新阶段；保存失败保持原状态，重试后只新增一条记录。
- 菜单栏时间与浮窗一致；点击展开、再次点击收起，Esc 收起后继续计时；点击外部或切换应用不收起。
- 点击记录/设置图标打开辅助窗口；辅助窗口切换不影响计时。拖动标题、时间、空白区域应移动整个窗口且不选中文字；开始/暂停、记录/设置按钮保持点击。
- 拖到指定位置后，分别从两屏菜单栏收起/打开，小窗保持原位置；退出重启恢复位置，原屏幕断开后能够找回。检查不同缩放、负坐标排列、全屏与菜单栏自动隐藏。
- 检查原生窗口四角、阴影和隐藏标题栏，浅/深主题保持一致。
- 英文切换完整；导出、修正、删除及取消操作有效。
- 正常退出与异常退出后打开，均不会自动累计离线时间。
- 通知被系统禁用时计时继续；存储失败不得假报保存成功。
