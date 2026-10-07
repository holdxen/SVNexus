//! 应用设置的持久化：读写配置目录下的 `settings.toml`。
//!
//! 前端在启动时调 `load_settings` 拿到设置并应用，用户点「确定」时调
//! `save_settings` 整体覆盖写回。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::app;
use crate::error;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
    System,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, ts_rs::TS)]
#[ts(export)]
pub enum LocalePreference {
    #[default]
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en-US")]
    EnUs,
    #[serde(rename = "system")]
    System,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ProxyType {
    #[default]
    Http,
    Https,
    Socks,
}

/// 设置弹窗里的全部设置项。字段缺失时取 `Default`，方便手工编辑或版本演进。
#[derive(Serialize, Deserialize, Debug, Clone, Default, ts_rs::TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct Settings {
    pub theme: ThemeMode,
    pub locale: LocalePreference,
    pub default_username: String,
    pub default_password: String,
    pub proxy_enabled: bool,
    pub proxy_type: ProxyType,
    pub proxy_host: String,
    pub proxy_port: Option<u16>,
    pub proxy_username: String,
    pub proxy_password: String,
}

fn settings_path() -> error::Result<PathBuf> {
    Ok(app::project()?.settings_file())
}

#[tauri::command]
pub async fn load_settings() -> error::Result<Settings> {
    let path = settings_path()?;

    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        // 首次启动还没有配置文件
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Settings::default())
        }
        Err(source) => return Err(source.into()),
    };

    match toml::from_str(&text) {
        Ok(settings) => Ok(settings),
        // 配置文件被改坏不该让应用起不来，退回默认值并留下日志
        Err(source) => {
            tracing::warn!("Failed to parse {}: {}", path.display(), source);
            Ok(Settings::default())
        }
    }
}

#[tauri::command]
pub async fn save_settings(settings: Settings) -> error::Result<()> {
    let path = settings_path()?;

    let text = toml::to_string_pretty(&settings)
        .map_err(|source| std::io::Error::new(std::io::ErrorKind::InvalidData, source))?;

    std::fs::write(&path, text)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let settings = Settings {
            theme: ThemeMode::Dark,
            locale: LocalePreference::EnUs,
            default_username: "alice".into(),
            proxy_enabled: true,
            proxy_type: ProxyType::Socks,
            proxy_port: Some(1080),
            ..Default::default()
        };

        let text = toml::to_string_pretty(&settings).unwrap();
        let parsed: Settings = toml::from_str(&text).unwrap();

        assert_eq!(parsed.theme, ThemeMode::Dark);
        assert_eq!(parsed.locale, LocalePreference::EnUs);
        assert_eq!(parsed.default_username, "alice");
        assert_eq!(parsed.proxy_type, ProxyType::Socks);
        assert_eq!(parsed.proxy_port, Some(1080));
        assert_eq!(parsed.proxy_password, "");
    }

    #[test]
    fn writes_expected_field_names() {
        let text = toml::to_string_pretty(&Settings::default()).unwrap();

        assert!(text.contains("theme = \"light\""), "{text}");
        assert!(text.contains("locale = \"zh-CN\""), "{text}");
        assert!(text.contains("proxyType = \"http\""), "{text}");
        assert!(text.contains("defaultUsername = \"\""), "{text}");
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let parsed: Settings = toml::from_str("theme = \"system\"\nproxyEnabled = true\n").unwrap();

        assert_eq!(parsed.theme, ThemeMode::System);
        assert!(parsed.proxy_enabled);
        assert_eq!(parsed.locale, LocalePreference::ZhCn);
        assert_eq!(parsed.proxy_type, ProxyType::Http);
        assert_eq!(parsed.proxy_port, None);
    }
}
