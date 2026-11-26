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
#[derive(Clone, Default, Serialize)]
struct Diagnostics {
    port_opened: bool,
    serial_configured: bool,
    error_stage: Option<String>,
    os_error: Option<i32>,
    bytes: u64,
    messages: u64,
    parse_errors: u64,
    heartbeats: u64,
    heartbeat_rejection: Option<String>,
    first_byte_ms: Option<u64>,
    last_byte_ms: Option<u64>,
    frame_rate_hz: Option<f64>,
    frame_rate_at_ms: Option<u64>,
    heartbeat_interval_ms: Option<u64>,
}
#[derive(Clone, Serialize)]
struct PreviousFailure {
    port: Option<String>,
    source: String,
    error: String,
    diagnostics: Diagnostics,
}
fn previous_failure(s: &Snapshot) -> Option<PreviousFailure> {
    if let Some(error) = &s.error {
        Some(PreviousFailure {
            port: s.port.clone(),
            source: s.source.clone(),
            error: error.clone(),
            diagnostics: s.diagnostics.clone(),
        })
    } else {
        s.previous_failure.clone()
    }
}
#[derive(Clone, Serialize)]
pub struct Snapshot {
    phase: Phase,
    port: Option<String>,
    source: String,
    vehicle: Option<Vehicle>,
    last_heartbeat_ms: Option<u64>,
    error: Option<String>,
    diagnostics: Diagnostics,
    previous_failure: Option<PreviousFailure>,
    #[serde(skip)]
    store: crate::telemetry::Store,
    telemetry: crate::telemetry::View,
    #[serde(skip)]
    monitor: Arc<Mutex<crate::safety::Monitor>>,
    safety: crate::safety::SafetyView,
}
#[derive(Serialize)]
pub struct ConnectionSummary {
    build_commit: &'static str,
    previous_failure: Option<PreviousFailure>,
    diagnostics: Diagnostics,
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
#[cfg(all(feature = "simulation", debug_assertions))]
#[derive(Default)]
struct VirtualInterlock {
    active: Option<u64>,
    replay: Option<u64>,
    next: u64,
}
pub struct Manager {
    snapshot: Arc<Mutex<Snapshot>>,
    worker: Mutex<Option<Worker>>,
    auto_allowed: Mutex<Option<AutoDevice>>,
    #[cfg(all(feature = "simulation", debug_assertions))]
    virtual_interlock: Mutex<VirtualInterlock>,
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
                store: Default::default(),
                telemetry: Default::default(),
                monitor: Arc::new(Mutex::new(Default::default())),
                safety: Default::default(),
                error: None,
                diagnostics: Default::default(),
                previous_failure: None,
            })),
            worker: Mutex::new(None),
            auto_allowed: Mutex::new(None),
            #[cfg(all(feature = "simulation", debug_assertions))]
            virtual_interlock: Mutex::new(VirtualInterlock::default()),
        }
    }
}
impl Manager {
    // Both claims and connection starts take this lock first. The virtual model
    // itself never receives a connection manager or a transport handle.
    #[cfg(all(feature = "simulation", debug_assertions))]
    pub(crate) fn claim_virtual(&self) -> Result<u64, String> {
        let mut active = self
            .virtual_interlock
            .lock()
            .map_err(|_| "권한 잠금 오류")?;
        if active.active.is_some() || active.replay.is_some() {
            return Err("가상 제어 또는 기록 재생이 이미 활성화되어 있습니다".into());
        }
        let worker = self.worker.lock().map_err(|_| "연결 잠금 오류")?;
        let snapshot = self.snapshot.lock().map_err(|_| "연결 상태 오류")?;
        if worker.as_ref().is_some_and(|w| !w.handle.is_finished())
            || snapshot.source != "OFFLINE"
            || !matches!(snapshot.phase, Phase::Disconnected | Phase::PortDetected)
        {
            return Err("연결을 해제한 후 가상 제어를 시작하세요".into());
        }
        active.next = active.next.checked_add(1).ok_or("가상 세션 번호 소진")?;
        let ticket = active.next;
        active.active = Some(ticket);
        Ok(ticket)
    }
    #[cfg(all(feature = "simulation", debug_assertions))]
    pub(crate) fn release_virtual(&self, ticket: u64) -> bool {
        if let Ok(mut active) = self.virtual_interlock.lock() {
            if active.active == Some(ticket) {
                active.active = None;
                return true;
            }
        }
        false
    }
    #[cfg(all(feature = "simulation", debug_assertions))]
    pub(crate) fn claim_replay(&self) -> Result<u64, String> {
        let mut state = self
            .virtual_interlock
            .lock()
            .map_err(|_| "권한 잠금 오류")?;
        if state.active.is_some() || state.replay.is_some() {
            return Err("가상 제어 또는 기록 재생이 이미 활성화되어 있습니다".into());
        }
        state.next = state.next.checked_add(1).ok_or("재생 세션 번호 소진")?;
        let ticket = state.next;
        state.replay = Some(ticket);
        Ok(ticket)
    }
    #[cfg(all(feature = "simulation", debug_assertions))]
    pub(crate) fn release_replay(&self, ticket: u64) -> bool {
        if let Ok(mut state) = self.virtual_interlock.lock() {
            if state.replay == Some(ticket) {
                state.replay = None;
                return true;
            }
        }
        false
    }
    #[cfg(all(feature = "simulation", debug_assertions))]
    pub(crate) fn require_replay(&self) -> Result<(), String> {
        let state = self
            .virtual_interlock
            .lock()
            .map_err(|_| "권한 잠금 오류")?;
        if state.replay.is_none() {
            return Err("기록 재생 세션이 필요합니다".into());
        }
        Ok(())
    }
    pub(crate) fn recording_handle(&self) -> Arc<Mutex<Snapshot>> {
        self.snapshot.clone()
    }
    pub(crate) fn recording_snapshot(
        handle: &Arc<Mutex<Snapshot>>,
    ) -> Result<serde_json::Value, String> {
        let mut s = handle.lock().map_err(|_| "통신 상태 읽기 실패")?;
        assess(&mut s);
        let mut value = serde_json::to_value(&*s).map_err(|_| "통신 상태 변환 실패")?;
        value["telemetry"] =
            serde_json::to_value(s.store.safety_view()).map_err(|_| "텔레메트리 변환 실패")?;
        Ok(value)
    }
    fn stop(&self) {
        if let Some(worker) = self.worker.lock().unwrap().take() {
            worker.cancel.store(true, Ordering::SeqCst);
            let _ = worker.handle.join();
        }
    }
    pub fn disconnect(&self) {
        *self.auto_allowed.lock().unwrap() = None;
        self.stop();
        let previous = previous_failure(&self.snapshot.lock().unwrap());
        let monitor = self.snapshot.lock().unwrap().monitor.clone();
        monitor.lock().unwrap().end_session();
        *self.snapshot.lock().unwrap() = Snapshot {
            phase: Phase::Disconnected,
            port: None,
            source: "OFFLINE".into(),
            vehicle: None,
            last_heartbeat_ms: None,
            store: Default::default(),
            telemetry: Default::default(),
            monitor,
            safety: Default::default(),
            error: None,
            diagnostics: Default::default(),
            previous_failure: previous,
        };
    }
    pub fn list(&self) -> Result<Vec<Port>, String> {
        #[allow(unused_mut)]
        let mut ports = serialport::available_ports()
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
        // An explicitly enabled, local PTY fixture for end-to-end development QA.
        // Production builds have no fixture discovery or alternate authorization path.
        #[cfg(all(feature = "simulation", debug_assertions, target_os = "linux"))]
        if let Ok(path) = std::env::var("FIRESCENE_QA_PTY") {
            if path
                .strip_prefix("/dev/pts/")
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
            {
                ports.push(Port {
                    access: access(&path),
                    name: path,
                    manufacturer: Some("QA fixture".into()),
                    description: Some("Synthetic PTY".into()),
                    vid: 0xfffe,
                    pid: 0xfffe,
                    serial_number: Some("FIRESCENE-QA-PTY".into()),
                });
            }
        }
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
        #[cfg(all(feature = "simulation", debug_assertions))]
        let _virtual_guard = {
            let guard = self
                .virtual_interlock
                .lock()
                .map_err(|_| "권한 잠금 오류")?;
            if guard.active.is_some() {
                return Err("가상 제어 종료 후 연결하세요".into());
            }
            guard
        };
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
        let monitor = self.snapshot.lock().unwrap().monitor.clone();
        let previous = previous_failure(&self.snapshot.lock().unwrap());
        monitor.lock().unwrap().new_session("HARDWARE_LINK");
        *self.snapshot.lock().unwrap() = Snapshot {
            phase: Phase::Connecting,
            port: Some(port.clone()),
            source: "HARDWARE_LINK".into(),
            vehicle: None,
            last_heartbeat_ms: None,
            store: Default::default(),
            telemetry: Default::default(),
            monitor,
            safety: Default::default(),
            error: None,
            diagnostics: Default::default(),
            previous_failure: previous,
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
    #[cfg(feature = "simulation")]
    pub fn connect_simulation(&self) -> Result<(), String> {
        self.simulation_at("127.0.0.1:14540")
    }
    #[cfg(feature = "simulation")]
    fn simulation_at(&self, address: &str) -> Result<(), String> {
        self.udp_at(address, "SIMULATION")
    }
    fn udp_at(&self, address: &str, source: &str) -> Result<(), String> {
        #[cfg(all(feature = "simulation", debug_assertions))]
        let _virtual_guard = {
            let guard = self
                .virtual_interlock
                .lock()
                .map_err(|_| "권한 잠금 오류")?;
            if guard.active.is_some() {
                return Err("가상 제어 종료 후 연결하세요".into());
            }
            guard
        };
        let mut worker = self.worker.lock().unwrap();
        if worker.as_ref().is_some_and(|w| !w.handle.is_finished()) {
            return Err("기존 연결을 먼저 해제하세요.".into());
        }
        if let Some(old) = worker.take() {
            let _ = old.handle.join();
        }
        let socket = std::net::UdpSocket::bind(address).map_err(|e| {
            if e.kind() == io::ErrorKind::AddrInUse {
                format!("UDP 포트 점유 중: {e}")
            } else {
                format!("UDP 수신 포트 열기 실패: {e}")
            }
        })?;
        socket
            .set_read_timeout(Some(Duration::from_millis(50)))
            .map_err(|e| e.to_string())?;
        let local_address = socket.local_addr().map_err(|e| e.to_string())?.to_string();
        let monitor = self.snapshot.lock().unwrap().monitor.clone();
        let previous = previous_failure(&self.snapshot.lock().unwrap());
        monitor.lock().unwrap().new_session(source);
        *self.snapshot.lock().unwrap() = Snapshot {
            phase: Phase::WaitingHeartbeat,
            port: Some(local_address),
            source: source.into(),
            vehicle: None,
            last_heartbeat_ms: None,
            store: Default::default(),
            telemetry: Default::default(),
            monitor,
            safety: Default::default(),
            error: None,
            diagnostics: Default::default(),
            previous_failure: previous,
        };
        let shared = self.snapshot.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let handle = thread::spawn(move || {
            receive(
                UdpReader::new(socket),
                shared,
                flag,
                Duration::from_secs(10),
                Duration::from_secs(5),
            )
        });
        *worker = Some(Worker { cancel, handle });
        Ok(())
    }
    fn connect_forwarded(&self, port: u16, route: &str) -> Result<(), String> {
        if port == 0 {
            return Err("1–65535 범위의 UDP 포트를 입력하세요".into());
        }
        let source = match route {
            "qgc" => "QGC_FORWARD",
            "router" => "ROUTER_FORWARD",
            _ => return Err("지원하지 않는 수신 경로입니다".into()),
        };
        self.udp_at(&format!("127.0.0.1:{port}"), source)
    }
    fn summary(&self) -> ConnectionSummary {
        let mut snapshot = self.snapshot.lock().unwrap();
        assess(&mut snapshot);
        ConnectionSummary {
            build_commit: env!("FIRESCENE_BUILD_COMMIT"),
            previous_failure: snapshot.previous_failure.clone(),
            diagnostics: snapshot.diagnostics.clone(),
            phase: snapshot.phase.clone(),
            port: snapshot.port.clone(),
            source: snapshot.source.clone(),
            vehicle: snapshot.vehicle.clone(),
            last_heartbeat_ms: snapshot.last_heartbeat_ms,
            error: snapshot.error.clone(),
        }
    }
    pub(crate) fn snapshot(&self) -> Snapshot {
        let mut snapshot = self.snapshot.lock().unwrap();
        assess(&mut snapshot);
        let mut result = snapshot.clone();
        result.telemetry = result.store.view();
        result
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
fn open_readonly(path: &str, baud: u32, opened: impl FnOnce()) -> io::Result<std::fs::File> {
    use std::{
        fs::OpenOptions,
        os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    };
    let f = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOCTTY | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)?;
    opened();
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
#[cfg(any(windows, test))]
fn windows_com_path(port: &str) -> io::Result<String> {
    let upper = port.to_ascii_uppercase();
    let number = upper
        .strip_prefix("COM")
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "COM 포트 형식 오류"))?;
    if number.is_empty()
        || number.len() > 5
        || !number.bytes().all(|b| b.is_ascii_digit())
        || number.parse::<u16>().ok().filter(|n| *n > 0).is_none()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "COM 포트 형식 오류",
        ));
    }
    Ok(format!(r"\\.\{upper}"))
}
#[cfg(windows)]
struct WindowsSerial(std::fs::File);
#[cfg(windows)]
impl Read for WindowsSerial {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.0.read(buf) {
            Ok(0) if !buf.is_empty() => Err(io::ErrorKind::WouldBlock.into()),
            other => other,
        }
    }
}
#[cfg(windows)]
fn open_readonly(port: &str, baud: u32, opened: impl FnOnce()) -> io::Result<WindowsSerial> {
    use std::{
        ffi::OsStr,
        os::windows::{
            ffi::OsStrExt,
            io::{AsRawHandle, FromRawHandle},
        },
        ptr,
    };
    use windows_sys::Win32::{
        Devices::Communication::{
            BuildCommDCBW, GetCommState, SetCommState, SetCommTimeouts, COMMTIMEOUTS, DCB,
        },
        Foundation::{GENERIC_READ, INVALID_HANDLE_VALUE},
        Storage::FileSystem::{CreateFileW, OPEN_EXISTING},
    };
    if ![57600, 115200, 230400, 460800, 921600].contains(&baud) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "baud"));
    }
    let path = windows_com_path(port)?;
    let wide: Vec<u16> = OsStr::new(&path).encode_wide().chain(Some(0)).collect();
    // Exclusive, synchronous, GENERIC_READ only. Never fall back to a writable handle.
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ,
            0,
            ptr::null(),
            OPEN_EXISTING,
            0,
            ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let file = unsafe { std::fs::File::from_raw_handle(handle.cast()) };
    opened();
    let raw = file.as_raw_handle().cast();
    let mut dcb = DCB::default();
    dcb.DCBlength = std::mem::size_of::<DCB>() as u32;
    if unsafe { GetCommState(raw, &mut dcb) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let config: Vec<u16> = format!("baud={baud} parity=n data=8 stop=1")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    if unsafe { BuildCommDCBW(config.as_ptr(), &mut dcb) } == 0
        || unsafe { SetCommState(raw, &dcb) } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let timeout = COMMTIMEOUTS {
        ReadIntervalTimeout: u32::MAX,
        ReadTotalTimeoutMultiplier: 0,
        ReadTotalTimeoutConstant: 0,
        WriteTotalTimeoutMultiplier: 0,
        WriteTotalTimeoutConstant: 0,
    };
    if unsafe { SetCommTimeouts(raw, &timeout) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(WindowsSerial(file))
}
#[cfg(not(any(target_os = "linux", windows)))]
fn open_readonly(_: &str, _: u32, _: impl FnOnce()) -> io::Result<std::fs::File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "이 플랫폼의 읽기 전용 직렬 연결은 아직 지원되지 않습니다",
    ))
}

fn heartbeat_rejection(raw: &MAVLinkMessageRaw) -> Option<&'static str> {
    if raw.message_id() != 0 {
        return None;
    }
    if raw.system_id() == 0 || raw.component_id() == 0 {
        return Some("시스템 또는 컴포넌트 ID가 0");
    }
    if let MAVLinkMessageRaw::V2(v) = raw {
        if v.incompatibility_flags() != 0 {
            return Some("지원하지 않는 MAVLink 2 호환성 플래그");
        }
    }
    let p = raw.payload();
    if p.len() != 9 {
        return Some("HEARTBEAT 길이 오류");
    }
    if p[5] != 12 {
        return Some("PX4 autopilot이 아님");
    }
    if !matches!(p[4], 1..=4 | 10..=15 | 19..=24 | 29 | 35 | 43 | 47) {
        return Some("지원하지 않는 기체 유형");
    }
    if p[8] != 3 {
        return Some("지원하지 않는 HEARTBEAT 버전");
    }
    None
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
fn assess(s: &mut Snapshot) {
    let phase = match s.phase {
        Phase::Disconnected => "DISCONNECTED",
        Phase::PortDetected => "PORT_DETECTED",
        Phase::Connecting => "CONNECTING",
        Phase::WaitingHeartbeat => "WAITING_HEARTBEAT",
        Phase::Connected => "CONNECTED",
        Phase::ConnectionLost => "CONNECTION_LOST",
        Phase::Error => "ERROR",
    };
    let identity = s.vehicle.as_ref().map(|v| (v.system_id, v.component_id));
    s.safety =
        s.monitor
            .lock()
            .unwrap()
            .evaluate(phase, &s.source, &s.store.safety_view(), identity);
}
// Bound work per decoder call so continuous junk cannot starve timeout or disconnect.
struct BoundedReader<R> {
    inner: R,
    remaining: usize,
    shared: Arc<Mutex<Snapshot>>,
}
impl<R: Read> Read for BoundedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let limit = buf.len().min(self.remaining);
        let n = self.inner.read(&mut buf[..limit])?;
        self.remaining -= n;
        if n > 0 {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let mut s = self.shared.lock().unwrap();
            s.diagnostics.bytes += n as u64;
            s.diagnostics.first_byte_ms.get_or_insert(now);
            s.diagnostics.last_byte_ms = Some(now);
        }
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
    let file = match open_readonly(&port, baud, || {
        shared.lock().unwrap().diagnostics.port_opened = true;
    }) {
        Ok(f) => f,
        Err(e) => {
            let mut s = shared.lock().unwrap();
            s.phase = Phase::Error;
            s.diagnostics.os_error = e.raw_os_error();
            let configured_stage = s.diagnostics.port_opened;
            s.diagnostics.error_stage = Some(
                if configured_stage {
                    "SERIAL_CONFIGURATION"
                } else {
                    "PORT_OPEN"
                }
                .into(),
            );
            s.error = Some(if configured_stage {
                format!("직렬 통신 설정 실패: {e}")
            } else {
                match e.kind() {
                io::ErrorKind::NotFound => format!("장치 미연결 또는 직렬 포트 없음: {e}"),
                io::ErrorKind::PermissionDenied if cfg!(windows) => format!("직렬 포트 접근 거부: 다른 프로그램의 포트 사용 또는 장치 권한을 확인하세요. {e}"),
                io::ErrorKind::PermissionDenied => format!("장치 접근 권한 부족: {e}"),
                _ if cfg!(windows) && e.raw_os_error() == Some(32) => format!("다른 프로그램에서 직렬 포트 사용 중: {e}"),
                _ if cfg!(unix) && e.raw_os_error() == Some(libc::EBUSY) => {
                    format!("다른 프로그램에서 직렬 포트 사용 중: {e}")
                }
                _ => format!("직렬 포트 열기 실패: {e}"),
            }
            });
            return;
        }
    };
    {
        let mut s = shared.lock().unwrap();
        s.phase = Phase::WaitingHeartbeat;
        s.diagnostics.serial_configured = true;
    }
    receive(&file, shared, cancel, wait_timeout, loss_timeout);
    // A PTY's exclusive flag survives closing the slave while its test master
    // remains open. Reset only the explicitly selected debug fixture.
    #[cfg(all(feature = "simulation", debug_assertions, target_os = "linux"))]
    if std::env::var("FIRESCENE_QA_PTY").ok().as_deref() == Some(port.as_str()) {
        use std::os::fd::AsRawFd;
        unsafe { libc::ioctl(file.as_raw_fd(), libc::TIOCNXCL) };
    }
}
// Local UDP listener is intentionally receive-only and cannot select remote destinations.
struct UdpReader {
    socket: std::net::UdpSocket,
    packet: Vec<u8>,
    pos: usize,
    len: usize,
}
impl UdpReader {
    fn new(socket: std::net::UdpSocket) -> Self {
        Self {
            socket,
            packet: vec![0; 65535],
            pos: 0,
            len: 0,
        }
    }
}
impl Read for UdpReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        while self.pos == self.len {
            self.len = self.socket.recv(&mut self.packet)?;
            self.pos = 0;
        }
        let n = buf.len().min(self.len - self.pos);
        buf[..n].copy_from_slice(&self.packet[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
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
    let mut last_assessment = Instant::now();
    let mut rate_started = Instant::now();
    let mut rate_frames: u64 = 0;
    let mut identity = None;
    let mut reader = MavlinkReader::new(BoundedReader {
        inner: input,
        remaining: 4096,
        shared: shared.clone(),
    });
    loop {
        if cancel.load(Ordering::SeqCst) {
            break;
        }
        reader.get_mut().remaining = 4096;
        match reader.read_any_raw_message::<MavMessage>() {
            Ok(raw) => {
                rate_frames = rate_frames.saturating_add(1);
                {
                    let mut s = shared.lock().unwrap();
                    s.diagnostics.messages += 1;
                    if raw.message_id() == 0 {
                        s.diagnostics.heartbeats += 1;
                        if let Some(reason) = heartbeat_rejection(&raw) {
                            s.diagnostics.heartbeat_rejection = Some(reason.to_owned());
                        }
                    }
                }
                if let Some(v) = valid_vehicle(&raw) {
                    let id = (v.system_id, v.component_id);
                    if identity.is_none() || identity == Some(id) {
                        identity = Some(id);
                        let now = Instant::now();
                        let interval = last.map(|previous: Instant| {
                            now.duration_since(previous)
                                .as_millis()
                                .min(u64::MAX as u128) as u64
                        });
                        last = Some(now);
                        let mut s = shared.lock().unwrap();
                        s.diagnostics.heartbeat_interval_ms = interval;
                        s.last_heartbeat_ms = Some(v.heartbeat_ms);
                        s.vehicle = Some(v);
                        s.store.consume(&raw);
                        s.phase = Phase::Connected;
                        s.error = None;
                        if last_assessment.elapsed() >= Duration::from_millis(250)
                            || s.safety.session == 0
                        {
                            assess(&mut s);
                            last_assessment = Instant::now();
                        }
                    }
                } else if let Some((sys, comp)) = identity {
                    if raw.message_id() != 0 && raw.system_id() == sys && raw.component_id() == comp
                    {
                        shared.lock().unwrap().store.consume(&raw);
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
            Err(MessageReadError::Parse(_)) => shared.lock().unwrap().diagnostics.parse_errors += 1,
            Err(e) => {
                let mut s = shared.lock().unwrap();
                if let MessageReadError::Io(ref io_error) = e {
                    s.diagnostics.os_error = io_error.raw_os_error();
                }
                s.diagnostics.error_stage = Some("READ".into());
                s.phase = Phase::ConnectionLost;
                s.vehicle = None;
                s.store = Default::default();
                s.error = Some(format!("포트 읽기 중단: {e}"));
                assess(&mut s);
                break;
            }
        }
        if rate_started.elapsed() >= Duration::from_secs(1) {
            let elapsed = rate_started.elapsed().as_secs_f64();
            let mut s = shared.lock().unwrap();
            s.diagnostics.frame_rate_hz = Some(rate_frames as f64 / elapsed);
            s.diagnostics.frame_rate_at_ms = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            );
            rate_frames = 0;
            rate_started = Instant::now();
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
            s.store = Default::default();
            s.diagnostics.error_stage = Some(
                if last.is_some() {
                    "HEARTBEAT_LOST"
                } else if s.diagnostics.bytes == 0 {
                    "RECEIVE_TIMEOUT"
                } else {
                    "HEARTBEAT_TIMEOUT"
                }
                .into(),
            );
            s.error = Some(
                if last.is_some() {
                    "HEARTBEAT 수신 중단"
                } else {
                    "PX4 HEARTBEAT 수신 시간 초과"
                }
                .into(),
            );
            assess(&mut s);
            break;
        }
        if last_assessment.elapsed() >= Duration::from_millis(250) {
            let mut s = shared.lock().unwrap();
            assess(&mut s);
            last_assessment = Instant::now();
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
#[cfg(feature = "simulation")]
#[tauri::command]
pub fn connect_simulation(manager: tauri::State<'_, Manager>) -> Result<(), String> {
    manager.connect_simulation()
}
#[tauri::command]
pub fn connect_forwarded(
    manager: tauri::State<'_, Manager>,
    port: u16,
    route: String,
) -> Result<(), String> {
    manager.connect_forwarded(port, &route)
}
#[tauri::command]
pub fn disconnect_serial(manager: tauri::State<'_, Manager>) {
    manager.disconnect();
}
#[tauri::command]
pub fn connection_summary(manager: tauri::State<'_, Manager>) -> ConnectionSummary {
    manager.summary()
}
#[tauri::command]
pub fn connection_status(manager: tauri::State<'_, Manager>) -> Snapshot {
    manager.snapshot()
}

#[tauri::command]
pub fn simulation_available() -> bool {
    cfg!(feature = "simulation")
}
#[tauri::command]
pub fn replay_guard_required() -> bool {
    cfg!(all(feature = "simulation", debug_assertions))
}

#[tauri::command]
pub fn acknowledge_alert(manager: tauri::State<'_, Manager>, id: u64) -> bool {
    let monitor = manager.snapshot.lock().unwrap().monitor.clone();
    let result = monitor.lock().unwrap().acknowledge(id);
    result
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::{ffi::CStr, fs::File, io::Write, os::fd::FromRawFd};
    #[cfg(all(feature = "simulation", debug_assertions))]
    #[test]
    fn virtual_claim_blocks_all_connection_starts_and_existing_link_blocks_claim() {
        let manager = Manager::default();
        let first = manager.claim_virtual().unwrap();
        assert!(manager.claim_virtual().is_err());
        assert!(manager
            .connect("/dev/does-not-exist".into(), 115200)
            .unwrap_err()
            .contains("가상 제어"));
        assert!(manager
            .connect_forwarded(14550, "qgc")
            .unwrap_err()
            .contains("가상 제어"));
        assert!(manager
            .connect_simulation()
            .unwrap_err()
            .contains("가상 제어"));
        assert!(manager.release_virtual(first));
        let second = manager.claim_virtual().unwrap();
        assert_ne!(first, second);
        assert!(!manager.release_virtual(first));
        assert!(manager.connect_simulation().is_err());
        assert!(manager.release_virtual(second));
        {
            let mut s = manager.snapshot.lock().unwrap();
            s.source = "HARDWARE_LINK".into();
            s.phase = Phase::Connecting;
        }
        assert!(manager.claim_virtual().is_err());
    }
    #[cfg(all(feature = "simulation", debug_assertions))]
    #[test]
    fn virtual_claim_and_connection_start_cannot_both_win() {
        use std::sync::Barrier;
        for _ in 0..10 {
            let manager = Arc::new(Manager::default());
            let barrier = Arc::new(Barrier::new(3));
            let a = manager.clone();
            let ba = barrier.clone();
            let virtual_thread = thread::spawn(move || {
                ba.wait();
                a.claim_virtual().ok()
            });
            let b = manager.clone();
            let bb = barrier.clone();
            let connection_thread = thread::spawn(move || {
                bb.wait();
                b.simulation_at("127.0.0.1:0").is_ok()
            });
            barrier.wait();
            let virtual_won = virtual_thread.join().unwrap();
            let connection_won = connection_thread.join().unwrap();
            assert_ne!(virtual_won.is_some(), connection_won);
            if let Some(ticket) = virtual_won {
                assert!(manager.release_virtual(ticket));
            }
            manager.disconnect();
        }
    }
    #[cfg(all(feature = "simulation", debug_assertions))]
    #[test]
    fn replay_and_virtual_claims_are_exclusive_and_tokens_are_scoped() {
        use std::sync::Barrier;
        let manager = Arc::new(Manager::default());
        let first = manager.claim_replay().unwrap();
        assert!(manager.claim_virtual().is_err());
        assert!(manager.release_replay(first));
        let second = manager.claim_replay().unwrap();
        assert!(!manager.release_replay(first));
        assert!(manager.claim_virtual().is_err());
        assert!(manager.release_replay(second));
        let virtual_token = manager.claim_virtual().unwrap();
        assert!(manager.claim_replay().is_err());
        assert!(!manager.release_replay(virtual_token));
        assert!(manager.release_virtual(virtual_token));
        for _ in 0..20 {
            let barrier = Arc::new(Barrier::new(3));
            let a = manager.clone();
            let ba = barrier.clone();
            let v = thread::spawn(move || {
                ba.wait();
                a.claim_virtual().ok()
            });
            let b = manager.clone();
            let bb = barrier.clone();
            let r = thread::spawn(move || {
                bb.wait();
                b.claim_replay().ok()
            });
            barrier.wait();
            let vt = v.join().unwrap();
            let rt = r.join().unwrap();
            assert_ne!(vt.is_some(), rt.is_some());
            if let Some(t) = vt {
                assert!(manager.release_virtual(t));
            }
            if let Some(t) = rt {
                assert!(manager.release_replay(t));
            }
        }
    }
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
            store: Default::default(),
            telemetry: Default::default(),
            monitor: Arc::new(Mutex::new(Default::default())),
            safety: Default::default(),
            error: None,
            diagnostics: Default::default(),
            previous_failure: None,
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
    fn failed_session_is_preserved_without_mixing_new_counters() {
        let manager = Manager::default();
        {
            let mut s = manager.snapshot.lock().unwrap();
            s.port = Some("COM4".into());
            s.source = "HARDWARE_LINK".into();
            s.error = Some("timeout".into());
            s.diagnostics.bytes = 123;
        }
        manager.disconnect();
        let snapshot = manager.snapshot();
        assert_eq!(snapshot.diagnostics.bytes, 0);
        let previous = snapshot.previous_failure.unwrap();
        assert_eq!(previous.diagnostics.bytes, 123);
        assert_eq!(previous.error, "timeout");
        manager.disconnect();
        assert_eq!(
            manager
                .snapshot()
                .previous_failure
                .unwrap()
                .diagnostics
                .bytes,
            123
        );
    }
    #[test]
    fn heartbeat_diagnostic_reports_rejection_without_relaxing_filter() {
        let mut reader = MavlinkReader::new(std::io::Cursor::new(frame(3)));
        let raw = reader.read_any_raw_message::<MavMessage>().unwrap();
        assert_eq!(heartbeat_rejection(&raw), Some("PX4 autopilot이 아님"));
        assert!(valid_vehicle(&raw).is_none());
        let mut reader = MavlinkReader::new(std::io::Cursor::new(frame(12)));
        let raw = reader.read_any_raw_message::<MavMessage>().unwrap();
        assert!(heartbeat_rejection(&raw).is_none());
        assert!(valid_vehicle(&raw).is_some());
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
    fn lightweight_connection_summary_excludes_telemetry_and_safety() {
        let manager = Manager::default();
        let value = serde_json::to_value(manager.summary()).unwrap();
        assert_eq!(value["phase"], "DISCONNECTED");
        assert!(value.get("telemetry").is_none());
        assert!(value.get("safety").is_none());
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
    fn forwarded_udp_reads_multiframe_datagram_and_is_receive_only() {
        let manager = Manager::default();
        assert!(manager.connect_forwarded(0, "qgc").is_err());
        assert!(manager.connect_forwarded(14560, "invalid").is_err());
        manager.udp_at("127.0.0.1:0", "QGC_FORWARD").unwrap();
        let target = manager.snapshot().port.unwrap();
        assert!(std::net::UdpSocket::bind(&target).is_err());
        assert!(manager.udp_at("127.0.0.1:0", "ROUTER_FORWARD").is_err());
        let sender = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        sender
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let mut packet = frame(12);
        packet.extend(frame(12));
        sender.send_to(&packet, &target).unwrap();
        wait_for(&manager.snapshot, |p| matches!(p, Phase::Connected));
        assert_eq!(manager.snapshot().source, "QGC_FORWARD");
        assert_eq!(manager.snapshot().vehicle.unwrap().system_id, 42);
        let mut reply = [0u8; 16];
        assert!(sender.recv_from(&mut reply).is_err());
        manager.disconnect();
        assert!(matches!(manager.snapshot().phase, Phase::Disconnected));
        manager.udp_at("127.0.0.1:0", "ROUTER_FORWARD").unwrap();
        assert_eq!(manager.snapshot().source, "ROUTER_FORWARD");
        manager.disconnect();
    }
    #[test]
    fn forwarded_udp_heartbeat_loss_clears_vehicle() {
        let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(30)))
            .unwrap();
        let addr = socket.local_addr().unwrap();
        let s = state();
        let shared = s.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let worker = thread::spawn(move || {
            receive(
                UdpReader::new(socket),
                shared,
                flag,
                Duration::from_secs(2),
                Duration::from_millis(200),
            )
        });
        let sender = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        sender.send_to(&frame(12), addr).unwrap();
        wait_for(&s, |p| matches!(p, Phase::Connected));
        wait_for(&s, |p| matches!(p, Phase::ConnectionLost));
        assert!(s.lock().unwrap().vehicle.is_none());
        worker.join().unwrap();
    }
    #[test]
    fn udp_new_session_discards_old_telemetry_and_rejects_other_system() {
        use mavlink::dialects::common::{MavMessage, ATTITUDE_DATA};
        fn attitude(system_id: u8, roll: f32) -> Vec<u8> {
            let mut bytes = Vec::new();
            mavlink::write_v1_msg(
                &mut bytes,
                mavlink::MavHeader {
                    system_id: system_id.into(),
                    component_id: 1,
                    sequence: 1,
                },
                &MavMessage::ATTITUDE(ATTITUDE_DATA {
                    roll,
                    ..ATTITUDE_DATA::DEFAULT
                }),
            )
            .unwrap();
            bytes
        }
        let manager = Manager::default();
        let sender = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        manager.udp_at("127.0.0.1:0", "QGC_FORWARD").unwrap();
        let old_target = manager.snapshot().port.unwrap();
        sender.send_to(&frame(12), &old_target).unwrap();
        wait_for(&manager.snapshot, |p| matches!(p, Phase::Connected));
        sender.send_to(&attitude(42, 0.5), &old_target).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !manager.snapshot().telemetry.fields.contains_key("roll") && Instant::now() < deadline
        {
            thread::sleep(Duration::from_millis(20));
        }
        assert!(manager.snapshot().telemetry.fields.contains_key("roll"));
        manager.disconnect();
        assert!(manager.snapshot().telemetry.fields.is_empty());
        // Reserve the old endpoint so the new session necessarily uses another port.
        let old_sink = std::net::UdpSocket::bind(&old_target).unwrap();
        manager.udp_at("127.0.0.1:0", "QGC_FORWARD").unwrap();
        let new_target = manager.snapshot().port.unwrap();
        sender.send_to(&frame(12), &new_target).unwrap();
        wait_for(&manager.snapshot, |p| matches!(p, Phase::Connected));
        sender.send_to(&attitude(42, 0.5), &old_target).unwrap();
        sender.send_to(&attitude(43, 0.8), &new_target).unwrap();
        thread::sleep(Duration::from_millis(200));
        assert!(!manager.snapshot().telemetry.fields.contains_key("roll"));
        sender.send_to(&attitude(42, 0.0), &new_target).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !manager.snapshot().telemetry.fields.contains_key("roll") && Instant::now() < deadline
        {
            thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(manager.snapshot().telemetry.fields["roll"].value, Some(0.0));
        manager.disconnect();
        drop(old_sink);
    }
    #[test]
    fn synthetic_udp_frame_rates_preserve_counts() {
        use mavlink::dialects::common::{MavMessage, ATTITUDE_DATA};
        let mut attitude = Vec::new();
        mavlink::write_v1_msg(
            &mut attitude,
            mavlink::MavHeader {
                system_id: 42,
                component_id: 1,
                sequence: 1,
            },
            &MavMessage::ATTITUDE(ATTITUDE_DATA {
                roll: 0.3,
                ..ATTITUDE_DATA::DEFAULT
            }),
        )
        .unwrap();
        for rate in [10u32, 30, 60, 100] {
            let manager = Manager::default();
            manager.udp_at("127.0.0.1:0", "QGC_FORWARD").unwrap();
            let target = manager.snapshot().port.unwrap();
            let sender = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
            let start = Instant::now();
            let total = rate * 3;
            let heartbeat = frame(12);
            for index in 0..total {
                let deadline = start + Duration::from_secs_f64(index as f64 / rate as f64);
                if let Some(delay) = deadline.checked_duration_since(Instant::now()) {
                    thread::sleep(delay);
                }
                sender
                    .send_to(
                        if index % rate == 0 {
                            &heartbeat
                        } else {
                            &attitude
                        },
                        &target,
                    )
                    .unwrap();
            }
            let deadline = Instant::now() + Duration::from_secs(2);
            while manager.snapshot().diagnostics.messages < total as u64
                && Instant::now() < deadline
            {
                thread::sleep(Duration::from_millis(10));
            }
            let snapshot = manager.snapshot();
            eprintln!("synthetic rate={rate} sent={total} received={} elapsed_s={:.3} connected={} attitude_live={}", snapshot.diagnostics.messages, start.elapsed().as_secs_f64(), matches!(snapshot.phase, Phase::Connected), snapshot.telemetry.fields.get("roll").is_some_and(|x| x.status == "LIVE"));
            assert_eq!(snapshot.diagnostics.messages, total as u64);
            assert!(matches!(snapshot.phase, Phase::Connected));
            assert_eq!(snapshot.telemetry.fields["roll"].status, "LIVE");
            manager.disconnect();
        }
    }
    #[test]
    fn udp_reader_preserves_a_datagram_across_small_reads() {
        let receiver = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let sender = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        sender
            .send_to(&[1, 2, 3, 4, 5], receiver.local_addr().unwrap())
            .unwrap();
        let mut reader = UdpReader::new(receiver);
        let mut b = [0; 2];
        assert_eq!(reader.read(&mut b).unwrap(), 2);
        assert_eq!(b, [1, 2]);
        assert_eq!(reader.read(&mut b).unwrap(), 2);
        assert_eq!(b, [3, 4]);
        assert_eq!(reader.read(&mut b).unwrap(), 1);
        assert_eq!(b[0], 5);
    }
    #[test]
    #[cfg(feature = "simulation")]
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
        let target = manager.snapshot().port.unwrap();
        let mut packet = Vec::new();
        let attitude = mavlink::dialects::common::MavMessage::ATTITUDE(
            mavlink::dialects::common::ATTITUDE_DATA {
                roll: 0.5,
                pitch: 0.25,
                ..mavlink::dialects::common::ATTITUDE_DATA::DEFAULT
            },
        );
        mavlink::write_v1_msg(
            &mut packet,
            mavlink::MavHeader {
                system_id: 43,
                component_id: 1,
                sequence: 0,
            },
            &attitude,
        )
        .unwrap();
        sender.send_to(&packet, &target).unwrap();
        thread::sleep(Duration::from_millis(100));
        assert!(!manager.snapshot().telemetry.fields.contains_key("roll"));
        packet.clear();
        mavlink::write_v1_msg(
            &mut packet,
            mavlink::MavHeader {
                system_id: 42,
                component_id: 1,
                sequence: 1,
            },
            &attitude,
        )
        .unwrap();
        sender.send_to(&packet, &target).unwrap();
        let until = Instant::now() + Duration::from_secs(2);
        while !manager.snapshot().telemetry.fields.contains_key("roll") && Instant::now() < until {
            thread::sleep(Duration::from_millis(20));
        }
        let snapshot = manager.snapshot();
        assert!((snapshot.telemetry.fields["roll"].value.unwrap() - 28.6479).abs() < 0.01);
        let recorded = Manager::recording_snapshot(&manager.recording_handle()).unwrap();
        println!(
            "synthetic UDP recording snapshot: {} bytes",
            serde_json::to_vec(&recorded).unwrap().len()
        );
        assert_eq!(recorded["source"], "SIMULATION");
        assert_eq!(recorded["vehicle"]["system_id"], 42);
        assert!(
            (recorded["telemetry"]["fields"]["roll"]["value"]
                .as_f64()
                .unwrap()
                - 28.6479)
                .abs()
                < 0.01
        );
        packet.clear();
        let low = mavlink::dialects::common::MavMessage::SYS_STATUS(
            mavlink::dialects::common::SYS_STATUS_DATA {
                battery_remaining: 10,
                voltage_battery: 12000,
                current_battery: -1,
                ..mavlink::dialects::common::SYS_STATUS_DATA::DEFAULT
            },
        );
        mavlink::write_v1_msg(
            &mut packet,
            mavlink::MavHeader {
                system_id: 42,
                component_id: 1,
                sequence: 2,
            },
            &low,
        )
        .unwrap();
        sender.send_to(&packet, &target).unwrap();
        let until = Instant::now() + Duration::from_secs(4);
        while !manager
            .snapshot()
            .safety
            .events
            .iter()
            .any(|e| e.code == "battery_critical" && e.resolved_ms.is_none())
            && Instant::now() < until
        {
            thread::sleep(Duration::from_millis(50));
        }
        let alert = manager
            .snapshot()
            .safety
            .events
            .into_iter()
            .find(|e| e.code == "battery_critical" && e.resolved_ms.is_none())
            .expect("critical event");
        assert_eq!(alert.source, "SIMULATION");
        assert_eq!(alert.system_id, Some(42));
        manager.disconnect();
        assert!(matches!(manager.snapshot().phase, Phase::Disconnected));
        assert_eq!(manager.snapshot().source, "OFFLINE");
        assert!(manager.snapshot().telemetry.fields.is_empty());
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
        assert!(open_readonly("/dev/firescene-no-such-port", 115200, || {}).is_err());
    }
    #[test]
    fn serial_open_and_configuration_errors_preserve_stage_and_os_code() {
        for (path, stage, opened) in [
            ("/dev/firescene-no-such-port", "PORT_OPEN", false),
            ("/dev/null", "SERIAL_CONFIGURATION", true),
        ] {
            let shared = state();
            run_with_timeouts(
                path.into(),
                115200,
                shared.clone(),
                Arc::new(AtomicBool::new(false)),
                Duration::from_millis(100),
                Duration::from_millis(100),
            );
            let s = shared.lock().unwrap();
            assert!(matches!(s.phase, Phase::Error));
            assert_eq!(s.diagnostics.port_opened, opened);
            assert!(!s.diagnostics.serial_configured);
            assert_eq!(s.diagnostics.error_stage.as_deref(), Some(stage));
            assert!(s.diagnostics.os_error.is_some());
        }
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
        {
            let snapshot = s.lock().unwrap();
            assert_eq!(snapshot.vehicle.as_ref().unwrap().system_id, 42);
            assert!(snapshot.diagnostics.port_opened);
            assert!(snapshot.diagnostics.bytes >= 34);
            assert_eq!(snapshot.diagnostics.messages, 2);
            assert_eq!(snapshot.diagnostics.heartbeats, 2);
            assert_eq!(
                snapshot.diagnostics.heartbeat_rejection.as_deref(),
                Some("PX4 autopilot이 아님")
            );
            assert!(snapshot.diagnostics.first_byte_ms.is_some());
        }
        drop(master);
        wait_for(&s, |p| matches!(p, Phase::ConnectionLost));
        h.join().unwrap();
        assert!(s.lock().unwrap().vehicle.is_none());
    }
    #[test]
    fn receive_rate_and_valid_heartbeat_interval_are_session_scoped() {
        let (mut master, path) = pty();
        let shared = state();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker = {
            let shared = shared.clone();
            let cancel = cancel.clone();
            thread::spawn(move || run(path, 115200, shared, cancel))
        };
        wait_for(&shared, |p| matches!(p, Phase::WaitingHeartbeat));
        master.write_all(&frame(12)).unwrap();
        wait_for(&shared, |p| matches!(p, Phase::Connected));
        thread::sleep(Duration::from_millis(120));
        master.write_all(&frame(3)).unwrap();
        master.write_all(&frame(12)).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let s = shared.lock().unwrap();
            if let (Some(rate), Some(interval)) = (
                s.diagnostics.frame_rate_hz,
                s.diagnostics.heartbeat_interval_ms,
            ) {
                assert!(rate > 0.0);
                assert!(interval >= 100 && interval < 2000);
                assert_eq!(s.diagnostics.heartbeats, 3);
                break;
            }
            drop(s);
            assert!(Instant::now() < deadline, "diagnostic sample timed out");
            thread::sleep(Duration::from_millis(20));
        }
        cancel.store(true, Ordering::SeqCst);
        worker.join().unwrap();
        let fresh = state();
        assert!(fresh.lock().unwrap().diagnostics.frame_rate_hz.is_none());
        assert!(fresh
            .lock()
            .unwrap()
            .diagnostics
            .heartbeat_interval_ms
            .is_none());
    }
    #[test]
    fn read_only_descriptor_cannot_write_and_port_is_exclusive() {
        use std::os::fd::AsRawFd;
        let (_master, path) = pty();
        let file = open_readonly(&path, 115200, || {}).unwrap();
        assert_eq!(
            unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) } & libc::O_ACCMODE,
            libc::O_RDONLY
        );
        assert_eq!(
            unsafe { libc::write(file.as_raw_fd(), b"x".as_ptr().cast(), 1) },
            -1
        );
        assert!(open_readonly(&path, 115200, || {}).is_err());
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

#[cfg(test)]
mod port_validation_tests {
    use super::windows_com_path;
    #[test]
    fn windows_port_names_are_strict_and_cannot_select_arbitrary_files() {
        assert_eq!(windows_com_path("COM3").unwrap(), r"\\.\COM3");
        assert_eq!(windows_com_path("com256").unwrap(), r"\\.\COM256");
        for value in [
            "COM0",
            "COM65536",
            "COM1/../x",
            "\\\\.\\COM1",
            "/dev/ttyUSB0",
            "COM1\0",
        ] {
            assert!(windows_com_path(value).is_err());
        }
    }
}
