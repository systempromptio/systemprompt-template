//! One running app per user and build. A later start hands its `LaunchRequest`
//! to the running app and exits.
//!
//! On Windows the rendezvous is a named pipe whose name carries the build's app
//! id and a hash of its data root, so each channel and each user has its own;
//! remote clients are refused. On macOS (the `.app` bundle and any other Unix
//! build) it is a Unix socket in the state directory. The wire is one JSON line
//! each way: the request, then `ok`. The listener runs as an owned task of the
//! services' `BackgroundTasks`, so it stops at shutdown.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::io;
use std::time::Duration;

use harness_tasks::BackgroundTasks;
use sp_brand::Identity;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::sync::mpsc::UnboundedSender;

use crate::app::deeplink::LaunchRequest;
use crate::app::paths::AppPaths;

const ACK: &str = "ok";
const MAX_REQUEST: u64 = 64 * 1024;
const ANSWER_WAIT: Duration = Duration::from_secs(3);
const ATTEMPTS: u64 = 3;
const LISTEN_TASK: &str = "single-instance";

/// Where the running app listens for later starts of the same build and user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    #[cfg(windows)]
    pipe: String,
    #[cfg(unix)]
    socket: std::path::PathBuf,
}

impl Endpoint {
    pub fn for_app(identity: &Identity, paths: &AppPaths) -> Self {
        platform::endpoint(identity, paths)
    }
}

/// The outcome of a start: this process owns the endpoint, handed its request
/// on, or runs without one.
#[derive(Debug)]
pub enum Instance {
    Primary(platform::Listener),
    Forwarded,
    Alone,
}

pub async fn acquire(endpoint: &Endpoint, request: &LaunchRequest) -> Instance {
    for attempt in 0..ATTEMPTS {
        match forward(endpoint, request).await {
            Ok(()) => return Instance::Forwarded,
            Err(error) => tracing::debug!(error = %error, attempt, "no running instance answered"),
        }
        match platform::listen(endpoint) {
            Ok(listener) => return Instance::Primary(listener),
            Err(error) => {
                tracing::debug!(error = %error, attempt, "the instance endpoint is taken");
                tokio::time::sleep(Duration::from_millis(100 * (attempt + 1))).await;
            },
        }
    }
    tracing::warn!("cannot reach or become the running instance; starting alone");
    Instance::Alone
}

pub fn serve(
    listener: platform::Listener,
    endpoint: Endpoint,
    tasks: &BackgroundTasks,
    requests: UnboundedSender<LaunchRequest>,
) {
    tasks.spawn_cancellable(LISTEN_TASK, move |cancel| async move {
        tokio::select! {
            () = cancel.cancelled() => {},
            result = platform::accept_loop(listener, &endpoint, requests) => {
                if let Err(error) = result {
                    tracing::warn!(error = %error, "stopped listening for other starts");
                }
            }
        }
    });
}

async fn forward(endpoint: &Endpoint, request: &LaunchRequest) -> io::Result<()> {
    let stream = platform::connect(endpoint).await?;
    let mut line = serde_json::to_string(request).map_err(io::Error::other)?;
    line.push('\n');
    let (read, mut write) = tokio::io::split(stream);
    write.write_all(line.as_bytes()).await?;
    write.flush().await?;
    let mut answer = String::new();
    let mut reader = BufReader::new(read.take(MAX_REQUEST));
    tokio::time::timeout(ANSWER_WAIT, reader.read_line(&mut answer))
        .await
        .map_err(|elapsed| io::Error::new(io::ErrorKind::TimedOut, elapsed))??;
    if answer.trim() == ACK {
        Ok(())
    } else {
        Err(io::Error::other(
            "the running instance did not take the request",
        ))
    }
}

async fn answer<S>(stream: S, requests: &UnboundedSender<LaunchRequest>) -> io::Result<bool>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let (read, mut write) = tokio::io::split(stream);
    let mut line = String::new();
    let mut reader = BufReader::new(read.take(MAX_REQUEST));
    tokio::time::timeout(ANSWER_WAIT, reader.read_line(&mut line))
        .await
        .map_err(|elapsed| io::Error::new(io::ErrorKind::TimedOut, elapsed))??;
    let request: LaunchRequest = serde_json::from_str(line.trim()).map_err(io::Error::other)?;
    let taken = requests.send(request).is_ok();
    let reply: &[u8] = if taken { b"ok\n" } else { b"closing\n" };
    write.write_all(reply).await?;
    write.flush().await?;
    Ok(taken)
}

pub fn pipe_name(app_id: &str, data_root: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in data_root.to_lowercase().bytes() {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
    }
    format!(r"\\.\pipe\{app_id}-{hash:016x}")
}

#[cfg(windows)]
mod platform {
    use std::io;
    use std::time::Duration;

    use sp_brand::Identity;
    use tokio::net::windows::named_pipe::{
        ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions,
    };
    use tokio::sync::mpsc::UnboundedSender;

    use super::{Endpoint, answer, pipe_name};
    use crate::app::deeplink::LaunchRequest;
    use crate::app::paths::AppPaths;

    const PIPE_BUSY: i32 = 231;
    const BUSY_RETRIES: u32 = 20;

    pub(crate) type Listener = NamedPipeServer;

    pub(super) fn endpoint(identity: &Identity, _paths: &AppPaths) -> Endpoint {
        Endpoint {
            pipe: pipe_name(identity.app_id(), &identity.data_root().to_string_lossy()),
        }
    }

    pub(super) fn listen(endpoint: &Endpoint) -> io::Result<NamedPipeServer> {
        ServerOptions::new()
            .first_pipe_instance(true)
            .reject_remote_clients(true)
            .create(&endpoint.pipe)
    }

    pub(super) async fn connect(endpoint: &Endpoint) -> io::Result<NamedPipeClient> {
        let mut retries = 0;
        loop {
            match ClientOptions::new().open(&endpoint.pipe) {
                Ok(client) => return Ok(client),
                Err(error) if error.raw_os_error() == Some(PIPE_BUSY) && retries < BUSY_RETRIES => {
                    retries += 1;
                    tokio::time::sleep(Duration::from_millis(50)).await;
                },
                Err(error) => return Err(error),
            }
        }
    }

    pub(super) async fn accept_loop(
        mut server: NamedPipeServer,
        endpoint: &Endpoint,
        requests: UnboundedSender<LaunchRequest>,
    ) -> io::Result<()> {
        loop {
            server.connect().await?;
            let next = ServerOptions::new()
                .reject_remote_clients(true)
                .create(&endpoint.pipe)?;
            let connected = std::mem::replace(&mut server, next);
            match answer(connected, &requests).await {
                Ok(true) => {},
                Ok(false) => return Ok(()),
                Err(error) => tracing::debug!(error = %error, "ignored a malformed start request"),
            }
        }
    }
}

#[cfg(unix)]
mod platform {
    use std::io;

    use sp_brand::Identity;
    use tokio::net::{UnixListener, UnixStream};
    use tokio::sync::mpsc::UnboundedSender;

    use super::{Endpoint, answer};
    use crate::app::deeplink::LaunchRequest;
    use crate::app::paths::AppPaths;

    pub(crate) type Listener = UnixListener;

    pub(super) fn endpoint(_identity: &Identity, paths: &AppPaths) -> Endpoint {
        Endpoint {
            socket: paths.instance_socket(),
        }
    }

    pub(super) fn listen(endpoint: &Endpoint) -> io::Result<UnixListener> {
        match UnixListener::bind(&endpoint.socket) {
            Ok(listener) => Ok(listener),
            Err(error) if error.kind() == io::ErrorKind::AddrInUse => {
                std::fs::remove_file(&endpoint.socket)?;
                UnixListener::bind(&endpoint.socket)
            },
            Err(error) => Err(error),
        }
    }

    pub(super) async fn connect(endpoint: &Endpoint) -> io::Result<UnixStream> {
        UnixStream::connect(&endpoint.socket).await
    }

    pub(super) async fn accept_loop(
        listener: UnixListener,
        _endpoint: &Endpoint,
        requests: UnboundedSender<LaunchRequest>,
    ) -> io::Result<()> {
        loop {
            let (stream, _) = listener.accept().await?;
            match answer(stream, &requests).await {
                Ok(true) => {},
                Ok(false) => return Ok(()),
                Err(error) => tracing::debug!(error = %error, "ignored a malformed start request"),
            }
        }
    }
}

#[cfg(not(any(windows, unix)))]
mod platform {
    use std::io;

    use sp_brand::Identity;
    use tokio::io::DuplexStream;
    use tokio::sync::mpsc::UnboundedSender;

    use super::Endpoint;
    use crate::app::deeplink::LaunchRequest;
    use crate::app::paths::AppPaths;

    pub(crate) type Listener = ();

    pub(super) const fn endpoint(_identity: &Identity, _paths: &AppPaths) -> Endpoint {
        Endpoint {}
    }

    pub(super) fn listen(_endpoint: &Endpoint) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }

    pub(super) async fn connect(_endpoint: &Endpoint) -> io::Result<DuplexStream> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }

    pub(super) async fn accept_loop(
        _listener: (),
        _endpoint: &Endpoint,
        _requests: UnboundedSender<LaunchRequest>,
    ) -> io::Result<()> {
        Ok(())
    }
}
