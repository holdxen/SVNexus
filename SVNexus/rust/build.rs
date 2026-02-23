use core::panic;
use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

// fn windows_deps() -> Vec<String> {

//     vec![
//         "libsvn_client-1".to_string(),
//         "libsvn_delta-1".to_string(),
//         "libsv_diff-1".to_string(),
//         "libsvn_fs_fs-1".to_string(),
//         "libsvn_fs_util-1".to_string(),
//         "libsvn_fs_x-1".to_string(),
//     ]
// }

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

#[allow(unreachable_code)]
fn svn_path() -> String {
    #[cfg(target_os = "windows")]
    {
        // #[cfg(target_pointer_width = "64")]
        // {
        //     return ".\\deps\\win-x64\\svn".to_string();
        // }
        //
        #[cfg(target_arch = "x86_64")]
        return ".\\deps\\win-x64\\svn".to_string();
    }

    #[cfg(target_os = "macos")]
    {
        #[cfg(target_arch = "aarch64")]
        return "./deps/macos-aarch64/svn".to_string();
    }

    #[cfg(target_os = "linux")]
    {
        #[cfg(target_arch = "x86_64")]
        return "./deps/linux-x64/svn".to_string();
    }

    panic!("Unsupported os")
}

#[allow(unreachable_code)]
fn library_suffix() -> String {
    #[cfg(target_os = "windows")]
    {
        return ".lib".to_string();
    }

    #[cfg(target_os = "macos")]
    {
        return ".dylib".to_string();
    }

    panic!("Unsupported os")
}

fn link_library(svn_path: &str) {
    let path = PathBuf::from(svn_path).join("lib");
    set_library_search_path(path.to_str().unwrap());

    for i in WalkDir::new(path) {
        if let Ok(i) = i {
            if i.file_type().is_file() {
                let file_name = i.file_name().to_str().unwrap();
                let size = file_name.chars().filter(|c| *c == '.').count();
                eprintln!("file_name={}, size = {}", file_name, size);
                if file_name.ends_with(library_suffix().as_str()) {
                    add_link_library(
                        file_name
                            .trim_start_matches("lib")
                            .trim_end_matches(library_suffix().as_str()),
                    );
                }
            }
        }
    }
}

fn generate_subversion(path: &str) {
    let path = PathBuf::from(path);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = PathBuf::from(out_dir);

    let include_path = path.join("include");

    let svn_include_path = include_path.join("subversion-1");

    let mut builder = bindgen::builder();

    // builder = builder.header(svn_include_path.join("svn_fs.h").to_str().unwrap());

    for i in WalkDir::new(svn_include_path) {
        if let Ok(i) = i {
            let file_name = i.file_name().to_str().unwrap();
            if i.file_type().is_file() && file_name.ends_with(".h") && file_name.starts_with("svn_")
            {
                builder = builder.header(i.path().to_str().unwrap());
            }
        }
    }

    builder = builder
        .clang_arg(format!("-I{}", include_path.to_str().unwrap()))
        .clang_arg(format!("-I{}/apr-1", include_path.display()))
        .clang_arg(format!("-I{}/subversion-1", include_path.display()))
        .clang_arg(format!("-I{}/serf-1", include_path.display()))
        .derive_default(true)
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .generate_comments(false)
        .allowlist_file(".*svn_.*\\.h")
        .blocklist_file(".*apr_.*\\.h");

    let bindings = builder.generate().expect("Failed to generate bindings");

    bindings
        .write_to_file(out_dir.join("subversion.rs"))
        .expect("Failed to write file");
}

fn generate_apr(path: &str) {
    let path = PathBuf::from(path);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = PathBuf::from(out_dir);

    let include_path = path.join("include");

    let mut builder = bindgen::builder();

    // builder = builder.header(svn_include_path.join("svn_fs.h").to_str().unwrap());

    for i in WalkDir::new(&include_path) {
        if let Ok(i) = i {
            let file_name = i.file_name().to_str().unwrap();
            if i.file_type().is_file() && file_name.ends_with(".h") && file_name.starts_with("apr_")
            {
                builder = builder.header(i.path().to_str().unwrap());
            }
        }
    }

    builder = builder
        .clang_arg(format!("-I{}", include_path.to_str().unwrap()))
        .clang_arg(format!("-I{}/apr-1", include_path.display()))
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .derive_default(true)
        .generate_comments(false)
        .allowlist_file(".*apr_.*\\.h");

    let bindings = builder.generate().expect("Failed to genrate bindings");
    bindings
        .write_to_file(out_dir.join("apr.rs"))
        .expect("Failed to write file");
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
            builder = builder.header(relative_path.join(i).to_str().unwrap());
        }

        for i in self.include_paths.iter() {
            builder = builder.clang_arg(format!("-I{}", relative_path.join(i).to_str().unwrap()));
        }

        for i in self.link_paths.iter() {
            set_library_search_path(relative_path.join(i).to_str().unwrap());
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
        let out_dir = std::env::var("OUT_DIR").unwrap();
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

fn main() {
    let path = svn_path();

    let options = deserialize_complie_options(Path::new(path.as_str()).join("compile.json"));

    options.compile(path);

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        // rpath = directory containing this cdylib
        println!("cargo:rustc-link-arg-cdylib=-Wl,-rpath,$ORIGIN");
        // 如果依赖在子目录，比如 lib/：
        // println!("cargo:rustc-link-arg-cdylib=-Wl,-rpath,$ORIGIN/lib");
    }

    // generate_apr(&path);

    // generate_subversion(&path);

    // link_library(&path);
}
