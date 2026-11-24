use mavlink::{dialects::common::MavMessage, MAVLinkMessageRaw, Message};
use serde::Serialize;
use std::{
    collections::{BTreeMap, VecDeque},
    time::{Instant, SystemTime, UNIX_EPOCH},
};
const CAP: usize = 120;
#[derive(Clone)]
struct Entry {
    value: Option<f64>,
    text: Option<String>,
    unit: &'static str,
    at: Instant,
    timestamp_ms: u64,
    source: &'static str,
    sys: u32,
    comp: u8,
    ttl_ms: u64,
}
#[derive(Clone, Serialize)]
pub struct Field {
    pub value: Option<f64>,
    pub text: Option<String>,
    pub unit: &'static str,
    pub status: &'static str,
    pub timestamp_ms: u64,
    pub source: &'static str,
    pub system_id: u32,
    pub component_id: u8,
}
#[derive(Clone, Serialize)]
pub struct Point {
    pub timestamp_ms: u64,
    pub value: Option<f64>,
}
#[derive(Clone, Default, Serialize)]
pub struct View {
    pub fields: BTreeMap<String, Field>,
    pub graphs: BTreeMap<String, Vec<Point>>,
}
#[derive(Clone, Default)]
pub struct Store {
    fields: BTreeMap<String, Entry>,
    graphs: BTreeMap<String, VecDeque<Point>>,
}
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
impl Store {
    pub fn view(&self) -> View {
        View {
            fields: self
                .fields
                .iter()
                .map(|(k, e)| {
                    (
                        k.clone(),
                        Field {
                            value: e.value,
                            text: e.text.clone(),
                            unit: e.unit,
                            status: if e.value.is_none() && e.text.is_none() {
                                "INVALID"
                            } else if e.at.elapsed().as_millis() > e.ttl_ms as u128 {
                                "STALE"
                            } else {
                                "LIVE"
                            },
                            timestamp_ms: e.timestamp_ms,
                            source: e.source,
                            system_id: e.sys,
                            component_id: e.comp,
                        },
                    )
                })
                .collect(),
            graphs: self
                .graphs
                .iter()
                .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
                .collect(),
        }
    }
    fn put(
        &mut self,
        key: &str,
        value: Option<f64>,
        unit: &'static str,
        source: &'static str,
        ttl_ms: u64,
        sys: u32,
        comp: u8,
    ) {
        let value = value.filter(|x| {
            x.is_finite()
                && match key {
                    "relative_alt" | "msl_alt" | "local_north" | "local_east" | "local_down" => {
                        x.abs() <= 100_000.0
                    }
                    "ground_speed" | "air_speed" => (0.0..=1000.0).contains(x),
                    "climb" | "local_climb" | "local_vx" | "local_vy" => x.abs() <= 1000.0,
                    "heading" => (0.0..360.0).contains(x),
                    _ => true,
                }
        });
        let timestamp_ms = now_ms();
        self.fields.insert(
            key.into(),
            Entry {
                value,
                text: None,
                unit,
                at: Instant::now(),
                timestamp_ms,
                source,
                sys,
                comp,
                ttl_ms,
            },
        );
        if matches!(
            key,
            "battery" | "relative_alt" | "ground_speed" | "roll" | "pitch" | "yaw"
        ) {
            let g = self.graphs.entry(key.into()).or_default();
            if g.back()
                .is_some_and(|p| timestamp_ms.saturating_sub(p.timestamp_ms) > ttl_ms)
            {
                g.push_back(Point {
                    timestamp_ms: timestamp_ms.saturating_sub(1),
                    value: None,
                });
            }
            g.push_back(Point {
                timestamp_ms,
                value,
            });
            while g.len() > CAP {
                g.pop_front();
            }
        }
    }
    fn text(
        &mut self,
        key: &str,
        value: Option<String>,
        source: &'static str,
        ttl_ms: u64,
        sys: u32,
        comp: u8,
    ) {
        self.fields.insert(
            key.into(),
            Entry {
                value: None,
                text: value,
                unit: "",
                at: Instant::now(),
                timestamp_ms: now_ms(),
                source,
                sys,
                comp,
                ttl_ms,
            },
        );
    }
    pub fn consume(&mut self, raw: &MAVLinkMessageRaw) {
        if let MAVLinkMessageRaw::V2(v) = raw {
            if v.incompatibility_flags() != 0 {
                return;
            }
        }
        let sys = raw.system_id();
        let comp = raw.component_id();
        let Ok(msg) = MavMessage::parse(raw.version(), raw.message_id(), raw.payload()) else {
            return;
        };
        match msg {
            MavMessage::HEARTBEAT(h) => {
                let armed = if h.base_mode.bits() & 128 != 0 {
                    "ARMED"
                } else {
                    "DISARMED"
                };
                self.text("armed", Some(armed.into()), "HEARTBEAT", 5000, sys, comp);
                let main = (h.custom_mode >> 16) & 255;
                let sub = (h.custom_mode >> 24) & 255;
                let mode = match main {
                    1 => "Manual",
                    2 => "Altitude",
                    3 => "Position",
                    4 => match sub {
                        1 => "Auto Ready",
                        2 => "Auto Takeoff",
                        3 => "Auto Hold",
                        4 => "Auto Mission",
                        5 => "Auto RTL",
                        6 => "Auto Land",
                        _ => "Auto (미상)",
                    },
                    5 => "Acro",
                    6 => "Offboard",
                    7 => "Stabilized",
                    _ => "확인 불가",
                };
                self.text(
                    "mode",
                    if h.base_mode.bits() & 1 != 0 && main != 0 {
                        Some(mode.into())
                    } else {
                        None
                    },
                    "HEARTBEAT",
                    5000,
                    sys,
                    comp,
                );
            }
            MavMessage::SYS_STATUS(m) => {
                self.put(
                    "battery",
                    (0..=100)
                        .contains(&m.battery_remaining)
                        .then_some(m.battery_remaining as f64),
                    "%",
                    "SYS_STATUS",
                    10000,
                    sys,
                    comp,
                );
                self.put(
                    "voltage",
                    (m.voltage_battery != u16::MAX).then_some(m.voltage_battery as f64 / 1000.0),
                    "V",
                    "SYS_STATUS",
                    10000,
                    sys,
                    comp,
                );
                self.put(
                    "current",
                    (m.current_battery != -1).then_some(m.current_battery as f64 / 100.0),
                    "A",
                    "SYS_STATUS",
                    10000,
                    sys,
                    comp,
                );
            }
            MavMessage::BATTERY_STATUS(m) if m.id == 0 => {
                self.put(
                    "battery",
                    (0..=100)
                        .contains(&m.battery_remaining)
                        .then_some(m.battery_remaining as f64),
                    "%",
                    "BATTERY_STATUS id 0",
                    10000,
                    sys,
                    comp,
                );
                self.put(
                    "current",
                    (m.current_battery != -1).then_some(m.current_battery as f64 / 100.0),
                    "A",
                    "BATTERY_STATUS id 0",
                    10000,
                    sys,
                    comp,
                ); /* No summed voltage: cell count and extensions can be unknown. */
            }
            MavMessage::GPS_RAW_INT(m) => {
                let fix = m.fix_type as u32;
                self.put(
                    "gps_fix",
                    Some(fix as f64),
                    "",
                    "GPS_RAW_INT",
                    5000,
                    sys,
                    comp,
                );
                self.put(
                    "satellites",
                    (m.satellites_visible != u8::MAX).then_some(m.satellites_visible as f64),
                    "",
                    "GPS_RAW_INT",
                    5000,
                    sys,
                    comp,
                );
                let valid = fix >= 2
                    && (-900000000..=900000000).contains(&m.lat)
                    && (-1800000000..=1800000000).contains(&m.lon);
                self.put(
                    "gps_lat",
                    valid.then_some(m.lat as f64 / 1e7),
                    "°",
                    "GPS_RAW_INT",
                    5000,
                    sys,
                    comp,
                );
                self.put(
                    "gps_lon",
                    valid.then_some(m.lon as f64 / 1e7),
                    "°",
                    "GPS_RAW_INT",
                    5000,
                    sys,
                    comp,
                );
            }
            MavMessage::GLOBAL_POSITION_INT(m) => {
                self.put(
                    "relative_alt",
                    Some(m.relative_alt as f64 / 1000.0),
                    "m",
                    "GLOBAL_POSITION_INT",
                    3000,
                    sys,
                    comp,
                );
                self.put(
                    "msl_alt",
                    Some(m.alt as f64 / 1000.0),
                    "m",
                    "GLOBAL_POSITION_INT",
                    3000,
                    sys,
                    comp,
                );
                self.put(
                    "ground_speed",
                    Some((m.vx as f64).hypot(m.vy as f64) / 100.0),
                    "m/s",
                    "GLOBAL_POSITION_INT",
                    3000,
                    sys,
                    comp,
                );
                self.put(
                    "climb",
                    Some(-(m.vz as f64) / 100.0),
                    "m/s",
                    "GLOBAL_POSITION_INT",
                    3000,
                    sys,
                    comp,
                );
                self.put(
                    "heading",
                    (m.hdg < 36000).then_some(m.hdg as f64 / 100.0),
                    "°",
                    "GLOBAL_POSITION_INT",
                    3000,
                    sys,
                    comp,
                );
            }
            MavMessage::ATTITUDE(m) => self.attitude(
                m.roll as f64,
                m.pitch as f64,
                m.yaw as f64,
                "ATTITUDE",
                sys,
                comp,
            ),
            MavMessage::ATTITUDE_QUATERNION(m) => {
                let (w, x, y, z) = (m.q1 as f64, m.q2 as f64, m.q3 as f64, m.q4 as f64);
                let n = (w * w + x * x + y * y + z * z).sqrt();
                if n.is_finite() && (0.99..=1.01).contains(&n) {
                    let (w, x, y, z) = (w / n, x / n, y / n, z / n);
                    self.attitude(
                        (2.0 * (w * x + y * z)).atan2(1.0 - 2.0 * (x * x + y * y)),
                        (2.0 * (w * y - z * x)).clamp(-1.0, 1.0).asin(),
                        (2.0 * (w * z + x * y)).atan2(1.0 - 2.0 * (y * y + z * z)),
                        "ATTITUDE_QUATERNION",
                        sys,
                        comp,
                    );
                } else {
                    self.attitude(
                        f64::NAN,
                        f64::NAN,
                        f64::NAN,
                        "ATTITUDE_QUATERNION",
                        sys,
                        comp,
                    );
                }
            }
            MavMessage::VFR_HUD(m) => {
                let f = |v: f32| (v.is_finite()).then_some(v as f64);
                self.put(
                    "air_speed",
                    f(m.airspeed).filter(|v| *v >= 0.0),
                    "m/s",
                    "VFR_HUD",
                    3000,
                    sys,
                    comp,
                );
                self.put(
                    "ground_speed",
                    f(m.groundspeed).filter(|v| *v >= 0.0),
                    "m/s",
                    "VFR_HUD",
                    3000,
                    sys,
                    comp,
                );
                self.put("msl_alt", f(m.alt), "m", "VFR_HUD", 3000, sys, comp);
                self.put("climb", f(m.climb), "m/s", "VFR_HUD", 3000, sys, comp);
                self.put(
                    "heading",
                    (0..=360)
                        .contains(&m.heading)
                        .then_some((m.heading % 360) as f64),
                    "°",
                    "VFR_HUD",
                    3000,
                    sys,
                    comp,
                );
            }
            MavMessage::LOCAL_POSITION_NED(m) => {
                self.put(
                    "local_north",
                    Some(m.x as f64),
                    "m",
                    "LOCAL_POSITION_NED",
                    3000,
                    sys,
                    comp,
                );
                self.put(
                    "local_east",
                    Some(m.y as f64),
                    "m",
                    "LOCAL_POSITION_NED",
                    3000,
                    sys,
                    comp,
                );
                self.put(
                    "local_down",
                    Some(m.z as f64),
                    "m",
                    "LOCAL_POSITION_NED",
                    3000,
                    sys,
                    comp,
                );
                self.put(
                    "local_vx",
                    Some(m.vx as f64),
                    "m/s",
                    "LOCAL_POSITION_NED",
                    3000,
                    sys,
                    comp,
                );
                self.put(
                    "local_vy",
                    Some(m.vy as f64),
                    "m/s",
                    "LOCAL_POSITION_NED",
                    3000,
                    sys,
                    comp,
                );
                self.put(
                    "local_climb",
                    Some(-(m.vz as f64)),
                    "m/s",
                    "LOCAL_POSITION_NED",
                    3000,
                    sys,
                    comp,
                );
            }
            MavMessage::EXTENDED_SYS_STATE(m) => self.text(
                "landed",
                (m.landed_state as u32 != 0).then(|| format!("{:?}", m.landed_state)),
                "EXTENDED_SYS_STATE",
                5000,
                sys,
                comp,
            ),
            MavMessage::STATUSTEXT(m) => self.text(
                "status_text",
                m.text
                    .to_str()
                    .ok()
                    .map(|s| format!("{:?} · {}", m.severity, s)),
                "STATUSTEXT",
                30000,
                sys,
                comp,
            ),
            _ => (),
        }
    }
    fn attitude(
        &mut self,
        roll: f64,
        pitch: f64,
        yaw: f64,
        source: &'static str,
        sys: u32,
        comp: u8,
    ) {
        for (k, v) in [("roll", roll), ("pitch", pitch), ("yaw", yaw)] {
            self.put(
                k,
                (v.is_finite()
                    && v.abs()
                        <= if k == "pitch" {
                            std::f64::consts::FRAC_PI_2 + 0.001
                        } else {
                            std::f64::consts::PI + 0.001
                        })
                .then_some(v.to_degrees()),
                "°",
                source,
                2000,
                sys,
                comp,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mavlink::{dialects::common::*, write_v1_msg, MavHeader, MavlinkReader};
    fn raw(m: MavMessage) -> MAVLinkMessageRaw {
        let mut b = Vec::new();
        write_v1_msg(
            &mut b,
            MavHeader {
                system_id: 42,
                component_id: 1,
                sequence: 1,
            },
            &m,
        )
        .unwrap();
        MavlinkReader::new(std::io::Cursor::new(b))
            .read_any_raw_message::<MavMessage>()
            .unwrap()
    }
    #[test]
    fn units_invalid_and_freshness_are_per_field() {
        let mut s = Store::default();
        s.consume(&raw(MavMessage::SYS_STATUS(SYS_STATUS_DATA {
            voltage_battery: 12000,
            current_battery: 123,
            battery_remaining: 0,
            ..SYS_STATUS_DATA::DEFAULT
        })));
        let v = s.view();
        assert_eq!(v.fields["voltage"].value, Some(12.0));
        assert_eq!(v.fields["current"].value, Some(1.23));
        assert_eq!(v.fields["battery"].value, Some(0.0));
        s.consume(&raw(MavMessage::SYS_STATUS(SYS_STATUS_DATA {
            voltage_battery: u16::MAX,
            current_battery: -1,
            battery_remaining: -1,
            ..SYS_STATUS_DATA::DEFAULT
        })));
        assert_eq!(s.view().fields["voltage"].status, "INVALID");
        s.consume(&raw(MavMessage::ATTITUDE(ATTITUDE_DATA {
            roll: 0.5,
            pitch: 0.0,
            yaw: f32::NAN,
            ..ATTITUDE_DATA::DEFAULT
        })));
        assert_eq!(s.view().fields["yaw"].status, "INVALID");
        s.fields.get_mut("roll").unwrap().at = Instant::now() - std::time::Duration::from_secs(3);
        assert_eq!(s.view().fields["roll"].status, "STALE");
        assert_eq!(s.view().fields["pitch"].status, "LIVE");
    }
    #[test]
    fn gps_quaternion_and_invalid_pitch() {
        let mut s = Store::default();
        s.consume(&raw(MavMessage::GPS_RAW_INT(GPS_RAW_INT_DATA {
            fix_type: GpsFixType::GPS_FIX_TYPE_3D_FIX,
            lat: 375000000,
            lon: 1270000000,
            satellites_visible: u8::MAX,
            ..GPS_RAW_INT_DATA::DEFAULT
        })));
        assert_eq!(s.view().fields["gps_lat"].value, Some(37.5));
        assert_eq!(s.view().fields["satellites"].status, "INVALID");
        s.consume(&raw(MavMessage::GPS_RAW_INT(GPS_RAW_INT_DATA {
            fix_type: GpsFixType::GPS_FIX_TYPE_NO_FIX,
            lat: 0,
            lon: 0,
            ..GPS_RAW_INT_DATA::DEFAULT
        })));
        assert_eq!(s.view().fields["gps_lat"].status, "INVALID");
        s.consume(&raw(MavMessage::ATTITUDE_QUATERNION(
            ATTITUDE_QUATERNION_DATA {
                q1: 1.0,
                ..ATTITUDE_QUATERNION_DATA::DEFAULT
            },
        )));
        assert_eq!(s.view().fields["roll"].value, Some(0.0));
        s.consume(&raw(MavMessage::ATTITUDE(ATTITUDE_DATA {
            pitch: 2.0,
            ..ATTITUDE_DATA::DEFAULT
        })));
        assert_eq!(s.view().fields["pitch"].status, "INVALID");
    }
    #[test]
    fn ned_heading_and_graph_limit() {
        let mut s = Store::default();
        for _ in 0..150 {
            s.consume(&raw(MavMessage::GLOBAL_POSITION_INT(
                GLOBAL_POSITION_INT_DATA {
                    relative_alt: 12000,
                    alt: 125000,
                    vx: 300,
                    vy: 400,
                    vz: -125,
                    hdg: u16::MAX,
                    ..GLOBAL_POSITION_INT_DATA::DEFAULT
                },
            )));
        }
        let v = s.view();
        assert_eq!(v.fields["relative_alt"].value, Some(12.0));
        assert_eq!(v.fields["msl_alt"].value, Some(125.0));
        assert_eq!(v.fields["ground_speed"].value, Some(5.0));
        assert_eq!(v.fields["climb"].value, Some(1.25));
        assert_eq!(v.fields["heading"].status, "INVALID");
        assert_eq!(v.graphs["relative_alt"].len(), CAP);
    }
}
