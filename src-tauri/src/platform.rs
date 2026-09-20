use std::{
    path::{Path, PathBuf},
    process::Command,
};

use serde::{Deserialize, Serialize};
use snafu::OptionExt;

use crate::{
    error::{self, builder},
    extensions::OptionExtension,
};

pub fn tar() -> error::Result<String> {
    cfg_if::cfg_if!(
        if #[cfg(unix)] {
            return Ok("tar".to_string());
        } else {
            use crate::error::builder;
            use snafu::OptionExt;
            let current = std::env::current_exe()?;
            let parent = current.parent().context(builder::General {
                detail: "Unexpected execute path",
            })?;
            let path = parent
                .join("tar.exe")
                .to_str()
                .context(builder::General {
                    detail: "Unexpected tar path",
                })?
                .to_string();
            return Ok(path);
        }
    );
}

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

    #[cfg(target_os = "macos")]
    {
        let fallback_path = std::path::Path::new(fallback);
        if fallback_path.exists() {
            return Ok(fallback_path.to_path_buf());
        }
    }

    Err(builder::General {
        detail: format!("{} not found in PATH or at {}", name, fallback),
    }
    .build())
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
            ExternalApplication::Explorer => todo!(),
            ExternalApplication::VSCode => {
                let exe = find_app_or_fallback(
                    "code",
                    "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code",
                )?;
                self.open_vscode(&exe, &path)?;
            }
            ExternalApplication::Zed => {
                let exe = find_app_or_fallback("zed", "/Applications/Zed.app/Contents/MacOS/cli")?;

                self.open_zed(&exe, &path)?;
            }
            ExternalApplication::Warp => {
                let exe = find_app_or_fallback(
                    "wezterm",
                    "/Applications/WezTerm.app/Contents/MacOS/wezterm",
                )?;

                self.open_wezterm(&exe, &path)?;
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

    fn open_sublime(&self, exe: &Path, path: &Path) -> error::Result<()> {
        Command::new(exe).arg(path).spawn()?;
        Ok(())
    }

    fn open_alacritty(&self, exe: &Path, path: &Path) -> error::Result<()> {
        Command::new(exe)
            .arg("--working-directory")
            .arg(path)
            .env(PATH, self.subversion_env()?.as_str())
            .spawn()?;
        Ok(())
    }

    #[tracing::instrument]
    fn open_wezterm(&self, exe: &Path, path: &Path) -> error::Result<()> {
        Command::new(exe)
            .arg("start")
            .arg("--cwd")
            .arg(path)
            .env(PATH, self.subversion_env()?.as_str())
            .spawn()?;
        Ok(())
    }

    fn open_zed(&self, exe: &Path, path: &Path) -> error::Result<()> {
        Command::new(exe).arg(path).spawn()?;
        Ok(())
    }

    fn open_vscode(&self, exe: &Path, path: &Path) -> error::Result<()> {
        Command::new(exe).arg(path).spawn()?;
        Ok(())
    }

    fn open_terminal(&self, path: &Path) -> error::Result<()>;

    fn combine_env(&self, name: &str, value: &str) -> String {
        let current_path = std::env::var(name).unwrap_or_default();
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
            .env(PATH, &new_path)
            .spawn()?;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct LinuxPlatform;

#[cfg(target_os = "linux")]
impl Platform for LinuxPlatform {
    fn open_terminal(&self, path: &Path) -> error::Result<()> {
        let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
        let shell_cmd = format!("export PATH='{}:$PATH'", self.subversion_path()?.display());

        let (terminal, args): (&str, Vec<String>) = match desktop.to_lowercase().as_str() {
            d if d.contains("kde") => {
                let cmd = format!(
                    "export PATH={}:$PATH; exec bash -i",
                    self.subversion_path()?.display()
                );
                (
                    "konsole",
                    vec![
                        "--workdir".to_string(),
                        path.to_string_lossy().to_string(),
                        "-e".to_string(),
                        "bash".to_string(),
                        "-c".to_string(),
                        cmd,
                    ],
                )
            }
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
                    "-e".to_string(),
                    shell_cmd,
                ],
            ),
            _ => {
                let current_path = std::env::var("PATH").unwrap_or_default();
                let new_path = format!("{}:{}", self.subversion_path()?.display(), current_path);
                let exe = which::which("xdg-open")?;
                Command::new(exe).arg(path).env("PATH", &new_path).spawn()?;
                return Ok(());
            }
        };

        let exe = which::which(terminal)?;
        Command::new(exe).args(&args).spawn()?;

        Ok(())
    }
}

#[derive(Debug)]
struct MacOSPlatform;

impl Platform for MacOSPlatform {
    fn open_terminal(&self, path: &Path) -> error::Result<()> {
        let shell_cmd = format!(
            "cd \\\"{}\\\" && export PATH=\\\"{}:$PATH\\\" && printf \\\"\\\\033[2J\\\\033[3J\\\\033[1;1H\\\"",
            path.display(), self.subversion_path()?.display()
        );
        let apple_script = format!(
            "tell application \"Terminal\" to do script \"{}\"",
            shell_cmd
        );
        Command::new("osascript")
            .arg("-e")
            .arg(&apple_script)
            .arg("-e")
            .arg("tell application \"Terminal\" to activate")
            .spawn()?;
        Ok(())
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
