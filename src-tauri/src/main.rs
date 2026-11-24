#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod connection;
mod telemetry;
fn main() {
    tauri::Builder::default()
        .manage(connection::Manager::default())
        .invoke_handler(tauri::generate_handler![
            connection::list_serial_ports,
            connection::connect_serial,
            connection::auto_connect_serial,
            connection::set_auto_device,
            connection::connect_simulation,
            connection::disconnect_serial,
            connection::connection_status
        ])
        .run(tauri::generate_context!())
        .expect("failed to start FireScene GCS");
}
