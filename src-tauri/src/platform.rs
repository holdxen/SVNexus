use std::{
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::Mutex,
};

use serde::{Deserialize, Serialize};
use snafu::OptionExt;

use crate::{
    error::{self, builder},
    extensions::OptionExtension,
};
//
// pub fn tar() -> error::Result<String> {
//     cfg_if::cfg_if!(
//         if #[cfg(unix)] {
//             return Ok("tar".to_string());
//         } else {
//             use crate::error::builder;
//             use snafu::OptionExt;
//             let current = std::env::current_exe()?;
//             let parent = current.parent().context(builder::General {
//                 detail: "Unexpected execute path",
//             })?;
//             let path = parent
//                 .join("tar.exe")
//                 .to_str()
//                 .context(builder::General {
//                     detail: "Unexpected tar path",
//                 })?
//                 .to_string();
//             return Ok(path);
//         }
//     );
// }
//
#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ExternalApplication {
    Terminal,
    Explorer,
    VSCode,
    Zed,
    Warp,
    Alacritty,
    Sublime,
    WezTerm,
}

fn find_app_or_fallback(name: &str, fallback: &str) -> error::Result<PathBuf> {
    use crate::error::builder;

    if let Ok(path) = which::which(name) {
        return Ok(path);
    }

    let fallback_path = Path::new(fallback);
    if fallback_path.exists() {
        return Ok(fallback_path.to_path_buf());
    }

    Err(builder::General {
        detail: format!("{} not found in PATH or at {}", name, fallback),
    }
    .build())
}

/// 已 spawn 的子进程；每次新 spawn 前回收已退出者，避免长期运行的主进程累积僵尸进程。
static SPAWNED_CHILDREN: Mutex<Vec<Child>> = Mutex::new(Vec::new());

fn spawn_reaped(command: &mut Command) -> error::Result<()> {
    let mut children = SPAWNED_CHILDREN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    children.retain_mut(|child| matches!(child.try_wait(), Ok(None)));
    children.push(command.spawn()?);
    Ok(())
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{:02X}", byte)),
        }
    }
    encoded
}

#[cfg(target_os = "windows")]
fn vscode_fallback() -> String {
    let candidates = [
        std::env::var("LOCALAPPDATA").ok().map(|base| {
            PathBuf::from(base)
                .join("Programs")
                .join("Microsoft VS Code")
                .join("bin")
                .join("code.cmd")
        }),
        std::env::var("ProgramFiles").ok().map(|base| {
            PathBuf::from(base)
                .join("Microsoft VS Code")
                .join("bin")
                .join("code.cmd")
        }),
    ];
    for candidate in candidates.into_iter().flatten() {
        if candidate.exists() {
            return candidate.display().to_string();
        }
    }
    String::from(r"C:\Program Files\Microsoft VS Code\bin\code.cmd")
}

#[cfg(not(target_os = "windows"))]
fn vscode_fallback() -> String {
    String::from("/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code")
}

#[cfg(target_os = "macos")]
fn shell_double_quote(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for c in value.chars() {
        if matches!(c, '"' | '\\' | '$' | '`') {
            quoted.push('\\');
        }
        quoted.push(c);
    }
    quoted.push('"');
    quoted
}

#[cfg(target_os = "macos")]
fn apple_script_quote(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for c in value.chars() {
        if matches!(c, '"' | '\\') {
            quoted.push('\\');
        }
        quoted.push(c);
    }
    quoted.push('"');
    quoted
}

#[cfg(target_os = "linux")]
fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

const PATH: &str = "PATH";

// #[easy_ext::ext]
// impl std::path::Path {
//     fn to_string_or_error(&self) -> error::Result<&str> {
//         self.as_os_str()
//             .to_str()
//             .any_context("Not UTF-8 valid string")
//     }
// }

pub trait Platform: std::fmt::Debug {
    #[tracing::instrument]
    fn open_external_app(
        &self,
        app: ExternalApplication,
        path: Option<String>,
    ) -> error::Result<()> {
        let path = match path {
            Some(path) => PathBuf::from(path),
            None => {
                let path = std::env::home_dir().any_context("Failed to query home dir")?;
                path
            }
        };
        match app {
            ExternalApplication::Terminal => {
                self.open_terminal(&path)?;
            }
            ExternalApplication::Explorer => {
                self.open_explorer(&path)?;
            }
            ExternalApplication::VSCode => {
                let exe = find_app_or_fallback("code", &vscode_fallback())?;
                self.open_vscode(&exe, &path)?;
            }
            ExternalApplication::Zed => {
                let exe = find_app_or_fallback("zed", "/Applications/Zed.app/Contents/MacOS/cli")?;

                self.open_zed(&exe, &path)?;
            }
            ExternalApplication::Warp => {
                self.open_warp(&path)?;
            }
            ExternalApplication::Alacritty => {
                let exe = find_app_or_fallback(
                    "alacritty",
                    "/Applications/Alacritty.app/Contents/MacOS/alacritty",
                )?;
                self.open_alacritty(&exe, &path)?;
            }
            ExternalApplication::Sublime => {
                let exe = find_app_or_fallback(
                    "subl",
                    "/Applications/Sublime Text.app/Contents/SharedSupport/bin/subl",
                )?;
                self.open_sublime(&exe, &path)?;
            }
            ExternalApplication::WezTerm => {
                let exe = find_app_or_fallback(
                    "wezterm",
                    "/Applications/WezTerm.app/Contents/MacOS/wezterm",
                )?;
                self.open_wezterm(&exe, &path)?;
            }
        }
        Ok(())
    }

    fn open_explorer(&self, path: &Path) -> error::Result<()> {
        let program = if cfg!(target_os = "macos") {
            "open"
        } else if cfg!(target_os = "windows") {
            "explorer"
        } else {
            "xdg-open"
        };
        spawn_reaped(Command::new(program).arg(path))
    }

    // Warp 不支持命令行目录参数（warpdotdev/Warp#4347 仍开放），官方途径是 warp:// URI Scheme
    fn open_warp(&self, path: &Path) -> error::Result<()> {
        let url = format!(
            "warp://action/new_window?path={}",
            percent_encode(&path.to_string_lossy())
        );
        cfg_if::cfg_if! {
            if #[cfg(target_os = "macos")] {
                spawn_reaped(Command::new("open").arg(&url))
            } else if #[cfg(target_os = "windows")] {
                spawn_reaped(
                    Command::new("rundll32").args(["url.dll,FileProtocolHandler", url.as_str()]),
                )
            } else if #[cfg(target_os = "linux")] {
                spawn_reaped(Command::new("xdg-open").arg(&url))
            } else {
                panic!("Unsupported plaform")
            }
        }
    }

    fn open_sublime(&self, exe: &Path, path: &Path) -> error::Result<()> {
        spawn_reaped(Command::new(exe).arg(path))
    }

    fn open_alacritty(&self, exe: &Path, path: &Path) -> error::Result<()> {
        spawn_reaped(
            Command::new(exe)
                .arg("--working-directory")
                .arg(path)
                .env(PATH, self.subversion_env()?.as_str()),
        )
    }

    #[tracing::instrument]
    fn open_wezterm(&self, exe: &Path, path: &Path) -> error::Result<()> {
        spawn_reaped(
            Command::new(exe)
                .arg("start")
                .arg("--cwd")
                .arg(path)
                .env(PATH, self.subversion_env()?.as_str()),
        )
    }

    fn open_zed(&self, exe: &Path, path: &Path) -> error::Result<()> {
        spawn_reaped(Command::new(exe).arg(path))
    }

    fn open_vscode(&self, exe: &Path, path: &Path) -> error::Result<()> {
        spawn_reaped(Command::new(exe).arg(path))
    }

    fn open_terminal(&self, path: &Path) -> error::Result<()>;

    fn combine_env(&self, name: &str, value: &str) -> String {
        let current_path = std::env::var(name).unwrap_or_default();
        if current_path.is_empty() {
            return value.to_string();
        }
        cfg_if::cfg_if! {
            if #[cfg(unix)] {
                format!("{}:{}", value, current_path)
            } else if #[cfg(target_os = "windows")] {
                format!("{};{}", value, current_path)
            } else {
                panic!("Unsupported plaform")
            }
        }
    }

    fn subversion_path(&self) -> error::Result<PathBuf> {
        let exe = std::env::current_exe()?;

        let parent = exe.parent().context(builder::General {
            detail: "Invalid directory",
        })?;

        cfg_if::cfg_if! {
            if #[cfg(target_os = "macos")] {
                Ok(parent.join("svnexus-svn").join("bin"))
            } else if #[cfg(target_os = "windows")] {
                Ok(parent.to_path_buf())
            } else if #[cfg(target_os = "linux")] {
                Ok(parent.join("svnexus-svn").join("bin"))
            } else {
                panic!("Unsupported plaform")
            }
        }
    }

    fn subversion_env(&self) -> error::Result<String> {
        Ok(self.combine_env(PATH, &self.subversion_path()?.to_string_lossy()))
    }
}

#[cfg(target_os = "windows")]
#[derive(Debug)]
struct WindowsPlatform;

#[cfg(target_os = "windows")]
impl Platform for WindowsPlatform {
    fn open_terminal(&self, path: &Path) -> error::Result<()> {
        use std::os::windows::process::CommandExt;
        let svn_path = self.subversion_path()?;
        let new_path = self.combine_env(PATH, &svn_path.to_string_lossy());
        const CREATE_NEW_CONSOLE: u32 = 0x00000010;
        spawn_reaped(
            Command::new("cmd")
                .arg("/K")
                .arg("cd")
                .arg("/D")
                .arg(path)
                .creation_flags(CREATE_NEW_CONSOLE)
                // .args(["/K", "cd", "/D", path.to_path_buf()])
                // Command::new("wt")
                //     .arg("-d")
                //     .arg(path)
                .current_dir(path)
                .env(PATH, &new_path),
        )
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct LinuxPlatform;

#[cfg(target_os = "linux")]
impl Platform for LinuxPlatform {
    fn open_terminal(&self, path: &Path) -> error::Result<()> {
        let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
        let shell_cmd = format!(
            "export PATH={}:$PATH; exec bash -i",
            shell_single_quote(&self.subversion_path()?.to_string_lossy())
        );

        let (terminal, args): (&str, Vec<String>) = match desktop.to_lowercase().as_str() {
            d if d.contains("kde") => (
                "konsole",
                vec![
                    "--workdir".to_string(),
                    path.to_string_lossy().to_string(),
                    "-e".to_string(),
                    "bash".to_string(),
                    "-c".to_string(),
                    shell_cmd,
                ],
            ),
            d if d.contains("gnome") => (
                "gnome-terminal",
                vec![
                    "--working-directory".to_string(),
                    path.to_string_lossy().to_string(),
                    "--".to_string(),
                    "bash".to_string(),
                    "-c".to_string(),
                    shell_cmd,
                ],
            ),
            d if d.contains("xfce") => (
                "xfce4-terminal",
                vec![
                    "--working-directory".to_string(),
                    path.to_string_lossy().to_string(),
                    "--".to_string(),
                    "bash".to_string(),
                    "-c".to_string(),
                    shell_cmd,
                ],
            ),
            _ => {
                let exe = which::which("x-terminal-emulator").or_else(|_| which::which("xterm"))?;
                return spawn_reaped(
                    Command::new(exe)
                        .current_dir(path)
                        .env(PATH, self.subversion_env()?.as_str()),
                );
            }
        };

        let exe = which::which(terminal)?;
        spawn_reaped(Command::new(exe).args(&args))
    }
}

#[cfg(target_os = "macos")]
#[derive(Debug)]
struct MacOSPlatform;

#[cfg(target_os = "macos")]
impl Platform for MacOSPlatform {
    fn open_terminal(&self, path: &Path) -> error::Result<()> {
        let shell_cmd = format!(
            "cd {} && export PATH={}:$PATH && printf \"\\033[2J\\033[3J\\033[1;1H\"",
            shell_double_quote(&path.to_string_lossy()),
            shell_double_quote(&self.subversion_path()?.to_string_lossy())
        );
        let apple_script = format!(
            "tell application \"Terminal\" to do script {}",
            apple_script_quote(&shell_cmd)
        );
        spawn_reaped(
            Command::new("osascript")
                .arg("-e")
                .arg(&apple_script)
                .arg("-e")
                .arg("tell application \"Terminal\" to activate"),
        )
    }
}

pub fn current() -> impl Platform {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            MacOSPlatform {}
        } else if #[cfg(target_os = "windows")] {
            WindowsPlatform {}
        } else if #[cfg(target_os = "linux")] {
            LinuxPlatform {}
        } else {
            panic!("Unsupported plaform")
        }
    }
}

/// `/usr/local/bin` 在 macOS 默认 PATH 里（`/etc/paths`），Zed、VS Code 的 shell command 也装在这。
#[cfg(target_os = "macos")]
const CLI_LINK_DIR: &str = "/usr/local/bin";

/// 提权时要执行的 shell 命令。路径一律过 `shell_double_quote`：这串命令会以管理员身份运行，
/// 路径里的引号若不转义就会把命令截断。
#[cfg(target_os = "macos")]
fn cli_install_shell_command(target: &Path, link_path: &Path) -> String {
    format!(
        "mkdir -p {} && ln -sfn {} {}",
        shell_double_quote(CLI_LINK_DIR),
        shell_double_quote(&target.to_string_lossy()),
        shell_double_quote(&link_path.to_string_lossy()),
    )
}

/// 把应用自身的可执行文件软链到 `/usr/local/bin/<exe 名>`，终端里就能直接 `svnexus /path`。
///
/// 不需要像 Zed 那样另带一个 cli 二进制：主程序本身就是 CLI —— main.rs 用 clap 解析路径，
/// 已有实例运行时由 single-instance 插件把 argv 转发过去（lib.rs 的 `cli-open-path`），
/// 冷启动则走 `CliInitialPath`。
///
/// 三步降级（对照 Zed `crates/install_cli/src/install_cli_binary.rs` 的 `install_script`）：
/// 1. 链接已指向当前 exe 就直接返回；app 升级或挪位置后 readlink 不一致，会落到下面重建；
/// 2. 先试免提权的 remove + symlink（用户对 `/usr/local/bin` 有写权限时不必弹授权）；
/// 3. 失败再用 osascript 申请管理员权限。
///
/// 返回 `None` 表示用户在授权弹窗里点了取消 —— 那是用户的主动选择，不算失败。
#[cfg(target_os = "macos")]
pub fn install_cli() -> error::Result<Option<PathBuf>> {
    let target = std::env::current_exe()?;
    let name = target
        .file_name()
        .any_context("Failed to get executable file name")?
        .to_str()
        .any_context("Executable file name is not valid UTF-8")?;
    let link_path = Path::new(CLI_LINK_DIR).join(name);

    if std::fs::read_link(&link_path).ok().as_deref() == Some(target.as_path()) {
        return Ok(Some(link_path));
    }

    let _ = std::fs::remove_file(&link_path);
    if std::os::unix::fs::symlink(&target, &link_path).is_ok() {
        return Ok(Some(link_path));
    }

    // 路径要经过两层引号：先是 shell 的双引号，再整体作为 AppleScript 字符串字面量。
    // 两层都由本模块已有的转义函数处理，避免路径里的引号把提权命令截断。
    let shell_command = cli_install_shell_command(&target, &link_path);
    let script = format!(
        "do shell script {} with administrator privileges",
        apple_script_quote(&shell_command)
    );

    let output = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(&script)
        .output()?;

    if output.status.success() {
        return Ok(Some(link_path));
    }

    // osascript 在用户取消授权时报 "User canceled."（error -128）
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("User canceled") || stderr.contains("-128") {
        return Ok(None);
    }

    // 提权后仍失败，最常见的就是当前用户不是管理员
    Err(builder::General {
        detail: format!("osascript failed: {}", stderr.trim()),
    }
    .build())
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    /// 提权命令里的路径必须被完整转义：路径中带 `"`、`$`、反引号时，
    /// 它们要变成 shell 双引号内的转义形式，单引号在双引号里本就是字面量，保持原样。
    #[test]
    fn escapes_hostile_paths_in_the_privileged_command() {
        let command = cli_install_shell_command(
            Path::new("/tmp/a'b\"c$d`e/svnexus"),
            Path::new("/usr/local/bin/svnexus"),
        );

        assert_eq!(
            command,
            r#"mkdir -p "/usr/local/bin" && ln -sfn "/tmp/a'b\"c\$d\`e/svnexus" "/usr/local/bin/svnexus""#
        );
    }
}
