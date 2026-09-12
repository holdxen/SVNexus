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
    extensions::ResultExtension,
    utils::Pointer,
};
use crate::{extensions::CommonExtension, utils::CStringer};

pub fn backup(path: impl AsRef<Path>, output: Option<impl AsRef<Path>>) -> error::Result<PathBuf> {
    // let dir = tempfile::tempdir()?;
    let path = path.as_ref();
    snafu::ensure!(
        path.exists(),
        builder::General {
            detail: "Path does not exist"
        }
    );
    let file_name = path
        .canonicalize()?
        .file_name()
        .map(|v| v.to_os_string())
        .unwrap_or(OsString::from("backup"));

    let output = match output {
        Some(output) => output.as_ref().to_path_buf(),
        None => tempfile::tempdir()?.keep(),
    };

    #[cfg(target_os = "linux")]
    {
        let mut index = 0;

        let file = loop {
            let name = file_name.clone().also_apply(|f| {
                if index == 0 {
                    f.push(".tar.gz")
                } else {
                    f.push(format!(".{}.tar.gz", index));
                }
            });

            let file = output.join(name);

            if !file.exists() {
                break file;
            }
            index += 1;
        };

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
        // file_name.push(".tar.gz");
        // let mut file = dir.keep();
        // file.push(file_name);

        let mut index = 0;

        let file = loop {
            let name = file_name.clone().also_apply(|f| {
                if index == 0 {
                    f.push(".tar.gz")
                } else {
                    f.push(format!(".{}.tar.gz", index));
                }
            });

            let file = output.join(name);

            if !file.exists() {
                break file;
            }
            index += 1;
        };

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

        let p = path
            .map(|p| unsafe { self.string(p) })
            .transpose()?
            .unwrap_or_default();

        unsafe {
            let mut error = svn_config_get_config(hash.pointer_mut(), p, self.as_mut_ptr());
            // 逻辑根据svn的源码
            if let Some(e) = error.as_ref() {
                if patches_apr_status_is_eacces(e.apr_err) != 0
                    || patches_svn_apr_status_is_enotdir(e.apr_err) != 0
                {
                    tracing::warn!(
                        "Failed to read config in {:?}: {}, fallback to default config",
                        path,
                        e.message.to_str()
                    );
                    svn_error_clear(error);
                    error = svn_config__get_default_config(hash.pointer_mut(), self.as_mut_ptr());
                }
            }
            super::SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
            assert!(!hash.is_null(), "Failed to read Subversion configuration");
        }

        Ok(hash)
    }
}

pub fn base64_encode(data: &[u8], break_lines: bool) -> error::Result<String> {
    unsafe {
        let mut pool = apr::Pool::create();
        let input = svn_string_ncreate(
            data.as_ptr() as _,
            data.len().try_into().expect("Failed to convert size"),
            pool.as_mut_ptr(),
        );
        let output = svn_base64_encode_string2(input, break_lines.into(), pool.as_mut_ptr())
            .as_ref()
            .expect("Unexpected failure");
        let output =
            std::slice::from_raw_parts(output.data as *const u8, output.len.try_into().expect("Failed to convert size"));

        Ok(std::str::from_utf8(output)
            .any_context("Invalid output string")?
            .to_string())
    }
}

pub fn base64_decode(data: &str) -> Vec<u8> {
    unsafe {
        let mut pool = apr::Pool::create();
        let input = svn_string_ncreate(
            data.as_ptr() as _,
            data.len().try_into().expect("Failed to convert size"),
            pool.as_mut_ptr(),
        );
        let output = svn_base64_decode_string(input, pool.as_mut_ptr())
            .as_ref()
            .expect("Unexpected failure");
        let output =
            std::slice::from_raw_parts(output.data as *const u8, output.len.try_into().expect("Failed to convert size"));

        output.to_vec()
    }
}
