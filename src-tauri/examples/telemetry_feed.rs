//! Development-only synthetic telemetry, hard-wired to loopback; never bundled with the app.
use mavlink::{dialects::common::*, write_v1_msg, MavHeader};
use std::{
    net::UdpSocket,
    thread,
    time::{Duration, Instant},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scenario = std::env::args().nth(1).unwrap_or_else(|| "normal".into());
    if ![
        "normal",
        "gps-loss",
        "stale-attitude",
        "invalid",
        "other-id",
        "stop",
        "corrupt",
        "battery-low",
        "battery-critical",
        "gps-stop",
        "heartbeat-stop",
        "multiple",
    ]
    .contains(&scenario.as_str())
    {
        return Err(
            "scenario: normal | gps-loss | gps-stop | stale-attitude | invalid | other-id | stop | corrupt | battery-low | battery-critical | heartbeat-stop | multiple"
                .into(),
        );
    }
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let start = Instant::now();
    let mut seq = 0u8;
    println!("TEST DATA ONLY · {scenario} · loopback 127.0.0.1:14540 · Ctrl+C to stop");
    loop {
        let t = start.elapsed().as_secs_f32();
        if scenario == "stop" && t > 8.0 {
            break;
        }
        let header = MavHeader {
            system_id: 42,
            component_id: 1,
            sequence: seq,
        };
        seq = seq.wrapping_add(1);
        let mut messages = vec![
            MavMessage::HEARTBEAT(HEARTBEAT_DATA {
                mavtype: MavType::MAV_TYPE_QUADROTOR,
                autopilot: MavAutopilot::MAV_AUTOPILOT_PX4,
                base_mode: MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED,
                custom_mode: 3 << 16,
                system_status: MavState::MAV_STATE_STANDBY,
                ..HEARTBEAT_DATA::DEFAULT
            }),
            MavMessage::EXTENDED_SYS_STATE(EXTENDED_SYS_STATE_DATA {
                landed_state: MavLandedState::MAV_LANDED_STATE_ON_GROUND,
                ..EXTENDED_SYS_STATE_DATA::DEFAULT
            }),
            MavMessage::SYS_STATUS(SYS_STATUS_DATA {
                onboard_control_sensors_present: MavSysStatusSensor::MAV_SYS_STATUS_SENSOR_3D_GYRO
                    | MavSysStatusSensor::MAV_SYS_STATUS_SENSOR_3D_ACCEL,
                onboard_control_sensors_enabled: MavSysStatusSensor::MAV_SYS_STATUS_SENSOR_3D_GYRO
                    | MavSysStatusSensor::MAV_SYS_STATUS_SENSOR_3D_ACCEL,
                onboard_control_sensors_health: MavSysStatusSensor::MAV_SYS_STATUS_SENSOR_3D_GYRO
                    | MavSysStatusSensor::MAV_SYS_STATUS_SENSOR_3D_ACCEL,
                voltage_battery: (15700.0 - t * 2.0).max(12000.0) as u16,
                current_battery: 250,
                battery_remaining: if (5.0..16.0).contains(&t)
                    && (scenario == "battery-critical" || scenario == "multiple")
                {
                    10
                } else if (5.0..16.0).contains(&t) && scenario == "battery-low" {
                    25
                } else {
                    85
                },
                ..SYS_STATUS_DATA::DEFAULT
            }),
            MavMessage::GLOBAL_POSITION_INT(GLOBAL_POSITION_INT_DATA {
                relative_alt: (12000.0 + 4000.0 * (t / 5.0).sin()) as i32,
                alt: 123000,
                vx: (350.0 + 100.0 * (t / 4.0).sin()) as i16,
                vy: 100,
                vz: -75,
                hdg: ((t * 500.0) as u16) % 36000,
                ..GLOBAL_POSITION_INT_DATA::DEFAULT
            }),
            MavMessage::GPS_RAW_INT(GPS_RAW_INT_DATA {
                fix_type: if (scenario == "gps-loss" && t > 8.0)
                    || (scenario == "multiple" && (5.0..16.0).contains(&t))
                {
                    GpsFixType::GPS_FIX_TYPE_NO_FIX
                } else {
                    GpsFixType::GPS_FIX_TYPE_3D_FIX
                },
                lat: 375000000,
                lon: 1270000000,
                satellites_visible: if (scenario == "gps-loss" && t > 8.0)
                    || (scenario == "multiple" && (5.0..16.0).contains(&t))
                {
                    0
                } else {
                    12
                },
                ..GPS_RAW_INT_DATA::DEFAULT
            }),
        ];
        if scenario != "stale-attitude" || t < 8.0 {
            messages.push(MavMessage::ATTITUDE(ATTITUDE_DATA {
                roll: if scenario == "invalid" && t > 5.0 {
                    f32::NAN
                } else {
                    0.35 * (t / 3.0).sin()
                },
                pitch: 0.18 * (t / 4.0).sin(),
                yaw: 0.4 * (t / 5.0).sin(),
                ..ATTITUDE_DATA::DEFAULT
            }));
        }
        for msg in messages {
            if scenario == "gps-stop" && t > 8.0 && matches!(msg, MavMessage::GPS_RAW_INT(_)) {
                continue;
            }
            if scenario == "heartbeat-stop"
                && (8.0..16.0).contains(&t)
                && matches!(msg, MavMessage::HEARTBEAT(_))
            {
                continue;
            }
            let mut frame = Vec::new();
            let h = if scenario == "other-id" && t > 8.0 && !matches!(msg, MavMessage::HEARTBEAT(_))
            {
                MavHeader {
                    system_id: 43,
                    ..header
                }
            } else {
                header
            };
            write_v1_msg(&mut frame, h, &msg)?;
            if scenario == "corrupt" && !matches!(msg, MavMessage::HEARTBEAT(_)) {
                if let Some(last) = frame.last_mut() {
                    *last ^= 0xff;
                }
            }
            socket.send_to(&frame, "127.0.0.1:14540")?;
        }
        thread::sleep(Duration::from_millis(500));
    }
    Ok(())
}
