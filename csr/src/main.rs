use std::path::{Path, PathBuf};

use camino::Utf8PathBuf;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "Project")]
struct Project {

    // <PropertyGroup>...</PropertyGroup>
    #[serde(rename = "PropertyGroup")]
    property_group: PropertyGroup,
}

impl Project {
    fn parse_from_xml(xml: &str) -> anyhow::Result<Self> {
        let project: Self = quick_xml::de::from_str(xml)?;
        Ok(project)
    }
}

#[derive(Debug, Deserialize)]
struct PropertyGroup {
    #[serde(rename = "TargetFramework")]
    target_framework: Option<String>,

    #[serde(rename = "TargetFrameworks")]
    target_frameworks: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Package,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
    edition: String,
}

impl CargoToml {
    fn parse_from_toml(toml_str: &str) -> anyhow::Result<Self> {
        let cargo_toml: Self = toml::from_str(toml_str)?;
        Ok(cargo_toml)
    }

    fn print_info(&self) {
        println!("Package Name: {}", self.package.name);
        println!("Version: {}", self.package.version);
        println!("Edition: {}", self.package.edition);
    }
}

impl Project {
    fn copy_library(&self, project_root: &Path, library_path: &Path) -> anyhow::Result<()> {
        let frameworks = self.property_group.frameworks();
        if frameworks.is_empty() {
            anyhow::bail!("No target frameworks found in the .csproj file");
        }

        let file_name = library_path.file_name().ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to get file name from library path: {:?}",
                library_path
            )
        })?;

        for framework in frameworks {
            let output_dir = project_root
                .join("bin")
                .join("Debug")
                .join(&framework);
            std::fs::create_dir_all(&output_dir)?;

            std::fs::copy(library_path, output_dir.join(file_name))?;

            let output_dir = project_root
                .join("bin")
                .join("Release")
                .join(&framework);
            std::fs::create_dir_all(&output_dir)?;

            std::fs::copy(library_path, output_dir.join(file_name))?;
        }

        Ok(())
    }
}

impl PropertyGroup {
    fn frameworks(&self) -> Vec<String> {
        if let Some(ref tf) = self.target_framework {
            vec![tf.clone()]
        } else if let Some(ref tfs) = self.target_frameworks {
            tfs.split(';').map(|s| s.trim().to_string()).collect()
        } else {
            vec![]
        }
    }
}

fn prepare_checking() -> anyhow::Result<()> {
    // Check for .csproj file
    // Check for rust directory with Cargo.toml
    // Parse .csproj to get target frameworks
    // Parse Cargo.toml to get package info
    let output = std::process::Command::new("cargo").arg("-V").output()?;

    anyhow::ensure!(
        output.status.success(),
        "Cargo is not installed or not found in PATH"
    );

    Ok(())
}

const OUT_DIRECTORY_NAME: &str = "Generated";

fn main() -> anyhow::Result<()> {
    prepare_checking()?;

    let current_dir = std::env::current_dir()?;

    let mut config = None;
    let mut csproj = None;
    let mut rust = None;

    for i in walkdir::WalkDir::new(&current_dir).max_depth(1) {
        let Ok(entry) = i else {
            continue;
        };

        if !entry.file_type().is_file() {
            if entry.file_type().is_dir() {
                if entry.file_name() == "rust" {
                    rust = Some(entry);
                }
            }
            continue;
        }

        if entry.file_name() == "uniffi.toml" {
            config = Some(entry)
        } else if entry.file_name().to_string_lossy().ends_with(".csproj") {
            if csproj.is_some() {
                anyhow::bail!("Multiple .csproj files found in the current directory");
            }
            csproj = Some(entry)
        }
    }

    let Some(csproj) = csproj else {
        anyhow::bail!("No .csproj file found in the current directory");
    };

    let Some(rust) = rust else {
        anyhow::bail!("No 'rust' directory found in the current directory");
    };

    let csproj_content = std::fs::read_to_string(csproj.path())?;

    let project = Project::parse_from_xml(&csproj_content)?;

    let cargo_toml_path = rust.path().join("Cargo.toml");

    let cargo_toml_content = std::fs::read_to_string(cargo_toml_path)?;

    let cargo_toml = CargoToml::parse_from_toml(&cargo_toml_content)?;

    cargo_toml.print_info();

    let library = build_rust_library(true, rust.path(), &cargo_toml.package.name)?.canonicalize()?;

    project.copy_library(&current_dir, &library)?;

    let library = Utf8PathBuf::from_path_buf(library).expect("Invalid library path");

    let config = config.map(|e| Utf8PathBuf::from_path_buf(e.into_path().canonicalize().expect("Failed to canonicalize")).expect(""));

    let config = config.as_ref().map(|e| e.as_path());

    let out_dir = current_dir.join(OUT_DIRECTORY_NAME);

    std::fs::create_dir_all(&out_dir)?;

    // let rust = Utf8PathBuf::from_path_buf(rust.path().to_path_buf())
    //     .expect("Invalid rust directory path");

    let out_dir = Utf8PathBuf::from_path_buf(out_dir.canonicalize()?).expect("Invalid output directory path");

    std::env::set_current_dir(rust.path())?;

    uniffi_bindgen_cs::generate_from_library(
        &library,
        Some(cargo_toml.package.name),
        false,
        config,
        &out_dir,
    )?;

    Ok(())
}

fn library_name(crate_name: &str, debug: bool) -> String {
    if cfg!(target_os = "windows") {
        if debug {
            format!("{}.dll", crate_name)
        } else {
            format!("{}.dll", crate_name)
        }
    } else if cfg!(target_os = "macos") {
        if debug {
            format!("lib{}.dylib", crate_name)
        } else {
            format!("lib{}.dylib", crate_name)
        }
    } else {
        // Assume Linux
        if debug {
            format!("lib{}.so", crate_name)
        } else {
            format!("lib{}.so", crate_name)
        }
    }
}

fn build_rust_library(
    debug: bool,
    project_root: &std::path::Path,
    crate_name: &str,
) -> anyhow::Result<PathBuf> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build");

    if !debug {
        cmd.arg("--release");
    }


    let status = cmd.current_dir(project_root).status()?;

    anyhow::ensure!(status.success(), "Failed to build the Rust library");

    let target = project_root
        .join("target")
        .join(if debug { "debug" } else { "release" })
        .join(library_name(crate_name, debug));

    if !target.exists() {
        anyhow::bail!("Expected library not found at {:?} after build", target);
    }
    if !target.is_file() {
        anyhow::bail!("Expected library path {:?} is not a file", target);
    }

    Ok(target)
}
