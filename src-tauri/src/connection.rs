use mavlink::{
    dialects::common::MavMessage, error::MessageReadError, MAVLinkMessageRaw, MavlinkReader,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{self, Read},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Phase {
    Disconnected,
    PortDetected,
    Connecting,
    WaitingHeartbeat,
    Connected,
    ConnectionLost,
    Error,
}
#[derive(Clone, Serialize)]
struct Vehicle {
    system_id: u32,
    component_id: u8,
    vehicle_type: u8,
    autopilot_type: u8,
    mavlink_version: u8,
    heartbeat_ms: u64,
}
#[derive(Clone, Serialize)]
pub struct Snapshot {
    phase: Phase,
    port: Option<String>,
    source: String,
    vehicle: Option<Vehicle>,
    last_heartbeat_ms: Option<u64>,
    error: Option<String>,
}
#[derive(Serialize)]
pub struct Port {
    name: String,
    manufacturer: Option<String>,
    description: Option<String>,
    vid: u16,
    pid: u16,
    serial_number: Option<String>,
    access: String,
}
#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct AutoDevice {
    vid: u16,
    pid: u16,
    serial: String,
    baud: u32,
}
struct Worker {
    cancel: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}
pub struct Manager {
    snapshot: Arc<Mutex<Snapshot>>,
    worker: Mutex<Option<Worker>>,
    auto_allowed: Mutex<Option<AutoDevice>>,
}
impl Default for Manager {
    fn default() -> Self {
        Self {
            snapshot: Arc::new(Mutex::new(Snapshot {
                phase: Phase::Disconnected,
                port: None,
                source: "OFFLINE".into(),
                vehicle: None,
                last_heartbeat_ms: None,
                error: None,
            })),
            worker: Mutex::new(None),
            auto_allowed: Mutex::new(None),
        }
    }
}
impl Manager {
    fn stop(&self) {
        if let Some(worker) = self.worker.lock().unwrap().take() {
            worker.cancel.store(true, Ordering::SeqCst);
            let _ = worker.handle.join();
        }
    }
    pub fn disconnect(&self) {
        *self.auto_allowed.lock().unwrap() = None;
        self.stop();
        *self.snapshot.lock().unwrap() = Snapshot {
            phase: Phase::Disconnected,
            port: None,
            source: "OFFLINE".into(),
            vehicle: None,
            last_heartbeat_ms: None,
            error: None,
        };
    }
    pub fn list(&self) -> Result<Vec<Port>, String> {
        let ports = serialport::available_ports()
            .map_err(|e| format!("포트 목록 조회 실패: {e}"))?
            .into_iter()
            .filter_map(|p| match p.port_type {
                serialport::SerialPortType::UsbPort(info) => Some(Port {
                    access: access(&p.port_name),
                    name: p.port_name,
                    manufacturer: info.manufacturer,
                    description: info.product,
                    vid: info.vid,
                    pid: info.pid,
                    serial_number: info.serial_number,
                }),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut s = self.snapshot.lock().unwrap();
        if matches!(s.phase, Phase::Disconnected | Phase::PortDetected) {
            s.phase = if ports.is_empty() {
                Phase::Disconnected
            } else {
                Phase::PortDetected
            };
        }
        Ok(ports)
    }
    pub fn connect(&self, port: String, baud: u32) -> Result<(), String> {
        if ![57600, 115200, 230400, 460800, 921600].contains(&baud) {
            return Err("지원하지 않는 통신 속도입니다.".into());
        }
        let mut worker = self.worker.lock().unwrap();
        if worker.as_ref().is_some_and(|w| !w.handle.is_finished()) {
            return Err("기존 연결을 먼저 해제하세요.".into());
        }
        if let Some(old) = worker.take() {
            let _ = old.handle.join();
        }
        if !self.list()?.iter().any(|p| p.name == port) {
            return Err("목록에서 확인된 USB 직렬 포트를 선택하세요.".into());
        }
        *self.snapshot.lock().unwrap() = Snapshot {
            phase: Phase::Connecting,
            port: Some(port.clone()),
            source: "HARDWARE_LINK".into(),
            vehicle: None,
            last_heartbeat_ms: None,
            error: None,
        };
        let shared = self.snapshot.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let handle = thread::spawn(move || run(port, baud, shared, flag));
        *worker = Some(Worker { cancel, handle });
        Ok(())
    }
    pub fn set_auto(&self, device: Option<AutoDevice>) -> Result<(), String> {
        if let Some(d) = &device {
            if d.serial.trim().is_empty()
                || ![57600, 115200, 230400, 460800, 921600].contains(&d.baud)
            {
                return Err("자동 연결 장치 정보가 올바르지 않습니다.".into());
            }
        }
        *self.auto_allowed.lock().unwrap() = device;
        Ok(())
    }
    pub fn connect_authorized(&self, port: String, device: AutoDevice) -> Result<(), String> {
        let allowed = self.auto_allowed.lock().unwrap();
        if allowed.as_ref() != Some(&device) {
            return Err("자동 연결을 허용하지 않은 장치입니다.".into());
        }
        let ports = self.list()?;
        let matches: Vec<_> = ports
            .iter()
            .filter(|p| {
                p.vid == device.vid
                    && p.pid == device.pid
                    && p.serial_number.as_deref() == Some(device.serial.as_str())
            })
            .collect();
        if matches.len() != 1 || matches[0].name != port {
            return Err("허용 장치가 없거나 식별 정보가 중복됩니다.".into());
        }
        self.connect(port, device.baud)
    }
    pub fn connect_simulation(&self) -> Result<(), String> {
        self.simulation_at("127.0.0.1:14540")
    }
    fn simulation_at(&self, address: &str) -> Result<(), String> {
        let mut worker = self.worker.lock().unwrap();
        if worker.as_ref().is_some_and(|w| !w.handle.is_finished()) {
            return Err("기존 연결을 먼저 해제하세요.".into());
        }
        if let Some(old) = worker.take() {
            let _ = old.handle.join();
        }
        let socket = std::net::UdpSocket::bind(address)
            .map_err(|e| format!("시뮬레이션 UDP 열기 실패: {e}"))?;
        socket
            .set_read_timeout(Some(Duration::from_millis(50)))
            .map_err(|e| e.to_string())?;
        let local_address = socket.local_addr().map_err(|e| e.to_string())?.to_string();
        *self.snapshot.lock().unwrap() = Snapshot {
            phase: Phase::WaitingHeartbeat,
            port: Some(local_address),
            source: "SIMULATION".into(),
            vehicle: None,
            last_heartbeat_ms: None,
            error: None,
        };
        let shared = self.snapshot.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let handle = thread::spawn(move || {
            receive(
                UdpReader(socket),
                shared,
                flag,
                Duration::from_secs(10),
                Duration::from_secs(5),
            )
        });
        *worker = Some(Worker { cancel, handle });
        Ok(())
    }
    fn snapshot(&self) -> Snapshot {
        self.snapshot.lock().unwrap().clone()
    }
}
impl Drop for Manager {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(target_os = "linux")]
fn access(path: &str) -> String {
    use std::ffi::CString;
    match CString::new(path) {
        Ok(c) => if unsafe { libc::access(c.as_ptr(), libc::R_OK) } == 0 {
            "readable"
        } else {
            "unavailable"
        }
        .into(),
        Err(_) => "unknown".into(),
    }
}
#[cfg(not(target_os = "linux"))]
fn access(_: &str) -> String {
    "unknown".into()
}

// The descriptor itself has no write permission; no command or byte-writing API exists.
#[cfg(target_os = "linux")]
fn open_readonly(path: &str, baud: u32) -> io::Result<std::fs::File> {
    use std::{
        fs::OpenOptions,
        os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    };
    let f = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOCTTY | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)?;
    let fd = f.as_raw_fd();
    unsafe {
        if libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) != 0 {
            return Err(io::Error::last_os_error());
        }
        if libc::ioctl(fd, libc::TIOCEXCL) != 0 {
            return Err(io::Error::last_os_error());
        }
        let mut t: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(fd, &mut t) != 0 {
            return Err(io::Error::last_os_error());
        }
        libc::cfmakeraw(&mut t);
        t.c_cflag &= !(libc::CRTSCTS | libc::CSTOPB);
        t.c_cflag |= libc::CLOCAL | libc::CREAD;
        let speed = match baud {
            57600 => libc::B57600,
            115200 => libc::B115200,
            230400 => libc::B230400,
            460800 => libc::B460800,
            921600 => libc::B921600,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "baud")),
        };
        if libc::cfsetispeed(&mut t, speed) != 0
            || libc::cfsetospeed(&mut t, speed) != 0
            || libc::tcsetattr(fd, libc::TCSANOW, &t) != 0
        {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(f)
}
#[cfg(not(target_os = "linux"))]
fn open_readonly(_: &str, _: u32) -> io::Result<std::fs::File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "이 플랫폼의 읽기 전용 직렬 연결은 아직 지원되지 않습니다",
    ))
}

fn valid_vehicle(raw: &MAVLinkMessageRaw) -> Option<Vehicle> {
    if raw.message_id() != 0 || raw.system_id() == 0 || raw.component_id() == 0 {
        return None;
    }
    if let MAVLinkMessageRaw::V2(v) = raw {
        if v.incompatibility_flags() != 0 {
            return None;
        }
    }
    let p = raw.payload();
    if p.len() != 9
        || p[5] != 12
        || !matches!(p[4], 1..=4 | 10..=15 | 19..=24 | 29 | 35 | 43 | 47)
        || p[8] != 3
    {
        return None;
    }
    Some(Vehicle {
        system_id: raw.system_id(),
        component_id: raw.component_id(),
        vehicle_type: p[4],
        autopilot_type: p[5],
        mavlink_version: match raw {
            MAVLinkMessageRaw::V1(_) => 1,
            MAVLinkMessageRaw::V2(_) => 2,
        },
        heartbeat_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    })
}
// Bound work per decoder call so continuous junk cannot starve timeout or disconnect.
struct BoundedReader<R> {
    inner: R,
    remaining: usize,
}
impl<R: Read> Read for BoundedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let limit = buf.len().min(self.remaining);
        let n = self.inner.read(&mut buf[..limit])?;
        self.remaining -= n;
        Ok(n)
    }
}
fn run(port: String, baud: u32, shared: Arc<Mutex<Snapshot>>, cancel: Arc<AtomicBool>) {
    run_with_timeouts(
        port,
        baud,
        shared,
        cancel,
        Duration::from_secs(10),
        Duration::from_secs(5),
    );
}
fn run_with_timeouts(
    port: String,
    baud: u32,
    shared: Arc<Mutex<Snapshot>>,
    cancel: Arc<AtomicBool>,
    wait_timeout: Duration,
    loss_timeout: Duration,
) {
    let file = match open_readonly(&port, baud) {
        Ok(f) => f,
        Err(e) => {
            let mut s = shared.lock().unwrap();
            s.phase = Phase::Error;
            s.error = Some(format!("포트 열기 실패 (권한 또는 점유 상태 확인): {e}"));
            return;
        }
    };
    {
        let mut s = shared.lock().unwrap();
        s.phase = Phase::WaitingHeartbeat;
    }
    receive(file, shared, cancel, wait_timeout, loss_timeout);
}
// Local UDP listener is intentionally receive-only and cannot select remote destinations.
struct UdpReader(std::net::UdpSocket);
impl Read for UdpReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.recv(buf)
    }
}
fn receive<R: Read>(
    input: R,
    shared: Arc<Mutex<Snapshot>>,
    cancel: Arc<AtomicBool>,
    wait_timeout: Duration,
    loss_timeout: Duration,
) {
    let started = Instant::now();
    let mut last = None;
    let mut identity = None;
    let mut reader = MavlinkReader::new(BoundedReader {
        inner: input,
        remaining: 4096,
    });
    loop {
        if cancel.load(Ordering::SeqCst) {
            break;
        }
        reader.get_mut().remaining = 4096;
        match reader.read_any_raw_message::<MavMessage>() {
            Ok(raw) => {
                if let Some(v) = valid_vehicle(&raw) {
                    let id = (v.system_id, v.component_id);
                    if identity.is_none() || identity == Some(id) {
                        identity = Some(id);
                        last = Some(Instant::now());
                        let mut s = shared.lock().unwrap();
                        s.last_heartbeat_ms = Some(v.heartbeat_ms);
                        s.vehicle = Some(v);
                        s.phase = Phase::Connected;
                        s.error = None;
                    }
                }
            }
            Err(MessageReadError::Io(e))
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::Interrupted
                ) =>
            {
                thread::sleep(Duration::from_millis(50))
            }
            Err(MessageReadError::Parse(_)) => (),
            Err(e) => {
                let mut s = shared.lock().unwrap();
                s.phase = Phase::ConnectionLost;
                s.vehicle = None;
                s.error = Some(format!("포트 읽기 중단: {e}"));
                break;
            }
        }
        let timed_out = match last {
            Some(t) => t.elapsed() >= loss_timeout,
            None => started.elapsed() >= wait_timeout,
        };
        if timed_out {
            let mut s = shared.lock().unwrap();
            s.phase = if last.is_some() {
                Phase::ConnectionLost
            } else {
                Phase::Error
            };
            s.vehicle = None;
            s.error = Some(
                if last.is_some() {
                    "HEARTBEAT 수신 중단"
                } else {
                    "PX4 HEARTBEAT 수신 시간 초과"
                }
                .into(),
            );
            break;
        }
    }
}
#[tauri::command]
pub fn list_serial_ports(manager: tauri::State<'_, Manager>) -> Result<Vec<Port>, String> {
    manager.list()
}
#[tauri::command]
pub fn connect_serial(
    manager: tauri::State<'_, Manager>,
    port: String,
    baud: u32,
) -> Result<(), String> {
    manager.connect(port, baud)
}
#[tauri::command]
pub fn set_auto_device(
    manager: tauri::State<'_, Manager>,
    device: Option<AutoDevice>,
) -> Result<(), String> {
    manager.set_auto(device)
}
#[tauri::command]
pub fn auto_connect_serial(
    manager: tauri::State<'_, Manager>,
    port: String,
    device: AutoDevice,
) -> Result<(), String> {
    manager.connect_authorized(port, device)
}
#[tauri::command]
pub fn connect_simulation(manager: tauri::State<'_, Manager>) -> Result<(), String> {
    manager.connect_simulation()
}
#[tauri::command]
pub fn disconnect_serial(manager: tauri::State<'_, Manager>) {
    manager.disconnect();
}
#[tauri::command]
pub fn connection_status(manager: tauri::State<'_, Manager>) -> Snapshot {
    manager.snapshot()
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::{ffi::CStr, fs::File, io::Write, os::fd::FromRawFd};
    fn pty() -> (File, String) {
        let (mut master, mut slave) = (0, 0);
        let mut name = [0i8; 128];
        assert_eq!(
            unsafe {
                libc::openpty(
                    &mut master,
                    &mut slave,
                    name.as_mut_ptr(),
                    std::ptr::null(),
                    std::ptr::null(),
                )
            },
            0
        );
        unsafe {
            libc::close(slave);
        }
        (
            unsafe { File::from_raw_fd(master) },
            unsafe { CStr::from_ptr(name.as_ptr()) }
                .to_string_lossy()
                .into_owned(),
        )
    }
    fn frame(autopilot: u8) -> Vec<u8> {
        let mut bytes = vec![0xfe, 9, 1, 42, 1, 0, 0, 0, 0, 0, 2, autopilot, 0, 0, 3];
        let mut crc: u16 = 0xffff;
        for byte in bytes[1..].iter().copied().chain([50]) {
            let mut t = byte ^ crc as u8;
            t ^= t << 4;
            crc = (crc >> 8) ^ ((t as u16) << 8) ^ ((t as u16) << 3) ^ ((t as u16) >> 4);
        }
        bytes.extend_from_slice(&crc.to_le_bytes());
        bytes
    }
    fn state() -> Arc<Mutex<Snapshot>> {
        Arc::new(Mutex::new(Snapshot {
            phase: Phase::Connecting,
            port: None,
            source: "OFFLINE".into(),
            vehicle: None,
            last_heartbeat_ms: None,
            error: None,
        }))
    }
    fn wait_for(s: &Arc<Mutex<Snapshot>>, pred: impl Fn(&Phase) -> bool) {
        let until = Instant::now() + Duration::from_secs(3);
        while Instant::now() < until {
            if pred(&s.lock().unwrap().phase) {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("state transition timed out");
    }
    #[test]
    fn mavlink2_crc_and_unverified_signatures() {
        fn v2(signed: bool) -> Vec<u8> {
            let mut bytes = vec![
                0xfd,
                9,
                u8::from(signed),
                0,
                1,
                42,
                1,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                2,
                12,
                0,
                0,
                3,
            ];
            let mut crc: u16 = 0xffff;
            for byte in bytes[1..].iter().copied().chain([50]) {
                let mut t = byte ^ crc as u8;
                t ^= t << 4;
                crc = (crc >> 8) ^ ((t as u16) << 8) ^ ((t as u16) << 3) ^ ((t as u16) >> 4);
            }
            bytes.extend_from_slice(&crc.to_le_bytes());
            if signed {
                bytes.extend_from_slice(&[0; 13]);
            }
            bytes
        }
        let mut reader = MavlinkReader::new(std::io::Cursor::new(v2(false)));
        let raw = reader.read_any_raw_message::<MavMessage>().unwrap();
        assert_eq!(valid_vehicle(&raw).unwrap().mavlink_version, 2);
        let mut reader = MavlinkReader::new(std::io::Cursor::new(v2(true)));
        let raw = reader.read_any_raw_message::<MavMessage>().unwrap();
        assert!(valid_vehicle(&raw).is_none());
        let mut bad = v2(false);
        bad[11] ^= 1;
        let mut reader = MavlinkReader::new(std::io::Cursor::new(bad));
        assert!(reader.read_any_raw_message::<MavMessage>().is_err());
    }
    #[test]
    fn auto_connect_requires_explicit_allowlist() {
        let manager = Manager::default();
        let device = AutoDevice {
            vid: 1,
            pid: 2,
            serial: "TEST-ONLY".into(),
            baud: 115200,
        };
        assert!(manager
            .connect_authorized("/dev/not-a-device".into(), device.clone())
            .unwrap_err()
            .contains("허용하지"));
        manager.set_auto(Some(device.clone())).unwrap();
        assert!(manager
            .connect_authorized("/dev/not-a-device".into(), device.clone())
            .is_err());
        manager.disconnect();
        assert!(manager
            .connect_authorized("/dev/not-a-device".into(), device)
            .unwrap_err()
            .contains("허용하지"));
    }
    #[test]
    fn udp_simulation_receive_is_loopback_and_exclusive() {
        let manager = Manager::default();
        manager.simulation_at("127.0.0.1:0").unwrap();
        assert!(manager.simulation_at("127.0.0.1:0").is_err());
        assert_eq!(manager.snapshot().source, "SIMULATION");
        let sender = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        sender
            .send_to(&frame(12), manager.snapshot().port.unwrap())
            .unwrap();
        wait_for(&manager.snapshot, |p| matches!(p, Phase::Connected));
        assert_eq!(manager.snapshot().vehicle.unwrap().system_id, 42);
        manager.disconnect();
        assert!(matches!(manager.snapshot().phase, Phase::Disconnected));
        assert_eq!(manager.snapshot().source, "OFFLINE");
    }
    #[test]
    fn serial_discovery_and_invalid_selection() {
        let manager = Manager::default();
        let ports = manager.list().expect("serial discovery");
        eprintln!("USB serial ports found: {}", ports.len());
        assert!(manager
            .connect("/dev/firescene-no-such-port".into(), 115200)
            .is_err());
        manager.disconnect();
        assert!(matches!(manager.snapshot().phase, Phase::Disconnected));
    }
    #[test]
    fn nonexistent_port_fails() {
        assert!(open_readonly("/dev/firescene-no-such-port", 115200).is_err());
    }
    #[test]
    fn invalid_and_valid_heartbeat_and_disconnect() {
        let (mut master, path) = pty();
        let s = state();
        let cancel = Arc::new(AtomicBool::new(false));
        let h = {
            let s = s.clone();
            let c = cancel.clone();
            thread::spawn(move || run(path, 115200, s, c))
        };
        wait_for(&s, |p| matches!(p, Phase::WaitingHeartbeat));
        let mut bad = frame(12);
        bad[8] ^= 1;
        master.write_all(&bad).unwrap();
        master.write_all(&frame(3)).unwrap();
        thread::sleep(Duration::from_millis(150));
        assert!(matches!(s.lock().unwrap().phase, Phase::WaitingHeartbeat));
        master.write_all(&frame(12)).unwrap();
        wait_for(&s, |p| matches!(p, Phase::Connected));
        assert_eq!(s.lock().unwrap().vehicle.as_ref().unwrap().system_id, 42);
        drop(master);
        wait_for(&s, |p| matches!(p, Phase::ConnectionLost));
        h.join().unwrap();
        assert!(s.lock().unwrap().vehicle.is_none());
    }
    #[test]
    fn read_only_descriptor_cannot_write_and_port_is_exclusive() {
        use std::os::fd::AsRawFd;
        let (_master, path) = pty();
        let file = open_readonly(&path, 115200).unwrap();
        assert_eq!(
            unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) } & libc::O_ACCMODE,
            libc::O_RDONLY
        );
        assert_eq!(
            unsafe { libc::write(file.as_raw_fd(), b"x".as_ptr().cast(), 1) },
            -1
        );
        assert!(open_readonly(&path, 115200).is_err());
    }
    #[test]
    fn heartbeat_timeout_and_loss_clear_identity() {
        let (_master, path) = pty();
        let s = state();
        let c = Arc::new(AtomicBool::new(false));
        let shared_first = s.clone();
        let h = thread::spawn(move || {
            run_with_timeouts(
                path,
                115200,
                shared_first,
                c,
                Duration::from_millis(200),
                Duration::from_millis(200),
            )
        });
        h.join().unwrap();
        assert!(matches!(s.lock().unwrap().phase, Phase::Error));
        let (mut master, path) = pty();
        let s = state();
        let c = Arc::new(AtomicBool::new(false));
        let shared = s.clone();
        let h = thread::spawn(move || {
            run_with_timeouts(
                path,
                115200,
                shared,
                c,
                Duration::from_secs(2),
                Duration::from_millis(250),
            )
        });
        wait_for(&s, |p| matches!(p, Phase::WaitingHeartbeat));
        master.write_all(&frame(12)).unwrap();
        wait_for(&s, |p| matches!(p, Phase::Connected));
        h.join().unwrap();
        let status = s.lock().unwrap();
        assert!(matches!(status.phase, Phase::ConnectionLost));
        assert!(status.vehicle.is_none());
    }
    #[test]
    fn user_disconnect_stops_reader() {
        let (_master, path) = pty();
        let s = state();
        let c = Arc::new(AtomicBool::new(false));
        let flag = c.clone();
        let state = s.clone();
        let h = thread::spawn(move || run(path, 115200, state, flag));
        wait_for(&s, |p| matches!(p, Phase::WaitingHeartbeat));
        c.store(true, Ordering::SeqCst);
        h.join().unwrap();
    }
}
