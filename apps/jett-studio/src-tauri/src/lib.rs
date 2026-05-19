mod simconnect_ffi;
mod simconnect_manager;
use simconnect_manager::{start_simconnect_thread, SimCommand};
use std::sync::{mpsc, Mutex};

struct AppState {
    tx: Mutex<Option<mpsc::Sender<SimCommand>>>,
}

#[tauri::command]
fn sim_connect(app: tauri::AppHandle, state: tauri::State<AppState>) -> Result<(), String> {
    let mut guard = state.tx.lock().unwrap();
    if guard.is_some() {
        return Err("Already connected".to_string());
    }
    let (tx, rx) = mpsc::channel();
    start_simconnect_thread(app, rx);
    *guard = Some(tx);
    Ok(())
}

#[tauri::command]
fn sim_disconnect(state: tauri::State<AppState>) -> Result<(), String> {
    let mut guard = state.tx.lock().unwrap();
    if let Some(tx) = guard.take() {
        let _ = tx.send(SimCommand::Disconnect);
    }
    Ok(())
}

#[tauri::command]
fn sim_subscribe_variable(
    name: String,
    unit: String,
    fps: i32,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let guard = state.tx.lock().unwrap();
    if let Some(tx) = guard.as_ref() {
        tx.send(SimCommand::SubscribeVariable { name, unit, fps })
            .map_err(|e| e.to_string())
    } else {
        Err("SimConnect not connected".to_string())
    }
}

#[tauri::command]
fn sim_unsubscribe_variable(name: String, state: tauri::State<AppState>) -> Result<(), String> {
    let guard = state.tx.lock().unwrap();
    if let Some(tx) = guard.as_ref() {
        tx.send(SimCommand::UnsubscribeVariable { name })
            .map_err(|e| e.to_string())
    } else {
        Err("SimConnect not connected".to_string())
    }
}

#[tauri::command]
fn sim_get_variable(
    name: String,
    unit: String,
    state: tauri::State<AppState>,
) -> Result<f64, String> {
    let (reply_tx, reply_rx) = mpsc::channel();
    {
        let guard = state.tx.lock().unwrap();
        if let Some(tx) = guard.as_ref() {
            tx.send(SimCommand::GetVariable { name, unit, reply_tx })
                .map_err(|e| e.to_string())?;
        } else {
            return Err("SimConnect not connected".to_string());
        }
    }
    reply_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No value received".to_string())
}

#[tauri::command]
fn sim_subscribe_event(event_name: String, state: tauri::State<AppState>) -> Result<(), String> {
    let guard = state.tx.lock().unwrap();
    if let Some(tx) = guard.as_ref() {
        tx.send(SimCommand::SubscribeEvent { event_name })
            .map_err(|e| e.to_string())
    } else {
        Err("SimConnect not connected".to_string())
    }
}

#[tauri::command]
fn sim_transmit_event(
    event_name: String,
    data: u32,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let guard = state.tx.lock().unwrap();
    if let Some(tx) = guard.as_ref() {
        tx.send(SimCommand::TransmitEvent { event_name, data })
            .map_err(|e| e.to_string())
    } else {
        Err("SimConnect not connected".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            tx: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            sim_connect,
            sim_disconnect,
            sim_subscribe_variable,
            sim_unsubscribe_variable,
            sim_get_variable,
            sim_subscribe_event,
            sim_transmit_event,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
