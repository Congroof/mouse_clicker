use mouse_clicker_core::{ClickType, HotKeyRegisterError, MouseClicker};

use crate::AppState;

#[tauri::command(rename_all = "snake_case")]
pub async fn register_hotkey(
    state: tauri::State<'_, AppState>,
    id: i32,
    modifiers: u32,
    vk: u32,
    click_type: ClickType,
) -> Result<(), HotKeyRegisterError> {
    MouseClicker::register_hotkey(&state.cmd_tx, id, modifiers, vk, click_type).await?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn unregister_hotkey(
    state: tauri::State<'_, AppState>,
    id: i32,
) -> Result<(), HotKeyRegisterError> {
    MouseClicker::unregister_hotkey(&state.cmd_tx, id).await?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn config_change(
    state: tauri::State<'_, AppState>,
    times: usize,
    duration: u64,
) -> Result<(), String> {
    MouseClicker::config_change(&state.cmd_tx, times, duration).await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn start_clicker(
    state: tauri::State<'_, AppState>,
    id: i32,
) -> Result<(), String> {
    MouseClicker::manual_toggle(&state.cmd_tx, id, true).await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn stop_clicker(
    state: tauri::State<'_, AppState>,
    id: i32,
) -> Result<(), String> {
    MouseClicker::manual_toggle(&state.cmd_tx, id, false).await
        .map_err(|e| e.to_string())?;
    Ok(())
}