use serde_json::Value;
use std::time::Duration;

// Fixed destination and bounded parameters: map requests cannot access arbitrary hosts.
#[tauri::command]
pub async fn search_places(key: String, query: String, kind: String) -> Result<Value, String> {
    let query = query.trim();
    if key.is_empty()
        || key.len() > 256
        || query.chars().count() < 2
        || query.chars().count() > 100
        || !matches!(kind.as_str(), "road" | "parcel" | "place")
    {
        return Err("입력값을 확인하세요".into());
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "지도 네트워크 초기화 실패")?;
    let mut response = client
        .get("https://api.vworld.kr/req/search")
        .query(&[
            ("service", "search"),
            ("request", "search"),
            ("version", "2.0"),
            ("crs", "EPSG:4326"),
            ("size", "10"),
            ("page", "1"),
            ("query", query),
            ("type", &kind),
            ("format", "json"),
            ("errorformat", "json"),
            ("key", &key),
        ])
        .send()
        .await
        .map_err(|_| "네트워크 오류 또는 시간 초과")?;
    if !response.status().is_success() {
        return Err("지도 서버 HTTP 오류".into());
    }
    if response.content_length().is_some_and(|n| n > 1_000_000) {
        return Err("지도 서버 응답 크기 초과".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "지도 서버 응답 오류")? {
        if bytes.len() + chunk.len() > 1_000_000 {
            return Err("지도 서버 응답 크기 초과".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| "지도 서버 응답 형식 오류")?;
    Ok(value.get("response").cloned().unwrap_or(Value::Null))
}

#[tauri::command]
pub fn load_map_key(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;
    let path = app
        .path()
        .app_config_dir()
        .map_err(|_| "지도 설정 경로 오류")?
        .join("vworld-api-key");
    match std::fs::read_to_string(path) {
        Ok(s) if s.len() <= 256 => Ok(s.trim().to_owned()),
        Ok(_) => Err("지도 설정 오류".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(_) => Err("지도 설정 읽기 실패".into()),
    }
}

#[tauri::command]
pub fn save_map_key(app: tauri::AppHandle, key: String) -> Result<(), String> {
    use std::io::Write;
    use tauri::Manager;
    if key.len() > 256 || key.contains(['\n', '\r']) {
        return Err("지도 키 형식 오류".into());
    }
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|_| "지도 설정 경로 오류")?;
    std::fs::create_dir_all(&dir).map_err(|_| "지도 설정 저장 실패")?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(dir.join("vworld-api-key"))
        .map_err(|_| "지도 설정 저장 실패")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))
            .map_err(|_| "지도 설정 저장 실패")?;
    }
    file.write_all(key.trim().as_bytes())
        .map_err(|_| "지도 설정 저장 실패".into())
}

#[tauri::command]
pub async fn buildings_at(key: String, lon: f64, lat: f64) -> Result<Value, String> {
    if key.is_empty()
        || key.len() > 256
        || !lon.is_finite()
        || !lat.is_finite()
        || !(-180.0..=180.0).contains(&lon)
        || !(-85.0..=85.0).contains(&lat)
    {
        return Err("입력값 오류".into());
    }
    let filter = format!(
        "BOX({},{},{},{})",
        lon - 0.00015,
        lat - 0.00015,
        lon + 0.00015,
        lat + 0.00015
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "네트워크 초기화 오류")?;
    let mut response = client
        .get("https://api.vworld.kr/req/data")
        .query(&[
            ("service", "data"),
            ("request", "GetFeature"),
            ("version", "2.0"),
            ("data", "LT_C_SPBD"),
            ("key", &key),
            ("format", "json"),
            ("errorformat", "json"),
            ("geometry", "true"),
            ("attribute", "true"),
            ("size", "100"),
            ("page", "1"),
            ("crs", "EPSG:4326"),
            ("geomFilter", &filter),
        ])
        .send()
        .await
        .map_err(|_| "네트워크 오류 또는 시간 초과")?;
    if !response.status().is_success() {
        return Err("건물 서버 HTTP 오류".into());
    }
    if response.content_length().is_some_and(|n| n > 2_000_000) {
        return Err("건물 응답 크기 초과".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "건물 응답 오류")? {
        if bytes.len() + chunk.len() > 2_000_000 {
            return Err("건물 응답 크기 초과".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| "건물 응답 형식 오류")?;
    Ok(value.get("response").cloned().unwrap_or(Value::Null))
}
