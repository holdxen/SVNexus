use std::{
    ffi::{c_char, c_int, c_void},
    path::PathBuf,
    sync::Arc,
};

use flatline::session::{AuthenticationMethod, InteractiveMethod, KeyboardInteractive, Prompt};
use snafu::ResultExt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::oneshot,
};

use crate::{
    apr::AprPool,
    error::{self, builder, FrontendError},
    subversion::{
        identity,
        stream::{ChannelStream, Streaming, StreamingExtension},
        svn_no_error, SubversionErrorCode,
    },
    utils::CStringer,
};

use super::ContextInner;
use super::{ffi, SSHAuthetication};

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
        let context = (baton as *mut ContextInner)
            .as_mut()
            .expect("Failed to cast baton to mutable reference");

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
        let context = (tunnel_baton as *mut ContextInner)
            .as_mut()
            .expect("Failed to cast baton to mutable reference");

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

#[derive(Clone, derive_more::Debug)]
struct SSHRunner {
    #[debug(skip)]
    notifier: Arc<dyn super::ContextNotifier>,
    host: String,
    ip: Option<String>,
}

impl SSHRunner {}

impl flatline::session::Notifier for SSHRunner {
    async fn verify_server_host_key(&mut self, r#type: &str, host_key: &[u8]) -> bool {
        let mut user_known_hosts = vec![];
        if let Some(home) = std::env::home_dir() {
            let file = home.join(".ssh").join("known_hosts");
            if file.exists() && file.is_file() {
                user_known_hosts.push(file);
            }
        }
        let mut system_known_hosts = vec![];

        #[cfg(unix)]
        {
            let file = PathBuf::from("/etc/ssh/known_hosts");
            if file.exists() && file.is_file() {
                system_known_hosts.push(file);
            }
        }

        let key_type = r#type.to_string();
        let host_key = host_key.to_vec();
        let host = self.host.clone();
        let ip = self.ip.clone();
        let notifier = self.notifier.clone();
        let result = tokio::task::spawn_blocking(move || {
            known_hosts::check_and_store_hostkey_in_files(
                &user_known_hosts,
                &system_known_hosts,
                &host,
                ip.as_ref().map(|v| v.as_str()),
                &key_type,
                &host_key,
                false,
                |confirm| {
                    //
                    match confirm {
                        known_hosts::HostkeyConfirm::NewHost(prompt) => {
                            //
                            notifier
                                .ssh_verify_new_host_key(
                                    prompt.host,
                                    prompt.ip,
                                    &prompt.key_type.to_string(),
                                    prompt.key_data,
                                    prompt.fingerprint.as_str(),
                                )
                                .unwrap_or(false)
                        }
                        known_hosts::HostkeyConfirm::IpChanged(prompt) => notifier
                            .ssh_verify_ip_changed(
                                prompt.host,
                                prompt.ip,
                                prompt.key_type.to_string().as_str(),
                                prompt.key_data,
                                prompt.fingerprint.as_str(),
                            )
                            .unwrap_or(false),
                    }
                },
                |w| tracing::warn!("Got ssh warning: {:?}", w),
            )
        })
        .await;

        match result {
            Ok(Ok(result)) => return result.accepted,
            result => {
                tracing::info!("Host key was rejected by user: {:?}", result);
                false
            }
        }
    }

    async fn server_host_keys(&mut self, _: &[&[u8]]) -> bool {
        true
    }

    async fn x11_forward(
        &mut self,
        originator: flatline::forward::SocketAddr,
        _receiver: oneshot::Receiver<flatline::forward::Stream>,
        _initial_window_size: &mut u32,
        _maximum_packet_size: &mut u32,
    ) -> bool {
        tracing::warn!("Unexpected x11 forward: {:?}", originator);
        false
    }

    async fn agent_forward(
        &mut self,
        _receiver: oneshot::Receiver<flatline::forward::Stream>,
        _initial_window_size: &mut u32,
        _maximum_packet_size: &mut u32,
    ) -> bool {
        tracing::warn!("Unexpected agent forward");
        false
    }

    async fn disconnected(
        &mut self,
        reason: flatline::ssh::msg::DisconnectReason,
        description: &str,
    ) {
        tracing::info!("SSH connection disconnected: {:?}:{}", reason, description)
    }

    async fn exited(&mut self, result: flatline::error::Result<()>) {
        tracing::info!("SSH connection destory: {:?}", result);
    }
}

#[async_trait::async_trait]
impl KeyboardInteractive for SSHRunner {
    async fn interactive(
        &mut self,
        name: &str,
        instruction: &str,
        prompts: &[Prompt<'_>],
    ) -> flatline::error::Result<Vec<String>> {
        if prompts.is_empty() {
            tracing::info!("Empty prompts");
            return Ok(vec![]);
        }
        let prompts: Vec<_> = prompts.into_iter().map(|v| (v.content, v.echo)).collect();
        let response = self
            .notifier
            .ssh_keyboard_interactive(name, instruction, &prompts)
            .await
            .whatever_context::<_, flatline::error::Error>(
                "Failed to authenticate with keyboard interactive",
            )?;
        Ok(response)
    }
}

/// 按 `ssh_identity` 给出的顺序尝试自动发现的私钥，等价于 OpenSSH 的
/// `sshconnect2.c pubkey_prepare()` + 逐把 offer。
///
/// 返回值：
/// * `Ok(None)` — 这一轮什么都没试（没有候选 / 拿不到用户名），调用方
///   继续走原来的人机交互流程；
/// * `Ok(Some(Success | PasswordChangeRequired))` — 已有结论，交给外层
///   循环处理；
/// * `Ok(Some(Failure))` — 最后一把被拒，外层据此刷新 `allow_methods`
///   后进入对话框。
async fn try_identities(
    session: &flatline::session::Session,
    notifier: &Arc<dyn super::ContextNotifier>,
    username: Option<&str>,
    candidates: &[identity::Candidate],
) -> Result<Option<flatline::session::AuthenticateResult>, FrontendError> {
    use flatline::session::AuthenticateResult;

    let Some(username) = username else {
        tracing::debug!("No username for public key authentication, identities skipped");
        return Ok(None);
    };

    // sshconnect2.c load_identity_file(): 口令最多问 number_of_password_prompts(3) 次
    const MAX_PASSPHRASE_PROMPTS: usize = 3;

    let mut last: Option<AuthenticateResult> = None;

    for candidate in candidates {
        let mut passphrase: Option<Vec<u8>> = None;
        let mut prompts = 0usize;
        let mut reported_wrong = false;

        loop {
            let path = candidate.path.display().to_string();

            // 需要口令的两种状态会落到 match 之后，其余分支要么有结论要么 break
            match identity::load(candidate, passphrase.as_deref()) {
                identity::Identity::Plain {
                    bytes,
                    key_type,
                    comment,
                } => {
                    let comment = if comment.is_empty() {
                        "<no comment>"
                    } else {
                        comment.as_str()
                    };
                    tracing::info!("Offering public key: {path} ({key_type}, {comment})");
                    let result = session
                        .authenticate_public_key(
                            username,
                            bytes,
                            None::<&[u8]>,
                            passphrase.as_deref(),
                        )
                        .await?;
                    tracing::info!("Public key {path} result: {result:?}");
                    match result {
                        // 已经有结论，交给外层循环
                        AuthenticateResult::Success
                        | AuthenticateResult::PasswordChangeRequired => {
                            return Ok(Some(result));
                        }
                        // 服务器不认这把，按 OpenSSH 的顺序换下一把
                        AuthenticateResult::Failure { .. } => {
                            last = Some(result);
                            break;
                        }
                    }
                }
                identity::Identity::Encrypted { key_type } => {
                    let key_type = key_type.as_deref().unwrap_or("unknown type");
                    tracing::debug!("Identity {path} is encrypted ({key_type})");
                }
                identity::Identity::WrongPassphrase { detail } => {
                    tracing::warn!("Wrong passphrase for {path}: {detail}");
                }
                // load() 已经按 user_provided 级别记过 "no such identity"
                identity::Identity::Missing => break,
                identity::Identity::PublicOnly {
                    key_type,
                    comment,
                    detail,
                } => {
                    let comment = comment.as_deref().unwrap_or("no comment");
                    tracing::debug!("Skipping identity {path} ({key_type}, {comment}): {detail}");
                    break;
                }
                identity::Identity::Unusable { detail } => {
                    tracing::debug!("Skipping identity {path}: {detail}");
                    break;
                }
            }

            if prompts >= MAX_PASSPHRASE_PROMPTS {
                tracing::info!("Too many passphrase prompts for {path}, try next key");
                break;
            }
            prompts += 1;

            match notifier.ssh_passphrase(&path, Some(username), reported_wrong).await? {
                Some(passphrase_text) => {
                    passphrase = Some(passphrase_text.into_bytes());
                    // 下一轮若还是没解开，就告诉前端"口令不对"
                    reported_wrong = true;
                }
                None => {
                    // sshconnect2.c: debug2("no passphrase given, try next key")
                    tracing::info!("No passphrase given for {path}, try next key");
                    break;
                }
            }
        }
    }

    Ok(last)
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

        // 身份文件发现要的是"裸"主机名与实际端口：下面的 host 会被改写成
        // [host]:port 形式（SSH 握手用），端口 0 也要折算成 22（%p）。
        let identity_host = host.to_string();
        let identity_port = if port == 0 { 22 } else { port };

        let host = if port == 0 || port == 22 {
            host.to_string()
        } else {
            format!("[{}]:{}", host, port)
        };
        let user = user.map(|v| v.to_string());
        // ssh_identity 负责 config 语义（默认列表 / Host / Include / % 展开），
        // 这里只做发现，文件是否可用由 identity::load 决定。
        let candidates = identity::discover(&identity_host, identity_port, user.as_deref());
        // URL 没带用户名时用本机用户名尝试自动发现的私钥（OpenSSH 同样默认
        // 本机用户）；连本机用户名都拿不到就只能交给对话框去问。
        let identity_user = user.clone().or_else(|| {
            std::env::var("USER")
                .ok()
                .or_else(|| std::env::var("USERNAME").ok())
        });
        let notifier = self.notifier.clone();
        let runtime_handle = tauri::async_runtime::handle().inner().clone();

        // 握手阶段（DNS / TCP / SSH 认证 / exec）完全不依赖 request1，可以在这里同步等待：
        // 失败的错误会作为 open_tunnel 的返回值，直接以 svn_error 的形式交给 svn。
        let (session, mut channel) = runtime_handle.block_on(async move {
            let hosts: Vec<_> = tokio::net::lookup_host(&addr).await?.collect();
            if hosts.is_empty() {
                tracing::error!("Failed to lookip host: {}", addr);
                return builder::General {
                    detail: format!("Failed to resolve host: {}", addr),
                }
                .fail();
            }
            let mut stream = None;
            let mut ip = None;
            for i in hosts {
                let result = tokio::net::TcpStream::connect(&i).await;
                if let Ok(s) = result {
                    stream = Some(s);
                    ip = Some(i.ip());
                    break;
                }
            }
            let Some(stream) = stream else {
                return builder::General {
                    detail: format!("Failed to connect {}", addr),
                }
                .fail();
            };
            let ip = ip.map(|v| {
                if port == 0 || port == 22 {
                    v.to_string()
                } else {
                    format!("[{}]:{}", v, port)
                }
            });
            let runner = SSHRunner {
                host,
                ip,
                notifier: notifier.clone(),
            };

            let config = flatline::session::Config::default();

            let session =
                flatline::session::Session::handshake(stream, config, runner.clone()).await?;

            session.request_authentication().await?;

            let mut password = true;

            let mut public_key = true;

            let mut keyboard_interactive = true;

            let mut first = true;

            // 自动发现的私钥只按顺序试一轮，试完（无论成败）都回到人机交互流程
            let mut tried_identities = false;

            loop {
                // 内层不再用 loop：每条路径都是 break，clippy::never_loop 会判为 error。
                let mut none_result = None;
                if first {
                    first = false;
                    if let Some(ref user) = user {
                        none_result = Some(session.authenticate_none(user).await?);
                    }
                }

                // none 探测（如果有）已经带回 allow_methods，据此决定要不要试公钥，
                // 免得服务器根本没提供 publickey 时白跑一趟。
                if none_result.is_none() && !tried_identities && public_key {
                    tried_identities = true;
                    none_result =
                        try_identities(&session, &notifier, identity_user.as_deref(), &candidates)
                            .await?;
                }

                let result = match none_result {
                    Some(result) => result,
                    None => {
                        let authentication = notifier
                            .ssh_authenticate(
                                password,
                                public_key,
                                keyboard_interactive,
                                user.as_deref(),
                            )
                            .await?;
                        match authentication {
                            SSHAuthetication::Password { password, username } => {
                                session.authenticate_password(username, password).await?
                            }
                            SSHAuthetication::PublicKey {
                                file,
                                username,
                                passphrase,
                            } => {
                                let content = tokio::fs::read(file).await?;
                                session
                                    .authenticate_public_key(
                                        &username,
                                        &content,
                                        None::<&[u8]>,
                                        passphrase.as_ref().map(|v| v.as_bytes()),
                                    )
                                    .await?
                            }
                            SSHAuthetication::KeyboardInteractive { username } => {
                                session
                                    .authenticate_keyboard_interactive(
                                        username,
                                        Box::new(runner.clone()),
                                        vec![InteractiveMethod::PAM, InteractiveMethod::BSD],
                                    )
                                    .await?
                            }
                        }
                    }
                };

                tracing::info!("SSH authenticate result: {:?}", result);

                match result {
                    flatline::session::AuthenticateResult::Success => break,
                    flatline::session::AuthenticateResult::PasswordChangeRequired => {
                        return builder::General {
                            detail: "Require to change password",
                        }
                        .fail();
                    }
                    flatline::session::AuthenticateResult::Failure { allow_methods, .. } => {
                        //
                        password = allow_methods.contains(&AuthenticationMethod::Password);
                        public_key = allow_methods.contains(&AuthenticationMethod::PublicKey);
                        keyboard_interactive =
                            allow_methods.contains(&AuthenticationMethod::KeyboardInteractive);
                    }
                }
            }

            let channel = session.channel_open_default().await?;

            channel.request_exec(true, "svnserve -t").await?;

            tracing::info!("Tunnel ssh handshake done");

            error::ok((session, flatline::channel::BufferChannel::new(channel)))
        })?;

        // 转发循环必须读 request1，而 request1 要等 open_tunnel 返回 svn 才会写入，
        // 所以这里只能 spawn，绝不能 block_on（否则两边互相等待，直接死锁）。
        let future = async move {
            // session 必须一直存活，否则 SSH 连接会被断开
            let _session = session;

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

                        channel.send(&request_buf[..size]).await?;

                        if size == request_buf.len() && request_buf.len() < max_len {
                            request_buf.resize(request_buf.len() * 2, 0);
                        }
                    }
                    result = channel.fill() => {
                        let buf = result?;
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
        };

        runtime_handle.spawn(async move {
            let result = future.await;
            tracing::info!("Tunnel ssh result: {:?}", result);
            result
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
