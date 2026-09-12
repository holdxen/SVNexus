fn main() {
    tauri_build::build();
    subversion();
}

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

fn set_library_search_path(path: &str) {
    println!("cargo:rustc-link-search=native={}", path);
}

fn add_link_library(lib: &str) {
    eprintln!(
        "link: >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>{}",
        lib
    );
    println!("cargo:rustc-link-lib={}", lib);
}

fn svn_path() -> String {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("Unexpected failure");
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").expect("Unexpected failure");

    if target_os == "windows" {
        if target_arch == "x86_64" {
            return ".\\deps\\win-x64\\svn".to_string();
        }
    } else if target_os == "linux" {
        if target_arch == "x86_64" {
            return "./deps/linux-x64/svn".to_string();
        } else if target_arch == "aarch64" {
            return "./deps/linux-aarch64/svn".to_string();
        } else if target_arch == "loongarch64" {
            return "./deps/linux-loongarch64/svn".to_string();
        }
    } else if target_os == "macos" {
        if target_arch == "aarch64" {
            return "./deps/macos-aarch64/svn".to_string();
        }
    }

    panic!("Unsupported os or arch: {}({})", target_os, target_arch)

    // use cfg_if::cfg_if;
    // cfg_if! {
    //     if #[cfg(target_os = "windows")] {
    //         cfg_if! {
    //             if #[cfg(target_arch = "x86_64")] {
    //                 ".\\deps\\win-x64\\svn".to_string()
    //             } else {
    //                 panic!("Unsupported Windows arch")
    //             }
    //         }
    //     } else if #[cfg(target_os = "macos")] {
    //         panic!("not macos test");
    //         cfg_if! {
    //             if #[cfg(target_arch = "aarch64")] {
    //                 "./deps/macos-aarch64/svn".to_string()
    //             } else {
    //                 panic!("Unsupported macOS arch")
    //             }
    //         }
    //     } else if #[cfg(target_os = "linux")] {
    //         cfg_if! {
    //             if #[cfg(target_arch = "x86_64")] {
    //                 "./deps/linux-x64/svn".to_string()
    //             } else if #[cfg(target_arch = "aarch64")] {
    //                 "./deps/linux-aarch64/svn".to_string()
    //             } else {
    //                 panic!("Unsupported Linux arch")
    //             }
    //         }
    //     } else {
    //         panic!("Unsupported os")
    //     }
    // }
}

#[derive(Serialize, Deserialize)]
pub struct CompileOptions {
    targets: CompileTargets,
    built_on: String,
}

#[derive(Serialize, Deserialize)]
pub struct CompileTargets {
    svn: CompileTarget,
    apr: CompileTarget,
}

#[derive(Serialize, Deserialize)]
pub struct CompileTarget {
    include_paths: Vec<String>,
    link_paths: Vec<String>,
    libraries: Vec<String>,
    headers: Vec<String>,
    blocklist_file: Vec<String>,
    allowlist_file: Vec<String>,
}

impl CompileTarget {
    fn compile(&self, relative_path: impl AsRef<Path>, out_file: impl AsRef<Path>) {
        let mut builder = bindgen::builder()
            .derive_default(true)
            .generate_comments(false);

        let relative_path = relative_path.as_ref();

        for i in self.headers.iter() {
            eprintln!("add header: {:?}", i);
            builder = builder.header(relative_path.join(i).to_str().expect("Invalid UTF-8 string"));
        }

        for i in self.include_paths.iter() {
            builder = builder.clang_arg(format!("-I{}", relative_path.join(i).to_str().expect("Invalid UTF-8 string")));
        }

        for i in self.link_paths.iter() {
            set_library_search_path(relative_path.join(i).to_str().expect("Invalid UTF-8 string"));
        }

        for i in self.libraries.iter() {
            add_link_library(i);
        }
        for i in self.allowlist_file.iter() {
            builder = builder.allowlist_file(i);
        }

        for i in self.blocklist_file.iter() {
            builder = builder.blocklist_file(i);
        }

        let bindings = builder.generate().expect("Failed to genrate bindings");

        bindings
            .write_to_file(out_file)
            .expect("Failed to write file");
    }
}

impl CompileOptions {
    fn compile(&self, relative_path: impl AsRef<Path>) {
        let out_dir = std::env::var("OUT_DIR").expect("Unexpected failure");
        let out_dir = PathBuf::from(out_dir);
        self.targets
            .svn
            .compile(relative_path.as_ref(), out_dir.join("subversion.rs"));
        self.targets
            .apr
            .compile(relative_path.as_ref(), out_dir.join("apr.rs"));
    }
}

fn deserialize_complie_options(json_file: impl AsRef<Path>) -> CompileOptions {
    let content = fs::read(json_file).expect("Failed to read compile.json");

    serde_json::from_slice(&content).expect("Invalid json file format")
}

fn subversion() {
    let path = svn_path();

    let path = PathBuf::from(path);

    let mut options = deserialize_complie_options(path.join("compile.json"));

    options
        .targets
        .apr
        .headers
        .push("../../../src/patches/apr_patch.h".to_string());
    options
        .targets
        .svn
        .headers
        .push("../../../src/patches/svn_patch.h".to_string());

    options.compile(&path);

    cc::Build::new()
        .file("src/patches/apr_patch.c")
        .file("src/patches/svn_patch.c")
        .include("src/patches")
        .includes(
            options
                .targets
                .apr
                .include_paths
                .iter()
                .map(|v| path.join(v)),
        )
        .includes(
            options
                .targets
                .svn
                .include_paths
                .iter()
                .map(|v| path.join(v)),
        )
        .compile("patches");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("Unexpected failure");

    if target_os == "macos" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/svnexus-svn/lib");
    } else if target_os == "linux" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/svnexus-svn/lib");
    }
}
