#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod companion;
mod connection;
mod map;
mod map3d;
mod missions;
mod recording;
mod safety;
mod telemetry;
fn main() {
    let builder = tauri::Builder::default()
        .manage(connection::Manager::default())
        .manage(companion::Manager::default())
        .manage(missions::MissionStore::default())
        .manage(std::sync::Arc::new(recording::Recorder::default()));
    #[cfg(feature = "simulation")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        recording::start_recording,
        recording::stop_recording,
        recording::recording_status,
        recording::list_recordings,
        recording::read_recording,
        recording::read_recording_window,
        recording::recording_events,
        recording::rename_recording,
        recording::delete_recording,
        recording::export_recording,
        missions::load_missions,
        missions::save_missions,
        map3d::map3d_available,
        map3d::map3d_is_open,
        map3d::open_map3d,
        map::search_places,
        map::buildings_at,
        map::load_map_key,
        map::save_map_key,
        companion::companion_connect,
        companion::companion_disconnect,
        companion::companion_status,
        connection::list_serial_ports,
        connection::connect_serial,
        connection::auto_connect_serial,
        connection::set_auto_device,
        connection::connect_simulation,
        connection::disconnect_serial,
        connection::connection_status,
        connection::connection_summary,
        connection::acknowledge_alert,
        connection::simulation_available
    ]);
    #[cfg(not(feature = "simulation"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        recording::start_recording,
        recording::stop_recording,
        recording::recording_status,
        recording::list_recordings,
        recording::read_recording,
        recording::read_recording_window,
        recording::recording_events,
        recording::rename_recording,
        recording::delete_recording,
        recording::export_recording,
        missions::load_missions,
        missions::save_missions,
        map3d::map3d_available,
        map3d::map3d_is_open,
        map3d::open_map3d,
        map::search_places,
        map::buildings_at,
        map::load_map_key,
        map::save_map_key,
        companion::companion_connect,
        companion::companion_disconnect,
        companion::companion_status,
        connection::list_serial_ports,
        connection::connect_serial,
        connection::auto_connect_serial,
        connection::set_auto_device,
        connection::disconnect_serial,
        connection::connection_status,
        connection::connection_summary,
        connection::acknowledge_alert,
        connection::simulation_available
    ]);
    builder
        .run(tauri::generate_context!())
        .expect("failed to start FireScene GCS");
}
