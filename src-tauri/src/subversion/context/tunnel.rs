use std::{
    ffi::{c_char, c_int, c_void},
    sync::Arc,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::oneshot,
};

use crate::{
    apr::AprPool,
    error::{self, FrontendError},
    subversion::{
        stream::{ChannelStream, Streaming, StreamingExtension},
        svn_no_error, SubversionErrorCode,
    },
    utils::CStringer,
};

use super::ffi;
use super::ContextInner;

pub struct TunnelResult {
    request: Box<dyn Streaming>,
    response: Box<dyn Streaming>,
}

pub trait Tuunel {
    fn tunnel(
        &self,
        user: Option<&str>,
        host: &str,
        port: u16,
        closed: oneshot::Receiver<()>,
    ) -> Result<TunnelResult, FrontendError>;
}

pub unsafe extern "C" fn check_tunnel(
    baton: *mut c_void,
    name: *const c_char,
) -> ffi::svn_boolean_t {
    unsafe {
        let context = (baton as *mut ContextInner).as_mut().expect("Failed to cast baton to mutable reference");

        if let Some(name) = name.to_nullable_str() {
            context.tunnel.contains_key(name).into()
        } else {
            1
        }
    }
}
pub unsafe extern "C" fn open_tunnel(
    request: *mut *mut ffi::svn_stream_t,
    response: *mut *mut ffi::svn_stream_t,
    close_func: *mut ffi::svn_ra_close_tunnel_func_t,
    close_baton: *mut *mut c_void,
    tunnel_baton: *mut c_void,
    tunnel_name: *const c_char,
    user: *const c_char, // may be null
    hostname: *const c_char,
    port: c_int,
    cancel_func: ffi::svn_cancel_func_t,
    cancel_baton: *mut c_void,
    pool: *mut ffi::apr_pool_t,
) -> *mut ffi::svn_error_t {
    unsafe extern "C" fn close(close_baton: *mut c_void, _: *mut c_void) {
        tracing::info!("Request to close: {:?}", close_baton);
        unsafe {
            let sender = (close_baton as *mut Option<oneshot::Sender<()>>)
                .as_mut()
                .expect("Unexpected failure");

            let Some(sender) = std::mem::take(sender) else {
                tracing::warn!("Tunnel has closed");
                return;
            };

            if sender.send(()).is_err() {
                tracing::warn!("Failed to send close signal");
            }
        }
    }
    if let Some(func) = cancel_func {
        unsafe {
            let e = func(cancel_baton);
            if !e.is_null() {
                return e;
            }
        }
    }
    let (sender, receiver) = oneshot::channel::<()>();
    unsafe {
        let context = (tunnel_baton as *mut ContextInner).as_mut().expect("Failed to cast baton to mutable reference");

        let transparent = transparent_tunnel();
        let tunnel = if let Some(tunnel_name) = tunnel_name.to_nullable_str() {
            let Some(tunnel) = context.tunnel.get(tunnel_name) else {
                return ffi::svn_error_create(
                    SubversionErrorCode::UnsupportedFeature.to_i32(),
                    std::ptr::null_mut(),
                    format!("unsupported tunnel: {}", tunnel_name).as_ptr() as _,
                );
            };
            tunnel
        } else {
            &transparent
        };
        let user = user.to_nullable_str();
        let host = hostname.to_str();
        let port = u16::try_from(port).expect("Unexpected failure");

        let result = match tunnel.tunnel(user, host, port, receiver) {
            Ok(v) => v,
            Err(e) => return e.native_error(),
        };

        let sender = pool.boxed(Some(sender));
        *close_func = Some(close);
        *close_baton = sender as _;
        *request = result.request.into_stream(pool);
        *response = result.response.into_stream(pool);
    }

    svn_no_error()
}

pub fn ssh_tunnel(notifier: Arc<dyn super::ContextNotifier>) -> Box<dyn Tuunel + Send> {
    Box::new(SSH { notifier })
}

struct SSH {
    notifier: Arc<dyn super::ContextNotifier>,
}

impl Tuunel for SSH {
    fn tunnel(
        &self,
        user: Option<&str>,
        host: &str,
        port: u16,
        closed: oneshot::Receiver<()>,
    ) -> Result<TunnelResult, FrontendError> {
        let (request1, mut request2) = ChannelStream::create();
        let (response1, response2) = ChannelStream::create();
        let addr = format!("{}:{}", host, if port == 0 { 22 } else { port });
        // let handle = uniffi::deps::async_compat::get_runtime_handle();
        // let _guard = handle.enter();
        tokio::spawn(async move {
            let stream = tokio::net::TcpStream::connect(&addr).await?;
            let config = flatline::session::Config::default();
            let session = flatline::session::Session::handshake(
                stream,
                config,
                flatline::session::DefaultNotifier::default(),
            )
            .await
            .expect("Unhandled");

            session
                .request_authentication()
                .await
                .expect("handle later");

            let status = session
                .authenticate_password("holdxen", "xxxxxxx")
                .await
                .expect("handle later");
            if !status.success() {
                tracing::error!("Failed to authenticate: {:?}", status);
            }

            let channel = session.channel_open_default().await.expect("Failed to open SSH channel");

            channel.request_exec(true, "svnserve -t").await.expect("Failed to execute command on SSH channel");

            let mut channel = flatline::channel::BufferChannel::new(channel);

            // let mut tcp = TcpStream::connect(addr).await?;
            let mut request_buf = vec![0; 8192];

            let max_len = 1024 * 1024 * 64;

            let mut closed = std::pin::pin!(closed);

            loop {
                tokio::select! {
                    result = &mut closed => {
                        if result.is_err() {
                            tracing::warn!("Unexpected shutdown");
                        }
                        break;
                    }
                    result = request2.read_async(&mut request_buf) => {
                        let size = result?;
                        tracing::info!("Got size: {}", size);
                        if size == 0 {
                            tracing::info!("Request closed");
                            break;
                        }

                        channel.send(&request_buf[..size]).await.expect("handle later");

                        if size == request_buf.len() && request_buf.len() < max_len {
                            request_buf.resize(request_buf.len() * 2, 0);
                        }
                    }
                    result = channel.fill() => {
                        let buf = result.expect("handle later");
                        let len = buf.len();
                        tracing::info!("Got size: {}", len);
                        if len == 0 {
                            tracing::info!("Tcp closed");
                            break;
                        }
                        response2.write_async(buf).await?;
                        channel.consumer_read_buffer(len);
                    }
                }
            }

            error::ok(())
        });

        Ok(TunnelResult {
            request: Box::new(request1),
            response: Box::new(response1),
        })
    }
}

struct Transparent {}

impl Tuunel for Transparent {
    #[tracing::instrument(skip(self))]
    fn tunnel(
        &self,
        _: Option<&str>,
        host: &str,
        port: u16,
        mut closed: oneshot::Receiver<()>,
    ) -> Result<TunnelResult, FrontendError> {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tracing::info!("Got handle: {:?} {:?}", handle.name(), handle.id());
        } else {
            tracing::info!("We don't have handle");
        }
        let (request1, mut request2) = ChannelStream::create();
        let (response1, response2) = ChannelStream::create();
        let addr = format!("{}:{}", host, if port == 0 { 3690 } else { port });
        // let handle = uniffi::deps::async_compat::get_runtime_handle();
        // let _guard = handle.enter();
        // tracing::info!("Enter handle: {:?} {:?}", handle.name(), handle.id());
        tokio::spawn(async move {
            tracing::info!("Start tunnel");
            let result = async {
                let mut tcp = TcpStream::connect(addr).await?;

                let mut buf = Vec::with_capacity(8192);
                let mut request_buf = vec![0; 8192];

                let max_len = 1024 * 1024 * 64;

                // let mut closed = std::pin::pin!(closed);

                loop {
                    tokio::select! {
                        result = &mut closed => {
                            if result.is_err() {
                                tracing::warn!("Unexpected shutdown");
                            }
                            tracing::info!("Shutdown now");
                            break;
                        }
                        result = request2.read_async(&mut request_buf) => {
                            let size = result?;
                            tracing::info!("Got size: {}", size);
                            if size == 0 {
                                tracing::info!("Request closed");
                                break;
                            }

                            tcp.write_all(&request_buf[..size]).await?;

                            if size == request_buf.len() && request_buf.len() < max_len {
                                request_buf.resize(request_buf.len() * 2, 0);
                            }
                        }
                        result = tcp.read_buf(&mut buf) => {
                            let size = result?;
                            tracing::info!("Got size: {}", size);
                            if size == 0 {
                                tracing::info!("Tcp closed");
                                break;
                            }
                            response2.write_async(&buf[..size]).await?;
                            buf.clear();
                        }
                    }
                }

                error::ok(())
            }
            .await;

            tracing::info!("transparent tunnel finshed: {:#?}", result);
        });

        tracing::info!("Return tunnel result");
        Ok(TunnelResult {
            request: Box::new(request1),
            response: Box::new(response1),
        })
    }
}

pub fn transparent_tunnel() -> Box<dyn Tuunel + Send> {
    Box::new(Transparent {})
}
