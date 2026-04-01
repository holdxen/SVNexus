use snafu::ResultExt;

use crate::{
    apr,
    error::{self, builder},
    utils::Boxed,
};

use super::ffi;
use std::{
    ffi::{c_char, c_void},
    marker::PhantomData,
};

pub struct Streamer<'a> {
    stream: *mut ffi::svn_stream_t,
    _mark: PhantomData<&'a ()>,
}

impl<'a> Streamer<'a> {
    pub fn write(&mut self, bytes: impl AsRef<[u8]>) -> error::Result<usize> {
        unsafe {
            let mut size: ffi::apr_size_t = bytes.as_ref().len().try_into().unwrap();
            let error = ffi::svn_stream_write(
                self.stream,
                bytes.as_ref().as_ptr() as _,
                &mut size as *mut _,
            );
            super::SVNError::from_nullable_ptr(error).context(builder::Svn)?;
            Ok(size.try_into().unwrap())
        }
    }
    pub fn close(&mut self) -> error::Result<()> {
        unsafe {
            let error = ffi::svn_stream_close(self.stream);
            super::SVNError::from_nullable_ptr(error).context(builder::Svn)
        }
    }
}

pub struct Stream {
    inner: Box<StreamInner>,
    stream: *mut ffi::svn_stream_t,
    pool: apr::Pool,
}

unsafe extern "C" fn svn_write_fn(
    baton: *mut c_void,
    data: *const c_char,
    len: *mut ffi::apr_size_t,
) -> *mut ffi::svn_error_t {
    unsafe {
        let inner = baton as *mut StreamInner;
        let inner = inner.as_mut().unwrap();
        inner
            .write_buffer
            .extend_from_slice(std::slice::from_raw_parts(
                data as _,
                (*len).try_into().unwrap(),
            ));
    }

    super::svn_no_error()
}

unsafe extern "C" fn svn_read_fn(
    baton: *mut c_void,
    buf: *mut c_char,
    len: *mut ffi::apr_size_t,
) -> *mut ffi::svn_error_t {
    unsafe {
        let inner = baton as *mut StreamInner;
        let inner = inner.as_mut().unwrap();

        let min = std::cmp::min(usize::try_from(*len).unwrap(), inner.read_buffer.len());

        let buf = std::slice::from_raw_parts_mut(buf as *mut u8, (*len).try_into().unwrap());

        buf.copy_from_slice(&inner.read_buffer[..min]);

        inner.read_buffer.drain(..min);

        *len = min.try_into().unwrap();
    }

    super::svn_no_error()
}

impl Stream {
    pub unsafe fn ptr(&mut self) -> *mut ffi::svn_stream_t {
        self.stream
    }
    pub fn base64(&mut self, break_lines: bool) -> Streamer<'_> {
        unsafe {
            let stream =
                ffi::svn_base64_encode2(self.stream, break_lines.into(), self.pool.as_mut_ptr());
            Streamer {
                stream,
                _mark: Default::default(),
            }
        }
    }

    pub fn create(read_buffer: Vec<u8>) -> Self {
        unsafe {
            let mut inner: Box<StreamInner> = Box::new(StreamInner {
                write_buffer: Default::default(),
                read_buffer,
            });
            let mut pool = apr::Pool::create();

            let stream =
                ffi::svn_stream_create(&mut *inner as *mut _ as *mut c_void, pool.as_mut_ptr());

            ffi::svn_stream_set_write(stream, Some(svn_write_fn));

            ffi::svn_stream_set_baton(stream, inner.inner_void_pointer_mut());

            ffi::svn_stream_set_read2(stream, Some(svn_read_fn), None);

            Stream {
                inner,
                stream,
                pool,
            }
        }
    }

    pub fn take_write_buffer(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.inner.write_buffer)
    }
}

#[derive(Default)]
pub struct StreamInner {
    write_buffer: Vec<u8>,
    read_buffer: Vec<u8>,
}
