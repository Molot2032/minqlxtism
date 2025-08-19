// Main library entry point - converted from dllmain.c
#![feature(c_variadic)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(static_mut_refs)]

use libc::{srand, strcmp, time};
use once_cell::sync::Lazy;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

// Module declarations
pub mod commands;
pub mod common;
pub mod hooks;
pub mod patterns;
pub mod quake;
pub mod utils;

// Re-exports
pub use commands::*;
pub use common::*;
pub use hooks::*;
pub use quake::*;
pub use utils::*;

// Global constants
#[cfg(target_arch = "x86_64")]
const QZERODED: &[u8] = b"qzeroded.x64\0";
#[cfg(target_arch = "x86")]
const QZERODED: &[u8] = b"qzeroded.x86\0";

#[cfg(target_arch = "x86_64")]
const QAGAME_NAME: &[u8] = b"qagamex64.so\0";
#[cfg(target_arch = "x86")]
const QAGAME_NAME: &[u8] = b"qagamei386.so\0";

// External program name for comparison
extern "C" {
    static __progname: *mut c_char;
}

// Global variables for qagame module info
pub static mut QAGAME: *mut c_void = std::ptr::null_mut();
pub static mut QAGAME_DLLENTRY: *mut c_void = std::ptr::null_mut();

// Debug printing macros
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        print!("{}", crate::common::DEBUG_PRINT_PREFIX);
        println!($($arg)*);
    };
}

#[macro_export]
macro_rules! debug_error {
    ($file:expr, $line:expr, $func:expr, $($arg:tt)*) => {
        unsafe {
            eprintln!("{}", format_args!(crate::common::DEBUG_ERROR_FORMAT, $file, $line, $func));
            eprintln!($($arg)*);
        }
    };
}

// Pattern search macro (simplified for initial compilation)
macro_rules! static_search {
    ($fn_var:expr, $pattern:expr, $mask:expr, $module:expr) => {
        unsafe {
            // TODO: Replace with actual pattern search using patternscan crate
            $fn_var = std::ptr::null_mut() as _;
            if $fn_var.is_none() {
                debug_print!("ERROR: Unable to find {}", stringify!($fn_var));
                return false;
            } else {
                debug_print!(
                    "{}: {:p}",
                    stringify!($fn_var),
                    $fn_var.unwrap() as *const c_void
                );
            }
        }
    };
}

// Search for necessary functions in the Quake Live binary
unsafe fn search_functions() -> bool {
    use crate::patterns::find_pattern;
    use crate::quake::patterns;

    debug_println!("Searching for necessary functions...");

    let mut failed = false;

    // COM_PRINTF
    if let Some(addr) = find_pattern(patterns::COM_PRINTF) {
        crate::quake::COM_PRINTF = Some(std::mem::transmute(addr));
        debug_println!("COM_PRINTF: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find COM_PRINTF");
        failed = true;
    }

    // CMD_ADD_COMMAND
    if let Some(addr) = find_pattern(patterns::CMD_ADDCOMMAND) {
        crate::quake::CMD_ADD_COMMAND = Some(std::mem::transmute(addr));
        debug_println!("CMD_ADD_COMMAND: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find CMD_ADD_COMMAND");
        failed = true;
    }

    // CMD_ARGS
    if let Some(addr) = find_pattern(patterns::CMD_ARGS) {
        crate::quake::CMD_ARGS = Some(std::mem::transmute(addr));
        debug_println!("CMD_ARGS: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find CMD_ARGS");
        failed = true;
    }

    // CMD_ARGV
    if let Some(addr) = find_pattern(patterns::CMD_ARGV) {
        crate::quake::CMD_ARGV = Some(std::mem::transmute(addr));
        debug_println!("CMD_ARGV: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find CMD_ARGV");
        failed = true;
    }

    // CMD_ARGC
    if let Some(addr) = find_pattern(patterns::CMD_ARGC) {
        crate::quake::CMD_ARGC = Some(std::mem::transmute(addr));
        debug_println!("CMD_ARGC: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find CMD_ARGC");
        failed = true;
    }

    // CMD_TOKENIZE_STRING
    if let Some(addr) = find_pattern(patterns::CMD_TOKENIZESTRING) {
        crate::quake::CMD_TOKENIZE_STRING = Some(std::mem::transmute(addr));
        debug_println!("CMD_TOKENIZE_STRING: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find CMD_TOKENIZE_STRING");
        failed = true;
    }

    // CBUF_EXECUTE_TEXT
    if let Some(addr) = find_pattern(patterns::CBUF_EXECUTETEXT) {
        crate::quake::CBUF_EXECUTE_TEXT = Some(std::mem::transmute(addr));
        debug_println!("CBUF_EXECUTE_TEXT: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find CBUF_EXECUTE_TEXT");
        failed = true;
    }

    // CVAR_FIND_VAR
    if let Some(addr) = find_pattern(patterns::CVAR_FINDVAR) {
        crate::quake::CVAR_FIND_VAR = Some(std::mem::transmute(addr));
        debug_println!("CVAR_FIND_VAR: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find CVAR_FIND_VAR");
        failed = true;
    }

    // CVAR_GET
    if let Some(addr) = find_pattern(patterns::CVAR_GET) {
        crate::quake::CVAR_GET = Some(std::mem::transmute(addr));
        debug_println!("CVAR_GET: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find CVAR_GET");
        failed = true;
    }

    // CVAR_GET_LIMIT
    if let Some(addr) = find_pattern(patterns::CVAR_GETLIMIT) {
        crate::quake::CVAR_GET_LIMIT = Some(std::mem::transmute(addr));
        debug_println!("CVAR_GET_LIMIT: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find CVAR_GET_LIMIT");
        failed = true;
    }

    // CVAR_SET2
    if let Some(addr) = find_pattern(patterns::CVAR_SET2) {
        crate::quake::CVAR_SET2 = Some(std::mem::transmute(addr));
        debug_println!("CVAR_SET2: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find CVAR_SET2");
        failed = true;
    }

    // SV_EXECUTE_CLIENT_COMMAND
    if let Some(addr) = find_pattern(patterns::SV_EXECUTECLIENTCOMMAND) {
        crate::quake::SV_EXECUTE_CLIENT_COMMAND = Some(std::mem::transmute(addr));
        debug_println!("SV_EXECUTE_CLIENT_COMMAND: {:p}", addr as *const ());
    } else {
        debug_println!("WARNING: Unable to find SV_EXECUTE_CLIENT_COMMAND");
    }

    // SV_SEND_SERVER_COMMAND
    if let Some(addr) = find_pattern(patterns::SV_SENDSERVERCOMMAND) {
        crate::quake::SV_SEND_SERVER_COMMAND = Some(std::mem::transmute(addr));
        debug_println!("SV_SEND_SERVER_COMMAND: {:p}", addr as *const ());
    } else {
        debug_println!("WARNING: Unable to find SV_SEND_SERVER_COMMAND");
    }

    // SV_CLIENT_ENTER_WORLD
    if let Some(addr) = find_pattern(patterns::SV_CLIENTENTERWORLD) {
        crate::quake::SV_CLIENT_ENTER_WORLD = Some(std::mem::transmute(addr));
        debug_println!("SV_CLIENT_ENTER_WORLD: {:p}", addr as *const ());
    } else {
        debug_println!("WARNING: Unable to find SV_CLIENT_ENTER_WORLD");
    }

    // SV_SET_CONFIGSTRING
    if let Some(addr) = find_pattern(patterns::SV_SETCONFIGSTRING) {
        crate::quake::SV_SET_CONFIGSTRING = Some(std::mem::transmute(addr));
        debug_println!("SV_SET_CONFIGSTRING: {:p}", addr as *const ());
    } else {
        debug_println!("WARNING: Unable to find SV_SET_CONFIGSTRING");
    }

    // SV_DROP_CLIENT
    if let Some(addr) = find_pattern(patterns::SV_DROPCLIENT) {
        crate::quake::SV_DROP_CLIENT = Some(std::mem::transmute(addr));
        debug_println!("SV_DROP_CLIENT: {:p}", addr as *const ());
    } else {
        debug_println!("WARNING: Unable to find SV_DROP_CLIENT");
    }

    // SV_SPAWN_SERVER
    if let Some(addr) = find_pattern(patterns::SV_SPAWNSERVER) {
        crate::quake::SV_SPAWN_SERVER = Some(std::mem::transmute(addr));
        debug_println!("SV_SPAWN_SERVER: {:p}", addr as *const ());
    } else {
        debug_println!("WARNING: Unable to find SV_SPAWN_SERVER");
    }

    if failed {
        debug_println!("Exiting.");
        std::process::exit(1);
    }

    true
}

/// Runs when the dynamic library is loaded
#[ctor::ctor]
unsafe fn ctor() {
    // Check if we're running in the correct process
    let progname = CStr::from_ptr(__progname);
    let qzeroded_str = CStr::from_bytes_with_nul_unchecked(QZERODED);

    if strcmp(progname.as_ptr(), qzeroded_str.as_ptr()) != 0 {
        return;
    }

    debug_println!("Shared library loaded!");

    // Initialize server static structure pointer
    // TODO: Get actual svs pointer from pattern search
    SVS = std::ptr::null_mut();

    // Search for necessary functions
    if !search_functions() {
        debug_println!("Failed to find necessary functions!");
        std::process::exit(1);
    }

    // Install hooks
    if let Err(e) = hook_static() {
        debug_println!("Failed to install hooks: {e}");
        std::process::exit(1);
    }

    debug_println!("minqlxtism Rust version initialized!");
}

// Library metadata
#[no_mangle]
pub static MINQLXTISM_VERSION_STRING: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();

// Stub implementations for Python functions (Python support removed)
// These are commented out versions of the original Python integration code:

/*
// PYTHON INTEGRATION STUBS - REMOVED IN RUST VERSION
//
// Original Python dispatcher functions:
// - NewGameDispatcher
// - ClientCommandDispatcher
// - ServerCommandDispatcher
// - ClientLoadedDispatcher
// - SetConfigstringDispatcher
// - ClientDisconnectDispatcher
// - ConsolePrintDispatcher
// - FrameDispatcher
// - ClientConnectDispatcher
// - ClientSpawnDispatcher
// - KamikazeUseDispatcher
// - KamikazeExplodeDispatcher
// - RconDispatcher
//
// All Python functionality has been removed and replaced with stubs
// that print messages indicating Python support is not available.
*/


// ===============================================================
// DISPATCHER FUNCTION STUBS - PYTHON REPLACEMENT
// ===============================================================
// These functions replace the original Python dispatchers from python_dispatchers.c
// In the future, these could be implemented using extism for WASM plugin support
// For now, they are stubs that print messages indicating the events

// Frame dispatcher - called every game frame 
#[no_mangle]
pub unsafe extern "C" fn frame_dispatcher() {
    // PSEUDOCODE FROM ORIGINAL:
    // if (!frame_handler) return; // No registered handler.
    // PyGILState_STATE gstate = PyGILState_Ensure();
    // PyObject* result = PyObject_CallObject(frame_handler, NULL);
    // Py_XDECREF(result);
    // PyGILState_Release(gstate);
    
    // Rust stub - could call extism plugin in future
    // debug_println!("FrameDispatcher stub - plugin support removed");
}

// Client command dispatcher - intercepts client commands like "say", "vote", etc.
#[no_mangle] 
pub unsafe extern "C" fn client_command_dispatcher(client_id: c_int, cmd: *const c_char) -> *mut c_char {
    // PSEUDOCODE FROM ORIGINAL:
    // char* ret = cmd; // Default to original command
    // static char ccmd_buf[4096]; 
    // if (!client_command_handler) return ret;
    // 
    // PyObject* cmd_string = PyUnicode_DecodeUTF8(cmd, strlen(cmd), "ignore");
    // PyObject* result = PyObject_CallFunction(client_command_handler, "iO", client_id, cmd_string);
    // 
    // if (result == NULL) {
    //     DebugError("PyObject_CallFunction() returned NULL");
    // } else if (PyBool_Check(result) && result == Py_False) {
    //     ret = NULL; // Block command
    // } else if (PyUnicode_Check(result)) {
    //     strncpy(ccmd_buf, PyUnicode_AsUTF8(result), sizeof(ccmd_buf));
    //     ret = ccmd_buf; // Modified command
    // }
    
    let cmd_str = CStr::from_ptr(cmd).to_string_lossy();
    debug_println!("ClientCommandDispatcher stub - client_id={}, cmd={}", client_id, cmd_str);
    cmd as *mut c_char // Return original command unchanged
}

// Server command dispatcher - intercepts server->client commands like "print", "cp", etc.
#[no_mangle]
pub unsafe extern "C" fn server_command_dispatcher(client_id: c_int, cmd: *const c_char) -> *mut c_char {
    // PSEUDOCODE FROM ORIGINAL:
    // char* ret = cmd; // Default to original command  
    // static char scmd_buf[4096];
    // if (!server_command_handler) return ret;
    //
    // Similar Python call pattern as client_command_dispatcher
    // Returns: original cmd, NULL to block, or modified command string
    
    let cmd_str = CStr::from_ptr(cmd).to_string_lossy();
    debug_println!("ServerCommandDispatcher stub - client_id={}, cmd={}", client_id, cmd_str);
    cmd as *mut c_char // Return original command unchanged
}

// Client connect dispatcher - called when player connects, can reject connection
#[no_mangle]
pub unsafe extern "C" fn client_connect_dispatcher(client_id: c_int, is_bot: c_int) -> *mut c_char {
    // PSEUDOCODE FROM ORIGINAL:
    // char* ret = NULL; // NULL = allow connection
    // static char connect_buf[4096];
    // if (!client_connect_handler) return ret;
    //
    // allow_free_client = client_id; // Allow getting player info for CS_FREE client
    // PyObject* result = PyObject_CallFunction(client_connect_handler, "iO", client_id, is_bot ? Py_True : Py_False);
    // allow_free_client = -1;
    //
    // if (result == NULL) {
    //     DebugError("PyObject_CallFunction() returned NULL");
    // } else if (PyBool_Check(result) && result == Py_False) {
    //     ret = "You are banned from this server."; // Reject with message
    // } else if (PyUnicode_Check(result)) {
    //     strncpy(connect_buf, PyUnicode_AsUTF8(result), sizeof(connect_buf));
    //     ret = connect_buf; // Custom rejection message
    // }
    
    debug_println!("ClientConnectDispatcher stub - client_id={}, is_bot={}", client_id, is_bot);
    std::ptr::null_mut() // Allow connection
}

// Client disconnect dispatcher - called when player disconnects  
#[no_mangle]
pub unsafe extern "C" fn client_disconnect_dispatcher(client_id: c_int, reason: *const c_char) {
    // PSEUDOCODE FROM ORIGINAL:
    // if (!client_disconnect_handler) return;
    //
    // allow_free_client = client_id; // Allow getting player info for CS_FREE client  
    // PyObject* result = PyObject_CallFunction(client_disconnect_handler, "is", client_id, reason);
    // allow_free_client = -1;
    
    let reason_str = CStr::from_ptr(reason).to_string_lossy();
    debug_println!("ClientDisconnectDispatcher stub - client_id={}, reason={}", client_id, reason_str);
}

// Client loaded dispatcher - called when client finishes loading (not bots)
#[no_mangle]
pub unsafe extern "C" fn client_loaded_dispatcher(client_id: c_int) -> c_int {
    // PSEUDOCODE FROM ORIGINAL:
    // int ret = 1; // Default allow
    // if (!client_loaded_handler) return ret;
    //
    // PyObject* result = PyObject_CallFunction(client_loaded_handler, "i", client_id);
    // if (PyBool_Check(result) && result == Py_False) {
    //     ret = 0; // Block client from entering
    // }
    
    debug_println!("ClientLoadedDispatcher stub - client_id={}", client_id);
    1 // Allow client to enter
}

// Client spawn dispatcher - called when player spawns
#[no_mangle]
pub unsafe extern "C" fn client_spawn_dispatcher(client_id: c_int) {
    // PSEUDOCODE FROM ORIGINAL:
    // if (!client_spawn_handler) return;
    // PyObject* result = PyObject_CallFunction(client_spawn_handler, "i", client_id);
    
    debug_println!("ClientSpawnDispatcher stub - client_id={}", client_id);
}

// New game dispatcher - called when map starts/restarts
#[no_mangle] 
pub unsafe extern "C" fn new_game_dispatcher(restart: c_int) {
    // PSEUDOCODE FROM ORIGINAL:
    // if (!new_game_handler) return;
    // PyObject* result = PyObject_CallFunction(new_game_handler, "O", restart ? Py_True : Py_False);
    
    debug_println!("NewGameDispatcher stub - restart={}", restart);
}

// Set configstring dispatcher - intercepts configstring changes
#[no_mangle]
pub unsafe extern "C" fn set_configstring_dispatcher(index: c_int, value: *const c_char) -> *mut c_char {
    // PSEUDOCODE FROM ORIGINAL:
    // char* ret = value; // Default to original value
    // static char setcs_buf[4096];
    // if (!set_configstring_handler) return ret;
    //
    // PyObject* value_string = PyUnicode_DecodeUTF8(value, strlen(value), "ignore");
    // PyObject* result = PyObject_CallFunction(set_configstring_handler, "iO", index, value_string);
    //
    // if (PyBool_Check(result) && result == Py_False) {
    //     ret = NULL; // Block configstring change
    // } else if (PyUnicode_Check(result)) {
    //     strncpy(setcs_buf, PyUnicode_AsUTF8(result), sizeof(setcs_buf));
    //     ret = setcs_buf; // Modified value
    // }
    
    let value_str = CStr::from_ptr(value).to_string_lossy();
    debug_println!("SetConfigstringDispatcher stub - index={}, value={}", index, value_str);
    value as *mut c_char // Return original value unchanged
}

// RCON dispatcher - called when RCON commands are executed
#[no_mangle]
pub unsafe extern "C" fn rcon_dispatcher(cmd: *const c_char) {
    // PSEUDOCODE FROM ORIGINAL:
    // if (!rcon_handler) return;
    // PyObject* result = PyObject_CallFunction(rcon_handler, "s", cmd);
    
    let cmd_str = CStr::from_ptr(cmd).to_string_lossy();
    debug_println!("RconDispatcher stub - cmd={}", cmd_str);
}

// Console print dispatcher - intercepts server console output
#[no_mangle]
pub unsafe extern "C" fn console_print_dispatcher(text: *const c_char) -> *mut c_char {
    // PSEUDOCODE FROM ORIGINAL:
    // char* ret = text; // Default to original text
    // static char print_buf[4096];
    // if (!console_print_handler) return ret;
    //
    // PyObject* text_string = PyUnicode_DecodeUTF8(text, strlen(text), "ignore");
    // PyObject* result = PyObject_CallFunction(console_print_handler, "O", text_string);
    //
    // if (PyBool_Check(result) && result == Py_False) {
    //     ret = NULL; // Block console output
    // } else if (PyUnicode_Check(result)) {
    //     strncpy(print_buf, PyUnicode_AsUTF8(result), sizeof(print_buf));
    //     ret = print_buf; // Modified text
    // }
    
    let text_str = CStr::from_ptr(text).to_string_lossy();
    debug_println!("ConsolePrintDispatcher stub - text={}", text_str);
    text as *mut c_char // Return original text unchanged
}

// Kamikaze use dispatcher - called when kamikaze powerup is used
#[no_mangle] 
pub unsafe extern "C" fn kamikaze_use_dispatcher(client_id: c_int) {
    // PSEUDOCODE FROM ORIGINAL:
    // if (!kamikaze_use_handler) return;
    // PyObject* result = PyObject_CallFunction(kamikaze_use_handler, "i", client_id);
    
    debug_println!("KamikazeUseDispatcher stub - client_id={}", client_id);
}

// Kamikaze explode dispatcher - called when kamikaze explodes
#[no_mangle]
pub unsafe extern "C" fn kamikaze_explode_dispatcher(client_id: c_int, is_used_on_demand: c_int) {
    // PSEUDOCODE FROM ORIGINAL:
    // if (!kamikaze_explode_handler) return;  
    // PyObject* result = PyObject_CallFunction(kamikaze_explode_handler, "ii", client_id, is_used_on_demand);
    
    debug_println!("KamikazeExplodeDispatcher stub - client_id={}, is_used_on_demand={}", client_id, is_used_on_demand);
}

// ===============================================================
// ORIGINAL LIBRARY EXPORTS
// ===============================================================

// Export the library functions that the Quake Live engine expects
#[no_mangle]
pub unsafe extern "C" fn dllEntry(_arg: *mut c_void) -> c_int {
    // This is the entry point that qagame calls
    debug_println!("dllEntry called");
    1
}

// Additional exports that might be needed
#[no_mangle]
pub unsafe extern "C" fn vmMain(
    _command: c_int,
    _arg0: c_int,
    _arg1: c_int,
    _arg2: c_int,
) -> c_int {
    // VM main entry point
    debug_println!("vmMain called with command: {}", _command);
    0
}
