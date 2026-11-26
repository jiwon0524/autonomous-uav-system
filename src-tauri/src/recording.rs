use crate::connection::Manager;
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::Manager as TauriManager;
const MAX_BYTES: u64 = 50 * 1024 * 1024;
const MAX_LINE: usize = 512 * 1024;
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
#[derive(Clone, Default, Serialize)]
pub struct Status {
    active: bool,
    id: Option<String>,
    started_ms: Option<u64>,
    bytes: u64,
    error: Option<String>,
}
struct Worker {
    stop: mpsc::Sender<()>,
    join: JoinHandle<()>,
}
#[derive(Default)]
pub struct Recorder {
    worker: Mutex<Option<Worker>>,
    status: Arc<Mutex<Status>>,
}
impl Recorder {
    fn request_stop(&self) -> Result<(), String> {
        let slot = self.worker.lock().map_err(|_| "기록 상태 오류")?;
        if let Some(worker) = slot.as_ref() {
            let _ = worker.stop.send(());
        }
        Ok(())
    }
    fn stop(&self) {
        if let Ok(mut slot) = self.worker.lock() {
            if let Some(worker) = slot.take() {
                let _ = worker.stop.send(());
                let _ = worker.join.join();
            }
        }
    }
}
impl Drop for Recorder {
    fn drop(&mut self) {
        self.stop();
    }
}
fn directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|_| "기록 경로 오류")?
        .join("recordings"))
}
fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 80 && id.bytes().all(|b| b.is_ascii_digit() || b == b'-')
}
fn path(dir: &Path, id: &str) -> Result<PathBuf, String> {
    if !valid_id(id) {
        return Err("기록 ID 오류".into());
    }
    Ok(dir.join(format!("{id}.jsonl")))
}
fn append(f: &mut File, value: &Value, bytes: &mut u64) -> Result<(), String> {
    let mut line = serde_json::to_vec(value).map_err(|_| "기록 데이터 변환 오류")?;
    line.push(b'\n');
    if line.len() > MAX_LINE || *bytes + line.len() as u64 > MAX_BYTES {
        return Err("기록 용량 제한 도달".into());
    }
    f.write_all(&line)
        .and_then(|_| f.flush())
        .map_err(|_| "기록 저장 실패 · 디스크 공간 및 권한 확인")?;
    *bytes += line.len() as u64;
    Ok(())
}
fn snapshot(
    handle: &std::sync::Arc<std::sync::Mutex<crate::connection::Snapshot>>,
) -> Result<Value, String> {
    Manager::recording_snapshot(handle)
}
#[tauri::command]
pub async fn start_recording(
    app: tauri::AppHandle,
    manager: tauri::State<'_, Manager>,
    recorder: tauri::State<'_, Arc<Recorder>>,
    name: String,
    mission_id: Option<String>,
) -> Result<(), String> {
    let handle = manager.recording_handle();
    let recorder = recorder.inner().clone();
    let dir = directory(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let initial = snapshot(&handle)?;
        begin_recording(&recorder, handle, initial, dir, name, mission_id)
    })
    .await
    .map_err(|_| "기록 시작 작업 실패")?
}
fn begin_recording(
    recorder: &Recorder,
    handle: Arc<Mutex<crate::connection::Snapshot>>,
    initial: Value,
    dir: PathBuf,
    name: String,
    mission_id: Option<String>,
) -> Result<(), String> {
    if name.trim().is_empty()
        || name.len() > 240
        || mission_id.as_ref().is_some_and(|s| {
            s.len() > 80 || !s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
    {
        return Err("기록 이름 또는 임무 ID 오류".into());
    }
    let mut slot = recorder.worker.lock().map_err(|_| "기록 상태 오류")?;
    if slot.as_ref().is_some_and(|w| !w.join.is_finished()) {
        return Err("이미 기록 중입니다".into());
    }
    if let Some(w) = slot.take() {
        let _ = w.join.join();
    }
    if initial["phase"] != "CONNECTED" || initial["vehicle"].is_null() {
        return Err("유효한 HEARTBEAT 연결을 확인한 후 기록을 시작하세요".into());
    }
    let session = initial["safety"]["session"]
        .as_u64()
        .ok_or("연결 세션 확인 실패")?;
    if session == 0 {
        return Err("연결 세션 확인 실패".into());
    }
    fs::create_dir_all(&dir).map_err(|_| "기록 폴더 생성 실패")?;
    let existing = fs::read_dir(&dir)
        .map_err(|_| "기록 목록 읽기 실패")?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|x| x == "jsonl"))
        .take(200)
        .count();
    if existing >= 200 {
        return Err("기록 개수 제한 도달 · 기존 기록을 정리하세요".into());
    }
    let started = now();
    let id = format!("{}-{}", started, std::process::id());
    let p = path(&dir, &id)?;
    let mut opt = OpenOptions::new();
    opt.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opt.mode(0o600);
    }
    let mut file = opt
        .open(p)
        .map_err(|_| "기록 파일 생성 실패 · 디스크 공간 및 권한 확인")?;
    let mut count = 0;
    let header = json!({"type":"header","version":1,"id":id,"name":name,"mission_id":mission_id,"started_ms":started,"source":initial["source"],"connection_session":session,"vehicle":initial["vehicle"]});
    append(&mut file, &header, &mut count)?;
    let (tx, rx) = mpsc::channel();
    let state = recorder.status.clone();
    let record_id = id.clone();
    *state.lock().map_err(|_| "기록 상태 오류")? = Status {
        active: true,
        id: Some(id),
        started_ms: Some(started),
        bytes: count,
        error: None,
    };
    let error_state = recorder.status.clone();
    let join = thread::Builder::new().name("firescene-recorder".into()).spawn(move || {
        let start = Instant::now();
        let mut events = HashMap::<String, String>::new();
        let mut previous_phase: Option<String> = None;
        let mut latest = Some(initial);
        let mut failure = None;
        loop {
            let mut data = match latest.take() {
                Some(s) => s,
                None => match snapshot(&handle) {
                    Ok(s) => s,
                    Err(e) => {
                        failure = Some(e);
                        break;
                    }
                },
            };
            let current_session = data["safety"]["session"].as_u64().unwrap_or(0);
            if current_session != 0 && current_session != session {
                failure = Some("새 연결 세션이 감지되어 기록을 종료했습니다".into());
                break;
            }
            let elapsed = start.elapsed().as_millis() as u64;
            let phase = data["phase"].as_str().unwrap_or("UNKNOWN").to_owned();
            let warnings = data["safety"]["events"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            data["safety"]["events"] = json!([]);
            if let Err(e) = append(
                &mut file,
                &json!({"type":"sample","elapsed_ms":elapsed,"time_ms":now(),"snapshot":data}),
                &mut count,
            ) {
                failure = Some(e);
                break;
            }
            if previous_phase.as_deref() != Some(&phase) {
                if let Err(e) = append(
                    &mut file,
                    &json!({"type":"connection","elapsed_ms":elapsed,"phase":phase,"time_ms":now()}),
                    &mut count,
                ) {
                    failure = Some(e);
                    break;
                }
                previous_phase = Some(phase);
            }
            for event in warnings {
                let occurred = event["occurred_ms"].as_u64().unwrap_or(started);
                if occurred < started
                    && event["resolved_ms"]
                        .as_u64()
                        .is_some_and(|end| end < started)
                {
                    continue;
                }
                let preexisting = occurred < started;
                let key = format!("{}:{}", event["session"], event["id"]);
                let serial = event.to_string();
                if events.get(&key) != Some(&serial) {
                    if let Err(e) = append(
                        &mut file,
                        &json!({"type":"event","elapsed_ms":elapsed,"preexisting":preexisting,"event":event}),
                        &mut count,
                    ) {
                        failure = Some(e);
                        break;
                    }
                    events.insert(key, serial);
                }
            }
            if let Ok(mut s) = state.lock() {
                s.bytes = count;
            }
            if failure.is_some() {
                break;
            }
            if elapsed / 1000 % 5 == 0 && file.sync_data().is_err() {
                failure = Some("기록 동기화 실패 · 디스크 공간 및 권한 확인".into());
                break;
            }
            if rx.recv_timeout(Duration::from_secs(1)).is_ok() {
                break;
            }
        }
        if failure.is_none() {
            if let Err(e) = append(
                &mut file,
                &json!({"type":"end","elapsed_ms":start.elapsed().as_millis() as u64,"ended_ms":now()}),
                &mut count,
            ) {
                failure = Some(e);
            }
        }
        if file.sync_all().is_err() && failure.is_none() {
            failure = Some("기록 최종 동기화 실패 · 디스크 공간 및 권한 확인".into());
        }
        if let Ok(mut s) = state.lock() {
            if s.id.as_deref() == Some(&record_id) {
                s.active = false;
                s.bytes = count;
                s.error = failure;
            }
        }
    }).map_err(|_| { if let Ok(mut status)=error_state.lock() { status.active=false; status.error=Some("기록 작업 스레드를 시작하지 못했습니다".into()); } "기록 작업 스레드를 시작하지 못했습니다".to_string() })?;
    *slot = Some(Worker { stop: tx, join });
    Ok(())
}
#[tauri::command]
pub fn stop_recording(recorder: tauri::State<'_, Arc<Recorder>>) -> Result<(), String> {
    recorder.request_stop()
}
#[tauri::command]
pub fn recording_status(recorder: tauri::State<'_, Arc<Recorder>>) -> Result<Status, String> {
    let finished = recorder
        .worker
        .lock()
        .map_err(|_| "기록 상태 오류")?
        .as_ref()
        .is_some_and(|w| w.join.is_finished());
    let mut status = recorder.status.lock().map_err(|_| "기록 상태 오류")?;
    if finished && status.active {
        status.active = false;
        status.error = Some("기록 작업이 예기치 않게 종료되었습니다".into());
    }
    Ok(status.clone())
}
#[derive(Serialize)]
pub struct Summary {
    id: String,
    name: String,
    mission_id: Option<String>,
    connection_session: u64,
    system_id: Option<u64>,
    component_id: Option<u64>,
    started_ms: u64,
    ended_ms: Option<u64>,
    duration_ms: u64,
    source: String,
    warnings: usize,
    bytes: u64,
    incomplete: bool,
    damaged: bool,
}
fn scan(
    p: &Path,
    from: u64,
    to: u64,
    max: usize,
    events_only: bool,
) -> Result<(Summary, Vec<Value>), String> {
    let meta = fs::metadata(p).map_err(|_| "기록 파일을 읽을 수 없습니다")?;
    if meta.len() > MAX_BYTES {
        return Err("기록 파일 크기 초과".into());
    }
    let file = File::open(p).map_err(|_| "기록 파일을 읽을 수 없습니다")?;
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();
    let mut header = None;
    let mut result = Vec::new();
    let mut end = None;
    let mut duration = 0;
    let mut events = HashMap::new();
    let mut incomplete = false;
    loop {
        buf.clear();
        let n = (&mut reader)
            .take((MAX_LINE + 1) as u64)
            .read_until(b'\n', &mut buf)
            .map_err(|_| "기록 읽기 실패")?;
        if n == 0 {
            break;
        }
        if buf.len() > MAX_LINE {
            return Err("기록 데이터 크기 오류".into());
        }
        if !buf.ends_with(b"\n") {
            incomplete = true;
            break;
        }
        let v: Value = serde_json::from_slice(&buf).map_err(|_| "기록 파일이 손상되었습니다")?;
        if end.is_some() {
            return Err("종료 후 기록이 있어 파일이 손상되었습니다".into());
        }
        match v["type"].as_str() {
            Some("header") if header.is_none() => {
                if v["version"] != 1 {
                    return Err("지원하지 않는 기록 버전".into());
                }
                header = Some(v);
            }
            Some("sample") | Some("event") | Some("connection") if header.is_some() => {
                let elapsed = v["elapsed_ms"].as_u64().ok_or("기록 시각 오류")?;
                if elapsed < duration {
                    return Err("기록 순서가 올바르지 않습니다".into());
                }
                duration = elapsed;
                if v["type"] == "event" {
                    let e = &v["event"];
                    let k = format!("{}:{}", e["session"], e["id"]);
                    events.insert(
                        k,
                        (
                            e["severity"].as_str().unwrap_or("").to_owned(),
                            v["preexisting"].as_bool().unwrap_or(false),
                        ),
                    );
                }
                if elapsed >= from
                    && elapsed <= to
                    && result.len() < max
                    && (!events_only || v["type"] == "event" || v["type"] == "connection")
                {
                    result.push(v);
                }
            }
            Some("end") if header.is_some() => {
                let last = v["elapsed_ms"].as_u64().ok_or("기록 종료 시각 오류")?;
                if last < duration {
                    return Err("기록 순서가 올바르지 않습니다".into());
                }
                duration = last;
                end = Some(v["ended_ms"].as_u64().ok_or("기록 종료 시각 오류")?);
            }
            _ => return Err("기록 형식 오류".into()),
        }
    }
    let h = header.ok_or("기록 헤더가 없습니다")?;
    let id = h["id"].as_str().ok_or("기록 ID 오류")?.to_owned();
    if !valid_id(&id) || p.file_name().and_then(|n| n.to_str()) != Some(&format!("{id}.jsonl")) {
        return Err("기록 식별자 오류".into());
    }
    let name = h["name"].as_str().ok_or("기록 이름 오류")?;
    if name.trim().is_empty() || name.len() > 240 {
        return Err("기록 이름 오류".into());
    }
    let source = h["source"].as_str().unwrap_or("UNKNOWN");
    if source.len() > 64 {
        return Err("기록 출처 오류".into());
    }
    let summary = Summary {
        id,
        name: name.to_owned(),
        mission_id: h["mission_id"].as_str().map(str::to_owned),
        connection_session: h["connection_session"].as_u64().unwrap_or(0),
        system_id: h["vehicle"]["system_id"].as_u64(),
        component_id: h["vehicle"]["component_id"].as_u64(),
        started_ms: h["started_ms"].as_u64().unwrap_or(0),
        ended_ms: end,
        duration_ms: duration,
        source: source.to_owned(),
        warnings: events
            .values()
            .filter(|(severity, preexisting)| {
                !preexisting && matches!(severity.as_str(), "WARNING" | "CRITICAL")
            })
            .count(),
        bytes: meta.len(),
        incomplete: incomplete || end.is_none(),
        damaged: false,
    };
    Ok((summary, result))
}
fn list_recordings_sync(app: tauri::AppHandle) -> Result<Vec<Summary>, String> {
    let dir = directory(&app)?;
    let entries = match fs::read_dir(&dir) {
        Ok(x) => x,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(_) => return Err("기록 목록 읽기 실패".into()),
    };
    let mut out = Vec::new();
    for entry in entries {
        let p = entry.map_err(|_| "기록 목록 읽기 실패")?.path();
        if p.extension().is_some_and(|x| x == "jsonl") {
            if out.len() >= 200 {
                return Err("기록 개수 제한 초과 · 기록을 정리하세요".into());
            }
            match scan(&p, 0, 0, 0, false) {
                Ok((mut s, _)) => {
                    if let Ok(file) = File::open(dir.join(format!("{}.name", s.id))) {
                        let mut name = String::new();
                        if file.take(241).read_to_string(&mut name).is_ok()
                            && name.len() <= 240
                            && !name.trim().is_empty()
                        {
                            s.name = name;
                        }
                    }
                    out.push(s)
                }
                Err(_) => {
                    if let Some(id) = p
                        .file_stem()
                        .and_then(|x| x.to_str())
                        .filter(|id| valid_id(id))
                    {
                        out.push(Summary {
                            id: id.to_owned(),
                            name: "손상된 기록".into(),
                            mission_id: None,
                            connection_session: 0,
                            system_id: None,
                            component_id: None,
                            started_ms: 0,
                            ended_ms: None,
                            duration_ms: 0,
                            source: "UNKNOWN".into(),
                            warnings: 0,
                            bytes: fs::metadata(&p).map(|m| m.len()).unwrap_or(0),
                            incomplete: true,
                            damaged: true,
                        });
                    }
                }
            }
        }
    }
    out.sort_by_key(|s| std::cmp::Reverse(s.started_ms));
    Ok(out)
}
#[derive(Serialize)]
pub struct Page {
    summary: Summary,
    records: Vec<Value>,
    truncated: bool,
}
fn read_recording_sync(
    app: tauri::AppHandle,
    id: String,
    from: u64,
    to: u64,
) -> Result<Page, String> {
    if to < from || to - from > 600_000 {
        return Err("조회 범위 오류".into());
    }
    let (summary, records) = scan(&path(&directory(&app)?, &id)?, from, to, 1600, false)?;
    let truncated = records.len() >= 1600;
    Ok(Page {
        summary,
        records,
        truncated,
    })
}
#[derive(Serialize)]
pub struct WindowPage {
    records: Vec<Value>,
    truncated: bool,
}
fn read_window_path(
    p: &Path,
    id: &str,
    from: u64,
    to: u64,
    max: usize,
) -> Result<WindowPage, String> {
    if to < from || to - from > 600_000 {
        return Err("조회 범위 오류".into());
    }
    if fs::metadata(p)
        .map_err(|_| "기록 파일을 읽을 수 없습니다")?
        .len()
        > MAX_BYTES
    {
        return Err("기록 파일 크기 초과".into());
    }
    let mut reader = BufReader::new(File::open(p).map_err(|_| "기록 파일을 읽을 수 없습니다")?);
    let mut line = Vec::new();
    let mut header = false;
    let mut previous = 0;
    let mut records = Vec::new();
    let mut truncated = false;
    loop {
        line.clear();
        let n = (&mut reader)
            .take((MAX_LINE + 1) as u64)
            .read_until(b'\n', &mut line)
            .map_err(|_| "기록 읽기 실패")?;
        if n == 0 {
            break;
        }
        if line.len() > MAX_LINE {
            return Err("기록 데이터 크기 오류".into());
        }
        if !line.ends_with(b"\n") {
            break;
        }
        let value: Value =
            serde_json::from_slice(&line).map_err(|_| "기록 파일이 손상되었습니다")?;
        match value["type"].as_str() {
            Some("header") if !header => {
                let name = value["name"].as_str().ok_or("기록 이름 오류")?;
                let source = value["source"].as_str().unwrap_or("UNKNOWN");
                if value["version"] != 1
                    || value["id"].as_str() != Some(id)
                    || !valid_id(id)
                    || name.trim().is_empty()
                    || name.len() > 240
                    || source.len() > 64
                {
                    return Err("기록 헤더 오류".into());
                }
                header = true;
            }
            Some("sample" | "event" | "connection") if header => {
                let time = value["elapsed_ms"].as_u64().ok_or("기록 시각 오류")?;
                if time < previous {
                    return Err("기록 순서가 올바르지 않습니다".into());
                }
                previous = time;
                if time > to {
                    break;
                }
                if time >= from {
                    if records.len() >= max {
                        truncated = true;
                        break;
                    }
                    records.push(value);
                }
            }
            Some("end") if header => {
                let time = value["elapsed_ms"].as_u64().ok_or("기록 종료 시각 오류")?;
                if time < previous || value["ended_ms"].as_u64().is_none() {
                    return Err("기록 종료 시각 오류".into());
                }
                break;
            }
            _ => return Err("기록 형식 오류".into()),
        }
    }
    if !header {
        return Err("기록 헤더가 없습니다".into());
    }
    Ok(WindowPage { records, truncated })
}
#[tauri::command]
pub async fn read_recording_window(
    app: tauri::AppHandle,
    id: String,
    from: u64,
    to: u64,
) -> Result<WindowPage, String> {
    #[cfg(all(feature = "simulation", debug_assertions))]
    {
        use tauri::Manager as _;
        app.state::<crate::connection::Manager>().require_replay()?;
    }
    let file = path(&directory(&app)?, &id)?;
    tauri::async_runtime::spawn_blocking(move || read_window_path(&file, &id, from, to, 1600))
        .await
        .map_err(|_| "기록 조회 작업 실패")?
}
fn recording_events_sync(app: tauri::AppHandle, id: String) -> Result<Vec<Value>, String> {
    let (_, records) = scan(&path(&directory(&app)?, &id)?, 0, u64::MAX, 1000, true)?;
    Ok(records)
}
#[tauri::command]
pub async fn list_recordings(app: tauri::AppHandle) -> Result<Vec<Summary>, String> {
    tauri::async_runtime::spawn_blocking(move || list_recordings_sync(app))
        .await
        .map_err(|_| "기록 목록 작업 실패")?
}
#[tauri::command]
pub async fn read_recording(
    app: tauri::AppHandle,
    id: String,
    from: u64,
    to: u64,
) -> Result<Page, String> {
    #[cfg(all(feature = "simulation", debug_assertions))]
    {
        use tauri::Manager as _;
        app.state::<crate::connection::Manager>().require_replay()?;
    }
    tauri::async_runtime::spawn_blocking(move || read_recording_sync(app, id, from, to))
        .await
        .map_err(|_| "기록 조회 작업 실패")?
}
#[tauri::command]
pub async fn recording_events(app: tauri::AppHandle, id: String) -> Result<Vec<Value>, String> {
    #[cfg(all(feature = "simulation", debug_assertions))]
    {
        use tauri::Manager as _;
        app.state::<crate::connection::Manager>().require_replay()?;
    }
    tauri::async_runtime::spawn_blocking(move || recording_events_sync(app, id))
        .await
        .map_err(|_| "이벤트 조회 작업 실패")?
}
#[tauri::command]
pub fn rename_recording(app: tauri::AppHandle, id: String, name: String) -> Result<(), String> {
    let dir = directory(&app)?;
    let p = path(&dir, &id)?;
    if !p.is_file() || name.trim().is_empty() || name.len() > 240 || name.contains(['\r', '\n']) {
        return Err("기록 이름 오류".into());
    }
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(dir.join(format!("{id}.name")))
        .and_then(|mut f| f.write_all(name.trim().as_bytes()))
        .map_err(|_| "기록 이름 저장 실패".into())
}
#[tauri::command]
pub fn delete_recording(
    app: tauri::AppHandle,
    recorder: tauri::State<'_, Arc<Recorder>>,
    id: String,
) -> Result<(), String> {
    {
        let status = recorder.status.lock().map_err(|_| "기록 상태 오류")?;
        if status.active && status.id.as_deref() == Some(&id) {
            return Err("기록 중에는 삭제할 수 없습니다".into());
        }
    }
    delete_file(&directory(&app)?, &id)
}
fn delete_file(dir: &Path, id: &str) -> Result<(), String> {
    fs::remove_file(path(dir, id)?).map_err(|_| "기록 삭제 실패")?;
    let _ = fs::remove_file(dir.join(format!("{id}.name")));
    Ok(())
}
#[tauri::command]
pub async fn export_recording(
    app: tauri::AppHandle,
    recorder: tauri::State<'_, Arc<Recorder>>,
    id: String,
) -> Result<String, String> {
    {
        let status = recorder.status.lock().map_err(|_| "기록 상태 오류")?;
        if status.active && status.id.as_deref() == Some(&id) {
            return Err("기록 종료 후 내보내세요".into());
        }
    }
    let dir = directory(&app)?;
    let exports = app
        .path()
        .app_data_dir()
        .map_err(|_| "내보내기 경로 오류")?
        .join("recording-exports");
    tauri::async_runtime::spawn_blocking(move || export_file(&dir, &id, &exports))
        .await
        .map_err(|_| "내보내기 작업 실패")?
}
fn export_file(dir: &Path, id: &str, exports: &Path) -> Result<String, String> {
    let source = path(dir, id)?;
    if fs::metadata(&source).map_err(|_| "기록 읽기 실패")?.len() > MAX_BYTES {
        return Err("기록 파일 크기 초과".into());
    }
    fs::create_dir_all(exports).map_err(|_| "내보내기 폴더 생성 실패")?;
    let dest = exports.join(format!("{id}-{}.jsonl", now()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut out = options.open(&dest).map_err(|_| "내보내기 파일 생성 실패")?;
    let result = (|| -> std::io::Result<()> {
        std::io::copy(&mut File::open(source)?, &mut out)?;
        out.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        drop(out);
        let _ = fs::remove_file(&dest);
        return Err("내보내기 실패 · 디스크 공간 및 권한 확인".into());
    }
    Ok(dest.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn temporary(id: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!(
                "firescene-recording-{}-{}",
                std::process::id(),
                std::thread::current()
                    .name()
                    .unwrap_or("test")
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                    .collect::<String>()
            ))
            .join(format!("{id}.jsonl"))
    }
    #[test]
    fn export_preserves_raw_bytes_and_delete_only_target() {
        let p = temporary("456-3");
        let dir = p.parent().unwrap();
        fs::create_dir_all(dir).unwrap();
        fs::write(&p, b"partial\n{broken").unwrap();
        let exports = dir.join("export");
        let output = export_file(dir, "456-3", &exports).unwrap();
        assert_eq!(fs::read(output).unwrap(), b"partial\n{broken");
        assert!(delete_file(dir, "../escape").is_err());
        delete_file(dir, "456-3").unwrap();
        assert!(!p.exists());
        assert!(exports.exists());
        let _ = fs::remove_dir_all(dir);
    }
    #[test]
    fn recorder_thread_writes_and_finalizes_without_controlling_connection() {
        let p = temporary("777-1");
        let dir = p.parent().unwrap().to_path_buf();
        let manager = Manager::default();
        let recorder = Recorder::default();
        let initial = json!({"phase":"CONNECTED","source":"SIMULATION","vehicle":{"system_id":42,"component_id":1},"safety":{"session":1,"events":[{"id":4,"session":1,"severity":"WARNING"},{"id":5,"session":1,"severity":"WARNING","occurred_ms":1,"resolved_ms":null},{"id":6,"session":1,"severity":"WARNING","occurred_ms":1,"resolved_ms":2}]},"telemetry":{"fields":{"battery":{"value":42,"status":"LIVE"}}}});
        begin_recording(
            &recorder,
            manager.recording_handle(),
            initial,
            dir.clone(),
            "test".into(),
            Some("mission-1".into()),
        )
        .unwrap();
        assert!(begin_recording(
            &recorder,
            manager.recording_handle(),
            json!({"phase":"CONNECTED","vehicle":{},"safety":{"session":1}}),
            dir.clone(),
            "duplicate".into(),
            None
        )
        .is_err());
        std::thread::sleep(Duration::from_millis(1100));
        recorder.request_stop().unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while recorder.status.lock().unwrap().active && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        recorder.stop();
        let status = recorder.status.lock().unwrap().clone();
        assert!(!status.active);
        assert!(status.error.is_none());
        println!("synthetic two-sample recording: {} bytes", status.bytes);
        let path = path(&dir, status.id.as_deref().unwrap()).unwrap();
        let (summary, records) = scan(&path, 0, 10000, 100, false).unwrap();
        assert!(!summary.incomplete);
        assert_eq!(summary.mission_id.as_deref(), Some("mission-1"));
        let missions = dir.join("missions.json");
        fs::write(&missions, b"synthetic").unwrap();
        fs::remove_file(missions).unwrap();
        let (after_deletion, _) = scan(&path, 0, 10000, 100, false).unwrap();
        assert_eq!(after_deletion.mission_id.as_deref(), Some("mission-1"));
        assert_eq!(summary.warnings, 1);
        let safety_events: Vec<_> = records.iter().filter(|v| v["type"] == "event").collect();
        assert_eq!(safety_events.len(), 2);
        assert!(safety_events.iter().any(|e| e["preexisting"] == true));
        let samples: Vec<_> = records.iter().filter(|v| v["type"] == "sample").collect();
        assert!(samples.len() >= 2);
        assert_eq!(
            samples[0]["snapshot"]["telemetry"]["fields"]["battery"]["value"],
            42
        );
        assert_eq!(samples[1]["snapshot"]["phase"], "DISCONNECTED");
        let transitions: Vec<_> = records
            .iter()
            .filter(|v| v["type"] == "connection")
            .collect();
        assert_eq!(transitions[0]["phase"], "CONNECTED");
        assert_eq!(transitions[1]["phase"], "DISCONNECTED");
        assert_eq!(
            serde_json::to_value(manager.snapshot()).unwrap()["phase"],
            "DISCONNECTED"
        );
        drop(recorder);
        let _ = fs::remove_dir_all(dir);
    }
    #[test]
    fn no_recording_without_connected_identified_session() {
        let p = temporary("778-1");
        let dir = p.parent().unwrap().to_path_buf();
        let manager = Manager::default();
        let recorder = Recorder::default();
        assert!(begin_recording(
            &recorder,
            manager.recording_handle(),
            json!({"phase":"DISCONNECTED","vehicle":null}),
            dir.clone(),
            "test".into(),
            None
        )
        .is_err());
        assert!(begin_recording(
            &recorder,
            manager.recording_handle(),
            json!({"phase":"CONNECTED","vehicle":{},"safety":{"session":0}}),
            dir.clone(),
            "test".into(),
            None
        )
        .is_err());
        assert!(!dir.exists());
    }
    #[test]
    #[ignore = "synthetic size and scan benchmark; not a performance guarantee"]
    fn synthetic_long_recording_scan() {
        let p = temporary("910-1");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        let mut file = File::create(&p).unwrap();
        let mut bytes = 0;
        append(&mut file,&json!({"type":"header","version":1,"id":"910-1","name":"synthetic","started_ms":0,"source":"SIMULATION"}),&mut bytes).unwrap();
        let sample = json!({"phase":"CONNECTED","telemetry":{"fields":{"battery":{"value":75,"status":"LIVE"},"gps_lat":{"value":37.5,"status":"LIVE"},"gps_lon":{"value":127.0,"status":"LIVE"},"relative_alt":{"value":30,"status":"LIVE"},"ground_speed":{"value":4,"status":"LIVE"},"roll":{"value":0,"status":"LIVE"},"pitch":{"value":0,"status":"LIVE"},"yaw":{"value":0,"status":"LIVE"}}}});
        for i in 0..36_000u64 {
            append(
                &mut file,
                &json!({"type":"sample","elapsed_ms":i*1000,"snapshot":sample}),
                &mut bytes,
            )
            .unwrap();
        }
        append(
            &mut file,
            &json!({"type":"end","elapsed_ms":36_000_000u64,"ended_ms":36_000_000u64}),
            &mut bytes,
        )
        .unwrap();
        drop(file);
        let started = Instant::now();
        let (_, page) = scan(&p, 18_000_000, 18_599_999, 1600, false).unwrap();
        let elapsed = started.elapsed();
        assert_eq!(page.len(), 600);
        println!(
            "synthetic 10-hour file: {bytes} bytes; 10-minute range full scan: {} ms; {} samples",
            elapsed.as_millis(),
            page.len()
        );
        let started = Instant::now();
        let fast = read_window_path(&p, "910-1", 18_000_000, 18_599_999, 1600).unwrap();
        let fast_elapsed = started.elapsed();
        assert_eq!(fast.records, page);
        println!(
            "synthetic 10-minute prefix scan: {} ms; {} samples",
            fast_elapsed.as_millis(),
            fast.records.len()
        );
        let _ = fs::remove_dir_all(p.parent().unwrap());
    }
    #[test]
    fn ordered_replay_and_interrupted_tail() {
        let p = temporary("123-7");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        let mut f = File::create(&p).unwrap();
        let mut count = 0;
        append(&mut f,&json!({"type":"header","version":1,"id":"123-7","name":"test","started_ms":123,"source":"SIMULATION"}),&mut count).unwrap();
        append(
            &mut f,
            &json!({"type":"sample","elapsed_ms":0,"snapshot":{"phase":"CONNECTED"}}),
            &mut count,
        )
        .unwrap();
        append(&mut f,&json!({"type":"event","elapsed_ms":1000,"event":{"id":1,"session":1,"severity":"WARNING"}}),&mut count).unwrap();
        append(
            &mut f,
            &json!({"type":"sample","elapsed_ms":2000,"snapshot":{"phase":"CONNECTION_LOST"}}),
            &mut count,
        )
        .unwrap();
        f.write_all(b"{\"type\":").unwrap();
        drop(f);
        let (summary, records) = scan(&p, 1000, 2000, 20, false).unwrap();
        assert!(summary.incomplete);
        assert_eq!(summary.warnings, 1);
        assert_eq!(records.len(), 2);
        assert_eq!(records[1]["snapshot"]["phase"], "CONNECTION_LOST");
        let window = read_window_path(&p, "123-7", 1000, 2000, 20).unwrap();
        assert_eq!(window.records, records);
        assert!(!window.truncated);
        let limited = read_window_path(&p, "123-7", 0, 2000, 1).unwrap();
        assert_eq!(limited.records.len(), 1);
        assert!(limited.truncated);
        assert!(read_window_path(&p, "wrong", 0, 2000, 1).is_err());
        let (_, events) = scan(&p, 0, u64::MAX, 10, true).unwrap();
        assert_eq!(events.len(), 1);
        let _ = fs::remove_dir_all(p.parent().unwrap());
    }
    #[test]
    fn window_stops_after_requested_range_and_full_scan_still_detects_later_corruption() {
        let p = temporary("987-2");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        let mut f = File::create(&p).unwrap();
        let mut bytes = 0;
        append(&mut f,&json!({"type":"header","version":1,"id":"987-2","name":"test","started_ms":1,"source":"SIMULATION"}),&mut bytes).unwrap();
        append(
            &mut f,
            &json!({"type":"sample","elapsed_ms":0,"snapshot":{}}),
            &mut bytes,
        )
        .unwrap();
        append(
            &mut f,
            &json!({"type":"sample","elapsed_ms":2000,"snapshot":{}}),
            &mut bytes,
        )
        .unwrap();
        f.write_all(b"not-json\n").unwrap();
        drop(f);
        let page = read_window_path(&p, "987-2", 0, 1000, 10).unwrap();
        assert_eq!(page.records.len(), 1);
        assert!(!page.truncated);
        assert!(scan(&p, 0, 1000, 10, false).is_err());
        let _ = fs::remove_dir_all(p.parent().unwrap());
    }
    #[test]
    fn rejects_corrupt_complete_line_and_invalid_path() {
        let p = temporary("234-8");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, b"not json\n").unwrap();
        assert!(scan(&p, 0, 10, 10, false).is_err());
        assert!(path(p.parent().unwrap(), "../secret").is_err());
        let _ = fs::remove_dir_all(p.parent().unwrap());
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn disk_write_failure_is_reported() {
        let mut f = OpenOptions::new().write(true).open("/dev/full").unwrap();
        let mut n = 0;
        assert!(append(&mut f, &json!({"type":"sample"}), &mut n).is_err());
        assert_eq!(n, 0);
    }
    #[test]
    fn write_limit_does_not_append() {
        let p = temporary("345-9");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        let mut f = File::create(&p).unwrap();
        let mut n = MAX_BYTES;
        assert!(append(&mut f, &json!({"type":"test"}), &mut n).is_err());
        assert_eq!(fs::metadata(&p).unwrap().len(), 0);
        let _ = fs::remove_dir_all(p.parent().unwrap());
    }
}
