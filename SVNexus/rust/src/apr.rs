use crate::subversion;

use super::error::builder;
use num_enum::TryFromPrimitive;
use std::{
    collections::HashMap,
    ffi::{CStr, CString, c_char, c_int},
    marker::PhantomData,
    sync::OnceLock,
};

use super::error;

#[allow(bad_style)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
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
        Some(unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_string())
    }
}

pub struct Map {
    ptr: *mut ffi::apr_hash_t,
    pool: Pool,
}

impl Map {
    fn new() -> Self {
        let pool = PoolFactory::instance().create_pool();
        let ptr = unsafe { ffi::apr_hash_make(pool.ptr) };

        Self { ptr, pool }
    }
}

pub struct Array<T> {
    ptr: *mut ffi::apr_array_header_t,
    pool: Pool,
    _p: PhantomData<T>,
}

impl Array<*const c_char> {
    pub unsafe fn to_string_list(ptr: *const ffi::apr_array_header_t) -> Vec<String> {
        let array = unsafe { ptr.as_ref().unwrap() };

        let mut ret = Vec::with_capacity(array.nelts.try_into().unwrap());

        let elements = array.elts as *const *const c_char;

        for i in 0..array.nelts {
            let element = unsafe { elements.offset(i.try_into().unwrap()).read() };

            let string = unsafe { CStr::from_ptr(element) }
                .to_str()
                .unwrap()
                .to_owned();

            ret.push(string)
        }

        ret
    }

    pub unsafe fn from_string_list<T, I>(len: usize, string_list: T) -> error::Result<Self>
    where
        T: IntoIterator<Item = I>,
        I: AsRef<str>,
    {
        let mut this = Array::with_capacity(len);

        let array = unsafe {
            ffi::apr_array_make(
                this.pool.ptr,
                len.try_into().unwrap(),
                size_of::<usize>().try_into().unwrap(),
            )
        };

        for i in string_list {
            let string = unsafe { this.pool.string(i.as_ref())? };

            let array = unsafe { ffi::apr_array_push(array) } as *mut *const c_char;

            unsafe {
                std::ptr::copy(
                    &string as _,
                    array as _,
                    size_of::<*const c_char>().try_into().unwrap(),
                )
            };
        }

        Ok(this)
    }
}

impl<T> Array<T> {
    pub fn with_capacity(len: usize) -> Self {
        let pool = PoolFactory::instance().create_pool();
        let ptr = unsafe {
            let ptr = ffi::apr_array_make(
                pool.ptr,
                len.try_into().unwrap(),
                size_of::<T>().try_into().unwrap(),
            );
            ptr
        };

        Self {
            ptr,
            pool,
            _p: Default::default(),
        }
    }

    pub fn push(&mut self, value: T) {
        let array = unsafe { ffi::apr_array_push(self.ptr) } as *mut T;

        unsafe { std::ptr::copy(&value as _, array as _, size_of::<T>().try_into().unwrap()) };

        std::mem::forget(value);
    }

    pub fn as_mut_ptr(&mut self) -> *mut ffi::apr_array_header_t {
        self.ptr
    }
}

#[derive(Clone, Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub msg: String,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error {
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

#[repr(i32)]
#[derive(Clone, Debug, Copy, TryFromPrimitive)]
pub enum ErrorCode {
    ENOSTAT = ffi::APR_ENOSTAT as _,
    ENOPOOL = ffi::APR_ENOPOOL as _,
    EBADDATE = ffi::APR_EBADDATE as _,
    EINVALSOCK = ffi::APR_EINVALSOCK as _,
    ENOPROC = ffi::APR_ENOPROC as _,
    ENOTIME = ffi::APR_ENOTIME as _,
    ENOLOCK = ffi::APR_ENOLOCK as _,
    ENOPOLL = ffi::APR_ENOPOLL as _,
    ENOSOCKET = ffi::APR_ENOSOCKET as _,
    ENOTHREAD = ffi::APR_ENOTHREAD as _,
    ENOTHDKEY = ffi::APR_ENOTHDKEY as _,
    EGENERAL = ffi::APR_EGENERAL as _,
    ENOSHMAVAIL = ffi::APR_ENOSHMAVAIL as _,
    EBADIP = ffi::APR_EBADIP as _,
    EBADMASK = ffi::APR_EBADMASK as _,
    EDSOOPEN = ffi::APR_EDSOOPEN as _,
    EABSOLUTE = ffi::APR_EABSOLUTE as _,
    ERELATIVE = ffi::APR_ERELATIVE as _,
    EINCOMPLETE = ffi::APR_EINCOMPLETE as _,
    EABOVEROOT = ffi::APR_EABOVEROOT as _,
    EBADPATH = ffi::APR_EBADPATH as _,
    EPATHWILD = ffi::APR_EPATHWILD as _,
    ESYMNOTFOUND = ffi::APR_ESYMNOTFOUND as _,
    EPROCUnknown = ffi::APR_EPROC_UNKNOWN as _,
    ENOTENOUGHENTROPY = ffi::APR_ENOTENOUGHENTROPY as _,
    INCHILD = ffi::APR_INCHILD as _,
    INPARENT = ffi::APR_INPARENT as _,
    DETACH = ffi::APR_DETACH as _,
    NOTDETACH = ffi::APR_NOTDETACH as _,
    ChildDone = ffi::APR_CHILD_DONE as _,
    ChildNotDone = ffi::APR_CHILD_NOTDONE as _,
    TIMEUP = ffi::APR_TIMEUP as _,
    INCOMPLETE = ffi::APR_INCOMPLETE as _,
    BADCH = ffi::APR_BADCH as _,
    BADARG = ffi::APR_BADARG as _,
    EOF = ffi::APR_EOF as _,
    NOTFOUND = ffi::APR_NOTFOUND as _,
    ANONYMOUS = ffi::APR_ANONYMOUS as _,
    FILEBASED = ffi::APR_FILEBASED as _,
    KEYBASED = ffi::APR_KEYBASED as _,
    EINIT = ffi::APR_EINIT as _,
    ENOTIMPL = ffi::APR_ENOTIMPL as _,
    EMISMATCH = ffi::APR_EMISMATCH as _,
    EBUSY = ffi::APR_EBUSY as _,
    EACCES = ffi::APR_EACCES as _,
    EEXIST = ffi::APR_EEXIST as _,
    ENAMETOOLONG = ffi::APR_ENAMETOOLONG as _,
    ENOENT = ffi::APR_ENOENT as _,
    ENOTDIR = ffi::APR_ENOTDIR as _,
    ENOSPC = ffi::APR_ENOSPC as _,
    ENOMEM = ffi::APR_ENOMEM as _,
    EMFILE = ffi::APR_EMFILE as _,
    ENFILE = ffi::APR_ENFILE as _,
    EBADF = ffi::APR_EBADF as _,
    EINVAL = ffi::APR_EINVAL as _,
    ESPIPE = ffi::APR_ESPIPE as _,
    EAGAIN = ffi::APR_EAGAIN as _,
    EINTR = ffi::APR_EINTR as _,
    ENOTSOCK = ffi::APR_ENOTSOCK as _,
    ECONNREFUSED = ffi::APR_ECONNREFUSED as _,
    EINPROGRESS = ffi::APR_EINPROGRESS as _,
    ECONNABORTED = ffi::APR_ECONNABORTED as _,
    ECONNRESET = ffi::APR_ECONNRESET as _,
    ETIMEDOUT = ffi::APR_ETIMEDOUT as _,
    EHOSTUNREACH = ffi::APR_EHOSTUNREACH as _,
    ENETUNREACH = ffi::APR_ENETUNREACH as _,
    EFTYPE = ffi::APR_EFTYPE as _,
    EPIPE = ffi::APR_EPIPE as _,
    EXDEV = ffi::APR_EXDEV as _,
    ENOTEMPTY = ffi::APR_ENOTEMPTY as _,
    EAFNOSUPPORT = ffi::APR_EAFNOSUPPORT as _,
    EOPNOTSUPP = ffi::APR_EOPNOTSUPP as _,
    ERANGE = ffi::APR_ERANGE as _,
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

            Error::check_error(status).expect("Failed to alloc memory");
            ptr
        };
        Pool::from_raw(ptr)
    }
}

extern "C" fn on_pool_abort(code: std::ffi::c_int) -> std::ffi::c_int {
    panic!("Out of memory: {}", code);
}

pub struct Pooling<'a> {
    ptr: *mut ffi::apr_pool_t,
    _p: PhantomData<&'a ()>,
}

impl<'a> Pooling<'a> {
    fn from_raw(ptr: *mut ffi::apr_pool_t) -> Self {
        Self {
            ptr,
            _p: PhantomData,
        }
    }
}

unsafe impl<'a> Send for Pooling<'a> {}

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
    pub unsafe fn apr_hash_to_hash_map(
        &mut self,
        hash: *mut ffi::apr_hash_t,
    ) -> HashMap<String, String> {
        let mut map = HashMap::new();

        unsafe {
            let mut it = ffi::apr_hash_first(self.ptr, hash);

            while !it.is_null() {
                let mut key = std::ptr::null();

                let mut value = std::ptr::null_mut();

                ffi::apr_hash_this(
                    it,
                    &mut key as *mut _,
                    std::ptr::null_mut(),
                    &mut value as *mut _,
                );

                let key = CStr::from_ptr(key as *const c_char)
                    .to_str()
                    .unwrap()
                    .to_string();

                let value = &*(value as *mut subversion::ffi::svn_string_t);

                let value = std::slice::from_raw_parts(
                    value.data as *const u8,
                    value.len.try_into().unwrap(),
                );

                let value = String::from_utf8(value.to_vec()).unwrap();

                map.insert(key, value);

                // ffi::apr_hash_this(it, key, klen, val);

                it = ffi::apr_hash_next(it);
            }
        }

        map
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

            Error::check_error(status).expect("Failed to alloc memory");
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

    pub fn create_child(&mut self) -> Pooling<'_> {
        let ptr: *mut *mut ffi::apr_pool_t = std::ptr::null_mut();
        let ptr = unsafe {
            ffi::apr_pool_create_ex(ptr, self.ptr, Some(on_pool_abort), std::ptr::null_mut());
            *ptr
        };
        Pooling::from_raw(ptr)
    }

    pub unsafe fn malloc<T: Sized>(&mut self) -> *mut T {
        let size = size_of::<T>();

        unsafe { ffi::apr_palloc(self.as_mut_ptr(), size.try_into().unwrap()) as *mut T }
    }

    pub unsafe fn string<T: AsRef<[u8]>>(&mut self, string: T) -> error::Result<*mut c_char> {
        if string.as_ref().contains(&0) {
            return builder::InvalidArgument {
                detail: "Invalid string",
            }
            .fail();
        }

        let string = unsafe {
            ffi::apr_pstrndup(
                self.as_mut_ptr(),
                string.as_ref().as_ptr() as _,
                string.as_ref().len(),
            )
        };

        Ok(string)
    }

    pub unsafe fn string_hash_map<T, K, V>(&mut self, map: T) -> error::Result<*mut ffi::apr_hash_t>
    where
        T: Iterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        unsafe {
            let table = ffi::apr_hash_make(self.as_mut_ptr());

            for (k, v) in map {
                let k = self.string(k.as_ref())?;

                let v = self.string(v.as_ref())?;

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
