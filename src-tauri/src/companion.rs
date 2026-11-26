//! Independent, receive-only companion status channel. No flight-control dependencies.
use serde::{Deserialize, Serialize};
use std::{
    io::{self, Read},
    net::{IpAddr, SocketAddr, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const MAX_LINE: usize = 4096;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const LOSS_TIMEOUT: Duration = Duration::from_secs(5);
const RETRIES: usize = 3;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    protocol_version: u8,
    device_id: String,
    timestamp_ms: u64,
    connection_state: DeviceState,
    cpu_usage_percent: Option<f64>,
    memory_usage_percent: Option<f64>,
    cpu_temperature_c: Option<f64>,
    ai_status: Option<ComponentState>,
    camera_status: Option<ComponentState>,
    mapping_status: Option<ComponentState>,
    sensor_status: Option<ComponentState>,
    error_status: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum DeviceState {
    Online,
    Degraded,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ComponentState {
    Ready,
    Running,
    Degraded,
    Error,
    Offline,
}
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}
fn parse_message(line: &[u8], now: u64) -> Result<Message, &'static str> {
    if line.len() > MAX_LINE {
        return Err("메시지 크기 초과");
    }
    let m: Message = serde_json::from_slice(line).map_err(|_| "잘못된 상태 메시지")?;
    if m.protocol_version != 1 {
        return Err("지원하지 않는 프로토콜 버전");
    }
    if m.device_id.is_empty()
        || m.device_id.len() > 64
        || !m
            .device_id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
    {
        return Err("장치 ID 오류");
    }
    if m.timestamp_ms.abs_diff(now) > 300_000 {
        return Err("메시지 시각 오류 (시계 동기화 확인)");
    }
    if [m.cpu_usage_percent, m.memory_usage_percent]
        .into_iter()
        .flatten()
        .any(|v| !v.is_finite() || !(0.0..=100.0).contains(&v))
        || m.cpu_temperature_c
            .is_some_and(|v| !v.is_finite() || !(-50.0..=150.0).contains(&v))
    {
        return Err("측정값 범위 오류");
    }
    if m.error_status
        .as_ref()
        .is_some_and(|s| s.len() > 256 || s.chars().any(char::is_control))
    {
        return Err("오류 상태 길이 또는 문자 오류");
    }
    Ok(m)
}
#[derive(Clone, Serialize)]
pub struct Snapshot {
    phase: &'static str,
    endpoint: Option<String>,
    last_received_ms: Option<u64>,
    message: Option<Message>,
    error: Option<String>,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            phase: "DISCONNECTED",
            endpoint: None,
            last_received_ms: None,
            message: None,
            error: None,
        }
    }
}
#[derive(Default)]
pub struct Manager {
    inner: Mutex<Inner>,
}
#[derive(Default)]
struct Inner {
    status: Arc<Mutex<Snapshot>>,
    cancel: Option<Arc<AtomicBool>>,
}
impl Manager {
    fn snapshot(&self) -> Snapshot {
        self.inner.lock().unwrap().status.lock().unwrap().clone()
    }
    fn disconnect(&self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(c) = inner.cancel.take() {
            c.store(true, Ordering::SeqCst);
        }
        inner.status = Arc::new(Mutex::new(Snapshot::default()));
    }
    fn connect(&self, ip: &str, port: u16) -> Result<(), String> {
        let ip: IpAddr = ip.parse().map_err(|_| "숫자 IP 주소를 입력하세요")?;
        if port == 0
            || ip.is_unspecified()
            || ip.is_multicast()
            || matches!(ip, IpAddr::V4(v4) if v4.is_broadcast())
        {
            return Err("유효한 단일 장치 IP와 포트를 입력하세요".into());
        }
        let addr = SocketAddr::new(ip, port);
        let mut inner = self.inner.lock().unwrap();
        if let Some(c) = inner.cancel.take() {
            c.store(true, Ordering::SeqCst);
        }
        let status = Arc::new(Mutex::new(Snapshot {
            phase: "CONNECTING",
            endpoint: Some(addr.to_string()),
            ..Snapshot::default()
        }));
        let cancel = Arc::new(AtomicBool::new(false));
        inner.status = status.clone();
        inner.cancel = Some(cancel.clone());
        thread::spawn(move || run(addr, status, cancel));
        Ok(())
    }
}
fn update(status: &Mutex<Snapshot>, phase: &'static str, error: Option<String>) {
    let mut s = status.lock().unwrap();
    s.phase = phase;
    s.error = error;
}
fn read_session(
    mut stream: TcpStream,
    status: &Mutex<Snapshot>,
    cancel: &AtomicBool,
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_millis(250)))
        .map_err(|e| e.to_string())?;
    let mut line = Vec::with_capacity(MAX_LINE);
    let mut buf = [0u8; 1024];
    let mut last = std::time::Instant::now();
    while !cancel.load(Ordering::SeqCst) {
        match stream.read(&mut buf) {
            Ok(0) => return Err("연결이 종료되었습니다".into()),
            Ok(n) => {
                for &b in &buf[..n] {
                    if b == b'\n' {
                        let m = parse_message(&line, now_ms()).map_err(str::to_owned)?;
                        line.clear();
                        last = std::time::Instant::now();
                        let mut s = status.lock().unwrap();
                        s.phase = "CONNECTED";
                        s.error = None;
                        s.last_received_ms = Some(now_ms());
                        s.message = Some(m);
                    } else {
                        if line.len() >= MAX_LINE {
                            return Err("메시지 크기 초과".into());
                        }
                        line.push(b);
                    }
                }
            }
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::Interrupted
                ) =>
            {
                ()
            }
            Err(e) => return Err(format!("수신 오류: {e}")),
        }
        if last.elapsed() >= LOSS_TIMEOUT {
            return Err("상태 메시지 수신 시간 초과".into());
        }
    }
    Ok(())
}
fn run(addr: SocketAddr, status: Arc<Mutex<Snapshot>>, cancel: Arc<AtomicBool>) {
    for attempt in 0..=RETRIES {
        if cancel.load(Ordering::SeqCst) {
            return;
        }
        update(
            &status,
            if attempt == 0 {
                "CONNECTING"
            } else {
                "RECONNECTING"
            },
            None,
        );
        let result = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT)
            .map_err(|e| format!("연결 실패: {e}"))
            .and_then(|stream| {
                update(&status, "WAITING", None);
                read_session(stream, &status, &cancel)
            });
        if cancel.load(Ordering::SeqCst) {
            return;
        }
        let err = result.err().unwrap_or_else(|| "연결 종료".into());
        update(
            &status,
            if attempt == RETRIES {
                "LOST"
            } else {
                "RECONNECTING"
            },
            Some(err),
        );
        if attempt < RETRIES {
            for _ in 0..10 {
                if cancel.load(Ordering::SeqCst) {
                    return;
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}
#[tauri::command]
pub fn companion_connect(
    manager: tauri::State<'_, Manager>,
    ip: String,
    port: u16,
) -> Result<(), String> {
    manager.connect(&ip, port)
}
#[tauri::command]
pub fn companion_disconnect(manager: tauri::State<'_, Manager>) {
    manager.disconnect();
}
#[tauri::command]
pub fn companion_status(manager: tauri::State<'_, Manager>) -> Snapshot {
    manager.snapshot()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn message(now: u64) -> String {
        format!(
            r#"{{"protocol_version":1,"device_id":"pi-1","timestamp_ms":{now},"connection_state":"online","cpu_usage_percent":12.5,"ai_status":"running"}}"#
        )
    }
    #[test]
    fn schema_and_invalid_input() {
        let now = now_ms();
        let valid = message(now);
        let m = parse_message(valid.as_bytes(), now).unwrap();
        assert!(m.camera_status.is_none());
        for bad in [
            valid.replace("\"protocol_version\":1", "\"protocol_version\":2"),
            valid.replace("pi-1", "bad/id"),
            valid.replace("12.5", "101"),
            valid.replace(&now.to_string(), "1"),
            valid.replace("running", "unknown"),
            format!("{} ", " ".repeat(MAX_LINE)),
        ] {
            assert!(parse_message(bad.as_bytes(), now).is_err());
        }
    }
    #[test]
    fn invalid_endpoint_and_independent_state() {
        let m = Manager::default();
        assert!(m.connect("localhost", 123).is_err());
        assert!(m.connect("0.0.0.0", 123).is_err());
        assert_eq!(m.snapshot().phase, "DISCONNECTED");
        m.disconnect();
        assert_eq!(m.snapshot().phase, "DISCONNECTED");
    }
    #[test]
    fn loopback_receive_and_loss() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            use std::io::Write;
            let (mut s, _) = listener.accept().unwrap();
            writeln!(s, "{}", message(now_ms())).unwrap();
            thread::sleep(Duration::from_millis(300));
        });
        let m = Manager::default();
        m.connect("127.0.0.1", port).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while m.snapshot().phase != "CONNECTED" && std::time::Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        let s = m.snapshot();
        assert_eq!(s.phase, "CONNECTED");
        assert!(s.last_received_ms.is_some());
        assert!(s.message.is_some());
        server.join().unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while m.snapshot().phase == "CONNECTED" && std::time::Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert_ne!(m.snapshot().phase, "CONNECTED");
        m.disconnect();
        assert_eq!(m.snapshot().phase, "DISCONNECTED");
    }
}
