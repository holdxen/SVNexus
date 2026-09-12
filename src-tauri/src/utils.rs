use std::ffi::{c_char, c_void, CStr};

use crate::subversion;

#[easy_ext::ext(SubversionStringer)]
pub impl *const subversion::ffi::svn_string_t {
    unsafe fn to_str<'a>(self) -> &'a str {
        unsafe {
            let slice = self.to_slice();

            std::str::from_utf8(slice).expect("Invalid UTF-8 data")
        }
    }
    unsafe fn to_nullable_string(self) -> Option<String> {
        if self.is_null() {
            None
        } else {
            unsafe { Some(self.to_str().to_string()) }
        }
    }

    #[cfg(false)]
    unsafe fn to_nullable_str<'a>(self) -> Option<&'a str> {
        if self.is_null() {
            None
        } else {
            unsafe { Some(self.to_str()) }
        }
    }

    unsafe fn to_slice<'a>(self) -> &'a [u8] {
        assert!(!self.is_null());

        unsafe {
            let ptr = self.as_ref().expect("Failed to get reference from pointer");

            assert!(!ptr.data.is_null());

            let slice = std::slice::from_raw_parts(
                ptr.data as *const u8,
                ptr.len.try_into().expect("Unexpected svn_string_t length"),
            );

            slice
        }
    }

    #[cfg(false)]
    unsafe fn to_nullable_slice<'a>(self) -> Option<&'a [u8]> {
        if self.is_null() {
            None
        } else {
            unsafe { Some(self.to_slice()) }
        }
    }
}

#[easy_ext::ext(CStringer)]
pub impl *const c_char {
    #[track_caller]
    unsafe fn to_str<'a>(self) -> &'a str {
        assert!(!self.is_null(), "Expected non-null pointer");
        unsafe { CStr::from_ptr(self).to_str().expect("Invalid UTF-8 data") }
    }
    unsafe fn to_nullable_str<'a>(self) -> Option<&'a str> {
        if self.is_null() {
            None
        } else {
            unsafe { Some(self.to_str()) }
        }
    }

    unsafe fn to_nullable_string(self) -> Option<String> {
        if self.is_null() {
            None
        } else {
            unsafe { Some(self.to_str().to_string()) }
        }
    }

    #[cfg(false)]
    unsafe fn to_slice<'a>(self) -> &'a [u8] {
        assert!(!self.is_null(), "Expected non-null pointer");

        unsafe { CStr::from_ptr(self).to_bytes() }
    }

    #[cfg(false)]
    unsafe fn to_nullable_slice<'a>(self) -> Option<&'a [u8]> {
        if self.is_null() {
            None
        } else {
            unsafe { Some(self.to_slice()) }
        }
    }
}

#[easy_ext::ext(Boxed)]
pub impl<T> Box<T> {
    #[cfg(false)]
    fn inner_void_pointer(&self) -> *const c_void {
        let inner: &T = &**self;
        inner as *const T as *const c_void
    }

    fn inner_void_pointer_mut(&mut self) -> *mut c_void {
        let inner: &mut T = &mut **self;
        inner as *mut T as *mut c_void
    }
}

#[easy_ext::ext(Pointer)]
pub impl<T> T {
    fn pointer_mut(&mut self) -> *mut T {
        self as *mut _
    }

    fn pointer(&self) -> *const T {
        self as *const _
    }
}

#[easy_ext::ext(PointerMapper)]
pub impl<T> *const T {
    fn map<V>(self, f: impl FnOnce(*const T) -> V) -> Option<V> {
        if self.is_null() {
            None
        } else {
            Some(f(self))
        }
    }
}

#[easy_ext::ext(PointerMutMapper)]
pub impl<T> *mut T {
    fn map_mut<V>(self, f: impl FnOnce(*mut T) -> V) -> Option<V> {
        if self.is_null() {
            None
        } else {
            Some(f(self))
        }
    }
}

// #[derive(Debug)]
// pub struct FormatSizeOptions {
//     size: u64,
// }

// impl FormatSizeOptions {
//     pub fn format(self) -> String {
//         humansize::format_size(self.size, humansize::DECIMAL)
//     }
// }

// const PROJECT_DIR: &str = env!("CARGO_MANIFEST_DIR");

// #[easy_ext::ext]
// impl<'a> &'a str {
//     fn project_relative_path(self) -> Option<&'a str> {
//         let path = std::path::Path::new(self);

//         let project_dir = std::path::Path::new(PROJECT_DIR).parent()?.parent()?;

//         path.strip_prefix(project_dir).ok()?.to_str()
//     }
// }

const LOG_TARGET: &str = "webview";

pub fn log_info(line: i32, file: &str, _member: &str, content: &str) {
    use log::{Level, Record};

    let content = format_args!("{}", content);

    let rec = Record::builder()
        .args(content)
        .level(Level::Info)
        .target(LOG_TARGET)
        .file(Some(file))
        .line(Some(line as u32))
        .build();

    let logger = log::logger();
    if logger.enabled(rec.metadata()) {
        logger.log(&rec);
    }
}

pub fn log_error(line: i32, file: &str, _member: &str, content: &str) {
    use log::{Level, Record};

    let content = format_args!("{}", content);

    let rec = Record::builder()
        .args(content)
        .level(Level::Error)
        .target(LOG_TARGET)
        .file(Some(file))
        .line(Some(line as u32))
        .build();

    let logger = log::logger();
    if logger.enabled(rec.metadata()) {
        logger.log(&rec);
    }
}

pub fn log_trace(line: i32, file: &str, _member: &str, content: &str) {
    use log::{Level, Record};

    let content = format_args!("{}", content);

    let rec = Record::builder()
        .args(content)
        .level(Level::Trace)
        .target(LOG_TARGET)
        .file(Some(file))
        .line(Some(line as u32))
        .build();

    let logger = log::logger();
    if logger.enabled(rec.metadata()) {
        logger.log(&rec);
    }
}

pub fn log_debug(line: i32, file: &str, _member: &str, content: &str) {
    use log::{Level, Record};

    let content = format_args!("{}", content);

    let rec = Record::builder()
        .args(content)
        .level(Level::Debug)
        .target(LOG_TARGET)
        .file(Some(file))
        .line(Some(line as u32))
        .build();

    let logger = log::logger();
    if logger.enabled(rec.metadata()) {
        logger.log(&rec);
    }
}

pub fn log_warn(line: i32, file: &str, _member: &str, content: &str) {
    use log::{Level, Record};

    let content = format_args!("{}", content);

    let rec = Record::builder()
        .args(content)
        .level(Level::Warn)
        .target(LOG_TARGET)
        .file(Some(file))
        .line(Some(line as u32))
        .build();

    let logger = log::logger();
    if logger.enabled(rec.metadata()) {
        logger.log(&rec);
    }
}

// fn find_which(name: &str) -> error::Result<Option<String>> {
//     let path = which::which(name);

//     let Ok(path) = path else {
//         return Ok(None);
//     };

//     path.to_str()
//         .map(|s| s.to_string().into_option_some())
//         .context(builder::General {
//             detail: "Invalid path",
//         })
// }
