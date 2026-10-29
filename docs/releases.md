# Beta 打包、安装与更新

当前版本：0.1.0-beta.4。发布至 GitHub Beta prerelease，提供 Apple Silicon macOS 安装包与签名更新包。

## 身份与数据

| 用途     | 应用         | 应用标识                    | macOS 数据目录                                             |
| -------- | ------------ | --------------------------- | ---------------------------------------------------------- |
| 日常使用 | Kairomo Beta | com.jacorycyjin.kairomo     | ~/Library/Application Support/com.jacorycyjin.kairomo/     |
| 开发测试 | Kairomo Dev  | com.jacorycyjin.kairomo.dev | ~/Library/Application Support/com.jacorycyjin.kairomo.dev/ |

`tauri.conf.json` 默认是开发身份；`tauri.beta.conf.json` 在 Beta 打包时覆盖应用名称、标识和打包选项。两者从同一代码构建，但不共享数据库。Beta 沿用已有应用标识，因此历史记录、暂停中的会话和设置不需要迁移。以后 Beta 升级继续保持该标识，不把版本号放入数据路径。未来正式版若继续使用该身份，也会保留数据。

SQLite 位于应用包外。更新只替换应用包，不清空数据库；数据库结构变更继续使用编号迁移。安装更新前暂停并保存、创建 SQLite 一致性备份，备份位于日常数据目录下的 `backups/`。

## 本地打包

在仓库根目录执行：

```bash
npm run build:beta
# 等价入口
bash scripts/dev.sh beta
```

macOS 需要 Xcode 构建工具。本机首次 release 编译会构建完整依赖；后续复用缓存。脚本不启动开发服务、不停止当前计时、不自动安装或发布。

应用位于 `target/release/bundle/macos/Kairomo Beta.app`；DMG、更新包、签名、`latest.json` 和 SHA-256 校验清单位于 `target/beta/0.1.0-beta.4/`。打包目标是构建机器的架构，不把 Apple Silicon 构建称为 Intel 或通用版本。

双击 DMG 后将 Kairomo Beta 拖入 Applications。安装后从 `/Applications/Kairomo Beta.app` 启动日常版本。开发构建位于 `target/debug/bundle/macos/Kairomo Dev.app`，开发启动继续使用 `bash scripts/dev.sh start`。

## 两种签名

- macOS 本地 Beta 使用 ad-hoc 签名，当前没有 Developer ID 与公证，不能声称已经具备 Apple 公证的对外发行条件。
- 更新包另用 Tauri 签名密钥，客户端内嵌公钥，并要求签名绑定发布版本。签名校验不能关闭；数据不传到更新服务器。

实际更新私钥在源码目录之外：`~/.local/share/kairomo-release/updater.key`，目录权限 700、私钥权限 600。配套公钥为 `.key.pub`，嵌入源码的公钥是 `src-tauri/updater.pub`。不要提交、打印或上传私钥；首次生成后应单独保管，未来发布继续使用同一把密钥。

构建脚本默认读取该私钥；可用 `KAIROMO_UPDATER_KEY` 指定另一个路径，但配套公钥必须与内嵌公钥一致，否则停止。新环境需先安全取得原密钥，不能随意重新生成替换。测试里的 `update_fixture.json` 只包含一次性测试公钥、签名和假应用归档，不包含私钥，不被生产构建引用。

## 检查与安装

设置页“检查更新”只在 Beta 启用。Rust 读取本仓库公开 GitHub Releases，选择带 `latest.json` 的已发布 Beta prerelease，排除 draft、正式版、alpha 和不可用的清单地址，按 SemVer 比较。不能用排除 prerelease 的 GitHub `/releases/latest` 作为 Beta 发现入口。

没有发布任何 Beta 时，明确显示“Beta 更新通道尚未发布”；网络错误显示失败，不伪装成最新版。检查不自动下载。发现新版后显示版本与发布说明，由用户选择“下载并安装”。

下载完成并验证签名后，锁定计时状态、暂停保存、备份 SQLite，再交给官方 updater 替换应用并重开。失败保留记录；不自动恢复计时。过期确认或重复更新操作被拒绝。开发版不能安装 Beta 更新。

更新清单中的版本必须与发布标签一致；更新包签名必须绑定相同版本。所有线上请求使用 HTTPS。测试使用的 HTTP 仅限隔离测试配置，不进入打包配置。

## 未来发布

本地生成发布素材不代表线上通道可用。用户明确授权发布后，再按 Git 规则提交对应代码、创建 `v<版本>` 标签与 GitHub prerelease，上传对应架构的 DMG、`.app.tar.gz`、`.sig`、`latest.json` 和校验清单。安装包不包含个人数据库、导出、测试密钥或日志。后续版本重复此流程，检查更新会发现更高 Beta 版本。

发布由用户明确授权后手动执行，不启用自动发布流程。

## 验证

- 配置测试确认 Dev 与 Beta 标识分开，Beta 保留原数据身份。
- 隔离更新测试通过本地服务提供签名假应用，执行官方 updater 的检查、下载、签名校验和实际替换，验证篡改包被拒绝、原数据库及暂停状态保留、备份可读。
- 验证 Beta 发现过滤、SemVer 排序、并发操作保护，以及已有计时、存储和界面检查。
- 安装后核对应用标识、版本、签名、数据数量、当前会话及单实例状态；只读检查用户数据，不用真实记录试验取消或完成。

参考：[Tauri 更新](https://v2.tauri.app/plugin/updater/)、[DMG](https://v2.tauri.app/distribute/dmg/)、[macOS 签名](https://v2.tauri.app/distribute/sign/macos/)、[GitHub Releases API](https://docs.github.com/en/rest/releases/releases)。
