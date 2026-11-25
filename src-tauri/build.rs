fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "load_missions",
            "save_missions",
            "search_places",
            "buildings_at",
            "load_map_key",
            "save_map_key",
            "list_serial_ports",
            "connect_serial",
            "auto_connect_serial",
            "set_auto_device",
            "connect_simulation",
            "disconnect_serial",
            "connection_status",
            "acknowledge_alert",
            "simulation_available",
            "map3d_available",
            "map3d_is_open",
            "open_map3d",
        ]),
    ))
    .expect("failed to configure application permissions");
}
