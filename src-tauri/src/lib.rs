use mouse_clicker_core::MouseClicker;
use std::sync::Arc;
use tauri::Emitter;

mod commands;

struct AppState {
    cmd_tx: mouse_clicker_core::CommandSender,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(10);
    let (evt_tx, evt_rx) = tokio::sync::mpsc::unbounded_channel();
    let evt_tx_clone = evt_tx.clone();

    tokio::task::spawn_blocking(|| {
        MouseClicker::event_loop_task(cmd_rx, evt_tx);
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState { cmd_tx })
        .invoke_handler(tauri::generate_handler![
            commands::register_hotkey,
            commands::unregister_hotkey,
            commands::config_change,
            commands::start_clicker,
            commands::stop_clicker
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();

            let status_callback: mouse_clicker_core::StatusCallback =
                Arc::new(move |running: bool| {
                    if let Err(e) = app_handle.emit_to(
                        tauri::EventTarget::any(),
                        "clicker-status-changed",
                        running,
                    ) {
                        eprintln!("Failed to emit status change event: {}", e);
                    }
                });

            tokio::spawn(async move {
                MouseClicker::keyboard_event_handler_task(
                    evt_rx,
                    evt_tx_clone,
                    Some(status_callback),
                )
                .await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
