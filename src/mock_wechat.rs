//! Test-only mock of the WeChat MP HTTP API.
//!
//! The production client talks to `https://api.weixin.qq.com` over `ureq`.
//! Pointing `WechatClient::with_base_url` at this listener lets tests exercise
//! the real request/response code path — URL building, JSON bodies, multipart
//! uploads, `errcode` handling — without network access or credentials.
//!
//! Each canned response answers exactly one request, in order. The server
//! thread stops after the last response (or a 20s idle deadline), so tests
//! never need an explicit shutdown call.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Recorded {
    pub method: String,
    /// Request target as sent by the client, e.g. `/cgi-bin/token?appid=x`.
    pub target: String,
    pub body: Vec<u8>,
}

impl Recorded {
    pub fn path(&self) -> &str {
        self.target.split('?').next().unwrap_or(&self.target)
    }

    pub fn query(&self) -> &str {
        self.target.split_once('?').map(|(_, q)| q).unwrap_or("")
    }

    pub fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

pub struct MockWechat {
    base_url: String,
    requests: Arc<Mutex<Vec<Recorded>>>,
}

impl MockWechat {
    /// Start a listener that answers `responses` one request at a time.
    pub fn start<I, S>(responses: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let responses: Vec<String> = responses.into_iter().map(Into::into).collect();
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
        listener
            .set_nonblocking(true)
            .expect("mock server nonblocking");
        let addr = listener.local_addr().expect("mock server addr");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let store = Arc::clone(&requests);

        std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(20);
            let mut served = 0usize;
            while served < responses.len() && Instant::now() < deadline {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_nonblocking(false).ok();
                        let Some(request) = read_request(&mut stream) else {
                            continue;
                        };
                        store.lock().expect("mock server store").push(request);
                        let body = &responses[served];
                        served += 1;
                        write_response(&mut stream, body);
                    }
                    Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            base_url: format!("http://{addr}"),
            requests,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn requests(&self) -> Vec<Recorded> {
        self.requests.lock().expect("mock server store").clone()
    }

    pub fn request_count(&self) -> usize {
        self.requests.lock().expect("mock server store").len()
    }

    /// Block until at least `count` requests have arrived (or 5s elapse).
    pub fn wait_for(&self, count: usize) -> Vec<Recorded> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let seen = self.requests();
            if seen.len() >= count || Instant::now() >= deadline {
                return seen;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

fn write_response(stream: &mut TcpStream, body: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn read_request(stream: &mut TcpStream) -> Option<Recorded> {
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
    let mut buf: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 8192];

    loop {
        if let Some(head_end) = find(&buf, b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
            let content_length = header_value(&head, "content-length")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            let body_start = head_end + 4;
            while buf.len() < body_start + content_length {
                let read = stream.read(&mut chunk).ok()?;
                if read == 0 {
                    break;
                }
                buf.extend_from_slice(&chunk[..read]);
            }
            let mut head_parts = head.lines().next().unwrap_or("").split_whitespace();
            let method = head_parts.next().unwrap_or_default().to_owned();
            let target = head_parts.next().unwrap_or_default().to_owned();
            let end = (body_start + content_length).min(buf.len());
            return Some(Recorded {
                method,
                target,
                body: buf[body_start..end].to_vec(),
            });
        }
        let read = stream.read(&mut chunk).ok()?;
        if read == 0 {
            return None;
        }
        buf.extend_from_slice(&chunk[..read]);
    }
}

fn header_value(head: &str, name: &str) -> Option<String> {
    head.lines().skip(1).find_map(|line| {
        let (key, value) = line.split_once(':')?;
        key.trim()
            .eq_ignore_ascii_case(name)
            .then(|| value.trim().to_owned())
    })
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
