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
const MAX_HEADER: usize = 16384;
const MAX_JPEG: usize = 512 * 1024;
const MAX_FPS: u32 = 10;
const TIMEOUT: Duration = Duration::from_secs(3);
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Detection {
    object_id: String,
    class_id: String,
    class_name: String,
    confidence: f64,
    bbox: [f64; 4],
    tracking_id: Option<String>,
    depth_m: Option<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    protocol_version: u8,
    device_id: String,
    frame_id: u64,
    timestamp_ms: u64,
    width: u32,
    height: u32,
    model_id: String,
    detection_frame_id: u64,
    detection_timestamp_ms: u64,
    detections: Vec<Detection>,
}
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
}
fn validate(h: &Header, now: u64) -> Result<(), &'static str> {
    if h.protocol_version != 1
        || !valid_id(&h.device_id)
        || !valid_id(&h.model_id)
        || h.detection_frame_id != h.frame_id
        || h.detection_timestamp_ms != h.timestamp_ms
        || h.frame_id > 9_007_199_254_740_991
        || h.timestamp_ms.abs_diff(now) > 5000
        || h.width == 0
        || h.height == 0
        || h.width > 1920
        || h.height > 1080
        || h.detections.len() > 50
    {
        return Err("영상 메타데이터 오류");
    }
    for d in &h.detections {
        let [x, y, w, ht] = d.bbox;
        if !valid_id(&d.object_id)
            || !valid_id(&d.class_id)
            || d.class_name.is_empty()
            || d.class_name.len() > 80
            || d.class_name.chars().any(char::is_control)
            || d.tracking_id.as_deref().is_some_and(|s| !valid_id(s))
            || !d.confidence.is_finite()
            || !(0.0..=1.0).contains(&d.confidence)
            || [x, y, w, ht].iter().any(|v| !v.is_finite())
            || x < 0.0
            || y < 0.0
            || w <= 0.0
            || ht <= 0.0
            || x + w > 1.0
            || y + ht > 1.0
            || d.depth_m
                .is_some_and(|v| !v.is_finite() || v <= 0.0 || v > 1000.0)
        {
            return Err("객체 탐지 데이터 오류");
        }
    }
    Ok(())
}
// Read JPEG SOF dimensions without decoding or allocating an image. WebView decoding is checked separately.
fn jpeg_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 4 || !data.starts_with(&[0xff, 0xd8]) || !data.ends_with(&[0xff, 0xd9]) {
        return None;
    }
    let mut i = 2;
    while i + 3 < data.len() {
        if data[i] != 0xff {
            return None;
        }
        while i < data.len() && data[i] == 0xff {
            i += 1;
        }
        if i >= data.len() {
            return None;
        }
        let marker = data[i];
        i += 1;
        if marker == 0xd9 || marker == 0xda {
            break;
        }
        if marker == 0x01 || (0xd0..=0xd7).contains(&marker) {
            continue;
        }
        if i + 2 > data.len() {
            return None;
        }
        let n = u16::from_be_bytes([data[i], data[i + 1]]) as usize;
        if n < 2 || i + n > data.len() {
            return None;
        }
        if matches!(marker, 0xc0 | 0xc1 | 0xc2) {
            if n < 8 {
                return None;
            }
            return Some((
                u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32,
                u16::from_be_bytes([data[i + 3], data[i + 4]]) as u32,
            ));
        }
        i += n;
    }
    None
}
#[derive(Clone, Serialize)]
pub struct Status {
    phase: &'static str,
    error: Option<String>,
    received_ms: Option<u64>,
    width: Option<u32>,
    height: Option<u32>,
    fps: f64,
}
impl Default for Status {
    fn default() -> Self {
        Self {
            phase: "OFFLINE",
            error: None,
            received_ms: None,
            width: None,
            height: None,
            fps: 0.0,
        }
    }
}
struct Shared {
    status: Status,
    latest: Option<Vec<u8>>,
}
impl Default for Shared {
    fn default() -> Self {
        Self {
            status: Status::default(),
            latest: None,
        }
    }
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
        let addr = SocketAddr::new(ip, port);
        let mut i = self.inner.lock().unwrap();
        if let Some(c) = i.cancel.take() {
            c.store(true, Ordering::SeqCst)
        }
        let shared = Arc::new(Mutex::new(Shared::default()));
        shared.lock().unwrap().status.phase = "CONNECTING";
        let cancel = Arc::new(AtomicBool::new(false));
        i.shared = shared.clone();
        i.cancel = Some(cancel.clone());
        thread::spawn(move || run(addr, shared, cancel));
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
fn read_exact_timeout(
    s: &mut TcpStream,
    buf: &mut [u8],
    cancel: &AtomicBool,
    last: &Instant,
) -> Result<(), String> {
    let mut n = 0;
    while n < buf.len() {
        if cancel.load(Ordering::SeqCst) {
            return Err("연결 해제".into());
        }
        if last.elapsed() > TIMEOUT {
            return Err("프레임 수신 시간 초과".into());
        }
        match s.read(&mut buf[n..]) {
            Ok(0) => return Err("영상 연결 종료".into()),
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
            Err(e) => return Err(format!("영상 수신 오류: {e}")),
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
    let mut previous = None;
    let mut interval = Instant::now();
    let mut count = 0u32;
    let mut fps = 0.;
    let mut last_id = None;
    loop {
        let mut lengths = [0; 8];
        read_exact_timeout(&mut stream, &mut lengths, cancel, &last)?;
        let hl = u32::from_be_bytes(lengths[..4].try_into().unwrap()) as usize;
        let jl = u32::from_be_bytes(lengths[4..].try_into().unwrap()) as usize;
        if hl == 0 || hl > MAX_HEADER || jl == 0 || jl > MAX_JPEG {
            return Err("영상 데이터 크기 초과".into());
        }
        let mut header = vec![0; hl];
        let mut jpeg = vec![0; jl];
        read_exact_timeout(&mut stream, &mut header, cancel, &last)?;
        read_exact_timeout(&mut stream, &mut jpeg, cancel, &last)?;
        let h: Header = serde_json::from_slice(&header).map_err(|_| "잘못된 영상 메타데이터")?;
        validate(&h, now_ms()).map_err(str::to_owned)?;
        if jpeg_dimensions(&jpeg) != Some((h.width, h.height)) {
            return Err("JPEG 형식 또는 크기 오류".into());
        }
        if last_id.is_some_and(|id| h.frame_id <= id) {
            return Err("프레임 순서 오류".into());
        }
        last_id = Some(h.frame_id);
        let now = Instant::now();
        if previous.is_some_and(|p: Instant| {
            now.duration_since(p) < Duration::from_millis(1000 / MAX_FPS as u64)
        }) {
            return Err("영상 전송 속도 초과".into());
        }
        previous = Some(now);
        last = now;
        count += 1;
        if interval.elapsed() >= Duration::from_secs(1) {
            fps = count as f64 / interval.elapsed().as_secs_f64();
            count = 0;
            interval = now;
        }
        let mut packet = Vec::with_capacity(4 + hl + jl);
        packet.extend_from_slice(&(hl as u32).to_be_bytes());
        packet.extend_from_slice(&header);
        packet.extend_from_slice(&jpeg);
        let mut s = shared.lock().unwrap();
        s.status = Status {
            phase: "LIVE",
            error: None,
            received_ms: Some(now_ms()),
            width: Some(h.width),
            height: Some(h.height),
            fps,
        };
        s.latest = Some(packet);
    }
}
fn run(addr: SocketAddr, shared: Arc<Mutex<Shared>>, cancel: Arc<AtomicBool>) {
    let result = TcpStream::connect_timeout(&addr, Duration::from_secs(3))
        .map_err(|e| format!("영상 연결 실패: {e}"))
        .and_then(|stream| receive(stream, &shared, &cancel));
    if !cancel.load(Ordering::SeqCst) {
        let mut s = shared.lock().unwrap();
        s.status.phase = "LOST";
        s.status.error = result.err();
        s.latest = None;
    }
}
#[tauri::command]
pub fn camera_connect(
    manager: tauri::State<'_, Manager>,
    ip: String,
    port: u16,
) -> Result<(), String> {
    manager.connect(&ip, port)
}
#[tauri::command]
pub fn camera_disconnect(manager: tauri::State<'_, Manager>) {
    manager.disconnect()
}
#[tauri::command]
pub fn camera_status(manager: tauri::State<'_, Manager>) -> Status {
    manager.status()
}
#[tauri::command]
pub fn camera_take_frame(manager: tauri::State<'_, Manager>) -> tauri::ipc::Response {
    tauri::ipc::Response::new(manager.take())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn header() -> Header {
        let now = now_ms();
        Header {
            protocol_version: 1,
            device_id: "pi".into(),
            frame_id: 1,
            timestamp_ms: now,
            width: 640,
            height: 480,
            model_id: "model".into(),
            detection_frame_id: 1,
            detection_timestamp_ms: now,
            detections: vec![Detection {
                object_id: "1".into(),
                class_id: "person".into(),
                class_name: "person".into(),
                confidence: 0.8,
                bbox: [0.1, 0.2, 0.3, 0.4],
                tracking_id: None,
                depth_m: Some(2.),
            }],
        }
    }
    #[test]
    fn validation() {
        let mut h = header();
        h.timestamp_ms = now_ms();
        h.detection_timestamp_ms = h.timestamp_ms;
        assert!(validate(&h, now_ms()).is_ok());
        h.detection_frame_id = 2;
        assert!(validate(&h, now_ms()).is_err());
        h.detection_frame_id = 1;
        h.detections[0].bbox = [0.9, 0.2, 0.3, 0.4];
        assert!(validate(&h, now_ms()).is_err());
        h = header();
        h.protocol_version = 2;
        assert!(validate(&h, now_ms()).is_err());
        h = header();
        h.timestamp_ms = 1;
        h.detection_timestamp_ms = 1;
        assert!(validate(&h, now_ms()).is_err());
        h = header();
        h.detections = vec![h.detections[0].clone(); 51];
        assert!(validate(&h, now_ms()).is_err());
    }
    #[test]
    fn jpeg_header() {
        let data = [
            0xff, 0xd8, 0xff, 0xc0, 0, 8, 8, 1, 0xe0, 2, 0x80, 0, 0xff, 0xd9,
        ];
        assert_eq!(jpeg_dimensions(&data), Some((640, 480)));
        assert_eq!(jpeg_dimensions(&data[..10]), None);
    }
    #[test]
    fn endpoint() {
        let m = Manager::default();
        assert!(m.connect("localhost", 1).is_err());
        assert_eq!(m.status().phase, "OFFLINE");
        assert!(m.take().is_empty());
    }
    #[test]
    fn receives_frame_and_rejects_oversized_packet() {
        use std::io::Write;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let t = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut frame = header();
            frame.timestamp_ms = now_ms();
            frame.detection_timestamp_ms = frame.timestamp_ms;
            let h = serde_json::to_vec(&frame).unwrap();
            let jpg = include_bytes!("../../tools/camera_test_fixture.jpg");
            stream.write_all(&(h.len() as u32).to_be_bytes()).unwrap();
            stream.write_all(&(jpg.len() as u32).to_be_bytes()).unwrap();
            stream.write_all(&h).unwrap();
            stream.write_all(jpg).unwrap();
            thread::sleep(Duration::from_millis(200));
            stream
                .write_all(&((MAX_HEADER + 1) as u32).to_be_bytes())
                .unwrap();
            stream.write_all(&1u32.to_be_bytes()).unwrap();
        });
        let shared = Arc::new(Mutex::new(Shared::default()));
        let copy = shared.clone();
        let worker = thread::spawn(move || {
            receive(
                TcpStream::connect(addr).unwrap(),
                &copy,
                &AtomicBool::new(false),
            )
        });
        thread::sleep(Duration::from_millis(100));
        {
            let s = shared.lock().unwrap();
            assert_eq!(s.status.phase, "LIVE");
            assert!(s.latest.is_some());
        }
        assert!(worker.join().unwrap().unwrap_err().contains("크기"));
        t.join().unwrap();
    }
}
