use crate::{subversion, utils::CStringer, utils::SubversionStringer};

use super::error::builder;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::{CStr, CString, c_char, c_int, c_void},
    hash::Hash,
    sync::OnceLock,
};

use super::error;

#[allow(bad_style)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
#[allow(unnecessary_transmutes)]
#[allow(unsafe_op_in_unsafe_fn)]
pub mod ffi {
    include!(concat!(env!("OUT_DIR"), "/apr.rs"));
}

pub fn initialize() -> error::Result<()> {
    static INIT: once_cell::sync::OnceCell<()> = once_cell::sync::OnceCell::new();
    INIT.get_or_try_init(|| {
        let status = unsafe { ffi::apr_initialize() };
        if status != 0 {
            return builder::General {
                detail: format!("Failed to initialize apache portable runtime: {}", status),
            }
            .fail();
        }
        error::ok(())
    })?;
    Ok(())
}

pub unsafe fn char_array_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(ptr) }.to_str().ok()?.to_string())
    }
}

// pub struct Map {
//     ptr: *mut ffi::apr_hash_t,
//     pool: Pool,
// }

// impl Map {
//     fn new() -> Self {
//         let pool = PoolFactory::instance().create_pool();
//         let ptr = unsafe { ffi::apr_hash_make(pool.ptr) };

//         Self { ptr, pool }
//     }
// }

// pub struct Array<T> {
//     ptr: *mut ffi::apr_array_header_t,
//     pool: Pool,
//     _p: PhantomData<T>,
// }

// impl Array<*const c_char> {
//     pub unsafe fn to_string_list(ptr: *const ffi::apr_array_header_t) -> Vec<String> {
//         let array = unsafe { ptr.as_ref().unwrap() };

//         let mut ret = Vec::with_capacity(array.nelts.try_into().unwrap());

//         let elements = array.elts as *const *const c_char;

//         for i in 0..array.nelts {
//             let element = unsafe { elements.offset(i.try_into().unwrap()).read() };

//             let string = unsafe { CStr::from_ptr(element) }
//                 .to_str()
//                 .unwrap()
//                 .to_owned();

//             ret.push(string)
//         }

//         ret
//     }

//     pub unsafe fn from_string_list<T, I>(len: usize, string_list: T) -> error::Result<Self>
//     where
//         T: IntoIterator<Item = I>,
//         I: AsRef<str>,
//     {
//         let mut this = Array::with_capacity(len);

//         let array = unsafe {
//             ffi::apr_array_make(
//                 this.pool.ptr,
//                 len.try_into().unwrap(),
//                 size_of::<usize>().try_into().unwrap(),
//             )
//         };

//         for i in string_list {
//             let string = unsafe { this.pool.string(i.as_ref())? };

//             let array = unsafe { ffi::apr_array_push(array) } as *mut *const c_char;

//             unsafe {
//                 std::ptr::copy(
//                     &string as _,
//                     array as _,
//                     size_of::<*const c_char>().try_into().unwrap(),
//                 )
//             };
//         }

//         Ok(this)
//     }
// }

// impl<T> Array<T> {
//     pub fn with_capacity(len: usize) -> Self {
//         let pool = PoolFactory::instance().create_pool();
//         let ptr = unsafe {
//             let ptr = ffi::apr_array_make(
//                 pool.ptr,
//                 len.try_into().unwrap(),
//                 size_of::<T>().try_into().unwrap(),
//             );
//             ptr
//         };

//         Self {
//             ptr,
//             pool,
//             _p: Default::default(),
//         }
//     }

//     pub fn push(&mut self, value: T) {
//         let array = unsafe { ffi::apr_array_push(self.ptr) } as *mut T;

//         unsafe { std::ptr::copy(&value as _, array as _, size_of::<T>().try_into().unwrap()) };

//         std::mem::forget(value);
//     }

//     pub fn as_mut_ptr(&mut self) -> *mut ffi::apr_array_header_t {
//         self.ptr
//     }
// }

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
pub struct AprError {
    pub code: ErrorCode,
    pub msg: String,
}

impl std::error::Error for AprError {}

impl std::fmt::Display for AprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl AprError {
    fn check_error(code: c_int) -> Result<(), Self> {
        if code == 0 {
            return Ok(());
        } else {
            todo!()
        }
    }
    fn from_code(code: ErrorCode) -> Self {
        let mut buf = vec![0; 512];
        unsafe { ffi::apr_strerror(code as _, buf.as_mut_ptr() as _, buf.len() as _) };

        let cstring = CString::from_vec_with_nul(buf).unwrap();

        Self {
            code,
            msg: cstring.to_str().unwrap().to_string(),
        }
    }
}

#[svnexus_macro::enum_converter(repr_type=u32)]
#[derive(Clone, Debug, Copy, Serialize, Deserialize, uniffi::Enum)]
pub enum ErrorCode {
    ENOSTAT = ffi::APR_ENOSTAT,
    ENOPOOL = ffi::APR_ENOPOOL,
    EBADDATE = ffi::APR_EBADDATE,
    EINVALSOCK = ffi::APR_EINVALSOCK,
    ENOPROC = ffi::APR_ENOPROC,
    ENOTIME = ffi::APR_ENOTIME,
    ENOLOCK = ffi::APR_ENOLOCK,
    ENOPOLL = ffi::APR_ENOPOLL,
    ENOSOCKET = ffi::APR_ENOSOCKET,
    ENOTHREAD = ffi::APR_ENOTHREAD,
    ENOTHDKEY = ffi::APR_ENOTHDKEY,
    EGENERAL = ffi::APR_EGENERAL,
    ENOSHMAVAIL = ffi::APR_ENOSHMAVAIL,
    EBADIP = ffi::APR_EBADIP,
    EBADMASK = ffi::APR_EBADMASK,
    EDSOOPEN = ffi::APR_EDSOOPEN,
    EABSOLUTE = ffi::APR_EABSOLUTE,
    ERELATIVE = ffi::APR_ERELATIVE,
    EINCOMPLETE = ffi::APR_EINCOMPLETE,
    EABOVEROOT = ffi::APR_EABOVEROOT,
    EBADPATH = ffi::APR_EBADPATH,
    EPATHWILD = ffi::APR_EPATHWILD,
    ESYMNOTFOUND = ffi::APR_ESYMNOTFOUND,
    EPROCUnknown = ffi::APR_EPROC_UNKNOWN,
    ENOTENOUGHENTROPY = ffi::APR_ENOTENOUGHENTROPY,
    INCHILD = ffi::APR_INCHILD,
    INPARENT = ffi::APR_INPARENT,
    DETACH = ffi::APR_DETACH,
    NOTDETACH = ffi::APR_NOTDETACH,
    ChildDone = ffi::APR_CHILD_DONE,
    ChildNotDone = ffi::APR_CHILD_NOTDONE,
    TIMEUP = ffi::APR_TIMEUP,
    INCOMPLETE = ffi::APR_INCOMPLETE,
    BADCH = ffi::APR_BADCH,
    BADARG = ffi::APR_BADARG,
    EOF = ffi::APR_EOF,
    NOTFOUND = ffi::APR_NOTFOUND,
    ANONYMOUS = ffi::APR_ANONYMOUS,
    FILEBASED = ffi::APR_FILEBASED,
    KEYBASED = ffi::APR_KEYBASED,
    EINIT = ffi::APR_EINIT,
    ENOTIMPL = ffi::APR_ENOTIMPL,
    EMISMATCH = ffi::APR_EMISMATCH,
    EBUSY = ffi::APR_EBUSY,
    EACCES = ffi::APR_EACCES,
    EEXIST = ffi::APR_EEXIST,
    ENAMETOOLONG = ffi::APR_ENAMETOOLONG,
    ENOENT = ffi::APR_ENOENT,
    ENOTDIR = ffi::APR_ENOTDIR,
    ENOSPC = ffi::APR_ENOSPC,
    ENOMEM = ffi::APR_ENOMEM,
    EMFILE = ffi::APR_EMFILE,
    ENFILE = ffi::APR_ENFILE,
    EBADF = ffi::APR_EBADF,
    EINVAL = ffi::APR_EINVAL,
    ESPIPE = ffi::APR_ESPIPE,
    EAGAIN = ffi::APR_EAGAIN,
    EINTR = ffi::APR_EINTR,
    ENOTSOCK = ffi::APR_ENOTSOCK,
    ECONNREFUSED = ffi::APR_ECONNREFUSED,
    EINPROGRESS = ffi::APR_EINPROGRESS,
    ECONNABORTED = ffi::APR_ECONNABORTED,
    ECONNRESET = ffi::APR_ECONNRESET,
    ETIMEDOUT = ffi::APR_ETIMEDOUT,
    EHOSTUNREACH = ffi::APR_EHOSTUNREACH,
    ENETUNREACH = ffi::APR_ENETUNREACH,
    EFTYPE = ffi::APR_EFTYPE,
    EPIPE = ffi::APR_EPIPE,
    EXDEV = ffi::APR_EXDEV,
    ENOTEMPTY = ffi::APR_ENOTEMPTY,
    EAFNOSUPPORT = ffi::APR_EAFNOSUPPORT,
    EOPNOTSUPP = ffi::APR_EOPNOTSUPP,
    ERANGE = ffi::APR_ERANGE,
}

pub struct PoolFactory;

impl Drop for PoolFactory {
    fn drop(&mut self) {
        unsafe {
            ffi::apr_pool_terminate();
        }
    }
}

impl PoolFactory {
    pub fn instance() -> &'static Self {
        pub static SELF: OnceLock<PoolFactory> = OnceLock::new();

        SELF.get_or_init(|| {
            unsafe {
                ffi::apr_initialize();
            }
            PoolFactory {}
        })
    }

    pub fn create_pool(&self) -> Pool {
        let mut ptr: *mut ffi::apr_pool_t = std::ptr::null_mut();
        let ptr = unsafe {
            let status = ffi::apr_pool_create_ex(
                &mut ptr as *mut _,
                std::ptr::null_mut(),
                Some(on_pool_abort),
                std::ptr::null_mut(),
            );

            AprError::check_error(status).expect("Failed to alloc memory");
            ptr
        };
        Pool::from_raw(ptr)
    }
}

extern "C" fn on_pool_abort(code: std::ffi::c_int) -> std::ffi::c_int {
    panic!("Out of memory: {}", code);
}

pub struct AutoPool<T> {
    pub pool: Pool,
    pub value: T,
}


pub struct Pool {
    ptr: *mut ffi::apr_pool_t,
}

unsafe impl Send for Pool {}

impl Drop for Pool {
    fn drop(&mut self) {
        unsafe {
            ffi::apr_pool_destroy(self.ptr);
        }
    }
}

impl Pool {
    pub unsafe fn convert_to_hash_map(
        &mut self,
        hash: *mut ffi::apr_hash_t,
    ) -> HashMap<String, String> {
        unsafe { self.ptr.convert_to_hash_map(hash) }
    }

    pub unsafe fn create() -> Self {
        initialize().expect("Failed to initialize apr portable runtime");
        let mut ptr: *mut ffi::apr_pool_t = std::ptr::null_mut();
        let ptr = unsafe {
            let status = ffi::apr_pool_create_ex(
                &mut ptr as *mut _,
                std::ptr::null_mut(),
                Some(on_pool_abort),
                std::ptr::null_mut(),
            );

            AprError::check_error(status).expect("Failed to alloc memory");
            ptr
        };

        Pool::from_raw(ptr)
    }

    pub fn as_mut_ptr(&mut self) -> *mut ffi::apr_pool_t {
        self.ptr
    }
    pub fn from_raw(ptr: *mut ffi::apr_pool_t) -> Self {
        Self { ptr }
    }

    // pub fn create_child(&mut self) -> Pooling<'_> {
    //     let ptr: *mut *mut ffi::apr_pool_t = std::ptr::null_mut();
    //     let ptr = unsafe {
    //         ffi::apr_pool_create_ex(ptr, self.ptr, Some(on_pool_abort), std::ptr::null_mut());
    //         *ptr
    //     };
    //     Pooling::from_raw(ptr)
    // }

    pub unsafe fn malloc<T: Sized>(&mut self) -> *mut T {
        unsafe { self.as_mut_ptr().malloc() }
    }

    pub unsafe fn string<T: AsRef<[u8]>>(&mut self, value: T) -> error::Result<*mut c_char> {
        unsafe { self.ptr.string(value) }
    }

    pub unsafe fn string_hash_map<T, K, V, F1, F2>(
        &mut self,
        map: T,
        key: F1,
        value: F2,
    ) -> error::Result<*mut ffi::apr_hash_t>
    where
        T: Iterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
        F1: Fn(&mut Pool, &str) -> error::Result<*const c_char>,
        F2: Fn(&mut Pool, &str) -> error::Result<*const c_char>,
    {
        unsafe {
            let table = ffi::apr_hash_make(self.as_mut_ptr());

            for (k, v) in map {
                let k = key(self, k.as_ref())?;

                let v = value(self, v.as_ref())?;

                ffi::apr_hash_set(
                    table,
                    k as _,
                    ffi::APR_HASH_KEY_STRING.try_into().unwrap(),
                    v as _,
                )
            }

            Ok(table)
        }
    }

    pub unsafe fn string_array<T, I>(
        &mut self,
        len: usize,
        string_list: T,
    ) -> error::Result<*mut ffi::apr_array_header_t>
    where
        T: Iterator<Item = I>,
        I: AsRef<str>,
    {
        // let array = Array::with_capacity(string_list.len());
        unsafe {
            let array = ffi::apr_array_make(
                self.as_mut_ptr(),
                len.try_into().unwrap(),
                size_of::<usize>().try_into().unwrap(),
            );

            for i in string_list {
                let string = self.string(i.as_ref())?;

                let ptr = ffi::apr_array_push(array) as *mut *const c_char;

                *ptr = string;

                // std::ptr::copy(
                //     &string as _,
                //     array as _,
                //     size_of::<*const c_char>().try_into().unwrap(),
                // )
            }

            Ok(array)
        }
    }
}

#[easy_ext::ext(AprPool)]
pub impl *mut ffi::apr_pool_t {
    unsafe fn convert_to_hash_map(self, hash: *mut ffi::apr_hash_t) -> HashMap<String, String> {
        unsafe {
            self.hash_map(hash, |(k, v)| {
                (
                    (k as *const c_char).to_str().to_string(),
                    (v as *mut subversion::ffi::svn_string_t)
                        .to_str()
                        .to_string(),
                )
            })
        }
    }

    unsafe fn hash_map<K: Eq + Hash, V>(
        self,
        hash: *mut ffi::apr_hash_t,
        f: impl Fn((*const c_void, *mut c_void)) -> (K, V),
    ) -> HashMap<K, V> {
        let mut map = HashMap::new();

        unsafe {
            let mut it = ffi::apr_hash_first(self, hash);

            while !it.is_null() {
                let mut key = std::ptr::null();

                let mut value = std::ptr::null_mut();

                ffi::apr_hash_this(
                    it,
                    &mut key as *mut _,
                    std::ptr::null_mut(),
                    &mut value as *mut _,
                );

                // let key = (key as *const c_char).to_str().to_string();

                // let value = (value as *mut subversion::ffi::svn_string_t).to_str().to_string();
                //

                let (key, value) = f((key, value));

                map.insert(key, value);

                it = ffi::apr_hash_next(it);
            }
        }

        map
    }

    unsafe fn string<T: AsRef<[u8]>>(self, string: T) -> error::Result<*mut c_char> {
        if string.as_ref().contains(&0) {
            return builder::InvalidArgument {
                detail: "Invalid string",
            }
            .fail();
        }

        let string = unsafe {
            ffi::apr_pstrndup(self, string.as_ref().as_ptr() as _, string.as_ref().len())
        };

        Ok(string)
    }

    unsafe fn malloc<T: Sized>(self) -> *mut T {
        let size = size_of::<T>();

        unsafe { ffi::apr_palloc(self, size.try_into().unwrap()) as *mut T }
    }
}

const POINTER_SIZE_BYTES: usize = std::mem::size_of::<usize>();

#[easy_ext::ext(AprArray)]
pub impl *const ffi::apr_array_header_t {
    fn len(self) -> usize {
        unsafe { self.as_ref().unwrap().nelts.try_into().unwrap() }
    }
    fn to_vec<T: Sized>(self, read: impl Fn(*const c_char) -> T) -> Vec<T> {
        let mut vec = Vec::with_capacity(self.len());
        unsafe {
            let this = self.as_ref().unwrap();
            for i in 0..this.nelts {
                let ptr = this.elts.byte_add(usize::try_from(i).unwrap() * POINTER_SIZE_BYTES);
                let value = read(std::ptr::read(ptr as _));
                vec.push(value);
            }
        }
        vec
    }

    fn to_value_vec<T>(self, read: impl Fn(*const c_char) -> T) -> Vec<T> {
        let mut vec = Vec::with_capacity(self.len());
        let element_size = std::mem::size_of::<T>();
        unsafe {
            let this = self.as_ref().unwrap();
            for i in 0..this.nelts {
                let ptr = this.elts.byte_add(usize::try_from(i).unwrap() * element_size);
                let value = read(ptr);
                vec.push(value);
            }
        }
        vec
    }
}
