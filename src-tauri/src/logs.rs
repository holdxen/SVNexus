//! 把日志目录打包成一个 zip，供反馈上报附带。
//!
//! 日志按天滚动且没有大小上限，实际见过单个文件四十多兆；后端把每个附件限制在
//! 1.5MB 以内，D1 单行也有上限，所以这里必须限流——从最新的文件往回收集，超出预算
//! 就只取文件末尾，压完仍然偏大就减半重来。

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use camino::{Utf8Path, Utf8PathBuf};
use snafu::OptionExt;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::app;
use crate::error::{self, builder};

/// 归档文件名。收集日志时要跳过它自己和上一次的产物。
const ARCHIVE_FILE: &str = "feedback-logs.zip";

/// README 首行，区分反馈附带的归档和用户手动导出的归档。
const FEEDBACK_TITLE: &str = "SVNexus feedback log archive";
const EXPORT_TITLE: &str = "SVNexus log export";

/// 未压缩输入的上限。日志是文本，压缩比通常十倍以上。
const RAW_BUDGET: usize = 16 * 1024 * 1024;

/// 减半重试的下限，再小就没什么可传的了。
const MIN_BUDGET: usize = 1024 * 1024;

/// 归档体积上限，压到后端单文件限制（1.5MB）以内。
const ARCHIVE_LIMIT: u64 = 1_400_000;

struct LogFile {
    name: String,
    path: Utf8PathBuf,
    size: u64,
    modified: std::time::SystemTime,
}

struct Entry {
    name: String,
    path: Utf8PathBuf,
    total: u64,
    taken: u64,
}

pub fn package() -> error::Result<Utf8PathBuf> {
    let project = app::project()?;
    let archive = Utf8PathBuf::from_path_buf(project.cache_directory().join(ARCHIVE_FILE))
        .ok()
        .context(builder::General {
            detail: "the cache directory is not valid UTF-8",
        })?;

    package_into(&archive, &project.log_directory())?;
    Ok(archive)
}

/// 用户手动导出：把日志目录下所有文件完整打包到指定路径，不做体积截断。
pub fn export(archive: &Utf8Path) -> error::Result<()> {
    export_from(archive, &app::project()?.log_directory())
}

/// 与目录解析分开，方便直接对着一个临时目录验证。
fn export_from(archive: &Utf8Path, log_directory: &std::path::Path) -> error::Result<()> {
    let logs = collect(log_directory)?;
    logs.first().context(builder::General {
        detail: format!("no log files in {}", log_directory.display()),
    })?;

    let entries = logs
        .into_iter()
        .map(|log| Entry {
            name: log.name,
            path: log.path,
            total: log.size,
            taken: log.size,
        })
        .collect::<Vec<_>>();
    write_archive(archive, &entries, EXPORT_TITLE)
}

/// 与目录解析分开，方便直接对着一个临时目录验证。
fn package_into(archive: &Utf8Path, log_directory: &std::path::Path) -> error::Result<()> {
    let logs = collect(log_directory)?;
    logs.first().context(builder::General {
        detail: format!("no log files in {}", log_directory.display()),
    })?;

    // 压缩比随日志内容浮动，压完超限就把预算减半重来；几次之后必然收敛。
    let mut budget = RAW_BUDGET;
    loop {
        let entries = select(&logs, budget);
        write_archive(archive, &entries, FEEDBACK_TITLE)?;

        let size = std::fs::metadata(archive)?.len();
        if size <= ARCHIVE_LIMIT || budget <= MIN_BUDGET {
            return Ok(());
        }
        budget /= 2;
    }
}

fn collect(directory: &std::path::Path) -> error::Result<Vec<LogFile>> {
    let mut files = Vec::new();

    if !directory.is_dir() {
        return Ok(files);
    }

    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }

        let path = Utf8PathBuf::from_path_buf(entry.path())
            .ok()
            .context(builder::General {
                detail: "a log path is not valid UTF-8",
            })?;
        let name = path.file_name().unwrap_or_default().to_string();

        // 上一次打包的产物就在同一层，别把它自己收进去
        if name == ARCHIVE_FILE {
            continue;
        }

        let metadata = entry.metadata()?;
        files.push(LogFile {
            name,
            path,
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(std::time::UNIX_EPOCH),
        });
    }

    // 新的在前：出问题的多半是最近这次运行
    files.sort_by(|left, right| right.modified.cmp(&left.modified));
    Ok(files)
}

fn select(logs: &[LogFile], budget: usize) -> Vec<Entry> {
    let mut entries = Vec::new();
    let mut used = 0usize;

    for log in logs {
        let size = log.size as usize;

        if used + size <= budget {
            used += size;
            entries.push(Entry {
                name: log.name.clone(),
                path: log.path.clone(),
                total: log.size,
                taken: log.size,
            });
            continue;
        }

        // 最新的一个就超预算：只留末尾，日志是追加写的，出错的地方在最后
        if entries.is_empty() {
            entries.push(Entry {
                name: log.name.clone(),
                path: log.path.clone(),
                total: log.size,
                taken: budget as u64,
            });
        }
        break;
    }

    entries
}

fn write_archive(archive: &Utf8Path, entries: &[Entry], title: &str) -> error::Result<()> {
    let file = File::create(archive)?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    // zip 的错误类型没进 error::Error 的 From 链，借 io::Error 带一层，细节不丢
    for entry in entries {
        writer
            .start_file(entry.name.clone(), options)
            .map_err(std::io::Error::other)?;
        writer.write_all(&read_tail(&entry.path, entry.taken)?)?;
    }

    writer
        .start_file("README.txt", options)
        .map_err(std::io::Error::other)?;
    writer.write_all(manifest(title, entries).as_bytes())?;

    writer.finish().map_err(std::io::Error::other)?;
    Ok(())
}

/// 只读需要的部分：一个四十兆的日志没必要整个读进内存。
fn read_tail(path: &Utf8Path, take: u64) -> error::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let length = file.metadata()?.len();

    if take < length {
        file.seek(SeekFrom::Start(length - take))?;
    }

    let mut buffer = Vec::with_capacity(take.min(length) as usize);
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// 附一份清单，接收方才知道日志被截断过。
fn manifest(title: &str, entries: &[Entry]) -> String {
    let mut lines = vec![
        title.to_string(),
        String::new(),
        "Files included (newest first):".to_string(),
    ];

    for entry in entries {
        if entry.taken < entry.total {
            lines.push(format!(
                "  {} — last {} of {} (truncated)",
                entry.name,
                human_size(entry.taken),
                human_size(entry.total)
            ));
        } else {
            lines.push(format!("  {} ({})", entry.name, human_size(entry.total)));
        }
    }

    lines.push(String::new());
    lines.push("Truncated entries keep only the end of the file.".to_string());
    lines.join("\n")
}

fn human_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    let value = bytes as f64;

    if value >= MB {
        format!("{:.1} MB", value / MB)
    } else if value >= KB {
        format!("{:.0} KB", value / KB)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> Utf8PathBuf {
        let dir = Utf8PathBuf::from_path_buf(
            std::env::temp_dir().join(format!("svnexus-logs-test-{name}")),
        )
        .unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// 不可压缩的文本，用来逼出体积上限分支
    fn noisy(len: usize) -> String {
        let mut state: u64 = 0x2545F4914F6CDD1D;
        (0..len)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                char::from(b'a' + (state % 26) as u8)
            })
            .collect()
    }

    #[test]
    fn archives_are_capped_and_truncated() {
        let logs = temp_dir("cap");
        std::fs::write(logs.join("svnexus.log.2026-01-01"), noisy(200_000)).unwrap();
        std::fs::write(logs.join("svnexus.log.2026-01-02"), noisy(6_000_000)).unwrap();
        std::fs::write(logs.join("svnexus.log.2026-01-03"), noisy(20_000_000)).unwrap();

        let archive = logs.join("out.zip");
        package_into(&archive, logs.as_std_path()).unwrap();

        let size = std::fs::metadata(&archive).unwrap().len();
        assert!(size <= ARCHIVE_LIMIT, "archive {size} exceeds the limit");

        let file = std::fs::File::open(&archive).unwrap();
        let mut zip = zip::ZipArchive::new(file).unwrap();
        let names: Vec<String> = (0..zip.len())
            .map(|i| zip.by_index(i).unwrap().name().to_string())
            .collect();
        assert!(
            names.contains(&"README.txt".to_string()),
            "names: {names:?}"
        );
        assert!(
            names.iter().any(|n| n.contains("2026-01-03")),
            "the newest log should be present: {names:?}"
        );

        let mut readme = String::new();
        zip.by_name("README.txt")
            .unwrap()
            .read_to_string(&mut readme)
            .unwrap();
        assert!(readme.contains("truncated"), "readme: {readme}");

        // 截断的条目必须保留文件末尾
        let mut entry = zip.by_name("svnexus.log.2026-01-03").unwrap();
        let mut content = Vec::new();
        entry.read_to_end(&mut content).unwrap();
        let source = std::fs::read(logs.join("svnexus.log.2026-01-03")).unwrap();
        assert_eq!(
            content,
            source[source.len() - content.len()..],
            "truncation must keep the end of the log"
        );

        let _ = std::fs::remove_dir_all(&logs);
    }

    #[test]
    fn export_keeps_every_file_whole() {
        let logs = temp_dir("export");
        std::fs::write(logs.join("svnexus.log.2026-03-01"), noisy(3_000_000)).unwrap();
        std::fs::write(logs.join("svnexus.log.2026-03-02"), "short\n").unwrap();

        // 即使超过反馈用的预算，导出也不截断
        let archive = logs.join("out.zip");
        export_from(&archive, logs.as_std_path()).unwrap();

        let file = std::fs::File::open(&archive).unwrap();
        let mut zip = zip::ZipArchive::new(file).unwrap();
        let mut readme = String::new();
        zip.by_name("README.txt")
            .unwrap()
            .read_to_string(&mut readme)
            .unwrap();
        assert!(readme.starts_with(EXPORT_TITLE), "readme: {readme}");
        assert!(!readme.contains("truncated"), "readme: {readme}");

        let mut entry = zip.by_name("svnexus.log.2026-03-01").unwrap();
        let mut content = Vec::new();
        entry.read_to_end(&mut content).unwrap();
        let source = std::fs::read(logs.join("svnexus.log.2026-03-01")).unwrap();
        assert_eq!(content, source, "export must not truncate");

        let _ = std::fs::remove_dir_all(&logs);
    }

    #[test]
    fn small_logs_are_included_whole() {
        let logs = temp_dir("small");
        std::fs::write(
            logs.join("svnexus.log.2026-02-01"),
            "first line
last line
",
        )
        .unwrap();
        std::fs::write(
            logs.join("svnexus.log.2026-02-02"),
            "only line
",
        )
        .unwrap();

        let archive = logs.join("out.zip");
        package_into(&archive, logs.as_std_path()).unwrap();

        let file = std::fs::File::open(&archive).unwrap();
        let mut zip = zip::ZipArchive::new(file).unwrap();
        let mut readme = String::new();
        zip.by_name("README.txt")
            .unwrap()
            .read_to_string(&mut readme)
            .unwrap();
        assert!(!readme.contains("truncated"), "readme: {readme}");

        let mut entry = zip.by_name("svnexus.log.2026-02-01").unwrap();
        let mut content = String::new();
        entry.read_to_string(&mut content).unwrap();
        assert_eq!(
            content,
            "first line
last line
"
        );

        // 上一次的产物不该被收进来
        std::fs::write(logs.join(ARCHIVE_FILE), b"stale").unwrap();
        package_into(&archive, logs.as_std_path()).unwrap();
        let file = std::fs::File::open(&archive).unwrap();
        let mut zip = zip::ZipArchive::new(file).unwrap();
        assert!(
            (0..zip.len()).all(|i| !zip.by_index(i).unwrap().name().contains("feedback-logs")),
            "the previous archive must be skipped"
        );

        let _ = std::fs::remove_dir_all(&logs);
    }
}
