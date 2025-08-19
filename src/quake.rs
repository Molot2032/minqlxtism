// Quake Live data structures and constants
// Converted from quake_common.h

use crate::common::PInt;
use std::os::raw::{c_char, c_float, c_int, c_uchar, c_void};

// Constants
pub const MAX_CLIENTS: i32 = 64;
pub const MAX_CHALLENGES: i32 = 1024;
pub const MAX_MSGLEN: i32 = 16384;
pub const MAX_PS_EVENTS: i32 = 2;
pub const MAX_MAP_AREA_BYTES: i32 = 32;
pub const MAX_INFO_STRING: i32 = 1024;
pub const MAX_RELIABLE_COMMANDS: i32 = 64;
pub const MAX_STRING_CHARS: i32 = 1024;
pub const MAX_NAME_LENGTH: i32 = 32;
pub const MAX_QPATH: i32 = 64;
pub const MAX_DOWNLOAD_WINDOW: i32 = 8;
pub const MAX_NETNAME: i32 = 36;
pub const PACKET_BACKUP: i32 = 32;
pub const PACKET_MASK: i32 = PACKET_BACKUP - 1;
pub const MAX_ENT_CLUSTERS: i32 = 16;
pub const MAX_MODELS: i32 = 256;
pub const MAX_SOUNDS: i32 = 256;
pub const MAX_LOCATIONS: i32 = 64;
pub const MAX_CONFIGSTRINGS: i32 = 1024;
pub const GENTITYNUM_BITS: i32 = 10;
pub const MAX_GENTITIES: i32 = 1 << GENTITYNUM_BITS;
pub const ENTITYNUM_NONE: i32 = MAX_GENTITIES - 1;
pub const ENTITYNUM_WORLD: i32 = MAX_GENTITIES - 2;
pub const ENTITYNUM_MAX_NORMAL: i32 = MAX_GENTITIES - 2;
pub const MAX_ITEM_MODELS: i32 = 4;
pub const MAX_SPAWN_VARS: i32 = 64;
pub const MAX_SPAWN_VARS_CHARS: i32 = 4096;
pub const BODY_QUEUE_SIZE: i32 = 8;

// Basic types
pub type QBoolean = c_int; // 0 = qfalse, 1 = qtrue
pub const QFALSE: QBoolean = 0;
pub const QTRUE: QBoolean = 1;

pub type Byte = c_uchar;
pub type Vec_t = c_float;
pub type Vec2_t = [Vec_t; 2];
pub type Vec3_t = [Vec_t; 3];
pub type Vec4_t = [Vec_t; 4];
pub type Vec5_t = [Vec_t; 5];

pub type FileHandle_t = c_int;

// Enums converted to constants/types
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Privileges {
    Banned = 0xFFFFFFFF,
    None = 0x0,
    Mod = 0x1,
    Admin = 0x2,
    Root = 0x3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ClientState {
    Free = 0,
    Zombie = 1,
    Connected = 2,
    Primed = 3,
    Active = 4,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Team {
    Free = 0,
    Red = 1,
    Blue = 2,
    Spectator = 3,
    NumTeams = 4,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SpectatorState {
    Not = 0,
    Free = 1,
    Follow = 2,
    Scoreboard = 3,
}

// Network address structure
#[repr(C)]
#[derive(Debug, Clone)]
pub struct NetAdr {
    pub addr_type: c_int,
    pub ip: [Byte; 4],
    pub ipx: [Byte; 10],
    pub port: u16,
}

// Basic structures (simplified for initial compilation)
#[repr(C)]
pub struct Cvar {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub reset_string: *mut c_char,
    pub latched_string: *mut c_char,
    pub default_string: *mut c_char,
    pub minimum_string: *mut c_char,
    pub maximum_string: *mut c_char,
    pub flags: c_int,
    pub modified: QBoolean,
    pub _unknown2: [u8; 4],
    pub modification_count: c_int,
    pub value: c_float,
    pub integer: c_int,
    pub _unknown3: [u8; 8],
    pub next: *mut Cvar,
    pub hash_next: *mut Cvar,
}

// User command structure  
#[repr(C)]
pub struct UserCmd {
    pub server_time: c_int,
    pub angles: [c_int; 3],
    pub buttons: c_int,
    pub weapon: c_uchar,
    pub weapon_primary: c_uchar,
    pub fov: c_uchar,
    pub forwardmove: c_char,
    pub rightmove: c_char,
    pub upmove: c_char,
}

// Client structure (from hooks.c usage)
#[repr(C)]
pub struct Client {
    pub state: ClientState,
    pub userinfo: [c_char; MAX_INFO_STRING as usize],
    pub reliable_commands: [[c_char; MAX_STRING_CHARS as usize]; MAX_RELIABLE_COMMANDS as usize],
    pub reliable_sequence: c_int,
    pub reliable_acknowledge: c_int,
    pub reliable_sent: c_int,
    pub message_acknowledge: c_int,
    pub gamestate_message_num: c_int,
    pub challenge: c_int,
    pub last_usercmd: UserCmd,
    pub last_message_num: c_int,
    pub last_client_command: c_int,
    pub last_client_command_string: [c_char; MAX_STRING_CHARS as usize],
    pub gentity: *mut GEntity,
    pub name: [c_char; MAX_NAME_LENGTH as usize],
    // Rest of structure (download data, networking, etc.)
    // Using large padding to match actual size
    #[cfg(target_arch = "x86_64")]
    pub _unknown: [u8; 36808],
    #[cfg(target_arch = "x86")]
    pub _unknown: [u8; 36836],
    pub steam_id: u64,
}

// Forward declarations for complex structures
pub struct GEntity;
pub struct GClient;
pub struct ServerStatic;
pub struct LevelLocals;
pub struct GItem;

// Function pointer types for Quake functions
pub type ComPrintfPtr = unsafe extern "C" fn(*const c_char, ...);
pub type CmdAddCommandPtr = unsafe extern "C" fn(*mut c_char, *mut c_void);
pub type CmdArgsPtr = unsafe extern "C" fn() -> *mut c_char;
pub type CmdArgvPtr = unsafe extern "C" fn(c_int) -> *mut c_char;
pub type CmdArgcPtr = unsafe extern "C" fn() -> c_int;
pub type CmdTokenizeStringPtr = unsafe extern "C" fn(*const c_char);
pub type CbufExecuteTextPtr = unsafe extern "C" fn(c_int, *const c_char);
pub type CvarFindVarPtr = unsafe extern "C" fn(*const c_char) -> *mut Cvar;
pub type CvarGetPtr = unsafe extern "C" fn(*const c_char, *const c_char, c_int) -> *mut Cvar;
pub type CvarGetLimitPtr = unsafe extern "C" fn(
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    c_int,
) -> *mut Cvar;
pub type CvarSet2Ptr = unsafe extern "C" fn(*const c_char, *const c_char, QBoolean) -> *mut Cvar;

// Engine-level function pointer types for dispatcher hooks
pub type SVExecuteClientCommandPtr = unsafe extern "C" fn(*mut Client, *mut c_char, QBoolean);
pub type SVSendServerCommandPtr = unsafe extern "C" fn(*mut Client, *const c_char, ...);
pub type SVClientEnterWorldPtr = unsafe extern "C" fn(*mut Client, *mut UserCmd);
pub type SVSetConfigstringPtr = unsafe extern "C" fn(c_int, *const c_char);
pub type SVDropClientPtr = unsafe extern "C" fn(*mut Client, *const c_char);
pub type SVSpawnServerPtr = unsafe extern "C" fn(*mut c_char, QBoolean);

// Global variables (will be initialized by pattern search)
pub static mut COM_PRINTF: Option<ComPrintfPtr> = None;
pub static mut CMD_ADD_COMMAND: Option<CmdAddCommandPtr> = None;
pub static mut CMD_ARGS: Option<CmdArgsPtr> = None;
pub static mut CMD_ARGV: Option<CmdArgvPtr> = None;
pub static mut CMD_ARGC: Option<CmdArgcPtr> = None;
pub static mut CMD_TOKENIZE_STRING: Option<CmdTokenizeStringPtr> = None;
pub static mut CBUF_EXECUTE_TEXT: Option<CbufExecuteTextPtr> = None;
pub static mut CVAR_FIND_VAR: Option<CvarFindVarPtr> = None;
pub static mut CVAR_GET: Option<CvarGetPtr> = None;
pub static mut CVAR_GET_LIMIT: Option<CvarGetLimitPtr> = None;
pub static mut CVAR_SET2: Option<CvarSet2Ptr> = None;

// Engine-level function pointers for dispatcher hooks
pub static mut SV_EXECUTE_CLIENT_COMMAND: Option<SVExecuteClientCommandPtr> = None;
pub static mut SV_SEND_SERVER_COMMAND: Option<SVSendServerCommandPtr> = None; 
pub static mut SV_CLIENT_ENTER_WORLD: Option<SVClientEnterWorldPtr> = None;
pub static mut SV_SET_CONFIGSTRING: Option<SVSetConfigstringPtr> = None;
pub static mut SV_DROP_CLIENT: Option<SVDropClientPtr> = None;
pub static mut SV_SPAWN_SERVER: Option<SVSpawnServerPtr> = None;

// Global structure pointers
pub static mut SVS: *mut ServerStatic = std::ptr::null_mut();
pub static mut G_ENTITIES: *mut GEntity = std::ptr::null_mut();
pub static mut LEVEL: *mut LevelLocals = std::ptr::null_mut();
pub static mut BG_ITEMLIST: *mut GItem = std::ptr::null_mut();
pub static mut BG_NUM_ITEMS: c_int = 0;

// Cvars
pub static mut SV_MAXCLIENTS: *mut Cvar = std::ptr::null_mut();

// VM Function types
pub type GInitGamePtr = unsafe extern "C" fn(c_int, c_int, c_int);
pub type GRunFramePtr = unsafe extern "C" fn(c_int);
pub type ClientConnectPtr = unsafe extern "C" fn(c_int, QBoolean, QBoolean) -> *mut c_char;
pub type ClientSpawnPtr = unsafe extern "C" fn(*mut GEntity);
pub type GDamagePtr = unsafe extern "C" fn(
    *mut GEntity,
    *mut GEntity,
    *mut GEntity,
    *mut Vec3_t,
    *mut Vec3_t,
    c_int,
    c_int,
    c_int,
);
pub type TouchItemPtr = unsafe extern "C" fn(*mut GEntity, *mut GEntity, *mut c_void);
pub type CmdCallVoteFPtr = unsafe extern "C" fn(*mut GEntity);

// VM function pointers (will be found by pattern search)
pub static mut G_INITGAME: Option<GInitGamePtr> = None;
pub static mut G_RUNFRAME: Option<GRunFramePtr> = None;
pub static mut CLIENTCONNECT: Option<ClientConnectPtr> = None;
pub static mut CLIENTSPAWN: Option<ClientSpawnPtr> = None;
pub static mut G_DAMAGE: Option<GDamagePtr> = None;
pub static mut TOUCH_ITEM: Option<TouchItemPtr> = None;
pub static mut CMD_CALLVOTE_F: Option<CmdCallVoteFPtr> = None;

// VM module info
pub static mut QAGAME_MODULE_BASE: usize = 0;
pub static mut QAGAME_MODULE_SIZE: usize = 0;

// Patterns for x86_64 - pre-converted to patternscan format
#[cfg(target_arch = "x86_64")]
pub mod patterns {
    // Engine functions
    pub const COM_PRINTF: &str = "41 54 55 53 48 81 EC ? ? ? ? 84 C0 48 89 B4 24 ? ? ? ? 48 89 94 24 ? ? ? ? 48 89 8C 24 ? ? ? ? 4C 89 84 24 ? ? ? ?";

    pub const CMD_ADDCOMMAND: &str = "41 55 49 89 F5 41 54 49 89 FC 55 53 48 83 EC ? 48 8B 1D ? ? ? ? 48 85 DB 75 ? EB ? 66 90 48 8B 1B 48 85 DB 74 ? 48 8B 73 ? 4C 89 E7";

    pub const CMD_ARGS: &str = "8B 05 ? ? ? ? C6 05 ? ? ? ? ? 83 F8 ? 0F 8E ? ? ? ? 41 54 44 8D 60 ? 83 E8 ? 55 48 8D 68 ? 53 31 DB 66 0F 1F 84 ? ? ? ? ?";

    pub const CMD_ARGV: &str = "3B 3D ? ? ? ? B8 ? ? ? ? 73 ? 48 63 FF 48 8B 04 FD ? ? ? ? F3 C3";

    pub const CMD_ARGC: &str = "8B 05 ? ? ? ? C3";

    pub const CMD_TOKENIZESTRING: &str = "48 85 FF 53 C7 05 ? ? 44 ? ? ? ? ? 48 89 FB 0F 84 ? ? ? ? 48 89 FE BA ? ? ? ? BF ? ? ? ? E8 ? ? ? ? 8B 0D ? ? ? ?";

    pub const CBUF_EXECUTETEXT: &str = "83 FF ? 74 ? 83 FF ? 74 ? 85 FF 74 ? BE ? ? ? ? 31 FF 31 C0 E9 ? ? ? ? 0F 1F 40 ? 48 85 F6 74 ? 80 3E ? 75 ? E9 ? ? ? ? 90";

    pub const CVAR_FINDVAR: &str = "55 48 89 FD 53 48 83 EC ? E8 ? ? ? ? 48 8B 1C C5 ? ? ? ? 48 85 DB 75 ? EB ? 0F 1F ? 48 8B 5B ? 48 85 DB 74 ? 48 8B 33 48 89 EF";

    pub const CVAR_GET: &str = "41 56 48 85 FF 41 55 41 89 D5 41 54 49 89 F4 55 48 89 FD 53 0F 84 ? ? ? ? 48 85 F6 0F 84 ? ? ? ? 48 89 EF E8 ? ? ? ? 85 C0";

    pub const CVAR_GETLIMIT: &str = "41 57 45 89 C7 41 56 49 89 D6 41 55 49 89 CD 41 54 49 89 F4 31 F6 55 48 89 FD 48 89 D7 53 48 83 EC ? E8 ? ? ? ? 66 0F 14 C0 31 F6 4C 89 EF";

    pub const CVAR_SET2: &str = "41 57 31 C0 41 56 41 89 D6 48 89 F2 41 55 41 54 49 89 F4 48 89 FE 55 48 89 FD BF ? ? ? ? 53 48 83 EC ? E8 ? ? ? ? 48 89 EF E8 ? ? ? ?";

    pub const SYS_SETMODULEOFFSET: &str = "55 48 89 F2 31 C0 48 89 F5 48 89 FE 53 48 89 FB BF ? ? ? ? 48 83 EC ? E8 ? ? ? ? BF ? ? ? ? B9 ? ? ? ? 48 89 DE F3 A6 74 ?";

    pub const SYS_ISLANADDRESS: &str =
        "8B 4C 24 ? 0F B6 54 24 ? 0F B6 74 24 ? 83 F9 ? 74 ? 31 C0 83 F9 ? 74 ? F3 C3";

    // Engine-level functions for dispatcher hooks (converted from minqlx patterns.h)
    
    // From PTRN_SV_EXECUTECLIENTCOMMAND + MASK_SV_EXECUTECLIENTCOMMAND 
    pub const SV_EXECUTECLIENTCOMMAND: &str = "41 55 41 89 D5 41 54 49 89 FC 48 89 F7 55 BD ?? ?? ?? ?? 53 48 83 EC ?? E8 ?? ?? ?? ?? 48 8B 1D ?? ?? ?? ?? 48 85 DB 75 ?? E9 ?? ?? ?? ?? 66 90";
    
    // From PTRN_SV_SENDSERVERCOMMAND + MASK_SV_SENDSERVERCOMMAND
    pub const SV_SENDSERVERCOMMAND: &str = "41 55 41 54 55 48 89 FD 53 48 81 EC ?? ?? ?? ?? 84 C0 48 89 94 24 ?? ?? ?? ?? 48 89 8C 24 ?? ?? ?? ?? 4C 89 84 24 ?? ?? ?? ?? 4C 89 8C 24 ?? ?? ?? ??";
    
    // From PTRN_SV_CLIENTENTERWORLD + MASK_SV_CLIENTENTERWORLD
    pub const SV_CLIENTENTERWORLD: &str = "41 55 31 C0 49 BD ?? ?? ?? ?? ?? ?? ?? ?? 41 54 49 89 F4 48 8D B7 ?? ?? ?? ?? 55 53 48 89 FB BF ?? ?? ?? ?? 48 89 DD 48 83 EC ?? E8 ?? ?? ?? ??";
    
    // From PTRN_SV_SETCONFIGSTRING + MASK_SV_SETCONFIGSTRING
    pub const SV_SETCONFIGSTRING: &str = "41 57 41 56 41 55 41 54 41 89 FC 55 53 48 81 EC ?? ?? ?? ?? 64 48 8B 04 25 ?? ?? ?? ?? 48 89 84 24 ?? ?? ?? ?? 31 C0 81 FF ?? ?? ?? ?? 48 89 74 24 ??";
    
    // From PTRN_SV_DROPCLIENT + MASK_SV_DROPCLIENT
    pub const SV_DROPCLIENT: &str = "41 54 55 48 89 FD 53 48 83 EC ?? 83 3F ?? 0F 84 ?? ?? ?? ?? 48 8B 87 ?? ?? ?? ?? 49 89 F4 48 85 C0 74 ?? F6 80 E0 01 00 00 00 75 ?? BB ?? ?? ?? ??";
    
    // From PTRN_SV_SPAWNSERVER + MASK_SV_SPAWNSERVER  
    pub const SV_SPAWNSERVER: &str = "41 55 41 54 41 89 F4 55 48 89 FD 53 48 81 EC ?? ?? ?? ?? 64 48 8B 04 25 ?? ?? ?? ?? 48 89 84 24 ?? ?? ?? ?? 31 C0 E8 ?? ?? ?? ?? 31 C0 BF ?? ?? ?? ??";

    // VM/qagame functions
    pub const _G_RUNFRAME: &str = "8B 05 ? ? ? ? 85 C0 74 ? F3 C3";

    pub const G_ADDEVENT: &str = "85 F6 74 ? 48 8B 8F ? ? ? ? 48 85 C9 74 ? 8B 81 ? ? ? ? 25 ? ? ? ? 05 ? ? ? ? 25 ? ? ? ? 09 F0 89 81 ? ? ? ?";

    pub const _G_INITGAME: &str = "41 54 55 53 48 81 EC ? ? ? ? 84 C0 48 89 B4 24 ? ? ? ? 48 89 94 24 ? ? ? ? 48 89 8C 24 ? ? ? ? 4C 89 84 24 ? ? ? ?";

    pub const CLIENTCONNECT: &str = "41 57 4C 63 FF 41 56 41 89 F6 41 55 41 54 55 4C 89 FD 48 C1 E5 ? 53 89 FB 48 81 EC ? ? ? ? 4C 8B 2D ? ? ? ? 64 48 8B 04 25 ? ? ? ?";

    pub const CLIENTSPAWN: &str = "41 57 41 56 49 89 FE 41 55 41 54 55 53 48 81 EC ? ? ? ? 4C 8B BF ? ? ? ? 64 48 8B 04 25 ? ? ? ? 48 89 84 24 ? ? ? ? 31 C0";

    pub const G_DAMAGE: &str = "41 57 41 56 41 55 41 54 55 53 48 89 FB 48 81 EC ? ? ? ? 44 8B 97 ? ? ? ? 48 8B AF ? ? ? ? 64 48 8B 04 25 ? ? ? ?";

    pub const TOUCH_ITEM: &str = "41 57 41 56 41 55 41 54 55 53 48 89 F3 48 81 EC ? ? ? ? 4C 8B 86 ? ? ? ? 4D 85 C0 74 ? 8B 96 ? ? ? ? 85 D2 7E ? 4C 8B 35 ? ? ? ?";

    pub const CMD_CALLVOTE_F: &str = "41 57 41 56 41 55 41 54 55 48 89 FD 53 48 81 EC ? ? ? ? 64 48 8B 04 25 ? ? ? ? 48 89 84 24 ? ? ? ? 31 C0 E8 ? ? ? ?";
}

// VM Call table offsets
pub const RELOFFSET_VM_CALL_INITGAME: PInt = 0x18;
pub const RELOFFSET_VM_CALL_RUNFRAME: PInt = 0x8;
