// Hooks converted from hooks.c
use crate::commands::*;
use crate::common::{COMMON_INITIALIZED, CVARS_INITIALIZED};
use crate::debug_println;
use crate::quake::*;
use crate::{QAGAME, QAGAME_DLLENTRY};
use retour::GenericDetour;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};

// Global variables moved to lib.rs to avoid duplicates
pub static mut SKIP_FRAME_DISPATCHER: bool = false;

// Hook function types
type CmdAddCommandHook = unsafe extern "C" fn(*mut c_char, *mut c_void);
type SysSetModuleOffsetHook = unsafe extern "C" fn(*mut c_char, *mut c_void);
type SysIsLANAddressHook = unsafe extern "C" fn(NetAdr) -> QBoolean;
type GInitGameHook = unsafe extern "C" fn(c_int, c_int, c_int);

// Engine-level hook function types
type SVExecuteClientCommandHook = unsafe extern "C" fn(*mut Client, *mut c_char, QBoolean);
type SVSendServerCommandHook = unsafe extern "C" fn(*mut Client, *const c_char, ...);
type SVClientEnterWorldHook = unsafe extern "C" fn(*mut Client, *mut UserCmd);
type SVSetConfigstringHook = unsafe extern "C" fn(c_int, *const c_char);
type SVDropClientHook = unsafe extern "C" fn(*mut Client, *const c_char);
type SVSpawnServerHook = unsafe extern "C" fn(*mut c_char, QBoolean);

// Static detours - using retour-rs for hooking
static mut CMD_ADD_COMMAND_HOOK: Option<GenericDetour<CmdAddCommandHook>> = None;
static mut SYS_SET_MODULE_OFFSET_HOOK: Option<GenericDetour<SysSetModuleOffsetHook>> = None;
static mut SYS_IS_LAN_ADDRESS_HOOK: Option<GenericDetour<SysIsLANAddressHook>> = None;

// Engine-level hook detours
static mut SV_EXECUTE_CLIENT_COMMAND_HOOK: Option<GenericDetour<SVExecuteClientCommandHook>> = None;
// NOTE: SV_SEND_SERVER_COMMAND_HOOK cannot use GenericDetour due to variadic arguments
// static mut SV_SEND_SERVER_COMMAND_HOOK: Option<GenericDetour<SVSendServerCommandHook>> = None;
static mut SV_CLIENT_ENTER_WORLD_HOOK: Option<GenericDetour<SVClientEnterWorldHook>> = None;
static mut SV_SET_CONFIGSTRING_HOOK: Option<GenericDetour<SVSetConfigstringHook>> = None;
static mut SV_DROP_CLIENT_HOOK: Option<GenericDetour<SVDropClientHook>> = None;
static mut SV_SPAWN_SERVER_HOOK: Option<GenericDetour<SVSpawnServerHook>> = None;

// Hook replacement functions
unsafe extern "C" fn my_cmd_add_command(cmd: *mut c_char, func: *mut c_void) {
    let cmd_str = CStr::from_ptr(cmd).to_string_lossy();
    debug_println!("my_cmd_add_command called: cmd={cmd_str}, func={func:p}");

    if !COMMON_INITIALIZED {
        initialize_static();
    }

    // Call original function
    if let Some(ref hook) = CMD_ADD_COMMAND_HOOK {
        hook.call(cmd, func);
    }
}

unsafe extern "C" fn my_sys_set_module_offset(module_name: *mut c_char, offset: *mut c_void) {
    let name_str = CStr::from_ptr(module_name);

    if name_str.to_bytes() == b"qagame" {
        // Despite the name, it's not the actual module, but vmMain.
        // We use dlinfo to get the base of the module so we can properly
        // initialize all the pointers relative to the base.
        QAGAME_DLLENTRY = offset;

        // TODO: Use dladdr equivalent to get module base
        QAGAME = offset; // Simplified for now

        debug_println!("Got qagame: {:#010x}", QAGAME as usize);
    } else {
        let name_str_safe = name_str.to_string_lossy();
        debug_println!("Unknown module: {}", name_str_safe);
    }

    // Call original function
    if let Some(ref hook) = SYS_SET_MODULE_OFFSET_HOOK {
        hook.call(module_name, offset);
    }

    if COMMON_INITIALIZED {
        // Search for VM functions
        if !crate::patterns::search_vm_functions() {
            crate::debug_println!("Failed to find VM functions!");
            std::process::exit(1);
        }

        // Install VM hooks
        if let Err(e) = crate::hooks::install_vm_hooks() {
            debug_println!("Failed to install VM hooks: {}", e);
            std::process::exit(1);
        }

        // Apply VM patches
        if let Err(e) = crate::hooks::patch_vm() {
            debug_println!("Failed to apply VM patches: {}", e);
            std::process::exit(1);
        }
    }
}

unsafe extern "C" fn my_sys_is_lan_address(_adr: NetAdr) -> QBoolean {
    // Always return qtrue - server will believe all IPs are LAN addresses
    QTRUE
}

// Engine-level hook replacement functions
unsafe extern "C" fn my_sv_execute_client_command(cl: *mut Client, s: *mut c_char, client_ok: QBoolean) {
    let mut res = s;
    
    if client_ok != 0 && !cl.is_null() && !(*cl).gentity.is_null() {
        // Calculate client ID from client pointer (need SVS pointer)
        if !SVS.is_null() {
            let client_id = (cl as usize - SVS as usize) / std::mem::size_of::<Client>();
            let _cmd_str = CStr::from_ptr(s).to_string_lossy();
            
            // Call client command dispatcher
            let dispatcher_result = crate::client_command_dispatcher(client_id as c_int, s);
            if dispatcher_result.is_null() {
                // Dispatcher blocked the command
                return;
            }
            res = dispatcher_result;
        }
    }

    // Call original function
    if let Some(ref hook) = SV_EXECUTE_CLIENT_COMMAND_HOOK {
        hook.call(cl, res, client_ok);
    }
}

unsafe extern "C" fn my_sv_send_server_command(cl: *mut Client, fmt: *const c_char, _args: ...) {
    // TODO: Implement proper variadic argument processing with c_variadic feature
    // use std::ffi::VaList;
    // This requires:
    // 1. Using VaList to extract arguments from the variadic parameter list
    // 2. Building a buffer using vsnprintf or similar
    // 3. Calling the server command dispatcher with the formatted buffer
    // 4. Calling the original function with potentially modified arguments
    //
    // For now, we'll implement a basic version that logs the call
    
    let fmt_str = CStr::from_ptr(fmt).to_string_lossy();
    debug_println!("SV_SendServerCommand: fmt={}", fmt_str);
    
    // TODO: Extract variadic arguments using VaList::copy()
    // TODO: Format the string using the arguments
    // TODO: Call dispatcher with formatted buffer
    
    if !cl.is_null() && !SVS.is_null() {
        let client_id = (cl as usize - SVS as usize) / std::mem::size_of::<Client>();
        let dispatcher_result = crate::server_command_dispatcher(client_id as c_int, fmt);
        if dispatcher_result.is_null() {
            // Dispatcher blocked the command
            return;
        }
    } else if cl.is_null() {
        let dispatcher_result = crate::server_command_dispatcher(-1, fmt);
        if dispatcher_result.is_null() {
            return;
        }
    }

    // TODO: Call original function with proper variadic argument forwarding
    // This requires reconstructing the variadic arguments or using a different approach
    debug_println!("TODO: SV_SendServerCommand variadic implementation needed");
}

unsafe extern "C" fn my_sv_client_enter_world(client: *mut Client, cmd: *mut UserCmd) {
    if !client.is_null() {
        let state = (*client).state;
        
        // Call original function first
        if let Some(ref hook) = SV_CLIENT_ENTER_WORLD_HOOK {
            hook.call(client, cmd);
        }

        // Check if this is the first time entering (CS_PRIMED) and gentity is not null
        if !(*client).gentity.is_null() && state == ClientState::Primed && !SVS.is_null() {
            let client_id = (client as usize - SVS as usize) / std::mem::size_of::<Client>();
            let _result = crate::client_loaded_dispatcher(client_id as c_int);
            // Note: Original returned result could block client, but we'll handle that later
        }
    }
}

unsafe extern "C" fn my_sv_set_configstring(index: c_int, value: *const c_char) {
    // Skip indices that are spammed frequently to avoid performance impact
    if index == 16 || (index >= 662 && index < 670) {
        if let Some(ref hook) = SV_SET_CONFIGSTRING_HOOK {
            hook.call(index, value);
        }
        return;
    }

    let value_to_use = if value.is_null() {
        let empty_cstr = std::ffi::CString::new("").unwrap();
        empty_cstr.as_ptr()
    } else {
        value
    };
    
    let dispatcher_result = crate::set_configstring_dispatcher(index, value_to_use);
    
    // Only call original if dispatcher didn't block it (NULL means block)
    if !dispatcher_result.is_null() {
        if let Some(ref hook) = SV_SET_CONFIGSTRING_HOOK {
            hook.call(index, dispatcher_result);
        }
    }
}

unsafe extern "C" fn my_sv_drop_client(drop: *mut Client, reason: *const c_char) {
    if !drop.is_null() && !SVS.is_null() {
        let client_id = (drop as usize - SVS as usize) / std::mem::size_of::<Client>();
        crate::client_disconnect_dispatcher(client_id as c_int, reason);
    }

    // Call original function
    if let Some(ref hook) = SV_DROP_CLIENT_HOOK {
        hook.call(drop, reason);
    }
}

unsafe extern "C" fn my_sv_spawn_server(server: *mut c_char, kill_bots: QBoolean) {
    SKIP_FRAME_DISPATCHER = true;
    
    // Call original function
    if let Some(ref hook) = SV_SPAWN_SERVER_HOOK {
        hook.call(server, kill_bots);
    }
    
    SKIP_FRAME_DISPATCHER = false;

    // Call NewGameDispatcher for non-restart case
    crate::new_game_dispatcher(0); // qfalse = not a restart
}

unsafe extern "C" fn my_com_printf(fmt: *const c_char, _args: ...) {
    // TODO: Implement proper variadic argument processing with c_variadic feature
    // use std::ffi::VaList;
    // This requires:
    // 1. Using VaList to extract arguments from the variadic parameter list  
    // 2. Building a formatted buffer using vsnprintf or similar
    // 3. Calling the console print dispatcher with the formatted buffer
    // 4. Calling the original Com_Printf function with the arguments if not blocked
    //
    // For now, we'll implement a basic version that logs the call
    
    let fmt_str = CStr::from_ptr(fmt).to_string_lossy();
    debug_println!("Com_Printf: fmt={}", fmt_str);
    
    // TODO: Extract variadic arguments using VaList::copy()
    // TODO: Format the string using the arguments (vsnprintf equivalent)
    // TODO: Call dispatcher with formatted buffer
    
    let dispatcher_result = crate::console_print_dispatcher(fmt);
    
    // Only call original if dispatcher didn't block it (NULL means block)
    if !dispatcher_result.is_null() {
        // TODO: Call original Com_Printf with proper variadic argument forwarding
        debug_println!("TODO: Com_Printf variadic implementation needed");
    }
}

// Initialization functions
pub unsafe fn initialize_static() {
    debug_println!("Initializing commands...");

    COMMON_INITIALIZED = true;

    // Set RNG seed
    libc::srand(libc::time(std::ptr::null_mut()) as u32);

    // Add commands
    if let Some(cmd_add_fn) = CMD_ADD_COMMAND {
        let cmd_str = std::ffi::CString::new("cmd").unwrap();
        cmd_add_fn(cmd_str.into_raw(), send_server_command as *mut c_void);

        let cp_str = std::ffi::CString::new("cp").unwrap();
        cmd_add_fn(cp_str.into_raw(), center_print as *mut c_void);

        let print_str = std::ffi::CString::new("print").unwrap();
        cmd_add_fn(print_str.into_raw(), regular_print as *mut c_void);

        let slap_str = std::ffi::CString::new("slap").unwrap();
        cmd_add_fn(slap_str.into_raw(), slap as *mut c_void);

        let slay_str = std::ffi::CString::new("slay").unwrap();
        cmd_add_fn(slay_str.into_raw(), slay as *mut c_void);

        let download_str = std::ffi::CString::new("steam_downloadugcdefer").unwrap();
        cmd_add_fn(
            download_str.into_raw(),
            download_workshop_item as *mut c_void,
        );

        let stop_str = std::ffi::CString::new("stopfollowing").unwrap();
        cmd_add_fn(stop_str.into_raw(), stop_following as *mut c_void);

        // Python command stubs
        let qlx_str = std::ffi::CString::new("qlx").unwrap();
        cmd_add_fn(qlx_str.into_raw(), py_rcon as *mut c_void);

        let pycmd_str = std::ffi::CString::new("pycmd").unwrap();
        cmd_add_fn(pycmd_str.into_raw(), py_command as *mut c_void);

        let pyrestart_str = std::ffi::CString::new("pyrestart").unwrap();
        cmd_add_fn(pyrestart_str.into_raw(), restart_python as *mut c_void);
    }
}

pub unsafe fn initialize_vm() {
    print!(
        "{}Initializing VM pointers...\n",
        crate::common::DEBUG_PRINT_PREFIX
    );
    // TODO: Initialize VM-specific pointers and structures
}

pub unsafe fn initialize_cvars() {
    if let Some(cvar_find_fn) = CVAR_FIND_VAR {
        let maxclients_str = std::ffi::CString::new("sv_maxclients").unwrap();
        SV_MAXCLIENTS = cvar_find_fn(maxclients_str.as_ptr());
    }

    CVARS_INITIALIZED = true;
}



pub unsafe fn hook_vm() {
    debug_println!("Hooking VM functions...");
    // Actual VM hooking is done later after VM functions are found
}

unsafe fn set_tag() {
    // Add minqlxtism tag to server
    if let Some(cvar_find_fn) = CVAR_FIND_VAR {
        if let Some(cbuf_exec_fn) = CBUF_EXECUTE_TEXT {
            let sv_tags_str = std::ffi::CString::new("sv_tags").unwrap();
            let sv_tags = cvar_find_fn(sv_tags_str.as_ptr());

            if !sv_tags.is_null() {
                let current_tags = CStr::from_ptr((*sv_tags).string);
                let current_len = current_tags.to_bytes().len();

                if current_len > 2 {
                    let new_tags = format!(
                        "sv_tags \"{},{}\"",
                        crate::common::SV_TAGS_PREFIX,
                        current_tags.to_string_lossy()
                    );
                    let new_tags_cstr = std::ffi::CString::new(new_tags).unwrap();
                    cbuf_exec_fn(1, new_tags_cstr.as_ptr()); // EXEC_INSERT = 1
                } else {
                    let new_tags = format!("sv_tags \"{}\"", crate::common::SV_TAGS_PREFIX);
                    let new_tags_cstr = std::ffi::CString::new(new_tags).unwrap();
                    cbuf_exec_fn(1, new_tags_cstr.as_ptr()); // EXEC_INSERT = 1
                }
            }
        }
    }
}

// Hook installation functions
pub unsafe fn hook_static() -> Result<(), Box<dyn std::error::Error>> {
    print!("{}Hooking...\n", crate::common::DEBUG_PRINT_PREFIX);

    // Install CMD_ADD_COMMAND hook
    if let Some(cmd_add_fn) = CMD_ADD_COMMAND {
        let detour = GenericDetour::new(cmd_add_fn, my_cmd_add_command)
            .map_err(|e| format!("Failed to create CMD_ADD_COMMAND detour: {}", e))?;
        detour
            .enable()
            .map_err(|e| format!("Failed to enable CMD_ADD_COMMAND detour: {}", e))?;
        CMD_ADD_COMMAND_HOOK = Some(detour);
        print!(
            "{}CMD_ADD_COMMAND hook installed\n",
            crate::common::DEBUG_PRINT_PREFIX
        );
    }

    // Search for and hook SYS_SET_MODULE_OFFSET
    if let Some(addr) = find_sys_setmoduleoffset() {
        let sys_setmod_fn: SysSetModuleOffsetHook = std::mem::transmute(addr);
        let detour = GenericDetour::new(sys_setmod_fn, my_sys_set_module_offset)
            .map_err(|e| format!("Failed to create SYS_SET_MODULE_OFFSET detour: {}", e))?;
        detour
            .enable()
            .map_err(|e| format!("Failed to enable SYS_SET_MODULE_OFFSET detour: {}", e))?;
        SYS_SET_MODULE_OFFSET_HOOK = Some(detour);
        print!(
            "{}SYS_SET_MODULE_OFFSET hook installed\n",
            crate::common::DEBUG_PRINT_PREFIX
        );
    }

    // Search for and hook SYS_IS_LAN_ADDRESS (for bypassing LAN checks)
    if let Some(addr) = find_sys_islanaddress() {
        let sys_lan_fn: SysIsLANAddressHook = std::mem::transmute(addr);
        let detour = GenericDetour::new(sys_lan_fn, my_sys_is_lan_address)
            .map_err(|e| format!("Failed to create SYS_IS_LAN_ADDRESS detour: {}", e))?;
        detour
            .enable()
            .map_err(|e| format!("Failed to enable SYS_IS_LAN_ADDRESS detour: {}", e))?;
        SYS_IS_LAN_ADDRESS_HOOK = Some(detour);
        print!(
            "{}SYS_IS_LAN_ADDRESS hook installed\n",
            crate::common::DEBUG_PRINT_PREFIX
        );
    }

    // Install engine-level hooks for dispatcher support
    
    // SV_ExecuteClientCommand hook
    if let Some(sv_exec_fn) = SV_EXECUTE_CLIENT_COMMAND {
        match GenericDetour::new(sv_exec_fn, my_sv_execute_client_command) {
            Ok(hook) => match hook.enable() {
                Ok(()) => {
                    SV_EXECUTE_CLIENT_COMMAND_HOOK = Some(hook);
                    print!("{}SV_ExecuteClientCommand hook installed\n", crate::common::DEBUG_PRINT_PREFIX);
                }
                Err(e) => {
                    debug_println!("WARNING: Failed to enable SV_ExecuteClientCommand hook: {}", e);
                }
            },
            Err(e) => {
                debug_println!("WARNING: Failed to create SV_ExecuteClientCommand hook: {}", e);
            }
        }
    }

    // SV_ClientEnterWorld hook  
    if let Some(sv_enter_fn) = SV_CLIENT_ENTER_WORLD {
        match GenericDetour::new(sv_enter_fn, my_sv_client_enter_world) {
            Ok(hook) => match hook.enable() {
                Ok(()) => {
                    SV_CLIENT_ENTER_WORLD_HOOK = Some(hook);
                    print!("{}SV_ClientEnterWorld hook installed\n", crate::common::DEBUG_PRINT_PREFIX);
                }
                Err(e) => {
                    debug_println!("WARNING: Failed to enable SV_ClientEnterWorld hook: {}", e);
                }
            },
            Err(e) => {
                debug_println!("WARNING: Failed to create SV_ClientEnterWorld hook: {}", e);
            }
        }
    }

    // SV_SetConfigstring hook
    if let Some(sv_setcs_fn) = SV_SET_CONFIGSTRING {
        match GenericDetour::new(sv_setcs_fn, my_sv_set_configstring) {
            Ok(hook) => match hook.enable() {
                Ok(()) => {
                    SV_SET_CONFIGSTRING_HOOK = Some(hook);
                    print!("{}SV_SetConfigstring hook installed\n", crate::common::DEBUG_PRINT_PREFIX);
                }
                Err(e) => {
                    debug_println!("WARNING: Failed to enable SV_SetConfigstring hook: {}", e);
                }
            },
            Err(e) => {
                debug_println!("WARNING: Failed to create SV_SetConfigstring hook: {}", e);
            }
        }
    }

    // SV_DropClient hook
    if let Some(sv_drop_fn) = SV_DROP_CLIENT {
        match GenericDetour::new(sv_drop_fn, my_sv_drop_client) {
            Ok(hook) => match hook.enable() {
                Ok(()) => {
                    SV_DROP_CLIENT_HOOK = Some(hook);
                    print!("{}SV_DropClient hook installed\n", crate::common::DEBUG_PRINT_PREFIX);
                }
                Err(e) => {
                    debug_println!("WARNING: Failed to enable SV_DropClient hook: {}", e);
                }
            },
            Err(e) => {
                debug_println!("WARNING: Failed to create SV_DropClient hook: {}", e);
            }
        }
    }

    // SV_SpawnServer hook
    if let Some(sv_spawn_fn) = SV_SPAWN_SERVER {
        match GenericDetour::new(sv_spawn_fn, my_sv_spawn_server) {
            Ok(hook) => match hook.enable() {
                Ok(()) => {
                    SV_SPAWN_SERVER_HOOK = Some(hook);
                    print!("{}SV_SpawnServer hook installed\n", crate::common::DEBUG_PRINT_PREFIX);
                }
                Err(e) => {
                    debug_println!("WARNING: Failed to enable SV_SpawnServer hook: {}", e);
                }
            },
            Err(e) => {
                debug_println!("WARNING: Failed to create SV_SpawnServer hook: {}", e);
            }
        }
    }

    // TODO: Enable SV_SendServerCommand hook once variadic implementation is complete
    // if let Some(sv_send_fn) = SV_SEND_SERVER_COMMAND {
    //     match GenericDetour::new(sv_send_fn, my_sv_send_server_command) {
    //         Ok(hook) => match hook.enable() {
    //             Ok(()) => {
    //                 SV_SEND_SERVER_COMMAND_HOOK = Some(hook);
    //                 print!("{}SV_SendServerCommand hook installed\n", crate::common::DEBUG_PRINT_PREFIX);
    //             }
    //             Err(e) => {
    //                 debug_println!("WARNING: Failed to enable SV_SendServerCommand hook: {}", e);
    //             }
    //         },
    //         Err(e) => {
    //             debug_println!("WARNING: Failed to create SV_SendServerCommand hook: {}", e);
    //         }
    //     }
    // }

    // TODO: Enable Com_Printf hook once variadic implementation is complete
    // if let Some(com_printf_fn) = COM_PRINTF {
    //     match GenericDetour::new(com_printf_fn, my_com_printf) {
    //         Ok(hook) => match hook.enable() {
    //             Ok(()) => {
    //                 // Need to add COM_PRINTF_HOOK static variable
    //                 print!("{}Com_Printf hook installed\n", crate::common::DEBUG_PRINT_PREFIX);
    //             }
    //             Err(e) => {
    //                 debug_println!("WARNING: Failed to enable Com_Printf hook: {}", e);
    //             }
    //         },
    //         Err(e) => {
    //             debug_println!("WARNING: Failed to create Com_Printf hook: {}", e);
    //         }
    //     }
    // }

    Ok(())
}

// Find SYS_SET_MODULE_OFFSET using pattern search
unsafe fn find_sys_setmoduleoffset() -> Option<usize> {
    use crate::patterns::find_pattern;
    use crate::quake::patterns;

    if let Some(addr) = find_pattern(patterns::SYS_SETMODULEOFFSET) {
        print!(
            "{}SYS_SET_MODULE_OFFSET: {:p}\n",
            crate::common::DEBUG_PRINT_PREFIX,
            addr as *const ()
        );
        Some(addr)
    } else {
        print!(
            "{}ERROR: Unable to find SYS_SET_MODULE_OFFSET\n",
            crate::common::DEBUG_PRINT_PREFIX
        );
        None
    }
}

// Find SYS_IS_LAN_ADDRESS using pattern search
unsafe fn find_sys_islanaddress() -> Option<usize> {
    use crate::patterns::find_pattern;
    use crate::quake::patterns;

    if let Some(addr) = find_pattern(patterns::SYS_ISLANADDRESS) {
        print!(
            "{}SYS_IS_LAN_ADDRESS: {:p}\n",
            crate::common::DEBUG_PRINT_PREFIX,
            addr as *const ()
        );
        Some(addr)
    } else {
        print!(
            "{}ERROR: Unable to find SYS_IS_LAN_ADDRESS\n",
            crate::common::DEBUG_PRINT_PREFIX
        );
        None
    }
}

// Function pointer types that will be set during pattern searching
pub static mut G_INIT_GAME: Option<unsafe extern "C" fn(c_int, c_int, c_int)> = None;

// Store original VM call table function pointers
static mut ORIGINAL_G_INITGAME: Option<GInitGamePtr> = None;
static mut ORIGINAL_G_RUNFRAME: Option<GRunFramePtr> = None;

// Regular function hooks using retour (for non-VM-call-table functions)
static mut CLIENTCONNECT_HOOK: Option<
    retour::GenericDetour<unsafe extern "C" fn(c_int, QBoolean, QBoolean) -> *mut c_char>,
> = None;
static mut CLIENTSPAWN_HOOK: Option<
    retour::GenericDetour<unsafe extern "C" fn(*mut crate::quake::GEntity)>,
> = None;

// VM hook functions
unsafe extern "C" fn my_g_initgame(level_time: c_int, random_seed: c_int, restart: c_int) {
    debug_println!(
        "G_InitGame called: level_time={}, random_seed={}, restart={}",
        level_time,
        random_seed,
        restart
    );

    if !CVARS_INITIALIZED {
        set_tag();
    }
    initialize_cvars();

    // Call dispatcher for new game events
    if restart != 0 {
        crate::new_game_dispatcher(restart);
    }

    // Call original function
    if let Some(original_fn) = ORIGINAL_G_INITGAME {
        original_fn(level_time, random_seed, restart);
    }

    // VM initialization handling
    debug_println!("VM initialized");
}

unsafe extern "C" fn my_g_runframe(level_time: c_int) {
    // Call frame dispatcher (unless we're skipping frame dispatchers)
    if !SKIP_FRAME_DISPATCHER {
        crate::frame_dispatcher();
    }

    // Call original function
    if let Some(original_fn) = ORIGINAL_G_RUNFRAME {
        original_fn(level_time);
    }
}

unsafe extern "C" fn my_clientconnect(
    client_num: c_int,
    first_time: QBoolean,
    is_bot: QBoolean,
) -> *mut c_char {
    debug_println!(
        "ClientConnect called: client_num={}, first_time={}, is_bot={}",
        client_num,
        first_time,
        is_bot
    );

    // Call dispatcher if it's the first time connecting
    if first_time != 0 {
        let dispatcher_result = crate::client_connect_dispatcher(client_num, is_bot);
        if !dispatcher_result.is_null() && is_bot == 0 {
            // Dispatcher returned rejection message for non-bot
            return dispatcher_result;
        }
    }

    // Call original function
    if let Some(hook) = &CLIENTCONNECT_HOOK {
        let result = hook.call(client_num, first_time, is_bot);
        return result;
    }

    std::ptr::null_mut()
}

unsafe extern "C" fn my_clientspawn(ent: *mut crate::quake::GEntity) {
    debug_println!("ClientSpawn called: ent={:p}", ent);

    // Call original function first
    if let Some(hook) = &CLIENTSPAWN_HOOK {
        hook.call(ent);
    }

    // Call dispatcher after original function (to allow weapon setup, etc.)
    // Calculate client_id from entity pointer
    if !ent.is_null() && !crate::quake::G_ENTITIES.is_null() {
        let client_id = (ent as usize - crate::quake::G_ENTITIES as usize) 
            / std::mem::size_of::<crate::quake::GEntity>();
        crate::client_spawn_dispatcher(client_id as c_int);
    }
}

// Install VM function hooks
pub unsafe fn install_vm_hooks() -> Result<(), Box<dyn std::error::Error>> {
    use crate::quake::*;

    debug_println!("Installing VM hooks...");

    // G_InitGame and G_RunFrame are in the VM call table, which is a table of pointers.
    // So no fancy detouring required. Just replace the address.

    // This must be done after Sys_SetModuleOffset so that we have QAGAME_DLLENTRY set.
    if QAGAME_DLLENTRY.is_null() {
        debug_println!("ERROR: QAGAME_DLLENTRY is null, cannot install VM hooks");
        return Err("QAGAME_DLLENTRY not set".into());
    }

    // Calculate VM call table address (same as in patterns.rs)
    const RELOFFSET_VM_CALL_INITGAME: usize = 0x18;
    const RELOFFSET_VM_CALL_RUNFRAME: usize = 0x8;

    #[cfg(target_arch = "x86_64")]
    let vm_call_table: usize = {
        let offset_relp_vm_call_table = QAGAME_DLLENTRY.offset(0x3) as usize;
        let rel_offset = (offset_relp_vm_call_table as *const i32).read_unaligned() as i32;
        (offset_relp_vm_call_table as i64 + rel_offset as i64 + 4) as usize
    };

    #[cfg(target_arch = "x86")]
    let vm_call_table: usize = {
        let offset_relp_vm_call_table = QAGAME_DLLENTRY.offset(0x11) as usize;
        let rel_offset = (offset_relp_vm_call_table as *const i32).read_unaligned() as i32;
        (rel_offset as i64 + 0xCEFF4 + QAGAME as usize as i64) as usize
    };

    let initgame_table_addr = vm_call_table + RELOFFSET_VM_CALL_INITGAME;
    let runframe_table_addr = vm_call_table + RELOFFSET_VM_CALL_RUNFRAME;

    debug_println!("VM call table at: {:#x}", vm_call_table);
    debug_println!("G_InitGame table entry at: {:#x}", initgame_table_addr);
    debug_println!("G_RunFrame table entry at: {:#x}", runframe_table_addr);

    // Store original function pointers
    ORIGINAL_G_INITGAME = Some(*(initgame_table_addr as *const GInitGamePtr));
    ORIGINAL_G_RUNFRAME = Some(*(runframe_table_addr as *const GRunFramePtr));

    debug_println!(
        "Original G_InitGame: {:p}",
        ORIGINAL_G_INITGAME.unwrap() as *const ()
    );
    debug_println!(
        "Original G_RunFrame: {:p}",
        ORIGINAL_G_RUNFRAME.unwrap() as *const ()
    );

    // Replace table entries with our hooks
    *(initgame_table_addr as *mut GInitGamePtr) = my_g_initgame;
    *(runframe_table_addr as *mut GRunFramePtr) = my_g_runframe;

    debug_println!("G_InitGame call table entry replaced with hook");
    debug_println!("G_RunFrame call table entry replaced with hook");

    // The rest we hook like normal...

    // Hook ClientConnect
    if let Some(clientconnect_ptr) = CLIENTCONNECT {
        debug_println!(
            "Attempting to hook ClientConnect at {:p}",
            clientconnect_ptr as *const ()
        );
        match retour::GenericDetour::new(clientconnect_ptr, my_clientconnect) {
            Ok(hook) => match hook.enable() {
                Ok(()) => {
                    CLIENTCONNECT_HOOK = Some(hook);
                    debug_println!("ClientConnect hook installed");
                }
                Err(e) => {
                    debug_println!("WARNING: Failed to enable ClientConnect hook: {}", e);
                }
            },
            Err(e) => {
                debug_println!("WARNING: Failed to create ClientConnect hook: {}", e);
            }
        }
    } else {
        debug_println!("WARNING: ClientConnect not found, skipping hook");
    }

    // Hook ClientSpawn
    if let Some(clientspawn_ptr) = CLIENTSPAWN {
        debug_println!(
            "Attempting to hook ClientSpawn at {:p}",
            clientspawn_ptr as *const ()
        );
        match retour::GenericDetour::new(clientspawn_ptr, my_clientspawn) {
            Ok(hook) => match hook.enable() {
                Ok(()) => {
                    CLIENTSPAWN_HOOK = Some(hook);
                    debug_println!("ClientSpawn hook installed");
                }
                Err(e) => {
                    debug_println!("WARNING: Failed to enable ClientSpawn hook: {}", e);
                }
            },
            Err(e) => {
                debug_println!("WARNING: Failed to create ClientSpawn hook: {}", e);
            }
        }
    } else {
        debug_println!("WARNING: ClientSpawn not found, skipping hook");
    }

    debug_println!("VM hooks installed successfully");
    Ok(())
}

// Memory patching functionality (from patches.c)
use libc::{mprotect, sysconf, PROT_EXEC, PROT_READ, PROT_WRITE, _SC_PAGESIZE};

// Apply memory patch using pattern and mask
unsafe fn patch_by_mask(
    offset: usize,
    pattern: &[u8],
    mask: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let page_size = sysconf(_SC_PAGESIZE);
    if page_size == -1 {
        return Err("Failed to get page size".into());
    }

    let aligned_offset = offset & !(page_size as usize - 1);
    let result = mprotect(
        aligned_offset as *mut libc::c_void,
        page_size as usize,
        PROT_READ | PROT_WRITE | PROT_EXEC,
    );

    if result != 0 {
        return Err(format!("mprotect failed: {}", std::io::Error::last_os_error()).into());
    }

    // Apply patch bytes where mask indicates 'X'
    for (i, &byte) in pattern.iter().enumerate() {
        if i < mask.len() {
            if let Some(mask_char) = mask.chars().nth(i) {
                if mask_char == 'X' {
                    *(offset.wrapping_add(i) as *mut u8) = byte;
                }
            }
        }
    }

    Ok(())
}

// Vote clientkick fix implementation
unsafe fn vote_clientkick_fix() -> Result<(), Box<dyn std::error::Error>> {
    use crate::patterns::find_pattern_in_module;
    use crate::quake::patterns;
    use crate::quake::*;

    debug_println!("Applying vote clientkick fix...");

    // Find Cmd_CallVote_f in qagame module
    let callvote_addr = if let Some(addr) = find_pattern_in_module(
        QAGAME_MODULE_BASE,
        QAGAME_MODULE_SIZE,
        patterns::CMD_CALLVOTE_F,
    ) {
        addr
    } else {
        debug_println!(
            "WARNING: Unable to find Cmd_CallVote_f. Skipping callvote-clientkick patch..."
        );
        return Ok(());
    };

    // Calculate patch address (offset from Cmd_CallVote_f)
    #[cfg(target_arch = "x86_64")]
    let patch_offset = 0x11C8;
    #[cfg(target_arch = "x86")]
    let patch_offset = 0x0F8C;

    let patch_addr = callvote_addr + patch_offset;

    // Patch bytes and mask
    #[cfg(target_arch = "x86_64")]
    let pattern_bytes: &[u8] = &[
        0x39, 0xFE, 0x0F, 0x8D, 0x90, 0x00, 0x00, 0x00, 0x48, 0x69, 0xD6, 0xF8, 0x0B, 0x00, 0x00,
        0x48, 0x01, 0xD0, 0x90, 0x90, 0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x85, 0x76, 0x00, 0x00, 0x00, 0x90, 0x90, 0x90, 0x90,
    ];
    #[cfg(target_arch = "x86_64")]
    let mask = "XXXXXXXXXXXXXXXXXXXXX-------XXXXXXXXXX";

    #[cfg(target_arch = "x86")]
    let pattern_bytes: &[u8] = &[
        0x69, 0xc8, 0xd0, 0x0b, 0x00, 0x00, 0x01, 0xca, 0x90, 0x00, 0x44, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6c, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90,
        0x90, 0x90,
    ];
    #[cfg(target_arch = "x86")]
    let mask = "XXXXXXXXX-X------------XXXXXXXXX";

    match patch_by_mask(patch_addr, pattern_bytes, mask) {
        Ok(()) => {
            debug_println!("Vote clientkick fix applied successfully");
        }
        Err(e) => {
            debug_println!("WARNING: Failed to apply vote clientkick fix: {}", e);
        }
    }

    Ok(())
}

// Main patch_vm function
pub unsafe fn patch_vm() -> Result<(), Box<dyn std::error::Error>> {
    debug_println!("Applying VM patches...");

    // Apply vote clientkick fix
    vote_clientkick_fix()?;

    debug_println!("VM patches applied successfully");
    Ok(())
}
