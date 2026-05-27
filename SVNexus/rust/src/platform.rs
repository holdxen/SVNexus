use crate::error;

pub fn tar() -> error::Result<String> {
    cfg_if::cfg_if!(
        if #[cfg(unix)] {
            return Ok("tar".to_string());
        } else {
            use crate::error::builder;
            use snafu::OptionExt;
            let current = std::env::current_exe()?;
            let parent = current.parent().context(builder::General {
                detail: "Unexpect execute path",
            })?;
            let path = parent
                .join("tar.exe")
                .to_str()
                .context(builder::General {
                    detail: "Unexpect tar path",
                })?
                .to_string();
            return Ok(path);
        }
    );
}
