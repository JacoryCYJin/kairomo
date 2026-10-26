# Kairomo 架构

## 选择与依据

采用 Tauri 2 + Vue 3 + TypeScript + Rust + SQLite。比较过单包分目录、单仓多包、多仓/后台服务三种组织方式：单包无法阻止业务依赖桌面细节；多仓/独立服务使一个本机工具承担发布与通信成本；单仓多包同时保持构建统一和可验证依赖边界。

依据：

- [Tauri Project Structure](https://v2.tauri.app/start/project-structure/)，前端与 Rust 桌面包的官方结构。
- [Tauri State Management](https://v2.tauri.app/develop/state-management/)，Rust 侧统一状态与多个入口共享。
- [Cargo Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)，统一锁文件、包依赖与验证。
- [Vue State Management](https://vuejs.org/guide/scaling-up/state-management.html)，单向状态流。

查阅于 2026-10-26。分层规范属于本项目设计，不宣称为 Tauri 强制结构。

## 包与职责

| 模块                  | 职责                                        | 允许依赖             | 禁止承担                        |
| --------------------- | ------------------------------------------- | -------------------- | ------------------------------- |
| timer-core            | 状态机、有效区间、设置校验、Repository 接口 | serde、thiserror     | Tauri、SQLite、系统时钟、通知   |
| timer-storage         | SQLite、编号迁移、事务、历史查询            | timer-core、rusqlite | 窗口、托盘、业务状态切换        |
| kairomo-desktop       | 组装和驱动应用                              | core、storage、Tauri | 复制核心状态机                  |
| commands              | 参数与调用结果转换                          | runtime              | 数据库 SQL、界面文案            |
| runtime               | 单调时钟、状态协调、持久化、恢复            | core、storage        | 系统窗口行为                    |
| platform              | 托盘、窗口、通知、睡眠适配                  | Tauri、runtime 入口  | 新建第二套计时器                |
| src/app               | 页面组装、读取与操作协调                    | features、shared     | 核心计时规则                    |
| src/features          | timer / history / settings 业务界面         | shared               | 跨 feature 内部导入、数据库访问 |
| src/shared/ipc        | 桌面调用、事件订阅、通信类型                | Tauri SDK            | 页面布局、业务状态机            |
| src/shared/ui、styles | 通用控件、视觉变量                          | Vue                  | 产品数据获取                    |

依赖是 desktop → core / storage，storage → core。`core` 对存储实现无感。当前没有独立 HTTP 服务、Node 常驻后台或浏览器模拟计时器。

## 数据流与一致性

1. Vue 提交开始、暂停、继续、切换、结束操作。
2. commands 将操作交给 runtime；一个 Mutex 串行化窗口、托盘、系统事件和后台心跳。
3. runtime 克隆核心状态、补齐有效经过时间，再让核心执行转移。
4. storage 在同一 SQLite 事务中写入完成记录与当前检查点。
5. 保存成功后才替换共享状态，并广播快照；失败返回错误，保持原状态。

Rust 状态是唯一权威来源。前端每秒显示后台快照，不通过定时器递增计时。通知失败不回滚已经保存的时间；存储失败会报告错误。

## 数据模型

- Settings：专注/休息目标、语言、通知开关。
- Session：ID、种类、标题、目标、总有效时长、起止时间、修正标记。
- Interval：实际工作的有效区间；暂停后恢复会创建新区间。
- Snapshot：当前会话、运行状态、恢复标记、检查点。

时间戳使用 UTC Unix 毫秒，运行期间有效时长由单调时钟提供。跨午夜显示按本地日期拆分区间。手工修正时长保留原区间位置并按比例分配有效时长，不伪造新的完成时间。

## 规范如何落实

- Cargo 包依赖、单元和存储集成测试验证 Rust 边界与行为。
- `npm run check` 运行类型、lint、架构导入检查。
- `scripts/check-architecture.mjs` 禁止 shared 反向引用、feature 之间引用内部实现及 UI 直接导入 Tauri SDK；同时检查核心/存储的反向依赖。
- 格式化使用 Prettier 与 rustfmt；不以任意文件行数作为拆分依据，按职责拆分。
- 新依赖先说明用途；跨平台行为放 platform，核心规则变化先补可验证案例。

## 窗口与产品范围

默认入口是菜单栏。main 为 360 × 280 的极简计时浮窗，macOS 保留原生圆角外框并隐藏标题与系统按钮，标题、时间和空白处可直接拖动，按钮不触发拖动。菜单栏只切换显示/隐藏，不按点击屏幕搬动小窗；切换应用不收起，Esc 收起。macOS 用 NSUserDefaults 保存 AppKit 全局逻辑点坐标，记住拖动位置，隐藏再打开和重启后恢复。仅首次使用或原屏幕断开导致拖动区域不可见时恢复到可见位置。隐藏不停止后台计时。macOS 使用 Accessory 激活策略，不常驻 Dock。

前端只显示模式、时间、开始/暂停，以及运行中的结束与休息按钮。记录、设置由图标打开独立 details 辅助窗口，两个窗口共享 Rust 权威状态。platform/windows.rs 负责窗口开关与跨窗口导航；macOS 的 popup.m 使用 AppKit 的逻辑点坐标校验保存的位置；鼠标所在屏幕仅用于首次使用或离屏恢复，避免混用不同屏幕的像素缩放。原生适配不放入计时核心。小窗常驻上层，跨应用保留；Spaces、多显示器和全屏行为分别验收。调试构建只在系统应用日志目录保存有大小上限的 window.log，记录窗口事件、屏幕编号与位置，不包含学习记录。

首版用会话标题关联所做的事；尚未实现独立任务实体、优先级、项目、截止日期或云同步。未来增加任务时新增任务模块，通过稳定 ID 关联会话，不把任务规则塞进计时状态机。
