use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashSet,
    fs,
    io::{Read, Write},
    path::Path,
    sync::Mutex,
};
use tauri::Manager;

#[derive(Default)]
pub struct MissionStore(Mutex<()>);
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mission {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub kind: String,
    pub status: String,
    pub building_id: String,
    pub building_name: String,
    pub address: String,
    pub lon: Option<f64>,
    pub lat: Option<f64>,
    pub geometry: Option<Value>,
    pub radius: f64,
    pub purpose: String,
    pub assignee: String,
    pub notes: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Database {
    pub version: u32,
    pub revision: u64,
    pub missions: Vec<Mission>,
}
impl Default for Database {
    fn default() -> Self {
        Self {
            version: 1,
            revision: 0,
            missions: Vec::new(),
        }
    }
}
fn valid_ring(value: &Value) -> bool {
    let Some(ring) = value.as_array() else {
        return false;
    };
    if ring.len() < 4 || ring.len() > 50_000 || ring.first() != ring.last() {
        return false;
    }
    ring.iter().all(|point| {
        point.as_array().is_some_and(|p| {
            if p.len() < 2 || p.len() > 3 {
                return false;
            }
            match (p[0].as_f64(), p[1].as_f64()) {
                (Some(lon), Some(lat)) => {
                    lon.is_finite()
                        && lat.is_finite()
                        && (-180.0..=180.0).contains(&lon)
                        && (-85.0..=85.0).contains(&lat)
                        && (p.len() == 2 || p[2].as_f64().is_some_and(f64::is_finite))
                }
                _ => false,
            }
        })
    })
}
fn valid_polygon(value: &Value) -> bool {
    value
        .as_array()
        .is_some_and(|rings| !rings.is_empty() && rings.iter().all(valid_ring))
}
fn valid_geometry(value: &Value) -> bool {
    let Some(coords) = value.get("coordinates") else {
        return false;
    };
    match value.get("type").and_then(Value::as_str) {
        Some("Polygon") => valid_polygon(coords),
        Some("MultiPolygon") => coords
            .as_array()
            .is_some_and(|polygons| !polygons.is_empty() && polygons.iter().all(valid_polygon)),
        _ => false,
    }
}
fn validate(db: &Database) -> Result<(), &'static str> {
    if db.version != 1 || db.missions.len() > 1000 {
        return Err("지원하지 않는 임무 데이터 형식 또는 개수 초과");
    }
    let mut ids = HashSet::new();
    for m in &db.missions {
        if m.id.is_empty()
            || m.id.len() > 80
            || !m.id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
            || !ids.insert(&m.id)
            || m.name.trim().is_empty()
            || m.name.len() > 400
            || m.created_at.len() > 40
            || m.created_at.is_empty()
            || m.updated_at.len() > 40
            || m.updated_at.is_empty()
        {
            return Err("임무 식별 정보가 올바르지 않습니다");
        }
        if !matches!(
            m.kind.as_str(),
            "FIRE_SEARCH" | "EXTERIOR" | "SURROUNDINGS" | "DAMAGE" | "CUSTOM"
        ) || !matches!(m.status.as_str(), "DRAFT" | "SAVED" | "ARCHIVED")
        {
            return Err("임무 유형 또는 상태가 올바르지 않습니다");
        }
        if !m.radius.is_finite()
            || !(1.0..=2000.0).contains(&m.radius)
            || m.lon
                .is_some_and(|n| !n.is_finite() || !(-180.0..=180.0).contains(&n))
            || m.lat
                .is_some_and(|n| !n.is_finite() || !(-85.0..=85.0).contains(&n))
            || m.lon.is_some() != m.lat.is_some()
        {
            return Err("좌표 또는 조회 범위가 올바르지 않습니다");
        }
        if m.status != "DRAFT" && (m.lon.is_none() || m.lat.is_none()) {
            return Err("저장하거나 보관할 임무에는 위치가 필요합니다");
        }
        if [
            &m.building_id,
            &m.building_name,
            &m.address,
            &m.purpose,
            &m.assignee,
            &m.notes,
        ]
        .iter()
        .any(|s| s.len() > 12000)
        {
            return Err("임무 텍스트가 너무 깁니다");
        }
        if let Some(g) = &m.geometry {
            if !valid_geometry(g)
                || m.lon.is_none()
                || serde_json::to_vec(g).map_err(|_| "건물 윤곽 오류")?.len() > 500_000
            {
                return Err("건물 윤곽이 올바르지 않습니다");
            }
        }
    }
    Ok(())
}
fn read(path: &Path) -> Result<Database, String> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Database::default()),
        Err(_) => return Err("임무 파일을 읽을 수 없습니다".into()),
    };
    let mut bytes = Vec::new();
    file.take(10_000_001)
        .read_to_end(&mut bytes)
        .map_err(|_| "임무 파일을 읽을 수 없습니다")?;
    if bytes.len() > 10_000_000 {
        return Err("임무 파일이 너무 큽니다. 원본을 보존했습니다".into());
    }
    let db: Database = serde_json::from_slice(&bytes)
        .map_err(|_| "임무 파일이 손상되었거나 형식이 다릅니다. 원본을 보존했습니다")?;
    validate(&db)
        .map_err(|_| "임무 파일 내용이 올바르지 않습니다. 원본을 보존했습니다".to_string())?;
    Ok(db)
}
fn write(path: &Path, mut db: Database, expected: u64) -> Result<Database, String> {
    let dir = path.parent().ok_or("임무 저장 경로 오류")?;
    fs::create_dir_all(dir).map_err(|_| "임무 폴더를 만들 수 없습니다")?;
    #[cfg(unix)]
    let _lock = {
        use std::os::fd::AsRawFd;
        use std::os::unix::fs::OpenOptionsExt;
        let file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .mode(0o600)
            .open(dir.join(".missions.lock"))
            .map_err(|_| "임무 잠금 파일 오류")?;
        // A file descriptor lock is released automatically, including after a crash.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err("다른 프로그램에서 임무를 저장 중입니다. 다시 시도하세요".into());
        }
        file
    };
    let old = read(path)?;
    if old.revision != expected {
        return Err("임무 목록이 변경되었습니다. 다시 불러오세요".into());
    }
    validate(&db).map_err(str::to_string)?;
    db.revision = expected.checked_add(1).ok_or("임무 버전 초과")?;
    let bytes = serde_json::to_vec_pretty(&db).map_err(|_| "임무 직렬화 오류")?;
    if bytes.len() > 10_000_000 {
        return Err("임무 데이터가 너무 큽니다".into());
    }
    let tmp = dir.join(format!(
        ".missions-{}-{}-{}.tmp",
        std::process::id(),
        db.revision,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "시각 오류")?
            .as_nanos()
    ));
    let mut created = false;
    let result = (|| {
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut f = options.open(&tmp)?;
        created = true;
        f.write_all(&bytes)?;
        f.sync_all()?;
        fs::rename(&tmp, path)?;
        #[cfg(unix)]
        if let Ok(directory) = fs::File::open(dir) {
            let _ = directory.sync_all();
        }
        Ok::<(), std::io::Error>(())
    })();
    if result.is_err() {
        if created {
            let _ = fs::remove_file(tmp);
        }
        return Err("임무 저장에 실패했습니다. 기존 파일은 유지됩니다".into());
    }
    Ok(db)
}
#[tauri::command]
pub fn load_missions(
    app: tauri::AppHandle,
    store: tauri::State<'_, MissionStore>,
) -> Result<Database, String> {
    let _guard = store.0.lock().map_err(|_| "임무 저장소 오류")?;
    let p = app
        .path()
        .app_data_dir()
        .map_err(|_| "임무 경로 오류")?
        .join("missions.json");
    read(&p)
}
#[tauri::command]
pub fn save_missions(
    app: tauri::AppHandle,
    store: tauri::State<'_, MissionStore>,
    database: Database,
    expected_revision: u64,
) -> Result<Database, String> {
    let _guard = store.0.lock().map_err(|_| "임무 저장소 오류")?;
    let p = app
        .path()
        .app_data_dir()
        .map_err(|_| "임무 경로 오류")?
        .join("missions.json");
    write(&p, database, expected_revision)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn path() -> std::path::PathBuf {
        std::env::temp_dir()
            .join(format!(
                "firescene-missions-test-{}-{}",
                std::process::id(),
                std::thread::current()
                    .name()
                    .unwrap_or("test")
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                    .collect::<String>()
            ))
            .join("missions.json")
    }
    fn sample() -> Mission {
        Mission {
            id: "test-1".into(),
            name: "Test".into(),
            created_at: "2026-01-01".into(),
            updated_at: "2026-01-01".into(),
            kind: "FIRE_SEARCH".into(),
            status: "SAVED".into(),
            building_id: "".into(),
            building_name: "".into(),
            address: "".into(),
            lon: Some(127.0),
            lat: Some(37.0),
            geometry: None,
            radius: 50.0,
            purpose: "".into(),
            assignee: "".into(),
            notes: "".into(),
        }
    }
    #[test]
    fn persistence_revision_and_corruption_protection() {
        let p = path();
        let db = Database {
            missions: vec![sample()],
            ..Default::default()
        };
        let saved = write(&p, db, 0).unwrap();
        assert_eq!(saved.revision, 1);
        assert_eq!(read(&p).unwrap().missions[0].name, "Test");
        assert!(write(&p, Database::default(), 0).is_err());
        fs::write(&p, b"broken").unwrap();
        assert!(write(&p, Database::default(), 1).is_err());
        assert_eq!(fs::read(&p).unwrap(), b"broken");
        let _ = fs::remove_dir_all(p.parent().unwrap());
    }
    #[test]
    fn create_edit_clone_archive_delete_and_reload() {
        let p = path();
        let mut draft = sample();
        draft.status = "DRAFT".into();
        draft.lon = None;
        draft.lat = None;
        let db = write(
            &p,
            Database {
                missions: vec![draft.clone()],
                ..Default::default()
            },
            0,
        )
        .unwrap();
        assert_eq!(read(&p).unwrap().missions[0].status, "DRAFT");
        draft.status = "SAVED".into();
        draft.lon = Some(127.0);
        draft.lat = Some(37.0);
        draft.name = "Edited".into();
        let db = write(
            &p,
            Database {
                missions: vec![draft.clone()],
                ..db
            },
            1,
        )
        .unwrap();
        let mut duplicate = draft.clone();
        duplicate.id = "test-2".into();
        duplicate.status = "DRAFT".into();
        draft.status = "ARCHIVED".into();
        let db = write(
            &p,
            Database {
                missions: vec![draft, duplicate],
                ..db
            },
            2,
        )
        .unwrap();
        let restored = read(&p).unwrap();
        assert_eq!(restored.missions.len(), 2);
        assert_eq!(restored.missions[0].status, "ARCHIVED");
        assert_eq!(restored.missions[1].status, "DRAFT");
        let db = write(
            &p,
            Database {
                missions: vec![restored.missions[1].clone()],
                ..db
            },
            3,
        )
        .unwrap();
        assert_eq!(db.revision, 4);
        assert_eq!(read(&p).unwrap().missions.len(), 1);
        let _ = fs::remove_dir_all(p.parent().unwrap());
    }
    #[cfg(unix)]
    #[test]
    fn concurrent_writer_is_rejected() {
        use std::os::fd::AsRawFd;
        let p = path();
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        let f = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(p.parent().unwrap().join(".missions.lock"))
            .unwrap();
        assert_eq!(
            unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
            0
        );
        assert!(write(&p, Database::default(), 0).is_err());
        assert!(!p.exists());
        drop(f);
        assert!(write(&p, Database::default(), 0).is_ok());
        let _ = fs::remove_dir_all(p.parent().unwrap());
    }
    #[test]
    fn rejects_bad_geometry_and_unlocated_saved_mission() {
        let mut m = sample();
        m.geometry = Some(
            serde_json::json!({"type":"Polygon","coordinates":[[[127,37],[127,38],[128,38],[127,37]]]}),
        );
        assert!(validate(&Database {
            missions: vec![m.clone()],
            ..Default::default()
        })
        .is_ok());
        m.geometry = Some(
            serde_json::json!({"type":"Polygon","coordinates":[[[127,37],[127,38],[128,38],[128,37]]]}),
        );
        assert!(validate(&Database {
            missions: vec![m.clone()],
            ..Default::default()
        })
        .is_err());
        m.geometry = None;
        m.lon = None;
        m.lat = None;
        assert!(validate(&Database {
            missions: vec![m],
            ..Default::default()
        })
        .is_err());
    }
    #[test]
    fn rejects_invalid_location_and_radius() {
        let mut m = sample();
        m.lat = Some(91.0);
        assert!(validate(&Database {
            missions: vec![m.clone()],
            ..Default::default()
        })
        .is_err());
        m.lat = Some(37.0);
        m.radius = 0.0;
        assert!(validate(&Database {
            missions: vec![m],
            ..Default::default()
        })
        .is_err());
    }
}
