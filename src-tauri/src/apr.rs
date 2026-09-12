use crate::{
    subversion,
    utils::{CStringer, SubversionStringer},
};

use super::error::builder;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::{c_char, c_void, CString},
    hash::Hash,
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

// pub unsafe fn char_array_to_string(ptr: *const c_char) -> Option<String> {
//     if ptr.is_null() {
//         None
//     } else {
//         Some(unsafe { CStr::from_ptr(ptr) }.to_str().ok()?.to_string())
//     }
// }

pub unsafe fn create() -> *mut ffi::apr_pool_t {
    let mut ptr: *mut ffi::apr_pool_t = std::ptr::null_mut();
    unsafe {
        let status = ffi::apr_pool_create_ex(
            &mut ptr as *mut _,
            std::ptr::null_mut(),
            Some(on_pool_abort),
            std::ptr::null_mut(),
        );

        AprError::check_error(status).expect("Failed to alloc memory");
    }
    ptr
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

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
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
    fn check_error(code: i32) -> Result<(), Self> {
        if code == 0 {
            return Ok(());
        }

        let mut buf = vec![0; 512];
        unsafe {
            ffi::apr_strerror(
                code as _,
                buf.as_mut_ptr() as _,
                buf.len().try_into().expect("Failed to convert size"),
            )
        };

        let Ok(code): Result<u32, _> = code.try_into() else {
            return Err(Self {
                code: ErrorCode::General,
                msg: format!("Unknow error: {}", code),
            });
        };
        let Ok(code) = ErrorCode::try_from(code) else {
            return Err(Self {
                code: ErrorCode::General,
                msg: format!("Unknow error: {}", code),
            });
        };

        let msg = CString::from_vec_with_nul(buf)
            .expect("Unexpected failure")
            .to_str()
            .expect("Unexpected failure")
            .to_string();

        Err(Self { code, msg })
    }
}

#[svnexus_macro::enum_converter(repr_type=u32)]
#[derive(Clone, Debug, Copy, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    // ═══════════════════════════════════════════════════════
    // 原生 APR 错误 (APR_OS_START_ERROR = 20000)
    // ═══════════════════════════════════════════════════════
    NoStat = ffi::APR_ENOSTAT,                     // 20001  无法 stat
    NoPool = ffi::APR_ENOPOOL,                     // 20002  未提供内存池
    BadDate = ffi::APR_EBADDATE,                   // 20004  无效日期
    InvalidSocket = ffi::APR_EINVALSOCK,           // 20005  无效 socket
    NoProc = ffi::APR_ENOPROC,                     // 20006  未提供进程结构
    NoTime = ffi::APR_ENOTIME,                     // 20007  未提供时间结构
    NoDir = ffi::APR_ENODIR,                       // 20008  未提供目录结构
    NoLock = ffi::APR_ENOLOCK,                     // 20009  未提供锁结构
    NoPoll = ffi::APR_ENOPOLL,                     // 20010  未提供 poll 结构
    NoSocket = ffi::APR_ENOSOCKET,                 // 20011  未提供 socket
    NoThread = ffi::APR_ENOTHREAD,                 // 20012  未提供线程结构
    NoThreadKey = ffi::APR_ENOTHDKEY,              // 20013  未提供线程 key
    General = ffi::APR_EGENERAL,                   // 20014  通用错误
    NoSharedMemory = ffi::APR_ENOSHMAVAIL,         // 20015  共享内存不足
    BadIp = ffi::APR_EBADIP,                       // 20016  无效 IP
    BadMask = ffi::APR_EBADMASK,                   // 20017  无效子网掩码
    DsoOpen = ffi::APR_EDSOOPEN,                   // 20019  无法打开动态库
    AbsolutePath = ffi::APR_EABSOLUTE,             // 20020  路径是绝对路径
    RelativePath = ffi::APR_ERELATIVE,             // 20021  路径是相对路径
    IncompletePath = ffi::APR_EINCOMPLETE,         // 20022  路径格式不完整
    AboveRoot = ffi::APR_EABOVEROOT,               // 20023  路径超出根目录
    BadPath = ffi::APR_EBADPATH,                   // 20024  路径格式错误
    PathWildcard = ffi::APR_EPATHWILD,             // 20025  路径含通配符
    SymbolNotFound = ffi::APR_ESYMNOTFOUND,        // 20026  找不到符号 (dlsym)
    ProcUnknown = ffi::APR_EPROC_UNKNOWN,          // 20027  进程无法识别
    NotEnoughEntropy = ffi::APR_ENOTENOUGHENTROPY, // 20028  熵不足

    // ═══════════════════════════════════════════════════════
    // APR 状态码 (APR_OS_START_STATUS = 70000)
    // ═══════════════════════════════════════════════════════
    InChild = ffi::APR_INCHILD,            // 70001  在子进程中
    InParent = ffi::APR_INPARENT,          // 70002  在父进程中
    Detached = ffi::APR_DETACH,            // 70003  线程已分离
    NotDetached = ffi::APR_NOTDETACH,      // 70004  线程未分离
    ChildDone = ffi::APR_CHILD_DONE,       // 70005  子进程执行完毕
    ChildNotDone = ffi::APR_CHILD_NOTDONE, // 70006  子进程未执行完毕
    TimeUp = ffi::APR_TIMEUP,              // 70007  操作超时
    Incomplete = ffi::APR_INCOMPLETE,      // 70008  操作部分完成
    BadOption = ffi::APR_BADCH,            // 70012  getopt 未知选项
    BadArg = ffi::APR_BADARG,              // 70013  getopt 缺少参数
    Eof = ffi::APR_EOF,                    // 70014  文件结束
    NotFound = ffi::APR_NOTFOUND,          // 70015  poll 中找不到 socket
    AnonymousShm = ffi::APR_ANONYMOUS,     // 70019  匿名共享内存
    FileBasedShm = ffi::APR_FILEBASED,     // 70020  文件名共享内存
    KeyBasedShm = ffi::APR_KEYBASED,       // 70021  key 共享内存
    Init = ffi::APR_EINIT,                 // 70022  初始化值
    NotImplemented = ffi::APR_ENOTIMPL,    // 70023  函数未实现
    Mismatch = ffi::APR_EMISMATCH,         // 70024  密码不匹配
    Busy = ffi::APR_EBUSY,                 // 70025  锁忙

    // ═══════════════════════════════════════════════════════
    // 跨平台 errno 映射
    // ═══════════════════════════════════════════════════════
    PermissionDenied = ffi::APR_EACCES,                // 权限不足
    AlreadyExists = ffi::APR_EEXIST,                   // 文件已存在
    NameTooLong = ffi::APR_ENAMETOOLONG,               // 文件名过长
    NotFound2 = ffi::APR_ENOENT,                       // 文件/目录不存在
    NotDir = ffi::APR_ENOTDIR,                         // 不是目录
    NoSpace = ffi::APR_ENOSPC,                         // 磁盘空间不足
    OutOfMemory = ffi::APR_ENOMEM,                     // 内存不足
    TooManyOpenFiles = ffi::APR_EMFILE,                // 进程打开文件过多
    FileTableOverflow = ffi::APR_ENFILE,               // 系统文件表溢出
    BadFileDescriptor = ffi::APR_EBADF,                // 无效文件描述符
    InvalidArg = ffi::APR_EINVAL,                      // 无效参数
    IllegalSeek = ffi::APR_ESPIPE,                     // 管道不可 seek
    WouldBlock = ffi::APR_EAGAIN,                      // 资源暂时不可用(需重试)
    Interrupted = ffi::APR_EINTR,                      // 系统调用被信号中断
    NotSock = ffi::APR_ENOTSOCK,                       // 不是 socket
    ConnectionRefused = ffi::APR_ECONNREFUSED,         // 连接被拒绝
    InProgress = ffi::APR_EINPROGRESS,                 // 操作进行中
    ConnectionAborted = ffi::APR_ECONNABORTED,         // 连接被中止
    ConnectionReset = ffi::APR_ECONNRESET,             // 连接被重置
    TimedOut = ffi::APR_ETIMEDOUT,                     // 连接超时
    HostUnreachable = ffi::APR_EHOSTUNREACH,           // 主机不可达
    NetworkUnreachable = ffi::APR_ENETUNREACH,         // 网络不可达
    BadFileType = ffi::APR_EFTYPE,                     // 文件类型错误
    BrokenPipe = ffi::APR_EPIPE,                       // 管道破裂
    CrossDevice = ffi::APR_EXDEV,                      // 跨设备链接
    DirNotEmpty = ffi::APR_ENOTEMPTY,                  // 目录非空
    AddressFamilyNotSupported = ffi::APR_EAFNOSUPPORT, // 地址族不支持
    OperationNotSupported = ffi::APR_EOPNOTSUPP,       // 操作不支持
    OutOfRange = ffi::APR_ERANGE,                      // 数值超出范围
}

// pub struct PoolFactory;

// impl Drop for PoolFactory {
//     fn drop(&mut self) {
//         unsafe {
//             ffi::apr_pool_terminate();
//         }
//     }
// }

// impl PoolFactory {
//     pub fn instance() -> &'static Self {
//         pub static SELF: OnceLock<PoolFactory> = OnceLock::new();

//         SELF.get_or_init(|| {
//             unsafe {
//                 ffi::apr_initialize();
//             }
//             PoolFactory {}
//         })
//     }

//     pub fn create_pool(&self) -> Pool {
//         let mut ptr: *mut ffi::apr_pool_t = std::ptr::null_mut();
//         let ptr = unsafe {
//             let status = ffi::apr_pool_create_ex(
//                 &mut ptr as *mut _,
//                 std::ptr::null_mut(),
//                 Some(on_pool_abort),
//                 std::ptr::null_mut(),
//             );

//             AprError::check_error(status).expect("Failed to alloc memory");
//             ptr
//         };
//         Pool::from_raw(ptr)
//     }
// }

extern "C" fn on_pool_abort(code: std::ffi::c_int) -> std::ffi::c_int {
    panic!("Out of memory: {}", code);
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
                    ffi::APR_HASH_KEY_STRING.try_into().expect("Failed to convert value"),
                    v as _,
                )
            }

            Ok(table)
        }
    }

    pub unsafe fn value_array<T>(
        &mut self,
        len: usize,
        iter: impl Iterator<Item = T>,
    ) -> error::Result<*mut ffi::apr_array_header_t> {
        unsafe {
            let array = ffi::apr_array_make(
                self.as_mut_ptr(),
                len.try_into().expect("Failed to convert size"),
                std::mem::size_of::<T>().try_into().expect("Failed to convert size"),
            );

            for i in iter {
                let ptr = ffi::apr_array_push(array) as *mut T;

                *ptr = i;
            }

            Ok(array)
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
                len.try_into().expect("Failed to convert size"),
                size_of::<usize>().try_into().expect("Failed to convert size"),
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

        unsafe { ffi::apr_palloc(self, size.try_into().expect("Failed to convert size")) as *mut T }
    }

    unsafe fn child(self) -> *mut ffi::apr_pool_t {
        unsafe {
            let mut child: *mut ffi::apr_pool_t = std::ptr::null_mut();
            let status = ffi::apr_pool_create_ex(
                &mut child as _,
                self,
                Some(on_pool_abort),
                Default::default(),
            );
            assert!(
                status == ffi::APR_SUCCESS as ffi::apr_status_t,
                "Unexpected error"
            );
            child
        }
    }

    unsafe fn boxed<T: Sized>(self, value: T) -> *mut T {
        unsafe extern "C" fn cleanup<T>(baton: *mut c_void) -> ffi::apr_status_t {
            tracing::debug!(
                "Drop: baton={:?}, type={}",
                baton,
                std::any::type_name::<T>()
            );
            unsafe {
                std::ptr::drop_in_place(baton as *mut T);
            }
            tracing::debug!(
                "Dropped: baton={:?}, type={}",
                baton,
                std::any::type_name::<T>()
            );
            0
        }
        unsafe {
            let this = self.malloc();

            std::ptr::write(this, value);

            ffi::apr_pool_cleanup_register(self, this as _, Some(cleanup::<T>), None);

            this
        }
    }
}

const POINTER_SIZE_BYTES: usize = std::mem::size_of::<usize>();

#[easy_ext::ext(AprArray)]
pub impl *const ffi::apr_array_header_t {
    fn len(self) -> usize {
        unsafe { self.as_ref().expect("Failed to get reference").nelts.try_into().expect("Failed to convert size") }
    }
    fn to_vec<T: Sized>(self, read: impl Fn(*const c_char) -> T) -> Vec<T> {
        let mut vec = Vec::with_capacity(self.len());
        unsafe {
            let this = self.as_ref().expect("Failed to get reference");
            for i in 0..this.nelts {
                let ptr = this
                    .elts
                    .byte_add(usize::try_from(i).expect("Unexpected failure") * POINTER_SIZE_BYTES);
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
            let this = self.as_ref().expect("Failed to get reference");
            for i in 0..this.nelts {
                let ptr = this
                    .elts
                    .byte_add(usize::try_from(i).expect("Unexpected failure") * element_size);
                let value = read(ptr);
                vec.push(value);
            }
        }
        vec
    }
}
