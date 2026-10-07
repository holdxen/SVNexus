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
