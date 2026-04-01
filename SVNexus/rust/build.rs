use core::panic;
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
