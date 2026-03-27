use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use snafu::ResultExt;

use super::ffi::*;
use crate::{
    apr::{self, ffi::*},
    error::{self, builder},
    utils::Pointer,
};

pub fn backup(path: impl AsRef<Path>) -> error::Result<PathBuf> {
    let dir = tempfile::tempdir()?;
    let path = path.as_ref();
    snafu::ensure!(
        path.exists(),
        builder::General {
            detail: "Path does not exist"
        }
    );
    let mut file_name = path
        .file_name()
        .map(|v| v.to_os_string())
        .unwrap_or(OsString::from("backup"));

    #[cfg(target_os = "linux")]
    {
        file_name.push(".tar.gz");
        let mut file = dir.keep();
        file.push(file_name);

        let status = Command::new("tar")
            .current_dir(path)
            .arg("zxcf")
            .arg(&file)
            .arg(".")
            .status()?;

        snafu::ensure!(
            status.success(),
            builder::General {
                detail: "Failed to backup"
            }
        );
        return Ok(file);
    }

    #[cfg(target_os = "macos")]
    {
        file_name.push(".tar.gz");
        let mut file = dir.keep();
        file.push(file_name);

        let status = Command::new("tar")
            .current_dir(path)
            .arg("-zcf")
            .arg(&file)
            .arg(".")
            .status()?;

        snafu::ensure!(
            status.success(),
            builder::General {
                detail: "Failed to backup"
            }
        );
        return Ok(file);
    }

    todo!()
}

pub fn clear_dir<P: AsRef<Path>>(dir: P) -> error::Result<()> {
    let dir = dir.as_ref();

    snafu::ensure!(
        dir.exists(),
        builder::General {
            detail: "Directory does not exist"
        }
    );

    snafu::ensure!(
        dir.is_dir(),
        builder::General {
            detail: "Path is no dir"
        }
    );

    for entry in walkdir::WalkDir::new(dir).max_depth(1) {
        let Ok(entry) = entry else {
            continue;
        };

        let path = entry.path();

        // 用 symlink_metadata：拿到“自身类型”，避免跟随 symlink
        let ft = entry.file_type();

        if ft.is_dir() {
            // 子目录：递归删除整个子目录
            std::fs::remove_dir_all(&path)?;
        } else {
            // 文件 / 软链接 / 其他：删文件即可（软链接本身也算“文件”删掉链接）
            std::fs::remove_file(&path)?;
        }
    }

    Ok(())
}

impl apr::Pool {
    pub unsafe fn read_subversion_config(
        &mut self,
        path: Option<&str>,
    ) -> error::Result<*mut apr_hash_t> {
        let mut hash: *mut apr_hash_t = std::ptr::null_mut();

        let path = path
            .map(|p| unsafe { self.string(p) }).transpose()?.unwrap_or_default();

        unsafe {
            let error = svn_config_get_config(hash.pointer_mut(), path, self.as_mut_ptr());
            super::SVNError::from_nullable_ptr(error).context(builder::Svn)?;
            assert!(!hash.is_null(), "Failed to read Subversion configuration");
        }

        Ok(hash)
    }
}
