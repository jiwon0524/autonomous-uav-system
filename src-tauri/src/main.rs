#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod connection;
mod safety;
mod telemetry;
fn main() {
    let builder = tauri::Builder::default().manage(connection::Manager::default());
    #[cfg(feature = "simulation")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        connection::list_serial_ports,
        connection::connect_serial,
        connection::auto_connect_serial,
        connection::set_auto_device,
        connection::connect_simulation,
        connection::disconnect_serial,
        connection::connection_status,
        connection::acknowledge_alert,
        connection::simulation_available
    ]);
    #[cfg(not(feature = "simulation"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        connection::list_serial_ports,
        connection::connect_serial,
        connection::auto_connect_serial,
        connection::set_auto_device,
        connection::disconnect_serial,
        connection::connection_status,
        connection::acknowledge_alert,
        connection::simulation_available
    ]);
    builder
        .run(tauri::generate_context!())
        .expect("failed to start FireScene GCS");
}
