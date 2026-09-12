<p align="center">
  <img src="assets/mouse.png" width="96" height="96" alt="Windows Mouse Fix 图标">
</p>

# Windows Mouse Fix

使用 Rust 编写的 Windows 鼠标按钮映射工具。支持中键、侧键与组合手势，关闭设置窗口后以低内存占用常驻系统托盘。

本项目受 [Mac Mouse Fix](https://github.com/noah-nuebling/mac-mouse-fix) 的功能设计启发，并参考其按钮交互行为，在 Windows 上重新实现。它是独立的非官方项目，与原作者没有隶属关系。

## 功能

- 支持中键、按钮 4、按钮 5。
- 支持单击、双击、长按、按住按钮向四个方向拖动。
- 支持配合 Ctrl、Shift、Alt、Win 修饰键。
- 支持录制自定义快捷键，以及返回、前进、任务视图、切换虚拟桌面、显示桌面、音量和媒体控制等动作。
- 提供按钮操作捕获区域、映射编辑、删除、恢复默认和拖动时锁定指针选项。
- 支持托盘启用 / 暂停、双击打开设置、右键打开或退出。
- 配置保存在本地；可选通过 GitHub Releases 检查更新，包括 Beta 版本。

默认不绑定任何动作，由用户自行添加规则。

## 获取与运行

目前主要支持 **Windows 11 x64**。其他 Windows 版本与 ARM64 原生构建尚未验证。

在本仓库的 [Releases](https://github.com/jeckleee/windows-mouse-fix/releases) 页面查找发布包；如果尚无正式发布，可自行编译，或在 [Actions](https://github.com/jeckleee/windows-mouse-fix/actions) 中下载成功构建的 `windows-mouse-fix-x64` 产物。下载 Actions 产物可能需要登录 GitHub。

1. 解压发布包，运行 `windows-mouse-fix.exe`。无需安装额外的字体文件。
2. 在“按钮”页面使用捕获区域添加操作，并为其选择动作或录制快捷键。
3. 点击窗口关闭按钮，设置界面退出，按钮映射继续在后台生效。
4. 双击托盘图标重新打开设置；右键图标选择“退出”，结束整个程序。

如果出现“访问被拒绝 / 0x00000005”导致托盘注册失败，请先完全退出程序，再尝试右键 exe → **以管理员身份运行**。操作管理员权限的目标窗口时，也可能需要相应权限。程序不会自动提升权限。

更新前请通过托盘“退出”结束整个程序，再替换 exe。

## 内存与运行方式

同一个 exe 有两种运行模式：

- **后台进程**：持有输入监听、映射规则、配置保存和托盘，不创建 egui 窗口或 OpenGL 渲染上下文。
- **设置进程**：由后台按需启动，负责窗口、字体和渲染；关闭窗口后退出，由系统回收该进程的资源。

窗口打开时，任务管理器通常能看到两个同名进程；关闭后只剩后台进程。内部参数 `--settings` 用于父子进程通信，不应手动单独运行。

在一台 Windows 11 电脑上的用户实测中，0.2.0 前台运行约占 40 MB，关闭窗口后后台约占 1.7 MB。这是特定环境的观察值，不是所有设备上的内存保证。

中文字体通过只读内存映射按需访问，避免完整读取和重复复制系统字体。

## 配置与网络访问

配置文件位于：

```text
%LOCALAPPDATA%\WindowsMouseFix\config.json
```

配置以 JSON 保存，包含映射规则与通用设置。后台统一写入配置，先写临时文件，再替换原文件。卸载时先退出程序，再删除 exe；如需清除设置，可另行删除上述 `WindowsMouseFix` 目录。

更新检查默认关闭。可在“通用”中将更新来源填为 `jeckleee/windows-mouse-fix`，并启用检查。程序只检查版本、提供下载链接，不会自动下载或安装更新。

输入监听用于识别映射和录制用户指定的快捷键，不保存完整键盘输入记录。未启用更新检查且不点击外部链接时，代码不会主动发起更新网络请求。

## 已知限制

- 当前实现的是通用设置与按钮映射，**不包含平滑滚动、鼠标加速度调整或设备驱动**。
- 只识别标准中键和两个侧键，不区分不同物理鼠标，不支持按应用分别设置规则。
- 双击识别使用系统双击时间；长按阈值为 400 ms，拖动阈值为累计 24 个输入坐标单位，尚无阈值设置界面。
- 单击与双击共享一个按钮时，部分单击需要等待双击判定。
- 新打开的设置窗口需要初始化渲染环境，速度可能比显示隐藏窗口稍慢。
- 关闭窗口会丢弃尚未点击保存的映射编辑草稿；已提交的规则由后台保留。
- macOS 仅用于界面预览与开发检查，不提供全局映射、托盘或双进程后台功能。Linux 未验证。
- 发布程序目前未进行代码签名。

## 从源码构建

### Windows 原生构建（推荐）

准备以下工具：

- [Rust / rustup](https://rustup.rs/)，使用最新 stable 工具链和 MSVC 目标。
- Visual Studio Build Tools，安装“使用 C++ 的桌面开发”和 Windows SDK；SDK 中的 `rc.exe` 用于嵌入图标资源。

```powershell
git clone https://github.com/jeckleee/windows-mouse-fix.git
cd windows-mouse-fix
rustup toolchain install stable --profile minimal --component rustfmt --component clippy
rustup target add x86_64-pc-windows-msvc
cargo build --locked --release --target x86_64-pc-windows-msvc
```

输出文件：

```text
target/x86_64-pc-windows-msvc/release/windows-mouse-fix.exe
```

项目的 `.cargo/config.toml` 为此目标启用静态 C 运行库链接。`Cargo.lock` 随仓库提交，用于固定依赖版本。

### macOS 交叉编译 Windows exe

先安装 Rust 和 Homebrew，再准备工具：

```bash
brew install llvm
cargo install cargo-xwin --locked
rustup target add x86_64-pc-windows-msvc
export PATH="$(brew --prefix llvm)/bin:$PATH"
export RC_PATH="$(brew --prefix llvm)/bin/llvm-rc"
cargo xwin build --locked --release --target x86_64-pc-windows-msvc
```

`cargo-xwin` 首次运行需要下载 Windows 构建组件，请遵循工具显示的许可提示。交叉编译成功不代表已经验证 Windows 托盘和输入行为，仍需在 Windows 实机测试。

### 代码检查

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
```

在 Mac 上检查 Windows 专属代码时，使用上述 LLVM 环境配置后执行：

```bash
cargo xwin clippy --locked --target x86_64-pc-windows-msvc --all-targets -- -D warnings
```

仓库的 Windows CI 会执行格式检查、Clippy 和 release 构建，并上传 exe 与许可说明。CI 不会自动发布 GitHub Release，也不能代替人工鼠标交互测试。

## 代码结构

| 文件 | 职责 |
| --- | --- |
| `src/main.rs` | 选择后台 / 设置运行模式，创建设置窗口 |
| `src/backend.rs` | 后台协调、配置保存、设置进程生命周期 |
| `src/editor_process.rs` | 启动和回收设置子进程，桥接管道与消息队列 |
| `src/ipc.rs` | 通过管道读写逐行 JSON 消息 |
| `src/platform.rs` | 命令与事件类型，界面连接及平台分支 |
| `src/windows.rs` | 单实例、鼠标 / 键盘钩子、快捷键发送、文件替换 |
| `src/windows_tray.rs` | 托盘图标、右键菜单、任务栏重建后的恢复 |
| `src/engine.rs` | 与 Windows API 分离的手势识别和动作生成 |
| `src/model.rs` | 配置、规则、动作、校验与序列化 |
| `src/ui.rs` | 通用设置、按钮列表、捕获和快捷键编辑界面 |
| `src/fonts.rs` | 系统中文字体的只读映射和生命周期管理 |
| `src/updates.rs` | GitHub Releases 查询和版本比较 |
| `build.rs`、`assets/` | 嵌入多尺寸图标和 Windows 版本资源 |

输入钩子与托盘使用不同线程，避免 Windows Shell 操作阻塞输入处理。进程管道 I/O 在独立线程执行，后台负责统一更新配置。

## 反馈与贡献

欢迎提交 Issue 和 Pull Request。报告问题时请提供 Windows 版本、程序版本、是否以管理员身份运行、复现步骤，以及相关映射规则。粘贴诊断信息前可隐去用户名和本地路径。

修改后请通过格式与 Clippy 检查，并在 Windows 上验证相关行为，尤其是：关闭后映射仍然生效、双击托盘恢复、快速重复打开只有一个设置窗口、退出后没有残留进程，以及修改后的规则能够重新加载。当前未随仓库保留自动化回归测试，CI 主要覆盖静态检查与构建。

## 许可证与致谢

本仓库原创 Rust 代码和鼠标图标采用 [MIT License](LICENSE)。第三方代码与字体遵循各自的许可证，详见 [第三方说明](THIRD_PARTY_NOTICES.md) 和 [许可文本汇总](THIRD_PARTY_LICENSES.txt)。

感谢 Noah Nuebling 的 [Mac Mouse Fix](https://github.com/noah-nuebling/mac-mouse-fix) 提供功能设计参考。原项目采用自定义 [MMF License](https://github.com/noah-nuebling/mac-mouse-fix/blob/master/License)，并非 MIT；本仓库的 MIT 授权不替代原项目的许可条件。本仓库不包含下载的 Mac 项目源码目录，也不包含其商业授权、试用或付款模块。

系统中文字体由操作系统提供，没有随本仓库重新分发。egui 自带的默认字体则随依赖嵌入程序，其许可证收录在第三方许可汇总中。
