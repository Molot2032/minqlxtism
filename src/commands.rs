// Commands converted from commands.c
use crate::quake::*;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

// Command implementations - converted from C
pub unsafe extern "C" fn send_server_command() {
    if let Some(cmd_args_fn) = CMD_ARGS {
        if let Some(sv_send_cmd_fn) = SV_SEND_SERVER_COMMAND {
            let args = cmd_args_fn();
            let fmt_str = CString::new("%s\n").unwrap();
            sv_send_cmd_fn(std::ptr::null_mut(), fmt_str.as_ptr(), args);
        }
    }
}

pub unsafe extern "C" fn center_print() {
    if let Some(cmd_args_fn) = CMD_ARGS {
        if let Some(sv_send_cmd_fn) = SV_SEND_SERVER_COMMAND {
            let args = cmd_args_fn();
            let fmt_str = CString::new("cp \"%s\"\n").unwrap();
            sv_send_cmd_fn(std::ptr::null_mut(), fmt_str.as_ptr(), args);
        }
    }
}

pub unsafe extern "C" fn regular_print() {
    if let Some(cmd_args_fn) = CMD_ARGS {
        if let Some(sv_send_cmd_fn) = SV_SEND_SERVER_COMMAND {
            let args = cmd_args_fn();
            let fmt_str = CString::new("print \"%s\n\"\n").unwrap();
            sv_send_cmd_fn(std::ptr::null_mut(), fmt_str.as_ptr(), args);
        }
    }
}

pub unsafe extern "C" fn slap() {
    let mut dmg = 0;

    // Get command arguments
    if let Some(cmd_argc_fn) = CMD_ARGC {
        if let Some(cmd_argv_fn) = CMD_ARGV {
            if let Some(com_printf_fn) = COM_PRINTF {
                let argc = cmd_argc_fn();

                if argc < 2 {
                    let usage_str = CString::new("Usage: %s <client_id> [damage]\n").unwrap();
                    let cmd_name = cmd_argv_fn(0);
                    com_printf_fn(usage_str.as_ptr(), cmd_name);
                    return;
                }

                let client_id_str = cmd_argv_fn(1);
                let client_id_cstr = CStr::from_ptr(client_id_str);
                let client_id = match client_id_cstr.to_str() {
                    Ok(s) => match s.parse::<i32>() {
                        Ok(id) => id,
                        Err(_) => {
                            let err_str = CString::new("Invalid client ID\n").unwrap();
                            com_printf_fn(err_str.as_ptr());
                            return;
                        }
                    },
                    Err(_) => {
                        let err_str = CString::new("Invalid client ID\n").unwrap();
                        com_printf_fn(err_str.as_ptr());
                        return;
                    }
                };

                // Validate client ID against sv_maxclients
                if SV_MAXCLIENTS.is_null() {
                    let err_str = CString::new("sv_maxclients not initialized\n").unwrap();
                    com_printf_fn(err_str.as_ptr());
                    return;
                }

                let max_clients = (*SV_MAXCLIENTS).integer;
                if client_id < 0 || client_id >= max_clients {
                    let err_str =
                        CString::new("client_id must be a number between 0 and %d\n").unwrap();
                    com_printf_fn(err_str.as_ptr(), max_clients);
                    return;
                }

                // Get damage if provided
                if argc > 2 {
                    let dmg_str = cmd_argv_fn(2);
                    let dmg_cstr = CStr::from_ptr(dmg_str);
                    if let Ok(s) = dmg_cstr.to_str() {
                        if let Ok(d) = s.parse::<i32>() {
                            dmg = d;
                        }
                    }
                }

                // TODO: Implement actual slap logic when G_ENTITIES is properly implemented
                let msg_str =
                    CString::new("Slapping functionality not yet implemented in Rust version\n")
                        .unwrap();
                com_printf_fn(msg_str.as_ptr());
            }
        }
    }
}

pub unsafe extern "C" fn slay() {
    if let Some(cmd_argc_fn) = CMD_ARGC {
        if let Some(cmd_argv_fn) = CMD_ARGV {
            if let Some(com_printf_fn) = COM_PRINTF {
                let argc = cmd_argc_fn();

                if argc < 2 {
                    let usage_str = CString::new("Usage: %s <client_id>\n").unwrap();
                    let cmd_name = cmd_argv_fn(0);
                    com_printf_fn(usage_str.as_ptr(), cmd_name);
                    return;
                }

                // TODO: Implement actual slay logic
                let msg_str =
                    CString::new("Slay functionality not yet implemented in Rust version\n")
                        .unwrap();
                com_printf_fn(msg_str.as_ptr());
            }
        }
    }
}

pub unsafe extern "C" fn download_workshop_item() {
    if let Some(cmd_argc_fn) = CMD_ARGC {
        if let Some(cmd_argv_fn) = CMD_ARGV {
            if let Some(com_printf_fn) = COM_PRINTF {
                let argc = cmd_argc_fn();

                if argc < 2 {
                    let usage_str = CString::new("Usage: %s <workshop_id>\n").unwrap();
                    let cmd_name = cmd_argv_fn(0);
                    com_printf_fn(usage_str.as_ptr(), cmd_name);
                    return;
                }

                // TODO: Implement workshop download logic
                let msg_str = CString::new(
                    "Workshop download functionality not yet implemented in Rust version\n",
                )
                .unwrap();
                com_printf_fn(msg_str.as_ptr());
            }
        }
    }
}

pub unsafe extern "C" fn stop_following() {
    if let Some(cmd_argc_fn) = CMD_ARGC {
        if let Some(cmd_argv_fn) = CMD_ARGV {
            if let Some(com_printf_fn) = COM_PRINTF {
                let argc = cmd_argc_fn();

                if argc < 2 {
                    let usage_str = CString::new("Usage: %s <client_id>\n").unwrap();
                    let cmd_name = cmd_argv_fn(0);
                    com_printf_fn(usage_str.as_ptr(), cmd_name);
                    return;
                }

                // TODO: Implement stop following logic
                let msg_str = CString::new(
                    "Stop following functionality not yet implemented in Rust version\n",
                )
                .unwrap();
                com_printf_fn(msg_str.as_ptr());
            }
        }
    }
}

// Python command stubs (Python support removed)
pub unsafe extern "C" fn py_rcon() {
    if let Some(com_printf_fn) = COM_PRINTF {
        let msg_str =
            CString::new("Python support has been removed from this Rust version\n").unwrap();
        com_printf_fn(msg_str.as_ptr());
    }
}

pub unsafe extern "C" fn py_command() {
    if let Some(com_printf_fn) = COM_PRINTF {
        let msg_str =
            CString::new("Python support has been removed from this Rust version\n").unwrap();
        com_printf_fn(msg_str.as_ptr());
    }
}

pub unsafe extern "C" fn restart_python() {
    if let Some(com_printf_fn) = COM_PRINTF {
        let msg_str =
            CString::new("Python support has been removed from this Rust version\n").unwrap();
        com_printf_fn(msg_str.as_ptr());
    }
}

// Additional function pointer types that will be needed
pub type SvSendServerCommandPtr = unsafe extern "C" fn(*mut c_void, *const c_char, ...);

// Global function pointer (will be initialized during pattern search)
pub static mut SV_SEND_SERVER_COMMAND: Option<SvSendServerCommandPtr> = None;
