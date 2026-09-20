# SVNexus

跨平台 Subversion 桌面客户端，基于 Tauri 2 构建。

**仓库地址：** [GitHub](https://github.com/holdxen/SVNexus) | [SourceForge](https://sourceforge.net/projects/svnexus/)

**[English](./README.md)**

![SVNexus 截图](./screenshots/mac.png)

## 功能特性

- **完整 SVN 操作** — 支持 checkout、commit、update、add、delete、revert、switch、merge、relocate 等常用操作
- **差异对比** — 内置代码编辑器，直观查看文件变更内容
- **提交历史** — 浏览日志、查看变更路径与版本快照
- **工作区管理** — 多工作区分组管理，多标签页并行操作互不干扰
- **冲突处理** — 冲突检测与解决向导
- **文件锁** — lock / unlock 操作支持
- **属性编辑** — 查看与修改文件、目录的 SVN 属性及版本属性

## TODO

- [ ] 支持 SSH 协议，兼容 OpenSSH 行为
- [ ] 支持文件历史版本浏览
- [ ] 支持远程仓库独立浏览
- [ ] 支持一键反馈 Bug
- [ ] 支持按文件块（hunk）粒度 Revert 工作副本变更
- [ ] History 界面显示当前工作副本对应的 Revision
- [ ] 错误 Toast 支持跨标签页跳转定位
- [ ] 支持 copy 和 move 操作
- [ ] 完善 merge 操作
- [ ] Rust 代码优化，通过 cargo clippy 和 fmt 检查

## 安装

前往 [Releases](../../releases) 页面下载对应平台的安装包：

- **macOS** — `.dmg`
- **Windows** — `.msi` / `.exe`
- **Linux** — `.deb` / `.rpm` / `.AppImage`

## 使用说明

### 添加工作区

启动后点击欢迎页的「添加工作区」，选择本地已有的 SVN 工作副本目录，或直接 checkout 远程仓库。

### 基本操作

在工作区视图中：

- **查看状态** — 左侧树形目录展示文件状态（已修改、未版本控制、冲突等）
- **提交变更** — 选中文件后点击提交，填写日志消息即可完成 commit
- **查看差异** — 双击已修改文件打开差异对比视图
- **查看历史** — 右键文件或目录查看提交历史
- **更新 / 回退** — 工具栏一键 update 或 revert 选中文件

### 多标签页

支持同时打开多个工作区，通过顶部标签页切换，适合需要同时管理多个仓库的场景。

## 系统要求

| 平台    | 最低版本                  |
| ------- | ------------------------- |
| macOS   | 11+（Apple Silicon 版本） |
| Windows | 10+                       |
| Linux   | GTK 3 主流发行版          |

## 反馈与问题

遇到问题或有功能建议，欢迎提交 [Issue](../../issues)。

## License

[AGPL-3.0](./LICENSE.txt)
