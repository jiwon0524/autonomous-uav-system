use serde::{Deserialize, Serialize};
use std::{
    io::{self, Read},
    net::{IpAddr, SocketAddr, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
const MAX_POINTS: usize = 100_000;
const MAX_HEADER: usize = 4096;
const STRIDE: usize = 15;
const TIMEOUT: Duration = Duration::from_secs(5);
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    protocol_version: u8,
    device_id: String,
    session_id: String,
    map_id: String,
    frame_id: u64,
    timestamp_ms: u64,
    coordinate_frame: String,
    point_count: usize,
    position_encoding: String,
    color_encoding: String,
    payload_length: usize,
    map_revision: u64,
    data_type: String,
    mission_id: Option<String>,
}
fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
}
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn validate(h: &Header, p: &[u8], now: u64) -> Result<(), &'static str> {
    if h.protocol_version != 1
        || ![&h.device_id, &h.session_id, &h.map_id]
            .iter()
            .all(|s| valid_id(s))
        || h.mission_id.as_deref().is_some_and(|s| !valid_id(s))
        || h.frame_id > 9_007_199_254_740_991
        || h.map_revision > 9_007_199_254_740_991
        || h.timestamp_ms.abs_diff(now) > 5000
        || !matches!(
            h.coordinate_frame.as_str(),
            "local_map" | "enu" | "ned" | "camera_optical" | "camera_body"
        )
        || h.point_count == 0
        || h.point_count > MAX_POINTS
        || h.position_encoding != "xyz_f32_le"
        || h.color_encoding != "rgb8"
        || h.payload_length != h.point_count * STRIDE
        || p.len() != h.payload_length
        || h.data_type != "snapshot"
    {
        return Err("점군 메타데이터 또는 길이 오류");
    }
    for pt in p.chunks_exact(STRIDE) {
        for c in pt[..12].chunks_exact(4) {
            let v = f32::from_le_bytes(c.try_into().unwrap());
            if !v.is_finite() || v.abs() > 100_000.0 {
                return Err("점군 좌표 오류");
            }
        }
    }
    Ok(())
}
#[derive(Clone, Serialize)]
pub struct Status {
    phase: &'static str,
    error: Option<String>,
    received_ms: Option<u64>,
    header: Option<Header>,
}
impl Default for Status {
    fn default() -> Self {
        Self {
            phase: "OFFLINE",
            error: None,
            received_ms: None,
            header: None,
        }
    }
}
#[derive(Default)]
struct Shared {
    status: Status,
    latest: Option<Vec<u8>>,
}
#[derive(Default)]
pub struct Manager {
    inner: Mutex<Inner>,
}
#[derive(Default)]
struct Inner {
    shared: Arc<Mutex<Shared>>,
    cancel: Option<Arc<AtomicBool>>,
}
impl Manager {
    fn disconnect(&self) {
        let mut i = self.inner.lock().unwrap();
        if let Some(c) = i.cancel.take() {
            c.store(true, Ordering::SeqCst)
        }
        i.shared = Arc::new(Mutex::new(Shared::default()));
    }
    fn connect(&self, ip: &str, port: u16) -> Result<(), String> {
        let ip: IpAddr = ip.parse().map_err(|_| "숫자 IP 주소를 입력하세요")?;
        if port == 0
            || ip.is_unspecified()
            || ip.is_multicast()
            || matches!(ip,IpAddr::V4(v) if v.is_broadcast())
        {
            return Err("유효한 장치 주소와 포트를 입력하세요".into());
        }
        let mut i = self.inner.lock().unwrap();
        if let Some(c) = i.cancel.take() {
            c.store(true, Ordering::SeqCst)
        }
        let shared = Arc::new(Mutex::new(Shared::default()));
        shared.lock().unwrap().status.phase = "CONNECTING";
        let cancel = Arc::new(AtomicBool::new(false));
        i.shared = shared.clone();
        i.cancel = Some(cancel.clone());
        thread::spawn(move || run(SocketAddr::new(ip, port), shared, cancel));
        Ok(())
    }
    fn status(&self) -> Status {
        self.inner
            .lock()
            .unwrap()
            .shared
            .lock()
            .unwrap()
            .status
            .clone()
    }
    fn take(&self) -> Vec<u8> {
        self.inner
            .lock()
            .unwrap()
            .shared
            .lock()
            .unwrap()
            .latest
            .take()
            .unwrap_or_default()
    }
}
fn read(s: &mut TcpStream, b: &mut [u8], c: &AtomicBool, last: &Instant) -> Result<(), String> {
    let mut n = 0;
    while n < b.len() {
        if c.load(Ordering::SeqCst) {
            return Err("연결 해제".into());
        }
        if last.elapsed() > TIMEOUT {
            return Err("점군 수신 시간 초과".into());
        }
        match s.read(&mut b[n..]) {
            Ok(0) => return Err("점군 연결 종료".into()),
            Ok(k) => n += k,
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
            Err(e) => return Err(format!("점군 수신 오류: {e}")),
        }
    }
    Ok(())
}
fn receive(
    mut stream: TcpStream,
    shared: &Mutex<Shared>,
    cancel: &AtomicBool,
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_millis(200)))
        .map_err(|e| e.to_string())?;
    let mut last = Instant::now();
    let mut previous: Option<(String, String, u64, u64)> = None;
    let mut last_packet = None;
    loop {
        let mut sizes = [0; 8];
        read(&mut stream, &mut sizes, cancel, &last)?;
        let hl = u32::from_be_bytes(sizes[..4].try_into().unwrap()) as usize;
        let pl = u32::from_be_bytes(sizes[4..].try_into().unwrap()) as usize;
        if hl == 0 || hl > MAX_HEADER || pl == 0 || pl > MAX_POINTS * STRIDE {
            return Err("점군 크기 초과".into());
        }
        let mut hdr = vec![0; hl];
        let mut payload = vec![0; pl];
        read(&mut stream, &mut hdr, cancel, &last)?;
        read(&mut stream, &mut payload, cancel, &last)?;
        let h: Header = serde_json::from_slice(&hdr).map_err(|_| "점군 헤더 오류")?;
        validate(&h, &payload, now_ms()).map_err(str::to_owned)?;
        if let Some((session, map, frame, rev)) = &previous {
            if *session == h.session_id
                && *map == h.map_id
                && (h.frame_id <= *frame || h.map_revision <= *rev)
            {
                return Err("프레임 또는 리비전 순서 오류".into());
            }
        }
        if last_packet.is_some_and(|p: Instant| p.elapsed() < Duration::from_millis(500)) {
            return Err("점군 전송 속도 초과".into());
        }
        last_packet = Some(Instant::now());
        last = Instant::now();
        previous = Some((
            h.session_id.clone(),
            h.map_id.clone(),
            h.frame_id,
            h.map_revision,
        ));
        let mut packet = Vec::with_capacity(4 + hl + pl);
        packet.extend_from_slice(&(hl as u32).to_be_bytes());
        packet.extend_from_slice(&hdr);
        packet.extend_from_slice(&payload);
        let mut s = shared.lock().unwrap();
        s.status = Status {
            phase: "LIVE",
            error: None,
            received_ms: Some(now_ms()),
            header: Some(h),
        };
        s.latest = Some(packet);
    }
}
fn run(addr: SocketAddr, shared: Arc<Mutex<Shared>>, cancel: Arc<AtomicBool>) {
    let result = TcpStream::connect_timeout(&addr, Duration::from_secs(3))
        .map_err(|e| format!("점군 연결 실패: {e}"))
        .and_then(|s| receive(s, &shared, &cancel));
    if !cancel.load(Ordering::SeqCst) {
        let mut s = shared.lock().unwrap();
        s.status.phase = "LOST";
        s.status.error = result.err();
        s.latest = None;
    }
}
#[tauri::command]
pub fn spatial_connect(
    manager: tauri::State<'_, Manager>,
    ip: String,
    port: u16,
) -> Result<(), String> {
    manager.connect(&ip, port)
}
#[tauri::command]
pub fn spatial_disconnect(manager: tauri::State<'_, Manager>) {
    manager.disconnect()
}
#[tauri::command]
pub fn spatial_status(manager: tauri::State<'_, Manager>) -> Status {
    manager.status()
}
#[tauri::command]
pub fn spatial_take(manager: tauri::State<'_, Manager>) -> tauri::ipc::Response {
    tauri::ipc::Response::new(manager.take())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn header() -> Header {
        Header {
            protocol_version: 1,
            device_id: "pi".into(),
            session_id: "s1".into(),
            map_id: "map".into(),
            frame_id: 1,
            timestamp_ms: now_ms(),
            coordinate_frame: "local_map".into(),
            point_count: 1,
            position_encoding: "xyz_f32_le".into(),
            color_encoding: "rgb8".into(),
            payload_length: 15,
            map_revision: 1,
            data_type: "snapshot".into(),
            mission_id: None,
        }
    }
    #[test]
    fn rejects_invalid_points_and_metadata() {
        let h = header();
        let p = [0u8; 15];
        assert!(validate(&h, &p, now_ms()).is_ok());
        assert!(validate(&h, &p[..14], now_ms()).is_err());
        let mut n = p;
        n[..4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(validate(&h, &n, now_ms()).is_err());
        let mut b = h.clone();
        b.point_count = MAX_POINTS + 1;
        assert!(validate(&b, &p, now_ms()).is_err());
        b = h.clone();
        b.data_type = "delta".into();
        assert!(validate(&b, &p, now_ms()).is_err());
    }
    #[test]
    fn local_stream_and_session_change() {
        use std::io::Write;
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = l.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut s, _) = l.accept().unwrap();
            for session in ["one", "two"] {
                let mut h = header();
                h.timestamp_ms = now_ms();
                h.session_id = session.into();
                let raw = serde_json::to_vec(&h).unwrap();
                s.write_all(&(raw.len() as u32).to_be_bytes()).unwrap();
                s.write_all(&15u32.to_be_bytes()).unwrap();
                s.write_all(&raw).unwrap();
                s.write_all(&[0u8; 15]).unwrap();
                thread::sleep(Duration::from_millis(550));
            }
        });
        let shared = Arc::new(Mutex::new(Shared::default()));
        let copy = shared.clone();
        let w = thread::spawn(move || {
            receive(
                TcpStream::connect(addr).unwrap(),
                &copy,
                &AtomicBool::new(false),
            )
        });
        server.join().unwrap();
        assert!(w.join().unwrap().is_err());
        let s = shared.lock().unwrap();
        assert_eq!(s.status.header.as_ref().unwrap().session_id, "two");
        assert!(s.latest.is_some());
    }
}
