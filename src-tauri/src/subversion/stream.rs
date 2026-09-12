use crate::{
    apr::{self, AprPool},
    error::{self},
    utils::{Boxed, Pointer},
};

use super::ffi;
use std::ffi::{c_char, c_void};

// pub struct Streamer<'a> {
//     stream: *mut ffi::svn_stream_t,
//     _mark: PhantomData<&'a ()>,
// }

// impl<'a> Streamer<'a> {
//     pub fn write(&mut self, bytes: impl AsRef<[u8]>) -> error::Result<usize> {
//         unsafe {
//             let mut size: ffi::apr_size_t = bytes.as_ref().len().try_into().unwrap();
//             let error = ffi::svn_stream_write(
//                 self.stream,
//                 bytes.as_ref().as_ptr() as _,
//                 &mut size as *mut _,
//             );
//             super::SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
//             Ok(size.try_into().unwrap())
//         }
//     }
//     pub fn close(&mut self) -> error::Result<()> {
//         unsafe {
//             let error = ffi::svn_stream_close(self.stream);
//             super::SubversionError::from_nullable_ptr(error).context(builder::Subversion)
//         }
//     }
// }

pub struct Stream {
    inner: Box<StreamInner>,
    stream: *mut ffi::svn_stream_t,
    _pool: apr::Pool,
}

unsafe extern "C" fn svn_write_fn(
    baton: *mut c_void,
    data: *const c_char,
    len: *mut ffi::apr_size_t,
) -> *mut ffi::svn_error_t {
    unsafe {
        let inner = baton as *mut StreamInner;
        let inner = inner.as_mut().expect("Failed to get mutable reference to inner");
        inner
            .write_buffer
            .extend_from_slice(std::slice::from_raw_parts(
                data as _,
                (*len).try_into().expect("Failed to convert size"),
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
        let inner = inner.as_mut().expect("Failed to get mutable reference to inner");

        let min = std::cmp::min(usize::try_from(*len).expect("Unexpected failure"), inner.read_buffer.len());

        let buf = std::slice::from_raw_parts_mut(buf as *mut u8, (*len).try_into().expect("Failed to convert size"));

        buf.copy_from_slice(&inner.read_buffer[..min]);

        inner.read_buffer.drain(..min);

        *len = min.try_into().expect("Failed to convert read length");
    }

    super::svn_no_error()
}

impl Stream {
    pub unsafe fn ptr(&mut self) -> *mut ffi::svn_stream_t {
        self.stream
    }
    // pub fn base64(&mut self, break_lines: bool) -> Streamer<'_> {
    //     unsafe {
    //         let stream =
    //             ffi::svn_base64_encode2(self.stream, break_lines.into(), self.pool.as_mut_ptr());
    //         Streamer {
    //             stream,
    //             _mark: Default::default(),
    //         }
    //     }
    // }

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
                _pool: pool,
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

// pub struct FunctionStream {
//     write: Box<dyn FnMut(&[u8]) -> Result<usize, error::FrontendError>>,
//     read: Box<dyn FnMut(&mut [u8]) -> Result<usize, error::FrontendError>>,
//     close: Box<dyn FnMut() -> Result<(), error::FrontendError>>,
// }

pub struct ChannelStream {
    receiver: tokio::sync::mpsc::Receiver<Vec<u8>>,
    sender: Option<tokio::sync::mpsc::Sender<Vec<u8>>>,
    read_buf: Vec<u8>,
    read_pos: usize,
}
impl ChannelStream {
    pub fn create() -> (Self, Self) {
        let (sender1, receiver1) = tokio::sync::mpsc::channel(128);
        let (sender2, receiver2) = tokio::sync::mpsc::channel(128);

        (
            Self {
                receiver: receiver1,
                sender: Some(sender2),
                read_buf: Default::default(),
                read_pos: 0,
            },
            Self {
                receiver: receiver2,
                sender: Some(sender1),
                read_buf: Default::default(),
                read_pos: 0,
            },
        )
    }

    #[tracing::instrument(skip(self, data))]
    pub async fn write_async(&self, data: &[u8]) -> Result<usize, error::FrontendError> {
        let Some(ref sender) = self.sender else {
            return error::UnexpectedSnafu {
                detail: "Channel has been closed",
            }
            .fail();
        };
        sender.send(data.to_vec()).await.map_err(|e| {
            error::UnexpectedSnafu {
                detail: format!("channel send failed: {e}"),
            }
            .build()
        })?;

        Ok(data.len())
    }

    #[tracing::instrument(skip(self, buffer))]
    pub async fn read_async(&mut self, buffer: &mut [u8]) -> Result<usize, error::FrontendError> {
        if buffer.is_empty() {
            return Ok(0);
        }

        while self.read_buf.is_empty() {
            // let data = self.receiver.recv().await.context(error::UnexpectedSnafu {
            //     detail: "channel receive failed",
            // })?;
            //
            let Some(data) = self.receiver.recv().await else {
                break;
            };

            self.read_buf.extend(data);
        }

        let available = self.read_buf.len() - self.read_pos;
        let n = available.min(buffer.len());

        buffer[..n].copy_from_slice(&self.read_buf[self.read_pos..self.read_pos + n]);

        self.read_pos += n;

        // 如果这一块已经读完，可以清空，释放元素内容
        if self.read_pos >= self.read_buf.len() {
            self.read_buf.clear();
            self.read_pos = 0;
        }

        Ok(n)
    }
}

impl Streaming for ChannelStream {
    #[tracing::instrument(skip(self, data))]
    fn write(&mut self, data: &[u8]) -> Result<usize, error::FrontendError> {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tracing::info!("Got handle: {:?} {:?}", handle.name(), handle.id());
        }
        // debug_assert!(
        //     tokio::runtime::Handle::try_current().is_err(),
        //     "blocking_recv/send should not be called from tokio runtime thread!"
        // );

        tracing::info!("Write data: {}", data.len());
        let Some(ref sender) = self.sender else {
            tracing::info!("Write fail");
            return error::UnexpectedSnafu {
                detail: "Channel has been closed",
            }
            .fail();
        };
        sender.blocking_send(data.to_vec()).map_err(|e| {
            tracing::info!("Write fail");
            error::UnexpectedSnafu {
                detail: format!("channel send failed: {e}"),
            }
            .build()
        })?;

        Ok(data.len())
    }

    #[tracing::instrument(skip(self, buffer))]
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, error::FrontendError> {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tracing::info!("Got handle: {:?} {:?}", handle.name(), handle.id());
        }
        // debug_assert!(
        //     tokio::runtime::Handle::try_current().is_err(),
        //     "blocking_recv/send should not be called from tokio runtime thread!"
        // );
        if buffer.is_empty() {
            return Ok(0);
        }

        while self.read_buf.is_empty() {
            let Some(data) = self.receiver.blocking_recv() else {
                break;
            };

            self.read_buf.extend(data);
        }

        let available = self.read_buf.len() - self.read_pos;
        if available == 0 {
            return Ok(0);
        }
        let n = available.min(buffer.len());

        buffer[..n].copy_from_slice(&self.read_buf[self.read_pos..self.read_pos + n]);

        self.read_pos += n;

        // 如果这一块已经读完，可以清空，释放元素内容
        if self.read_pos >= self.read_buf.len() {
            self.read_buf.clear();
            self.read_pos = 0;
        }

        Ok(n)
    }

    #[tracing::instrument(skip(self))]
    fn close(&mut self) -> Result<(), error::FrontendError> {
        self.sender = None;
        Ok(())
    }

    fn read_available(&mut self) -> Result<bool, error::FrontendError> {
        Ok(self.read_buf.len() > 0 || self.receiver.len() > 0)
    }
}

pub trait Streaming {
    fn write(&mut self, data: &[u8]) -> Result<usize, error::FrontendError>;
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, error::FrontendError>;
    fn close(&mut self) -> Result<(), error::FrontendError>;
    fn read_available(&mut self) -> Result<bool, error::FrontendError>;
}

#[easy_ext::ext(StreamingExtension)]
pub impl Box<dyn Streaming> {
    // pub fn channel_stream() {
    //     let default_size = 128;

    //     let (sender1, mut receiver1) = mpsc::channel::<Vec<u8>>(128);

    //     let this = FunctionStream {
    //         write: Box::new(move |data| {
    //             sender1.blocking_send(data.to_vec()).unwrap();
    //             Ok(data.len())
    //         }),
    //         read: Box::new(|data| Ok(0)),
    //         close: Box::new(|| Ok(())),
    //     };

    //     todo!()
    // }

    unsafe fn into_stream(self, pool: *mut ffi::apr_pool_t) -> *mut ffi::svn_stream_t {
        unsafe {
            let this = pool.boxed(self);
            this.as_mut().expect("Failed to get mutable reference").as_stream(pool)
        }
    }

    unsafe fn as_stream(&mut self, pool: *mut ffi::apr_pool_t) -> *mut ffi::svn_stream_t {
        unsafe extern "C" fn data_available(
            baton: *mut c_void,
            data_available: *mut ffi::svn_boolean_t,
        ) -> *mut ffi::svn_error_t {
            let context = (baton as *mut Box<dyn Streaming>).as_mut().expect("Failed to cast baton to mutable reference");
            *data_available = context.read_available().expect("Failed to check data availability").into();
            super::svn_no_error()
        }

        unsafe extern "C" fn write(
            baton: *mut c_void,
            data: *const c_char,
            len: *mut ffi::apr_size_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let data =
                    std::slice::from_raw_parts(data as *const u8, (*len).try_into().expect("Failed to convert size"));
                match (baton as *mut Box<dyn Streaming>)
                    .as_mut()
                    .expect("Unexpected failure")
                    .write(data)
                {
                    Ok(l) => *len = l.try_into().expect("Failed to convert size"),
                    Err(e) => return e.native_error(),
                };
            }
            super::svn_no_error()
        }
        unsafe extern "C" fn read(
            baton: *mut c_void,
            buffer: *mut c_char,
            len: *mut ffi::apr_size_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let data =
                    std::slice::from_raw_parts_mut(buffer as *mut u8, (*len).try_into().expect("Failed to convert size"));
                match (baton as *mut Box<dyn Streaming>)
                    .as_mut()
                    .expect("Unexpected failure")
                    .read(data)
                {
                    Ok(l) => *len = l.try_into().expect("Failed to convert size"),
                    Err(e) => return e.native_error(),
                }
            }
            super::svn_no_error()
        }
        unsafe extern "C" fn close(baton: *mut c_void) -> *mut ffi::svn_error_t {
            unsafe {
                if let Err(e) = (baton as *mut Box<dyn Streaming>).as_mut().expect("Failed to cast baton to mutable reference").close() {
                    return e.native_error();
                }
            }
            super::svn_no_error()
        }
        // ffi::apr_pool_cleanup_register(pool, data, plain_cleanup, child_cleanup);
        unsafe {
            let stream = ffi::svn_stream_create(self.pointer_mut() as _, pool);

            ffi::svn_stream_set_write(stream, Some(write));
            ffi::svn_stream_set_read2(stream, Some(read), None);
            ffi::svn_stream_set_close(stream, Some(close));
            ffi::svn_stream_set_data_available(stream, Some(data_available));

            stream
        }
    }
}
