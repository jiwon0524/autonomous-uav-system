#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod connection;
fn main() {
    tauri::Builder::default()
        .manage(connection::Manager::default())
        .invoke_handler(tauri::generate_handler![
            connection::list_serial_ports,
            connection::connect_serial,
            connection::disconnect_serial,
            connection::connection_status
        ])
        .run(tauri::generate_context!())
        .expect("failed to start FireScene GCS");
}
