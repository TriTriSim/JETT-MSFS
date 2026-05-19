//! Raw SimConnect FFI declarations.
//! Links against SimConnect.lib (bundled in lib/).
//! No bindgen — types and function signatures are written manually
//! based on SimConnect.h and verified bindgen output.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::os::raw::{c_char, c_int};

// ── Windows primitive types ────────────────────────────────────────────────
pub type HANDLE = *mut std::ffi::c_void;
pub type HWND   = *mut std::ffi::c_void;
pub type DWORD  = u32;
pub type BOOL   = i32;
pub type HRESULT = i32;

// ── SimConnect ID type aliases (all DWORD) ────────────────────────────────
pub type SIMCONNECT_OBJECT_ID              = DWORD;
pub type SIMCONNECT_DATA_DEFINITION_ID     = DWORD;
pub type SIMCONNECT_CLIENT_EVENT_ID        = DWORD;
pub type SIMCONNECT_DATA_REQUEST_ID        = DWORD;
pub type SIMCONNECT_NOTIFICATION_GROUP_ID  = DWORD;

// ── Enum base types ───────────────────────────────────────────────────────
pub type SIMCONNECT_RECV_ID   = c_int;
pub type SIMCONNECT_DATATYPE  = c_int;
pub type SIMCONNECT_PERIOD    = c_int;

// ── Constants ─────────────────────────────────────────────────────────────
pub const SIMCONNECT_OBJECT_ID_USER:                DWORD = 0;
pub const SIMCONNECT_GROUP_PRIORITY_HIGHEST:        DWORD = 1;
pub const SIMCONNECT_EVENT_FLAG_GROUPID_IS_PRIORITY: DWORD = 16;
pub const SIMCONNECT_UNUSED:                        DWORD = 0xFFFF_FFFF;

// SIMCONNECT_RECV_ID values
pub const SIMCONNECT_RECV_ID_NULL:           SIMCONNECT_RECV_ID = 0;
pub const SIMCONNECT_RECV_ID_EXCEPTION:      SIMCONNECT_RECV_ID = 1;
pub const SIMCONNECT_RECV_ID_OPEN:           SIMCONNECT_RECV_ID = 2;
pub const SIMCONNECT_RECV_ID_QUIT:           SIMCONNECT_RECV_ID = 3;
pub const SIMCONNECT_RECV_ID_EVENT:          SIMCONNECT_RECV_ID = 4;
pub const SIMCONNECT_RECV_ID_SIMOBJECT_DATA: SIMCONNECT_RECV_ID = 8;

// SIMCONNECT_DATATYPE values
pub const SIMCONNECT_DATATYPE_FLOAT64: SIMCONNECT_DATATYPE = 4;

// SIMCONNECT_PERIOD values
pub const SIMCONNECT_PERIOD_NEVER:        SIMCONNECT_PERIOD = 0;
pub const SIMCONNECT_PERIOD_ONCE:         SIMCONNECT_PERIOD = 1;
pub const SIMCONNECT_PERIOD_VISUAL_FRAME: SIMCONNECT_PERIOD = 2;
pub const SIMCONNECT_PERIOD_SIM_FRAME:    SIMCONNECT_PERIOD = 3;
pub const SIMCONNECT_PERIOD_SECOND:       SIMCONNECT_PERIOD = 4;

// ── Structs ───────────────────────────────────────────────────────────────

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SIMCONNECT_RECV {
    pub dwSize:    DWORD,
    pub dwVersion: DWORD,
    pub dwID:      DWORD,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SIMCONNECT_RECV_EVENT {
    pub _base:    SIMCONNECT_RECV,
    pub uGroupID: DWORD,
    pub uEventID: DWORD,
    pub dwData:   DWORD,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SIMCONNECT_RECV_SIMOBJECT_DATA {
    pub _base:         SIMCONNECT_RECV,
    pub dwRequestID:   DWORD,
    pub dwObjectID:    DWORD,
    pub dwDefineID:    DWORD,
    pub dwFlags:       DWORD,
    pub dwentrynumber: DWORD,
    pub dwoutof:       DWORD,
    pub dwDefineCount: DWORD,
    pub dwData:        DWORD, // first 4 bytes of variable-length data
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SIMCONNECT_RECV_EXCEPTION {
    pub _base:       SIMCONNECT_RECV,
    pub dwException: DWORD,
    pub dwSendID:    DWORD,
    pub dwIndex:     DWORD,
}

// SIMCONNECT_EXCEPTION codes (partial)
pub const SIMCONNECT_EXCEPTION_NONE:              DWORD = 0;
pub const SIMCONNECT_EXCEPTION_ERROR:             DWORD = 1;
pub const SIMCONNECT_EXCEPTION_UNRECOGNIZED_ID:   DWORD = 3;
pub const SIMCONNECT_EXCEPTION_NAME_UNRECOGNIZED: DWORD = 7;
pub const SIMCONNECT_EXCEPTION_INVALID_DATA_TYPE: DWORD = 18;
pub const SIMCONNECT_EXCEPTION_DEFINITION_ERROR:  DWORD = 28;

// ── Extern function declarations ──────────────────────────────────────────
// SimConnect API uses __stdcall (= "system" on Windows).
// All symbols are extern "C" in SimConnect.h (no C++ mangling).

extern "system" {
    pub fn SimConnect_Open(
        phSimConnect:  *mut HANDLE,
        szName:        *const c_char,
        hWnd:          HWND,
        UserEventWin32: DWORD,
        hEventHandle:  HANDLE,
        ConfigIndex:   DWORD,
    ) -> HRESULT;

    pub fn SimConnect_Close(hSimConnect: HANDLE) -> HRESULT;

    pub fn SimConnect_GetNextDispatch(
        hSimConnect: HANDLE,
        ppData:      *mut *mut SIMCONNECT_RECV,
        pcbData:     *mut DWORD,
    ) -> HRESULT;

    pub fn SimConnect_AddToDataDefinition(
        hSimConnect: HANDLE,
        DefineID:    SIMCONNECT_DATA_DEFINITION_ID,
        DatumName:   *const c_char,
        UnitsName:   *const c_char,
        DatumType:   SIMCONNECT_DATATYPE,
        fEpsilon:    f32,
        DatumID:     DWORD,
    ) -> HRESULT;

    pub fn SimConnect_ClearDataDefinition(
        hSimConnect: HANDLE,
        DefineID:    SIMCONNECT_DATA_DEFINITION_ID,
    ) -> HRESULT;

    pub fn SimConnect_RequestDataOnSimObject(
        hSimConnect: HANDLE,
        RequestID:   SIMCONNECT_DATA_REQUEST_ID,
        DefineID:    SIMCONNECT_DATA_DEFINITION_ID,
        ObjectID:    SIMCONNECT_OBJECT_ID,
        Period:      SIMCONNECT_PERIOD,
        Flags:       DWORD,
        origin:      DWORD,
        interval:    DWORD,
        limit:       DWORD,
    ) -> HRESULT;

    pub fn SimConnect_MapClientEventToSimEvent(
        hSimConnect: HANDLE,
        EventID:     SIMCONNECT_CLIENT_EVENT_ID,
        EventName:   *const c_char,
    ) -> HRESULT;

    pub fn SimConnect_AddClientEventToNotificationGroup(
        hSimConnect: HANDLE,
        GroupID:     SIMCONNECT_NOTIFICATION_GROUP_ID,
        EventID:     SIMCONNECT_CLIENT_EVENT_ID,
        bMaskable:   BOOL,
    ) -> HRESULT;

    pub fn SimConnect_SetNotificationGroupPriority(
        hSimConnect: HANDLE,
        GroupID:     SIMCONNECT_NOTIFICATION_GROUP_ID,
        uPriority:   DWORD,
    ) -> HRESULT;

    pub fn SimConnect_TransmitClientEvent(
        hSimConnect: HANDLE,
        ObjectID:    SIMCONNECT_OBJECT_ID,
        EventID:     SIMCONNECT_CLIENT_EVENT_ID,
        dwData:      DWORD,
        GroupID:     SIMCONNECT_NOTIFICATION_GROUP_ID,
        dwFlags:     DWORD,
    ) -> HRESULT;
}
