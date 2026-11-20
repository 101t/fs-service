//! Native async client for the FreeSWITCH Event Socket (mod_event_socket, inbound mode).
//!
//! Replaces the SWIG/C++ `ESL` binding from v1. One TCP connection is shared by the whole
//! service: commands are pipelined (ESL answers in order), events are pushed to a channel,
//! and the connection re-establishes itself with exponential backoff.

use std::{
    collections::{HashMap, VecDeque},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::{mpsc, oneshot},
    time::{sleep, timeout},
};

const REPLY_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_BODY: usize = 16 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum EslError {
    #[error("FreeSWITCH is not connected")]
    Disconnected,
    #[error("FreeSWITCH did not answer in time")]
    Timeout,
    #[error("FreeSWITCH: {0}")]
    Command(String),
    #[error("invalid command: line breaks are not allowed")]
    InvalidCommand,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// One ESL message: headers plus an optional body.
#[derive(Debug, Clone, Default)]
pub struct Frame {
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Frame {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(key).map(String::as_str)
    }
}

/// A FreeSWITCH event (`text/event-plain`), values already URL-decoded.
pub type Event = Frame;

type Pending = (String, oneshot::Sender<Frame>);

pub struct Esl {
    commands: mpsc::Sender<Pending>,
    connected: Arc<AtomicBool>,
}

impl Esl {
    /// Connects in the background and keeps reconnecting forever.
    /// Events named in `subscribe` are delivered to `events`.
    pub fn spawn(addr: String, password: String, subscribe: &'static str, events: mpsc::Sender<Event>) -> Arc<Self> {
        let (tx, mut rx) = mpsc::channel::<Pending>(1024);
        let connected = Arc::new(AtomicBool::new(false));
        let flag = connected.clone();
        tokio::spawn(async move {
            let mut backoff = Duration::from_secs(1);
            loop {
                let result = session(&addr, &password, subscribe, &mut rx, &events, &flag).await;
                if flag.swap(false, Ordering::SeqCst) {
                    backoff = Duration::from_secs(1);
                }
                match result {
                    Ok(()) => tracing::warn!("FreeSWITCH closed the event socket"),
                    Err(e) => {
                        tracing::warn!(error = %e, "FreeSWITCH event socket unavailable, retrying in {backoff:?}")
                    }
                }
                while rx.try_recv().is_ok() {} // fail queued commands fast: their reply senders drop here
                sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(30));
            }
        });
        Arc::new(Self { commands: tx, connected })
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Runs a blocking API command (`api <cmd>`) and returns its output.
    pub async fn api(&self, cmd: &str) -> Result<String, EslError> {
        let reply = self.send_raw(format!("api {cmd}"), cmd).await?;
        let body = reply.body.unwrap_or_default();
        match body.strip_prefix("-ERR") {
            Some(err) => Err(EslError::Command(err.trim().to_string())),
            None => Ok(body),
        }
    }

    /// Queues a background API command under our own `job_uuid`; the result arrives
    /// later as a `BACKGROUND_JOB` event carrying the same `Job-UUID`.
    pub async fn bgapi(&self, cmd: &str, job_uuid: &str) -> Result<(), EslError> {
        if job_uuid.contains(['\n', '\r']) {
            return Err(EslError::InvalidCommand);
        }
        let reply = self.send_raw(format!("bgapi {cmd}\nJob-UUID: {job_uuid}"), cmd).await?;
        match reply.get("Reply-Text") {
            Some(t) if t.starts_with("+OK") => Ok(()),
            other => Err(EslError::Command(other.unwrap_or("no reply").to_string())),
        }
    }

    /// `checked` is the caller-controlled part: it must stay on one line, otherwise a
    /// value like "x\n\napi shutdown" would smuggle a second command onto the socket.
    async fn send_raw(&self, wire: String, checked: &str) -> Result<Frame, EslError> {
        if checked.contains(['\n', '\r']) {
            return Err(EslError::InvalidCommand);
        }
        if !self.is_connected() {
            return Err(EslError::Disconnected);
        }
        let (tx, rx) = oneshot::channel();
        self.commands.send((wire, tx)).await.map_err(|_| EslError::Disconnected)?;
        match timeout(REPLY_TIMEOUT, rx).await {
            Ok(Ok(frame)) => Ok(frame),
            Ok(Err(_)) => Err(EslError::Disconnected),
            Err(_) => Err(EslError::Timeout),
        }
    }
}

async fn session(
    addr: &str,
    password: &str,
    subscribe: &str,
    commands: &mut mpsc::Receiver<Pending>,
    events: &mpsc::Sender<Event>,
    connected: &AtomicBool,
) -> Result<(), EslError> {
    let stream = timeout(Duration::from_secs(5), TcpStream::connect(addr)).await.map_err(|_| EslError::Timeout)??;
    stream.set_nodelay(true)?;
    let (read, mut write) = stream.into_split();

    // A dedicated reader task keeps frame parsing cancel-safe inside the select! below.
    let (frame_tx, mut frames) = mpsc::channel::<Frame>(1024);
    let reader = tokio::spawn(async move {
        let mut read = BufReader::new(read);
        while let Ok(frame) = read_frame(&mut read).await {
            if frame_tx.send(frame).await.is_err() {
                break;
            }
        }
    });

    let result = async {
        let mut next = async || frames.recv().await.ok_or(EslError::Disconnected);
        if next().await?.get("Content-Type") != Some("auth/request") {
            return Err(EslError::Command("unexpected greeting".into()));
        }
        for cmd in [format!("auth {password}"), format!("event plain {subscribe}")] {
            write.write_all(format!("{cmd}\n\n").as_bytes()).await?;
            let reply = next().await?;
            if !reply.get("Reply-Text").is_some_and(|t| t.starts_with("+OK")) {
                let what = cmd.split(' ').next().unwrap_or_default();
                return Err(EslError::Command(format!("{what} rejected: {}", reply.get("Reply-Text").unwrap_or("?"))));
            }
        }
        connected.store(true, Ordering::SeqCst);
        tracing::info!(%addr, "connected to FreeSWITCH event socket");

        let mut waiting: VecDeque<oneshot::Sender<Frame>> = VecDeque::new();
        loop {
            tokio::select! {
                frame = frames.recv() => {
                    let frame = frame.ok_or(EslError::Disconnected)?;
                    match frame.get("Content-Type") {
                        Some("command/reply" | "api/response") => {
                            if let Some(tx) = waiting.pop_front() {
                                let _ = tx.send(frame);
                            }
                        }
                        Some("text/event-plain") => {
                            if let Some(body) = &frame.body {
                                // Backpressure instead of dropping: losing an event means losing a CDR.
                                let _ = events.send(parse_event(body)).await;
                            }
                        }
                        Some("text/disconnect-notice") => return Ok(()),
                        _ => {}
                    }
                }
                cmd = commands.recv() => {
                    let Some((cmd, tx)) = cmd else { return Ok(()) };
                    write.write_all(format!("{cmd}\n\n").as_bytes()).await?;
                    waiting.push_back(tx);
                }
            }
        }
    }
    .await;
    reader.abort();
    result
}

/// Reads one `Header: value` block terminated by an empty line, then `Content-Length` bytes of body.
async fn read_frame<R: AsyncBufRead + Unpin>(r: &mut R) -> Result<Frame, EslError> {
    let mut frame = Frame::default();
    let mut line = String::new();
    loop {
        line.clear();
        if r.read_line(&mut line).await? == 0 {
            return Err(EslError::Disconnected);
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            if frame.headers.is_empty() {
                continue;
            }
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            frame.headers.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    if let Some(len) = frame.get("Content-Length") {
        let len: usize = len.parse().map_err(|_| EslError::Command("bad Content-Length".into()))?;
        if len > MAX_BODY {
            return Err(EslError::Command("frame too large".into()));
        }
        let mut buf = vec![0; len];
        r.read_exact(&mut buf).await?;
        frame.body = Some(String::from_utf8_lossy(&buf).into_owned());
    }
    Ok(frame)
}

/// Parses a `text/event-plain` body: URL-encoded headers, a blank line, then an optional body.
pub fn parse_event(raw: &str) -> Event {
    let (head, body) = raw.split_once("\n\n").unwrap_or((raw, ""));
    let headers = head
        .lines()
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.to_string(), url_decode(v.strip_prefix(' ').unwrap_or(v))))
        .collect();
    Event { headers, body: (!body.is_empty()).then(|| body.to_string()) }
}

fn url_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = (b[i] == b'%' && i + 2 < b.len())
            .then(|| std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()))
            .flatten();
        match hex {
            Some(byte) => {
                out.push(byte);
                i += 3;
            }
            None => {
                out.push(b[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_percent_escapes() {
        assert_eq!(url_decode("Outbound%20Call%3A%20ok"), "Outbound Call: ok");
        assert_eq!(url_decode("100%"), "100%");
        assert_eq!(url_decode("%4"), "%4");
        assert_eq!(url_decode("%zz"), "%zz");
        assert_eq!(url_decode("caf%C3%A9"), "café");
    }

    #[test]
    fn parses_plain_event() {
        let ev = parse_event("Event-Name: BACKGROUND_JOB\nJob-UUID: abc\nContent-Length: 12\n\n-ERR NO_ROUTE");
        assert_eq!(ev.get("Event-Name"), Some("BACKGROUND_JOB"));
        assert_eq!(ev.get("Job-UUID"), Some("abc"));
        assert_eq!(ev.body.as_deref(), Some("-ERR NO_ROUTE"));
    }

    #[tokio::test]
    async fn reads_frames_with_body() {
        let wire =
            b"Content-Type: api/response\nContent-Length: 5\n\n+OK\n\nContent-Type: command/reply\nReply-Text: +OK\n\n";
        let mut r = BufReader::new(&wire[..]);
        let a = read_frame(&mut r).await.unwrap();
        assert_eq!(a.body.as_deref(), Some("+OK\n\n"));
        let b = read_frame(&mut r).await.unwrap();
        assert_eq!(b.get("Reply-Text"), Some("+OK"));
        assert!(read_frame(&mut r).await.is_err());
    }

    /// End-to-end against a fake FreeSWITCH: auth, pipelined commands, events, injection guard.
    #[tokio::test]
    async fn talks_to_fake_freeswitch() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        tokio::spawn(async move {
            let (s, _) = listener.accept().await.unwrap();
            let (r, mut w) = s.into_split();
            let mut r = BufReader::new(r);
            w.write_all(b"Content-Type: auth/request\n\n").await.unwrap();
            let mut line = String::new();
            loop {
                line.clear();
                if r.read_line(&mut line).await.unwrap() == 0 {
                    break;
                }
                let cmd = line.trim().to_string();
                if cmd.is_empty() || cmd.starts_with("Job-UUID") {
                    continue;
                }
                let reply = if cmd == "auth secret" || cmd.starts_with("event ") || cmd.starts_with("bgapi ") {
                    "Content-Type: command/reply\nReply-Text: +OK\n\n".to_string()
                } else if cmd == "api status" {
                    let ev = "Event-Name: CUSTOM\nX-Test: hello%20world\n\n";
                    format!(
                        "Content-Type: text/event-plain\nContent-Length: {}\n\n{ev}Content-Type: api/response\nContent-Length: 2\n\nUP",
                        ev.len()
                    )
                } else {
                    "Content-Type: api/response\nContent-Length: 13\n\n-ERR no such\n".to_string()
                };
                w.write_all(reply.as_bytes()).await.unwrap();
            }
        });

        let (etx, mut erx) = mpsc::channel(8);
        let esl = Esl::spawn(addr, "secret".into(), "ALL", etx);
        for _ in 0..50 {
            if esl.is_connected() {
                break;
            }
            sleep(Duration::from_millis(20)).await;
        }
        assert_eq!(esl.api("status").await.unwrap(), "UP");
        assert_eq!(erx.recv().await.unwrap().get("X-Test"), Some("hello world"));
        assert!(matches!(esl.api("bogus").await, Err(EslError::Command(_))));
        esl.bgapi("originate x", "job-1").await.unwrap();
        assert!(matches!(esl.api("status\n\napi shutdown").await, Err(EslError::InvalidCommand)));
    }
}
