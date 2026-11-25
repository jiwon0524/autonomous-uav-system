use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[tauri::command]
pub fn find_qgroundcontrol() -> Option<String> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    [
        home.join("Downloads/QGroundControl.AppImage"),
        home.join("Downloads/GroundControl.AppImage"),
        home.join("GroundControl.AppImage"),
    ]
    .into_iter()
    .find(|p| p.is_file())
    .map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn launch_qgroundcontrol(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.is_absolute() || !p.is_file() {
        return Err("실행 파일을 찾을 수 없습니다. 절대 경로를 확인하세요.".into());
    }
    let name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !matches!(
        name.as_str(),
        "qgroundcontrol.appimage"
            | "groundcontrol.appimage"
            | "qgroundcontrol"
            | "qgroundcontrol.exe"
    ) {
        return Err("QGroundControl 실행 파일 이름을 확인하세요.".into());
    }
    Command::new(p)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("QGroundControl 실행 실패: {e}. 실행 권한과 설치 상태를 확인하세요."))
}
