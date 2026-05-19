use std::collections::HashMap;
use std::ffi::CString;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::simconnect_ffi as ffi;

pub enum SimCommand {
    SubscribeVariable { name: String, unit: String, fps: i32 },
    UnsubscribeVariable { name: String },
    GetVariable { name: String, unit: String, reply_tx: mpsc::Sender<Option<f64>> },
    SubscribeEvent { event_name: String },
    TransmitEvent { event_name: String, data: u32 },
    Disconnect,
}

#[derive(Clone, serde::Serialize)]
struct VariablePayload {
    name: String,
    value: f64,
}

#[derive(Clone, serde::Serialize)]
struct EventPayload {
    name: String,
    data: u32,
}

fn fps_to_period(fps: i32) -> ffi::SIMCONNECT_PERIOD {
    match fps {
        -1    => ffi::SIMCONNECT_PERIOD_SIM_FRAME,
        0 | 1 => ffi::SIMCONNECT_PERIOD_SECOND,
        _     => ffi::SIMCONNECT_PERIOD_VISUAL_FRAME,
    }
}

/// Convert a &str to CString, replacing interior nulls with `?`.
fn to_cstring(s: &str) -> CString {
    let safe: String = s.chars().map(|c| if c == '\0' { '?' } else { c }).collect();
    CString::new(safe).unwrap_or_else(|_| CString::new("?").unwrap())
}

pub fn start_simconnect_thread(app: AppHandle, rx: mpsc::Receiver<SimCommand>) {
    thread::spawn(move || {
        let mut handle: ffi::HANDLE = std::ptr::null_mut();
        let app_name = to_cstring("JETT Studio");
        let hr = unsafe {
            ffi::SimConnect_Open(
                &mut handle,
                app_name.as_ptr(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                0,
            )
        };
        if hr != 0 || handle.is_null() {
            let _ = app.emit("jett-error", "Failed to connect to SimConnect");
            return;
        }
        let _ = app.emit("jett-connected", ());

        let mut next_define_id: u32 = 1;
        let mut next_event_id: u32 = 1;
        let mut variables: HashMap<u32, String> = HashMap::new();
        let mut variables_by_name: HashMap<String, u32> = HashMap::new();
        let mut events: HashMap<u32, String> = HashMap::new();
        let mut events_by_name: HashMap<String, u32> = HashMap::new();
        let mut pending_replies: HashMap<u32, mpsc::Sender<Option<f64>>> = HashMap::new();

        'main: loop {
            loop {
                match rx.try_recv() {
                    Ok(cmd) => match cmd {
                        SimCommand::Disconnect => break 'main,

                        SimCommand::SubscribeVariable { name, unit, fps } => {
                            if variables_by_name.contains_key(&name) {
                                continue;
                            }
                            let define_id = next_define_id;
                            next_define_id += 1;
                            let c_name = to_cstring(&name);
                            let c_unit = to_cstring(&unit);
                            unsafe {
                                ffi::SimConnect_AddToDataDefinition(
                                    handle, define_id,
                                    c_name.as_ptr(), c_unit.as_ptr(),
                                    ffi::SIMCONNECT_DATATYPE_FLOAT64, 0.0,
                                    ffi::SIMCONNECT_UNUSED,
                                );
                                ffi::SimConnect_RequestDataOnSimObject(
                                    handle, define_id, define_id,
                                    ffi::SIMCONNECT_OBJECT_ID_USER,
                                    fps_to_period(fps),
                                    0, 0, 0, 0,
                                );
                            }
                            variables.insert(define_id, name.clone());
                            variables_by_name.insert(name, define_id);
                        }

                        SimCommand::UnsubscribeVariable { name } => {
                            if let Some(define_id) = variables_by_name.remove(&name) {
                                unsafe { ffi::SimConnect_ClearDataDefinition(handle, define_id); }
                                variables.remove(&define_id);
                            }
                        }

                        SimCommand::GetVariable { name, unit, reply_tx } => {
                            let define_id = next_define_id;
                            next_define_id += 1;
                            let c_name = to_cstring(&name);
                            let c_unit = to_cstring(&unit);
                            unsafe {
                                ffi::SimConnect_AddToDataDefinition(
                                    handle, define_id,
                                    c_name.as_ptr(), c_unit.as_ptr(),
                                    ffi::SIMCONNECT_DATATYPE_FLOAT64, 0.0,
                                    ffi::SIMCONNECT_UNUSED,
                                );
                                ffi::SimConnect_RequestDataOnSimObject(
                                    handle, define_id, define_id,
                                    ffi::SIMCONNECT_OBJECT_ID_USER,
                                    ffi::SIMCONNECT_PERIOD_ONCE,
                                    0, 0, 0, 0,
                                );
                            }
                            pending_replies.insert(define_id, reply_tx);
                        }

                        SimCommand::SubscribeEvent { event_name } => {
                            if events_by_name.contains_key(&event_name) {
                                continue;
                            }
                            let event_id = next_event_id;
                            next_event_id += 1;
                            let c_name = to_cstring(&event_name);
                            unsafe {
                                ffi::SimConnect_MapClientEventToSimEvent(
                                    handle, event_id, c_name.as_ptr(),
                                );
                                ffi::SimConnect_AddClientEventToNotificationGroup(
                                    handle, 0, event_id, 0,
                                );
                                ffi::SimConnect_SetNotificationGroupPriority(
                                    handle, 0, ffi::SIMCONNECT_GROUP_PRIORITY_HIGHEST,
                                );
                            }
                            events.insert(event_id, event_name.clone());
                            events_by_name.insert(event_name, event_id);
                        }

                        SimCommand::TransmitEvent { event_name, data } => {
                            let event_id = if let Some(&id) = events_by_name.get(&event_name) {
                                id
                            } else {
                                let id = next_event_id;
                                next_event_id += 1;
                                let c_name = to_cstring(&event_name);
                                unsafe {
                                    ffi::SimConnect_MapClientEventToSimEvent(
                                        handle, id, c_name.as_ptr(),
                                    );
                                }
                                events.insert(id, event_name.clone());
                                events_by_name.insert(event_name, id);
                                id
                            };
                            unsafe {
                                ffi::SimConnect_TransmitClientEvent(
                                    handle,
                                    ffi::SIMCONNECT_OBJECT_ID_USER,
                                    event_id,
                                    data,
                                    ffi::SIMCONNECT_GROUP_PRIORITY_HIGHEST,
                                    ffi::SIMCONNECT_EVENT_FLAG_GROUPID_IS_PRIORITY,
                                );
                            }
                        }
                    },
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => break 'main,
                }
            }

            loop {
                let mut data_ptr: *mut ffi::SIMCONNECT_RECV = std::ptr::null_mut();
                let mut data_size: ffi::DWORD = 0;
                let hr = unsafe {
                    ffi::SimConnect_GetNextDispatch(handle, &mut data_ptr, &mut data_size)
                };
                if hr != 0 || data_ptr.is_null() {
                    break;
                }

                let recv_id = unsafe { std::ptr::read_unaligned(std::ptr::addr_of!((*data_ptr).dwID)) as ffi::SIMCONNECT_RECV_ID };
                match recv_id {
                    ffi::SIMCONNECT_RECV_ID_SIMOBJECT_DATA => {
                        let data = unsafe {
                            &*(data_ptr as *const ffi::SIMCONNECT_RECV_SIMOBJECT_DATA)
                        };
                        let define_id = unsafe { std::ptr::read_unaligned(std::ptr::addr_of!(data.dwDefineID)) };
                        let value: f64 = unsafe {
                            std::ptr::read_unaligned(
                                std::ptr::addr_of!(data.dwData) as *const f64,
                            )
                        };
                        if let Some(reply_tx) = pending_replies.remove(&define_id) {
                            let _ = reply_tx.send(Some(value));
                            unsafe { ffi::SimConnect_ClearDataDefinition(handle, define_id); }
                        } else if let Some(name) = variables.get(&define_id) {
                            let _ = app.emit(
                                "jett-variable",
                                VariablePayload { name: name.clone(), value },
                            );
                        }
                    }
                    ffi::SIMCONNECT_RECV_ID_EVENT => {
                        let data = unsafe {
                            &*(data_ptr as *const ffi::SIMCONNECT_RECV_EVENT)
                        };
                        let event_id = unsafe { std::ptr::read_unaligned(std::ptr::addr_of!(data.uEventID)) };
                        let event_data = unsafe { std::ptr::read_unaligned(std::ptr::addr_of!(data.dwData)) };
                        if let Some(name) = events.get(&event_id) {
                            let _ = app.emit(
                                "jett-event",
                                EventPayload { name: name.clone(), data: event_data },
                            );
                        }
                    }
                    ffi::SIMCONNECT_RECV_ID_QUIT => break 'main,
                    _ => {}
                }
            }

            thread::sleep(Duration::from_millis(16));
        }

        unsafe { ffi::SimConnect_Close(handle); }
        let _ = app.emit("jett-disconnected", ());
    });
}
