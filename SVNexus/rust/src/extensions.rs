use std::{any::Any, ffi::c_char, sync::Arc};

use sea_orm::ActiveValue;
use snafu::{OptionExt, ResultExt};

use crate::{AnyValue, apr, error::{self, Error, builder}, subversion::SVNError, utils::Pointer};

// #[easy_ext::ext(JsonExtension)]
// pub impl<T> T {
//     fn as_json(&self) -> error::Result<String>
//     where
//         Self: Serialize,
//     {
//         let v = serde_json::to_string(self)?;
//         Ok(v)
//     }
//     fn from_json<'de>(json: &'de str) -> error::Result<Self>
//     where
//         Self: Deserialize<'de>,
//     {
//         let v = serde_json::from_str(json)?;
//         Ok(v)
//     }
// }

#[easy_ext::ext(ActiveValueExtension)]
pub impl<T: Into<sea_query::Value>> ActiveValue<T> {
    fn set_value(&mut self, value: T) {
        *self = Self::Set(value);
    }
}

#[easy_ext::ext(DefaultExtension)]
pub impl<T: Default> T {
    fn default_value() -> Self {
        Default::default()
    }
}

// pub enum ValueOrRef<'a, T> {
//     Ref(&'a T),
//     Value(T),
// }

// impl<'a, T> AsRef<T> for ValueOrRef<'a, T> {
//     fn as_ref(&self) -> &T {
//         match *self {
//             ValueOrRef::Ref(v) => v,
//             ValueOrRef::Value(ref v) => v,
//         }
//     }
// }

#[easy_ext::ext(OptionExtension)]
pub impl<T> Option<T> {
    #[track_caller]
    fn any_context<S>(self, context: S) -> Result<T, Error>
    where
        S: Into<String>,
    {
        self.whatever_context::<_, Error>(context)
    }

    #[track_caller]
    fn with_any_context<F, S>(self, context: F) -> Result<T, Error>
    where
        F: FnOnce() -> S,
        S: Into<String>,
    {
        self.with_whatever_context::<_, _, Error>(context)
    }

    fn inner_into<V: From<T>>(self) -> Option<V> {
        self.map(|v| V::from(v))
    }
}

#[easy_ext::ext(ResultExtension)]
pub impl<T, E: std::error::Error + Send + Sync + 'static> Result<T, E> {
    #[track_caller]
    fn any_context<S>(self, context: S) -> Result<T, Error>
    where
        S: Into<String>,
    {
        self.whatever_context::<_, Error>(context)
    }

    #[track_caller]
    fn with_any_context<F, S>(self, context: F) -> Result<T, Error>
    where
        F: FnOnce(&mut E) -> S,
        S: Into<String>,
    {
        self.with_whatever_context::<_, _, Error>(context)
    }
}

#[easy_ext::ext(CommonExtension)]
pub impl<T: Sized> T {
    fn if_some<V>(self, option: Option<V>, f: impl FnOnce(Self, V) -> Self) -> Self {
        if let Some(v) = option {
            f(self, v)
        } else {
            self
        }
    }

    fn if_or<R>(self, value: bool, i: impl FnOnce(Self) -> R, o: impl FnOnce(Self) -> R) -> R {
        if value { i(self) } else { o(self) }
    }

    fn so_if_or<R>(
        self,
        b: impl FnOnce(&Self) -> bool,
        i: impl FnOnce(Self) -> R,
        o: impl FnOnce(Self) -> R,
    ) -> R {
        if b(&self) { i(self) } else { o(self) }
    }

    #[cfg(false)]
    fn if_or_ref<R>(
        &self,
        value: bool,
        i: impl FnOnce(&Self) -> R,
        o: impl FnOnce(&Self) -> R,
    ) -> R {
        if value { i(self) } else { o(self) }
    }

    fn also_build(self, f: impl FnOnce(Self) -> Self) -> Self {
        f(self)
    }

    fn into_option_some(self) -> Option<Self> {
        Some(self)
    }

    fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    fn into_any_value(self) -> AnyValue
    where
        Self: Any + Send + Sync + 'static,
    {
        AnyValue::new(self)
    }
}



#[easy_ext::ext(Canonicalization)] 
pub impl apr::Pool {

    fn canonicalize_target(&mut self, target: &str) -> error::Result<*const c_char> {

        use crate::subversion::ffi;

        unsafe {
            let t = self.string(target)?;

            if ffi::svn_path_is_url(t) == 0 {
                self.canonicalize_dirent(target)
            } else {
                self.canonicalize_uri(target)
            }
        }
        
    }

    unsafe fn canonicalize_target_array<T, I>(
        &mut self,
        len: usize,
        targets: T,
    ) -> error::Result<*mut crate::apr::ffi::apr_array_header_t>
    where
        T: Iterator<Item = I>,
        I: AsRef<str>,
    {
        use crate::apr::ffi;
        // let array = Array::with_capacity(string_list.len());
        unsafe {
            let array = ffi::apr_array_make(
                self.as_mut_ptr(),
                len.try_into().unwrap(),
                size_of::<usize>().try_into().unwrap(),
            );

            for i in targets {
                let string = self.canonicalize_target(i.as_ref())?;

                let ptr = ffi::apr_array_push(array) as *mut *const c_char;

                *ptr = string;
            }

            Ok(array)
        }
    }


    fn canonicalize_relpath(&mut self, u: &str) -> error::Result<*const c_char> {
        unsafe {
            let uri = self.string(u)?;
            use crate::subversion::ffi;

            let mut result: *const c_char = std::ptr::null();

            let error = ffi::svn_relpath_canonicalize_safe(result.pointer_mut(), std::ptr::null_mut(), uri, self.as_mut_ptr(), self.as_mut_ptr());

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            if ffi::svn_relpath_is_canonical(uri) == 0 {
                return builder::InvalidArgument {
                    detail: format!("{} is not canonical", u),
                }.fail();
            }

            Ok(result)

        }
    }

    fn canonicalize_uri(&mut self, uri: &str) -> error::Result<*const c_char> {
        unsafe {
            let uri = self.string(uri)?;
            use crate::subversion::ffi;

            let mut result: *const c_char = std::ptr::null();

            let error = ffi::svn_uri_canonicalize_safe(result.pointer_mut(), std::ptr::null_mut(), uri, self.as_mut_ptr(), self.as_mut_ptr());
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            Ok(result)

        }
    }


    unsafe fn canonicalize_dirent_array<T, I>(
        &mut self,
        len: usize,
        targets: T,
    ) -> error::Result<*mut crate::apr::ffi::apr_array_header_t>
    where
        T: Iterator<Item = I>,
        I: AsRef<str>,
    {
        use crate::apr::ffi;
        // let array = Array::with_capacity(string_list.len());
        unsafe {
            let array = ffi::apr_array_make(
                self.as_mut_ptr(),
                len.try_into().unwrap(),
                size_of::<usize>().try_into().unwrap(),
            );

            for i in targets {
                let string = self.canonicalize_dirent(i.as_ref())?;

                let ptr = ffi::apr_array_push(array) as *mut *const c_char;

                *ptr = string;
            }

            Ok(array)
        }
    }

    fn canonicalize_dirent(&mut self, path: &str) -> error::Result<*const c_char> {

        use crate::subversion::ffi;

        unsafe {
            let path = self.string(path)?;

            let internal = ffi::svn_dirent_internal_style(path, self.as_mut_ptr());

            let mut absolute: *const c_char = std::ptr::null();

            let error = ffi::svn_dirent_get_absolute(absolute.pointer_mut(), internal, self.as_mut_ptr());

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            let mut result: *const c_char = std::ptr::null();

            let error = ffi::svn_dirent_canonicalize_safe(result.pointer_mut(), std::ptr::null_mut(), absolute, self.as_mut_ptr(), self.as_mut_ptr());

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            Ok(result)
        }

    }
}