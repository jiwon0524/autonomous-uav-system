use tauri::AppHandle;

// Enabling this experimental feature does not imply that WebView permissions or
// provider domain registration have been verified. It is off in normal builds.
#[tauri::command]
pub fn map3d_available() -> bool {
    cfg!(feature = "experimental-vworld-3d") && option_env!("FIRESCENE_3D_PAGE_URL").is_some()
}

#[tauri::command]
pub fn map3d_is_open(app: AppHandle) -> bool {
    use tauri::Manager;
    app.get_webview_window("map3d").is_some()
}

#[tauri::command]
pub fn open_map3d(app: AppHandle, lon: f64, lat: f64, radius: Option<f64>) -> Result<(), String> {
    if !lon.is_finite()
        || !lat.is_finite()
        || !(-180.0..=180.0).contains(&lon)
        || !(-85.0..=85.0).contains(&lat)
        || radius.is_some_and(|r| !r.is_finite() || !(1.0..=2000.0).contains(&r))
    {
        return Err("잘못된 지도 좌표 또는 범위".into());
    }
    open_window(app, lon, lat, radius)
}

#[cfg(not(feature = "experimental-vworld-3d"))]
fn open_window(_: AppHandle, _: f64, _: f64, _: Option<f64>) -> Result<(), String> {
    Err("3D 지도는 권한 및 도메인 검증 전까지 비활성화되어 있습니다".into())
}

#[cfg(feature = "experimental-vworld-3d")]
fn open_window(app: AppHandle, lon: f64, lat: f64, radius: Option<f64>) -> Result<(), String> {
    use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
    let configured = option_env!("FIRESCENE_3D_PAGE_URL")
        .ok_or("승인된 HTTPS 3D 페이지가 설정되지 않았습니다")?;
    let mut url = tauri::Url::parse(configured).map_err(|_| "3D 페이지 주소 오류")?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("3D 페이지는 인증 정보와 쿼리가 없는 HTTPS 주소여야 합니다".into());
    }
    if app.get_webview_window("map3d").is_some() {
        return Err("3D 창이 이미 열려 있습니다".into());
    }
    let base = url.clone();
    // The explicitly configured HTTPS host receives the same client key it would
    // receive from a user typing it into that page. It never gains file access.
    let key = crate::map::load_map_key(app.clone())?;
    if key.is_empty()
        || key.len() > 256
        || !key.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
    {
        return Err("VWorld API 키를 확인하세요".into());
    }
    let expected = serde_json::to_string(base.as_str()).map_err(|_| "3D 페이지 설정 오류")?;
    let encoded_key = serde_json::to_string(&key).map_err(|_| "지도 키 형식 오류")?;
    let initialize = format!("if (window.top === window && location.origin + location.pathname === {expected}) {{ try {{ if (!sessionStorage.getItem('firescene-key-initialized')) {{ sessionStorage.setItem('firescene-vworld-key', {encoded_key}); sessionStorage.setItem('firescene-key-initialized', '1'); }} }} catch (_) {{}} }}");
    url.set_fragment(Some(&format!(
        "lon={lon}&lat={lat}&radius={}",
        radius.unwrap_or(0.0)
    )));
    let map = WebviewWindowBuilder::new(&app, "map3d", WebviewUrl::External(url))
        .title("FireScene GCS · VWorld 3D · 실험")
        .inner_size(1200.0, 800.0)
        .initialization_script(&initialize)
        .on_navigation(move |url| {
            url.scheme() == base.scheme()
                && url.host_str() == base.host_str()
                && url.port_or_known_default() == base.port_or_known_default()
                && url.path() == base.path()
                && url.query().is_none()
        })
        .build()
        .map_err(|_| "3D 창 생성 실패")?;
    let main = app.clone();
    map.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
            let _ = main.emit_to("main", "map3d-closed", ());
        }
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn map3d_is_disabled_in_normal_builds() {
        if !cfg!(feature = "experimental-vworld-3d") {
            assert!(!map3d_available());
        }
    }
}
