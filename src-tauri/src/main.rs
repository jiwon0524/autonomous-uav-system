#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod camera;
mod companion;
mod connection;
mod map;
mod map3d;
mod missions;
mod qgc;
mod recording;
mod safety;
mod spatial;
mod telemetry;
#[cfg(all(feature = "simulation", debug_assertions))]
mod virtual_control;
fn main() {
    let builder = tauri::Builder::default()
        .manage(connection::Manager::default())
        .manage(companion::Manager::default())
        .manage(camera::Manager::default())
        .manage(spatial::Manager::default())
        .manage(missions::MissionStore::default())
        .manage(std::sync::Arc::new(recording::Recorder::default()));
    #[cfg(all(feature = "simulation", debug_assertions))]
    let builder = builder.manage(virtual_control::VirtualManager::default());
    #[cfg(all(feature = "simulation", debug_assertions))]
    let builder = builder.setup(|app| {
        virtual_control::start_watchdog(app.handle().clone());
        // Isolated desktop QA fixture; the extra window receives no capability.
        if std::env::var_os("FIRESCENE_QA_UNTRUSTED_WINDOW").as_deref()
            == Some(std::ffi::OsStr::new("1"))
        {
            tauri::WebviewWindowBuilder::new(
                app,
                "qa-untrusted",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("FireScene QA untrusted")
            .inner_size(800.0, 600.0)
            .build()?;
        }
        Ok(())
    });
    #[cfg(all(feature = "simulation", debug_assertions))]
    let builder = builder.on_window_event(|window, event| {
        use tauri::Manager as _;
        if window.label() == "main"
            && matches!(
                event,
                tauri::WindowEvent::Focused(false) | tauri::WindowEvent::Destroyed
            )
        {
            window
                .state::<virtual_control::VirtualManager>()
                .mark_focus_lost();
        }
    });
    #[cfg(all(feature = "simulation", debug_assertions))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        virtual_control::virtual_start,
        virtual_control::virtual_input,
        virtual_control::virtual_status,
        virtual_control::virtual_stop,
        virtual_control::virtual_lock,
        virtual_control::virtual_fault,
        virtual_control::virtual_replay_start,
        virtual_control::virtual_replay_stop,
        qgc::find_qgroundcontrol,
        qgc::launch_qgroundcontrol,
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
        spatial::spatial_connect,
        spatial::spatial_disconnect,
        spatial::spatial_status,
        spatial::spatial_take,
        camera::camera_connect,
        camera::camera_disconnect,
        camera::camera_status,
        camera::camera_take_frame,
        companion::companion_connect,
        companion::companion_disconnect,
        companion::companion_status,
        connection::connect_forwarded,
        connection::list_serial_ports,
        connection::connect_serial,
        connection::auto_connect_serial,
        connection::set_auto_device,
        connection::connect_simulation,
        connection::disconnect_serial,
        connection::connection_status,
        connection::connection_summary,
        connection::acknowledge_alert,
        connection::simulation_available,
        connection::replay_guard_required
    ]);
    #[cfg(all(feature = "simulation", not(debug_assertions)))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        qgc::find_qgroundcontrol,
        qgc::launch_qgroundcontrol,
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
        spatial::spatial_connect,
        spatial::spatial_disconnect,
        spatial::spatial_status,
        spatial::spatial_take,
        camera::camera_connect,
        camera::camera_disconnect,
        camera::camera_status,
        camera::camera_take_frame,
        companion::companion_connect,
        companion::companion_disconnect,
        companion::companion_status,
        connection::connect_forwarded,
        connection::list_serial_ports,
        connection::connect_serial,
        connection::auto_connect_serial,
        connection::set_auto_device,
        connection::connect_simulation,
        connection::disconnect_serial,
        connection::connection_status,
        connection::connection_summary,
        connection::acknowledge_alert,
        connection::simulation_available,
        connection::replay_guard_required
    ]);
    #[cfg(not(feature = "simulation"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        qgc::find_qgroundcontrol,
        qgc::launch_qgroundcontrol,
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
        spatial::spatial_connect,
        spatial::spatial_disconnect,
        spatial::spatial_status,
        spatial::spatial_take,
        camera::camera_connect,
        camera::camera_disconnect,
        camera::camera_status,
        camera::camera_take_frame,
        companion::companion_connect,
        companion::companion_disconnect,
        companion::companion_status,
        connection::connect_forwarded,
        connection::list_serial_ports,
        connection::connect_serial,
        connection::auto_connect_serial,
        connection::set_auto_device,
        connection::disconnect_serial,
        connection::connection_status,
        connection::connection_summary,
        connection::acknowledge_alert,
        connection::simulation_available,
        connection::replay_guard_required
    ]);
    builder
        .run(tauri::generate_context!())
        .expect("failed to start FireScene GCS");
}
