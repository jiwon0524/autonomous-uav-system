//! Observational development policy, independent of PX4 failsafe.
use crate::telemetry::{Field, View};
use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
const CAP: usize = 200;
#[derive(Clone, Copy)]
struct Policy {
    low: f64,
    critical: f64,
    low_clear: f64,
    critical_clear: f64,
    confirm: Duration,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            low: 30.0,
            critical: 15.0,
            low_clear: 33.0,
            critical_clear: 18.0,
            confirm: Duration::from_secs(2),
        }
    }
}
#[derive(Clone, Serialize)]
pub struct Event {
    pub id: u64,
    pub code: String,
    pub severity: String,
    pub message: String,
    pub occurred_ms: u64,
    pub resolved_ms: Option<u64>,
    pub resolution: Option<String>,
    pub acknowledged: bool,
    pub session: u64,
    pub system_id: Option<u32>,
    pub component_id: Option<u8>,
    pub source: String,
    pub message_source: Option<String>,
}
#[derive(Clone, Serialize)]
pub struct SafetyView {
    pub overall: String,
    pub connection: String,
    pub battery: String,
    pub gps: String,
    pub sensors: String,
    pub freshness: String,
    pub armed: String,
    pub mode: String,
    pub landed: String,
    pub source: String,
    pub session: u64,
    pub events: Vec<Event>,
}
impl Default for SafetyView {
    fn default() -> Self {
        Self {
            overall: "UNKNOWN".into(),
            connection: "UNKNOWN".into(),
            battery: "UNKNOWN".into(),
            gps: "UNKNOWN".into(),
            sensors: "UNKNOWN".into(),
            freshness: "UNKNOWN".into(),
            armed: "UNKNOWN".into(),
            mode: "UNKNOWN".into(),
            landed: "UNKNOWN".into(),
            source: "OFFLINE".into(),
            session: 0,
            events: Vec::new(),
        }
    }
}
fn ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn live<'a>(v: &'a View, key: &str) -> Option<&'a Field> {
    v.fields.get(key).filter(|f| f.status == "LIVE")
}
fn text(v: &View, key: &str) -> Option<String> {
    live(v, key).and_then(|f| f.text.clone())
}
fn number(v: &View, key: &str) -> Option<f64> {
    live(v, key).and_then(|f| f.value)
}
fn severity_rank(x: &str) -> u8 {
    match x {
        "CRITICAL" => 4,
        "WARNING" => 3,
        "CAUTION" => 2,
        "INFORMATION" => 1,
        _ => 0,
    }
}
#[derive(Clone)]
pub struct Monitor {
    policy: Policy,
    events: VecDeque<Event>,
    pending: HashMap<String, Instant>,
    field_sources: HashMap<String, String>,
    session: u64,
    source: String,
    sys: Option<u32>,
    comp: Option<u8>,
    prev_connection: String,
    prev_armed: Option<String>,
    prev_mode: Option<String>,
    last_status_ms: u64,
    sequence: u64,
    view: SafetyView,
    saw_loss: bool,
}
impl Default for Monitor {
    fn default() -> Self {
        Self {
            policy: Policy::default(),
            events: VecDeque::new(),
            pending: HashMap::new(),
            field_sources: HashMap::new(),
            session: 0,
            source: "OFFLINE".into(),
            sys: None,
            comp: None,
            prev_connection: "DISCONNECTED".into(),
            prev_armed: None,
            prev_mode: None,
            last_status_ms: 0,
            sequence: 0,
            view: SafetyView::default(),
            saw_loss: false,
        }
    }
}
impl Monitor {
    fn append(&mut self, code: &str, severity: &str, message: String, active: bool) {
        self.sequence += 1;
        let t = ms();
        if self.events.len() >= CAP {
            if let Some(i) = self.events.iter().position(|e| e.resolved_ms.is_some()) {
                self.events.remove(i);
            } else {
                return;
            }
        }
        self.events.push_back(Event {
            id: self.sequence,
            code: code.into(),
            severity: severity.into(),
            message,
            occurred_ms: t,
            resolved_ms: if active { None } else { Some(t) },
            resolution: if active {
                None
            } else {
                Some("단발 이벤트 · 상태 해제 여부 미확인".into())
            },
            acknowledged: false,
            session: self.session,
            system_id: self.sys,
            component_id: self.comp,
            source: self.source.clone(),
            message_source: match code {
                "battery_low" | "battery_critical" | "battery_data" => {
                    self.field_sources.get("battery").cloned()
                }
                "gps_fix_lost" | "gps_data" => self.field_sources.get("gps_fix").cloned(),
                "satellites_unknown" => self.field_sources.get("satellites").cloned(),
                "sensor_health" => self.field_sources.get("sensors_health").cloned(),
                "attitude_stale" | "attitude_unknown" => self.field_sources.get("roll").cloned(),
                "px4_statustext"
                | "px4_status_critical"
                | "px4_status_warning"
                | "px4_status_caution" => self.field_sources.get("status_text").cloned(),
                "mav_state_critical"
                | "connection_lost"
                | "connection_recovered"
                | "connection_established"
                | "armed_changed"
                | "mode_changed" => Some("HEARTBEAT".into()),
                _ => None,
            },
        });
    }
    fn active(&self, code: &str) -> bool {
        self.events
            .iter()
            .any(|e| e.code == code && e.resolved_ms.is_none())
    }
    fn resolve(&mut self, code: &str, reason: &str) {
        self.pending.remove(code);
        for e in self
            .events
            .iter_mut()
            .filter(|e| e.code == code && e.resolved_ms.is_none())
        {
            e.resolved_ms = Some(ms());
            e.resolution = Some(reason.into());
        }
    }
    fn condition(
        &mut self,
        code: &str,
        severity: &str,
        message: &str,
        condition: bool,
        delay: Duration,
    ) {
        if condition {
            if self.active(code) {
                self.pending.remove(code);
                if let Some(e) = self
                    .events
                    .iter_mut()
                    .find(|e| e.code == code && e.resolved_ms.is_none())
                {
                    e.message = message.into();
                }
                return;
            }
            let since = *self.pending.entry(code.into()).or_insert_with(Instant::now);
            if since.elapsed() >= delay {
                self.pending.remove(code);
                self.append(code, severity, message.into(), true);
            }
        } else {
            self.resolve(code, "관측 조건 해제");
        }
    }
    pub fn new_session(&mut self, source: &str) {
        let codes: Vec<String> = self
            .events
            .iter()
            .filter(|e| e.resolved_ms.is_none())
            .map(|e| e.code.clone())
            .collect();
        for c in codes {
            self.resolve(&c, "세션 종료 · 실제 문제 해제 여부 미확인");
        }
        self.session += 1;
        self.field_sources.clear();
        self.source = source.into();
        self.sys = None;
        self.comp = None;
        self.prev_connection = "CONNECTING".into();
        self.prev_armed = None;
        self.prev_mode = None;
        self.last_status_ms = 0;
        self.pending.clear();
    }
    pub fn end_session(&mut self) {
        let codes: Vec<String> = self
            .events
            .iter()
            .filter(|e| e.resolved_ms.is_none())
            .map(|e| e.code.clone())
            .collect();
        for c in codes {
            self.resolve(&c, "연결 종료 · 실제 문제 해제 여부 미확인");
        }
        self.pending.clear();
        self.prev_armed = None;
        self.prev_mode = None;
        self.saw_loss = false;
    }
    pub fn acknowledge(&mut self, id: u64) -> bool {
        if let Some(e) = self.events.iter_mut().find(|e| e.id == id) {
            e.acknowledged = true;
            true
        } else {
            false
        }
    }
    pub fn evaluate(
        &mut self,
        phase: &str,
        source: &str,
        v: &View,
        identity: Option<(u32, u8)>,
    ) -> SafetyView {
        self.source = source.into();
        for (key, field) in &v.fields {
            self.field_sources.insert(key.clone(), field.source.into());
        }
        if let Some((s, c)) = identity {
            self.sys = Some(s);
            self.comp = Some(c);
        }
        let connected = phase == "CONNECTED";
        if phase == "CONNECTION_LOST" && self.prev_connection != "CONNECTION_LOST" {
            self.saw_loss = true;
            self.condition(
                "connection_lost",
                "WARNING",
                "HEARTBEAT 중단 또는 연결 손실",
                true,
                Duration::ZERO,
            );
        }
        if connected && self.prev_connection != "CONNECTED" {
            if self.saw_loss {
                self.append(
                    "connection_recovered",
                    "INFORMATION",
                    "HEARTBEAT 재수신 · 연결 복구".into(),
                    false,
                );
                self.saw_loss = false;
            } else {
                self.append(
                    "connection_established",
                    "INFORMATION",
                    "HEARTBEAT 수신 시작".into(),
                    false,
                );
            }
            self.resolve("connection_lost", "HEARTBEAT 재수신");
        }
        if phase == "DISCONNECTED" && self.prev_connection != "DISCONNECTED" {
            self.resolve("connection_lost", "사용자 해제 · 상태 미확인");
        }
        self.prev_connection = phase.into();
        let mut bstate = "UNKNOWN";
        let mut gstate = "UNKNOWN";
        let mut sstate = "UNKNOWN";
        let mut fstate = "UNKNOWN";
        if connected {
            let battery_field = v.fields.get("battery");
            let battery_missing =
                !battery_field.is_some_and(|f| f.status == "LIVE" && f.value.is_some());
            if battery_missing {
                self.pending.remove("battery_low");
                self.pending.remove("battery_critical");
            }
            let battery_note = match battery_field.map(|f| f.status) {
                Some("STALE") => "배터리 데이터 지연 · 상태 미확인",
                Some("INVALID") => "배터리 데이터 무효 · 상태 미확인",
                _ => "배터리 데이터 미수신 · 상태 미확인",
            };
            self.condition(
                "battery_data",
                "UNKNOWN",
                battery_note,
                battery_missing,
                Duration::ZERO,
            );
            let gps_field = v.fields.get("gps_fix");
            let gps_missing = !gps_field.is_some_and(|f| f.status == "LIVE" && f.value.is_some());
            if gps_missing {
                self.pending.remove("gps_fix_lost");
            }
            let gps_note = match gps_field.map(|f| f.status) {
                Some("STALE") => "GPS 데이터 지연 · 상태 미확인",
                Some("INVALID") => "GPS 데이터 무효 · 상태 미확인",
                _ => "GPS 데이터 미수신 · 상태 미확인",
            };
            self.condition("gps_data", "UNKNOWN", gps_note, gps_missing, Duration::ZERO);
            self.condition(
                "satellites_unknown",
                "UNKNOWN",
                "위성 수 미수신·무효 또는 0",
                number(v, "satellites").is_none_or(|n| n < 1.0),
                Duration::ZERO,
            );
            if let Some(b) = number(v, "battery") {
                let critical = b <= self.policy.critical
                    || (self.active("battery_critical") && b < self.policy.critical_clear);
                let low = b <= self.policy.low
                    || (self.active("battery_low") && b < self.policy.low_clear);
                self.condition(
                    "battery_critical",
                    "CRITICAL",
                    "배터리 위험 · 개발용 기준 ≤15%",
                    critical,
                    self.policy.confirm,
                );
                self.condition(
                    "battery_low",
                    "WARNING",
                    "배터리 부족 · 개발용 기준 ≤30%",
                    low && !(critical && self.active("battery_critical")),
                    self.policy.confirm,
                );
                bstate = if self.active("battery_critical") {
                    "CRITICAL"
                } else if self.active("battery_low") {
                    "WARNING"
                } else if critical || low {
                    "CAUTION"
                } else {
                    "NORMAL"
                };
            }
            if let Some(g) = number(v, "gps_fix") {
                let bad = g < 2.0;
                self.condition(
                    "gps_fix_lost",
                    "WARNING",
                    "GPS Fix 손실 또는 미확보",
                    bad,
                    self.policy.confirm,
                );
                gstate = if self.active("gps_fix_lost") {
                    "WARNING"
                } else if bad {
                    "CAUTION"
                } else if number(v, "satellites").is_none_or(|n| n < 1.0) {
                    "UNKNOWN"
                } else {
                    "NORMAL"
                };
            }
            if let (Some(p), Some(e), Some(h)) = (
                number(v, "sensors_present"),
                number(v, "sensors_enabled"),
                number(v, "sensors_health"),
            ) {
                let enabled = (p as u32) & (e as u32);
                if enabled != 0 {
                    let bad = enabled & !(h as u32);
                    self.condition(
                        "sensor_health",
                        "WARNING",
                        "SYS_STATUS에서 활성 센서 건강 오류 보고",
                        bad != 0,
                        self.policy.confirm,
                    );
                    sstate = if self.active("sensor_health") {
                        "WARNING"
                    } else if bad != 0 {
                        "CAUTION"
                    } else {
                        "NORMAL"
                    };
                }
            }
            if sstate == "UNKNOWN" {
                self.pending.remove("sensor_health");
            }
            let roll = v.fields.get("roll");
            let pitch = v.fields.get("pitch");
            let attitude_live = [roll, pitch]
                .iter()
                .all(|f| f.is_some_and(|f| f.status == "LIVE" && f.value.is_some()));
            let attitude_stale = [roll, pitch]
                .iter()
                .any(|f| f.is_some_and(|f| f.status == "STALE"));
            self.condition(
                "attitude_unknown",
                "UNKNOWN",
                "자세 데이터 미수신 또는 무효",
                !attitude_live && !attitude_stale,
                Duration::ZERO,
            );
            if attitude_live {
                fstate = "NORMAL";
                self.condition(
                    "attitude_stale",
                    "CAUTION",
                    "자세 데이터 수신 지연",
                    false,
                    Duration::ZERO,
                );
            } else if attitude_stale {
                fstate = "CAUTION";
                self.condition(
                    "attitude_stale",
                    "CAUTION",
                    "자세 데이터 수신 지연",
                    true,
                    Duration::ZERO,
                );
            }
            for (code, key) in [("armed_changed", "armed"), ("mode_changed", "mode")] {
                if let Some(value) = text(v, key) {
                    let previous = if key == "armed" {
                        self.prev_armed.replace(value.clone())
                    } else {
                        self.prev_mode.replace(value.clone())
                    };
                    if previous.as_deref().is_some_and(|p| p != value) {
                        self.append(
                            code,
                            "INFORMATION",
                            format!("{key} 변경: {} → {value}", previous.unwrap()),
                            false,
                        );
                    }
                }
            }
            if let Some(f) = live(v, "status_text") {
                if f.timestamp_ms != self.last_status_ms {
                    self.last_status_ms = f.timestamp_ms;
                    let level = number(v, "status_severity").unwrap_or(7.0) as u8;
                    let sev = match level {
                        0..=2 => "CRITICAL",
                        3 => "WARNING",
                        4 => "CAUTION",
                        _ => "INFORMATION",
                    };
                    let message = format!(
                        "기체 메시지: {} · 해제 여부 미확인",
                        f.text.as_deref().unwrap_or("확인 불가")
                    );
                    match sev {
                        "CRITICAL" => self.condition(
                            "px4_status_critical",
                            sev,
                            &message,
                            true,
                            Duration::ZERO,
                        ),
                        "WARNING" => self.condition(
                            "px4_status_warning",
                            sev,
                            &message,
                            true,
                            Duration::ZERO,
                        ),
                        "CAUTION" => self.condition(
                            "px4_status_caution",
                            sev,
                            &message,
                            true,
                            Duration::ZERO,
                        ),
                        _ => self.append("px4_statustext", sev, message, false),
                    }
                }
            }
            if let Some(state) = number(v, "mav_state") {
                self.condition(
                    "mav_state_critical",
                    "CRITICAL",
                    "HEARTBEAT가 CRITICAL/EMERGENCY/FLIGHT_TERMINATION 상태를 보고함",
                    state == 5.0 || state == 6.0 || state == 8.0,
                    Duration::ZERO,
                );
            }
        }
        if !connected {
            self.pending.clear();
        }
        let active_rank = self
            .events
            .iter()
            .filter(|e| e.resolved_ms.is_none())
            .map(|e| severity_rank(&e.severity))
            .max()
            .unwrap_or(0);
        let overall = if active_rank >= 4 {
            "CRITICAL"
        } else if active_rank >= 3 {
            "WARNING"
        } else if active_rank >= 2 || [bstate, gstate, sstate, fstate].contains(&"CAUTION") {
            "CAUTION"
        } else if !connected
            || [bstate, gstate, sstate, fstate].contains(&"UNKNOWN")
            || text(v, "armed").is_none()
            || text(v, "landed").is_none()
            || text(v, "mode").is_none_or(|mode| mode == "확인 불가")
            || !number(v, "mav_state").is_some_and(|state| state == 3.0 || state == 4.0)
        {
            "UNKNOWN"
        } else {
            "NORMAL"
        };
        self.view = SafetyView {
            overall: overall.into(),
            connection: if connected { "INFORMATION" } else { "UNKNOWN" }.into(),
            battery: bstate.into(),
            gps: gstate.into(),
            sensors: sstate.into(),
            freshness: fstate.into(),
            armed: text(v, "armed").unwrap_or_else(|| "UNKNOWN".into()),
            mode: text(v, "mode").unwrap_or_else(|| "UNKNOWN".into()),
            landed: text(v, "landed").unwrap_or_else(|| "UNKNOWN".into()),
            source: source.into(),
            session: self.session,
            events: self.events.iter().rev().cloned().collect(),
        };
        self.view.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn field(value: Option<f64>, text: Option<&str>, status: &'static str) -> Field {
        Field {
            value,
            text: text.map(str::to_string),
            unit: "",
            status,
            timestamp_ms: 1,
            source: "TEST",
            system_id: 42,
            component_id: 1,
        }
    }
    fn ready() -> View {
        let mut v = View::default();
        for (k, n) in [
            ("battery", 80.0),
            ("gps_fix", 3.0),
            ("satellites", 10.0),
            ("sensors_present", 3.0),
            ("sensors_enabled", 3.0),
            ("sensors_health", 3.0),
            ("roll", 0.0),
            ("pitch", 0.0),
            ("mav_state", 3.0),
        ] {
            v.fields.insert(k.into(), field(Some(n), None, "LIVE"));
        }
        v.fields
            .insert("armed".into(), field(None, Some("DISARMED"), "LIVE"));
        v.fields
            .insert("mode".into(), field(None, Some("Position"), "LIVE"));
        v.fields
            .insert("landed".into(), field(None, Some("ON_GROUND"), "LIVE"));
        v
    }
    fn monitor() -> Monitor {
        let mut m = Monitor::default();
        m.policy.confirm = Duration::ZERO;
        m.new_session("SIMULATION");
        m
    }
    #[test]
    fn incomplete_vehicle_state_is_not_normal() {
        let mut m = monitor();
        let mut v = ready();
        v.fields.remove("landed");
        assert_eq!(
            m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)))
                .overall,
            "UNKNOWN"
        );
        v = ready();
        v.fields
            .insert("satellites".into(), field(Some(0.0), None, "LIVE"));
        assert_eq!(
            m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1))).gps,
            "UNKNOWN"
        );
        v = ready();
        v.fields.remove("pitch");
        assert_eq!(
            m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)))
                .freshness,
            "UNKNOWN"
        );
    }
    #[test]
    fn transient_low_does_not_open_warning() {
        let mut m = monitor();
        m.policy.confirm = Duration::from_millis(40);
        let mut v = ready();
        m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)));
        v.fields
            .insert("battery".into(), field(Some(20.0), None, "LIVE"));
        let pending = m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)));
        assert_eq!(pending.battery, "CAUTION");
        assert!(!pending
            .events
            .iter()
            .any(|e| e.code == "battery_low" && e.resolved_ms.is_none()));
        v.fields
            .insert("battery".into(), field(Some(50.0), None, "LIVE"));
        std::thread::sleep(Duration::from_millis(45));
        let cleared = m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)));
        assert!(!cleared
            .events
            .iter()
            .any(|e| e.code == "battery_low" && e.resolved_ms.is_none()));
    }
    #[test]
    fn status_message_critical_is_not_auto_cleared_by_ack() {
        let mut m = monitor();
        let mut v = ready();
        v.fields.insert(
            "status_text".into(),
            field(None, Some("test critical"), "LIVE"),
        );
        v.fields
            .insert("status_severity".into(), field(Some(2.0), None, "LIVE"));
        let out = m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)));
        let id = out
            .events
            .iter()
            .find(|e| e.code == "px4_status_critical")
            .unwrap()
            .id;
        assert!(m.acknowledge(id));
        v.fields.remove("status_text");
        let out = m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)));
        let event = out.events.iter().find(|e| e.id == id).unwrap();
        assert!(event.acknowledged && event.resolved_ms.is_none());
    }
    #[test]
    fn unknown_never_normal_and_stale_does_not_resolve_battery() {
        let mut m = monitor();
        let empty = View::default();
        assert_eq!(
            m.evaluate("DISCONNECTED", "OFFLINE", &empty, None).overall,
            "UNKNOWN"
        );
        let mut v = ready();
        assert_eq!(
            m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)))
                .overall,
            "NORMAL"
        );
        v.fields
            .insert("battery".into(), field(Some(20.0), None, "LIVE"));
        let out = m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)));
        let id = out
            .events
            .iter()
            .find(|x| x.code == "battery_low" && x.resolved_ms.is_none())
            .unwrap()
            .id;
        assert!(m.acknowledge(id));
        v.fields
            .insert("battery".into(), field(Some(20.0), None, "STALE"));
        let out = m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)));
        let e = out.events.iter().find(|x| x.id == id).unwrap();
        assert!(e.acknowledged && e.resolved_ms.is_none());
        assert_eq!(out.battery, "UNKNOWN");
    }
    #[test]
    fn thresholds_hysteresis_and_dedup() {
        let mut m = monitor();
        let mut v = ready();
        m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)));
        v.fields
            .insert("battery".into(), field(Some(30.0), None, "LIVE"));
        let first = m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)));
        assert_eq!(first.overall, "WARNING");
        let n = first.events.len();
        assert_eq!(
            m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)))
                .events
                .len(),
            n
        );
        v.fields
            .insert("battery".into(), field(Some(32.0), None, "LIVE"));
        assert_eq!(
            m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)))
                .battery,
            "WARNING"
        );
        v.fields
            .insert("battery".into(), field(Some(14.0), None, "LIVE"));
        assert_eq!(
            m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)))
                .overall,
            "CRITICAL"
        );
        v.fields
            .insert("battery".into(), field(Some(34.0), None, "LIVE"));
        let out = m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)));
        assert_eq!(out.battery, "NORMAL");
        assert!(out
            .events
            .iter()
            .any(|e| e.code == "battery_critical" && e.resolved_ms.is_some()));
    }
    #[test]
    fn loss_recovery_sessions_and_independent_alerts() {
        let mut m = monitor();
        let mut v = ready();
        m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)));
        v.fields
            .insert("gps_fix".into(), field(Some(1.0), None, "LIVE"));
        v.fields
            .insert("sensors_health".into(), field(Some(1.0), None, "LIVE"));
        v.fields
            .insert("roll".into(), field(Some(0.0), None, "STALE"));
        let out = m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)));
        for code in ["gps_fix_lost", "sensor_health", "attitude_stale"] {
            assert!(out
                .events
                .iter()
                .any(|e| e.code == code && e.resolved_ms.is_none()));
        }
        let out = m.evaluate("CONNECTION_LOST", "SIMULATION", &View::default(), None);
        assert!(out
            .events
            .iter()
            .any(|e| e.code == "connection_lost" && e.resolved_ms.is_none()));
        m.new_session("SIMULATION");
        let out = m.evaluate("CONNECTED", "SIMULATION", &ready(), Some((42, 1)));
        assert!(out.events.iter().any(|e| e.code == "connection_recovered"));
        assert_eq!(out.session, 2);
        assert!(out
            .events
            .iter()
            .filter(|e| e.session == 1 && e.code == "gps_fix_lost")
            .all(|e| e.resolved_ms.is_some()));
    }
    #[test]
    fn disabled_sensor_is_not_a_failure_and_history_is_bounded() {
        let mut m = monitor();
        let mut v = ready();
        v.fields
            .insert("sensors_enabled".into(), field(Some(1.0), None, "LIVE"));
        v.fields
            .insert("sensors_health".into(), field(Some(1.0), None, "LIVE"));
        let out = m.evaluate("CONNECTED", "SIMULATION", &v, Some((42, 1)));
        assert_eq!(out.sensors, "NORMAL");
        for i in 0..250 {
            m.append("test", "INFORMATION", format!("{i}"), false);
        }
        assert!(m.events.len() <= CAP);
    }
}
