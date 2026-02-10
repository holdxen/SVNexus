pub mod context;
pub mod stream;
pub mod version;
mod wc;

#[cfg(test)]
mod tests;

use crate::apr;

#[allow(bad_style)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub mod ffi {
    pub use crate::apr::ffi::*;
    include!(concat!(env!("OUT_DIR"), "/subversion.rs"));
}

// pub struct WorkingCopyNotify {
//     path: String,
//     action: ffi::svn_wc_notify_action_t,
//     kind: ffi::svn_node_kind_t,
//     mime_type: Option<String>,
// }

// pub struct WCNotify {
//     path: String,
//     action: ffi::svn_wc_notify_action_t,
//     kind: ffi::svn_node_kind_t,
//     mime_type: Option<String>,
// }

#[derive(Debug, Clone, uniffi::Record)]
pub struct ErrorInfo {
    code: i32,
    msg: String,
    file: String,
    line: i32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SVNError {
    pub msg: String,
    pub code: i32,
    pub info: Vec<ErrorInfo>,
}

impl std::error::Error for SVNError {}

impl std::fmt::Display for SVNError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.msg, self.code)
    }
}

impl SVNError {
    fn from_nullable_ptr(err: *mut ffi::svn_error_t) -> Result<(), Self> {
        if err.is_null() {
            Ok(())
        } else {
            let result = Err(Self::from_not_null_ptr(err));

            unsafe {
                ffi::svn_error_clear(err);
            }

            result
        }
    }

    fn from_not_null_ptr(err: *const ffi::svn_error_t) -> Self {
        unsafe {
            assert!(!err.is_null());

            let mut buf = vec![0; 512];

            let msg = ffi::svn_err_best_message(err, buf.as_mut_ptr(), buf.len() as _);

            let msg = apr::char_array_to_string(msg).unwrap();

            let code = err.as_ref().unwrap().apr_err as i32;

            // let err = unsafe { &*err };

            let mut next = err;

            let mut info = vec![];

            while !next.is_null() {
                let err = next.as_ref().unwrap();

                let msg = apr::char_array_to_string(err.message).unwrap_or_default();

                let file = apr::char_array_to_string(err.file).unwrap_or_default();
                let error_info = ErrorInfo {
                    code: err.apr_err as _,
                    msg,
                    file,
                    line: err.line as _,
                };
                info.push(error_info);

                next = err.child;
            }


            Self { msg, code, info }
        }
    }
}

const fn svn_no_error() -> *mut ffi::svn_error_t {
    ffi::SVN_NO_ERROR as *mut _
}

pub fn encode_base64(bytes: &[u8], break_lines: bool) -> String {
    let mut stream = stream::Stream::create(Default::default());

    let mut input = stream.base64(break_lines);

    input.write(bytes).unwrap();

    input.close().unwrap();

    String::from_utf8(stream.take_write_buffer()).unwrap_or_default()

    // unsafe {
    //     let mut pool = apr::Pool::create();
    //     let string = ffi::svn_string_create(
    //         CString::from_str(string).unwrap().as_ptr(),
    //         pool.as_mut_ptr(),
    //     );

    //     let base64 = ffi::svn_base64_encode_string2(string, 1, pool.as_mut_ptr())
    //         .as_ref()
    //         .unwrap();
    //     std::slice::from_raw_parts(base64.data as *const u8, base64.len.try_into().unwrap())
    //         .to_vec()
    // }
}
