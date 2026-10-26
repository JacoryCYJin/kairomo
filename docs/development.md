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
| npm run tauri -- build --debug --bundles app          | macOS 调试应用                 |

CI 为 macOS/Windows 分别检查和构建，不发布、不上传安装包、不创建 Release。Windows 真机的托盘、通知、睡眠恢复仍需单独验收。macOS 的睡眠实测不得通过直接让用户电脑睡眠完成，使用核心测试并由用户在真实日常使用中确认系统链路。

## 本机数据

macOS：`~/Library/Application Support/com.jacorycyjin.kairomo/kairomo.sqlite3`。其他平台通过 Tauri app_data_dir 定位。不要把数据库、CSV 导出、日志、target、dist、签名文件提交到 Git。

浏览器 `npm run dev` 只提供明确标识的界面预览，按钮不可运行计时；只有 Tauri 桌面应用可使用本机状态。没有隐藏模拟数据模式。

## 发布边界

0.1.0 为首个本地版本。正式 macOS 分发另做 Developer ID、公证、签名与更新策略；Windows 另做签名和安装验证。当前不申请系统级权限、不注册自动启动、不配置开机服务。

## 手工验收

- 开始、暂停等待、恢复、提前结束；历史时长排除暂停。
- 达到目标后仍在 running，主动休息才新增记录。
- 菜单栏时间与浮窗一致；点击展开、再次点击收起，Esc 收起后继续计时；点击外部或切换应用不收起。
- 点击记录/设置图标打开辅助窗口；辅助窗口切换不影响计时。拖动标题、时间、空白区域应移动整个窗口且不选中文字；开始/暂停、记录/设置按钮保持点击。
- 拖到指定位置后，分别从两屏菜单栏收起/打开，小窗保持原位置；退出重启恢复位置，原屏幕断开后能够找回。检查不同缩放、负坐标排列、全屏与菜单栏自动隐藏。
- 检查原生窗口四角、阴影和隐藏标题栏，浅/深主题保持一致。
- 英文切换完整；导出、修正、删除及取消操作有效。
- 正常退出与异常退出后打开，均不会自动累计离线时间。
- 通知被系统禁用时计时继续；存储失败不得假报保存成功。
