use std::path::PathBuf;

use camino::Utf8PathBuf;
use clap::Parser;
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

    #[serde(rename = "AllowUnsafeBlocks")]
    allow_unsafe_blocks: Option<bool>,
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

// impl Project {
//     fn copy_library(&self, project_root: &Path, library_path: &Path) -> anyhow::Result<()> {
//         let frameworks = self.property_group.frameworks();
//         if frameworks.is_empty() {
//             anyhow::bail!("No target frameworks found in the .csproj file");
//         }

//         let file_name = library_path.file_name().ok_or_else(|| {
//             anyhow::anyhow!(
//                 "Failed to get file name from library path: {:?}",
//                 library_path
//             )
//         })?;

//         for framework in frameworks {
//             let output_dir = project_root.join("bin").join("Debug").join(&framework);
//             std::fs::create_dir_all(&output_dir)?;

//             std::fs::copy(library_path, output_dir.join(file_name))?;

//             let output_dir = project_root.join("bin").join("Release").join(&framework);
//             std::fs::create_dir_all(&output_dir)?;

//             std::fs::copy(library_path, output_dir.join(file_name))?;
//         }

//         Ok(())
//     }
// }

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

// fn prepare_checking() -> anyhow::Result<()> {
//     // Check for .csproj file
//     // Check for rust directory with Cargo.toml
//     // Parse .csproj to get target frameworks
//     // Parse Cargo.toml to get package info
//     let output = std::process::Command::new("cargo").arg("-V").output()?;

//     anyhow::ensure!(
//         output.status.success(),
//         "Cargo is not installed or not found in PATH"
//     );

//     Ok(())
// }

const OUT_DIRECTORY_NAME: &str = "Generated";

#[derive(Debug, Parser)]
pub struct Command {
    #[arg(long)]
    release: bool,
    #[arg(long)]
    format: bool,
}

pub struct Generator {
    project: Project,
    cargo_toml: CargoToml,
    csharp_root: PathBuf,
    rust_root: PathBuf,
    cmd: Command,
    config: Option<PathBuf>,
}

impl Generator {
    fn find() -> anyhow::Result<Self> {
        let cmd = Command::parse();

        let mut config = None;
        let mut csproj = None;
        let mut rust = None;

        let current_dir = std::env::current_dir()?;

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

        let generator = Generator {
            project,
            cargo_toml,
            csharp_root: current_dir,
            rust_root: rust.path().to_path_buf(),
            cmd,
            config: config.map(|e| e.into_path()),
        };

        Ok(generator)
    }

    fn prepare() -> anyhow::Result<()> {
        let output = std::process::Command::new("cargo").arg("-V").output()?;

        anyhow::ensure!(
            output.status.success(),
            "Cargo is not installed or not found in PATH"
        );

        let text = String::from_utf8(output.stdout).unwrap_or_default();

        let version = text.trim();

        println!("Found Cargo version: {}", version);

        Ok(())
    }

    fn check(&self) -> anyhow::Result<()> {
        let frameworks = self.project.property_group.frameworks();
        if frameworks.is_empty() {
            anyhow::bail!("No target frameworks found in the .csproj file");
        }

        println!(
            "Target frameworks found in .csproj: {}",
            frameworks.join(";")
        );

        if self
            .project
            .property_group
            .allow_unsafe_blocks
            .is_none_or(|v| !v)
        {
            println!(
                "Warning: 'AllowUnsafeBlocks' is not set to true in the .csproj file. This may cause issues if the generated code contains unsafe blocks."
            );
        }

        self.cargo_toml.print_info();

        println!(
            "Building Rust library in {:?} mode...",
            if self.cmd.release { "release" } else { "debug" }
        );

        Ok(())
    }

    fn build_rust_library(&self) -> anyhow::Result<()> {
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("build");

        if self.cmd.release {
            cmd.arg("--release");
        }

        let status = cmd.current_dir(&self.rust_root).status()?;

        anyhow::ensure!(status.success(), "Failed to build the Rust library");

        Ok(())
    }

    fn rust_library_name(&self) -> String {
        let debug = !self.cmd.release;
        let crate_name = &self.cargo_toml.package.name;
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

    fn rust_library_path(&self) -> PathBuf {
        self.rust_root
            .join("target")
            .join(if self.cmd.release { "release" } else { "debug" })
            .join(self.rust_library_name())
    }

    fn copy_rust_library(&self) -> anyhow::Result<()> {
        let file_name = self.rust_library_name();

        let target = self.rust_library_path();

        if !target.exists() {
            anyhow::bail!("Expected library not found at {:?} after build", target);
        }
        if !target.is_file() {
            anyhow::bail!("Expected library path {:?} is not a file", target);
        }

        // let file_name = target.file_name().ok_or_else(|| {
        //     anyhow::anyhow!("Failed to get file name from library path: {:?}", target)
        // })?;

        for framework in self.project.property_group.frameworks() {
            let output_dir = self.csharp_root.join("bin").join("Debug").join(&framework);
            std::fs::create_dir_all(&output_dir)?;

            std::fs::copy(&target, output_dir.join(&file_name))?;

            let output_dir = self
                .csharp_root
                .join("bin")
                .join("Release")
                .join(&framework);
            std::fs::create_dir_all(&output_dir)?;

            std::fs::copy(&target, output_dir.join(&file_name))?;
        }

        Ok(())
    }

    fn generate(&self) -> anyhow::Result<()> {
        std::env::set_current_dir(&self.rust_root)?;

        let out_dir = self.csharp_root.join(OUT_DIRECTORY_NAME);

        std::fs::create_dir_all(&out_dir)?;

        let out_dir = Utf8PathBuf::from_path_buf(out_dir.canonicalize()?)
            .expect("Invalid output directory path");

        let library = Utf8PathBuf::from_path_buf(self.rust_library_path().canonicalize()?)
            .expect("Invalid library path");


        let mut config = None;

        if let Some(ref config_path) = self.config {
            config = Some(
                Utf8PathBuf::from_path_buf(config_path.canonicalize()?)
                    .expect("Invalid config file path"),
            );
        }

        std::env::set_current_dir(&self.rust_root)?;

        uniffi_bindgen_cs::generate_from_library(
            &library,
            Some(self.cargo_toml.package.name.clone()),
            self.cmd.format,
            config.as_ref().map(|e| e.as_path()),
            &out_dir,
        )?;

        Ok(())
    }

    fn exec(&self) -> anyhow::Result<()> {
        self.check()?;
        self.build_rust_library()?;
        self.copy_rust_library()?;
        self.generate()?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    Generator::prepare()?;

    let generator = Generator::find()?;

    generator.exec()?;

    Ok(())
}

// fn main() -> anyhow::Result<()> {
//     prepare_checking()?;

//     let current_dir = std::env::current_dir()?;

//     let mut config = None;
//     let mut csproj = None;
//     let mut rust = None;

//     for i in walkdir::WalkDir::new(&current_dir).max_depth(1) {
//         let Ok(entry) = i else {
//             continue;
//         };

//         if !entry.file_type().is_file() {
//             if entry.file_type().is_dir() {
//                 if entry.file_name() == "rust" {
//                     rust = Some(entry);
//                 }
//             }
//             continue;
//         }

//         if entry.file_name() == "uniffi.toml" {
//             config = Some(entry)
//         } else if entry.file_name().to_string_lossy().ends_with(".csproj") {
//             if csproj.is_some() {
//                 anyhow::bail!("Multiple .csproj files found in the current directory");
//             }
//             csproj = Some(entry)
//         }
//     }

//     let Some(csproj) = csproj else {
//         anyhow::bail!("No .csproj file found in the current directory");
//     };

//     let Some(rust) = rust else {
//         anyhow::bail!("No 'rust' directory found in the current directory");
//     };

//     let csproj_content = std::fs::read_to_string(csproj.path())?;

//     let project = Project::parse_from_xml(&csproj_content)?;

//     let cargo_toml_path = rust.path().join("Cargo.toml");

//     let cargo_toml_content = std::fs::read_to_string(cargo_toml_path)?;

//     let cargo_toml = CargoToml::parse_from_toml(&cargo_toml_content)?;

//     cargo_toml.print_info();

//     let library =
//         build_rust_library(true, rust.path(), &cargo_toml.package.name)?.canonicalize()?;

//     project.copy_library(&current_dir, &library)?;

//     let library = Utf8PathBuf::from_path_buf(library).expect("Invalid library path");

//     let config = config.map(|e| {
//         Utf8PathBuf::from_path_buf(
//             e.into_path()
//                 .canonicalize()
//                 .expect("Failed to canonicalize"),
//         )
//         .expect("")
//     });

//     let config = config.as_ref().map(|e| e.as_path());

//     let out_dir = current_dir.join(OUT_DIRECTORY_NAME);

//     std::fs::create_dir_all(&out_dir)?;

//     // let rust = Utf8PathBuf::from_path_buf(rust.path().to_path_buf())
//     //     .expect("Invalid rust directory path");

//     let out_dir =
//         Utf8PathBuf::from_path_buf(out_dir.canonicalize()?).expect("Invalid output directory path");

//     std::env::set_current_dir(rust.path())?;

//     uniffi_bindgen_cs::generate_from_library(
//         &library,
//         Some(cargo_toml.package.name),
//         false,
//         config,
//         &out_dir,
//     )?;

//     Ok(())
// }

// fn library_name(crate_name: &str, debug: bool) -> String {
//     if cfg!(target_os = "windows") {
//         if debug {
//             format!("{}.dll", crate_name)
//         } else {
//             format!("{}.dll", crate_name)
//         }
//     } else if cfg!(target_os = "macos") {
//         if debug {
//             format!("lib{}.dylib", crate_name)
//         } else {
//             format!("lib{}.dylib", crate_name)
//         }
//     } else {
//         // Assume Linux
//         if debug {
//             format!("lib{}.so", crate_name)
//         } else {
//             format!("lib{}.so", crate_name)
//         }
//     }
// }

// fn build_rust_library(
//     debug: bool,
//     project_root: &std::path::Path,
//     crate_name: &str,
// ) -> anyhow::Result<PathBuf> {
//     let mut cmd = std::process::Command::new("cargo");
//     cmd.arg("build");

//     if !debug {
//         cmd.arg("--release");
//     }

//     let v: Vec<Vec<u8>> = vec![Vec::new(); 10];

//     let status = cmd.current_dir(project_root).status()?;

//     anyhow::ensure!(status.success(), "Failed to build the Rust library");

//     let target = project_root
//         .join("target")
//         .join(if debug { "debug" } else { "release" })
//         .join(library_name(crate_name, debug));

//     if !target.exists() {
//         anyhow::bail!("Expected library not found at {:?} after build", target);
//     }
//     if !target.is_file() {
//         anyhow::bail!("Expected library path {:?} is not a file", target);
//     }

//     Ok(target)
// }
