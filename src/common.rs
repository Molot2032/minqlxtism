use std::os::raw::{c_char, c_int, c_void};

// Version definition - will be set by build script
pub const MINQLXTISM_VERSION: &str = env!("CARGO_PKG_VERSION");

// Debug macros
pub const DEBUG_PRINT_PREFIX: &str = "[minqlxtism] ";
pub const DEBUG_ERROR_FORMAT: &str = "[minqlxtism] ERROR @ {}:{} in {}:\n[minqlxtism] ";

// Tags prefix for server info
pub const SV_TAGS_PREFIX: &str = "minqlxtism";

// Architecture-specific types
#[cfg(target_arch = "x86_64")]
pub type PInt = u64;
#[cfg(target_arch = "x86_64")]
pub type SInt = i64;

#[cfg(target_arch = "x86")]
pub type PInt = u32;
#[cfg(target_arch = "x86")]
pub type SInt = i32;

// Global state variables
pub static mut COMMON_INITIALIZED: bool = false;
pub static mut CVARS_INITIALIZED: bool = false;

// Function type definitions for external functions we'll need to call
pub type PatternSearchFn =
    unsafe extern "C" fn(*mut c_void, usize, *const c_char, *const c_char) -> *mut c_void;
pub type PatternSearchModuleFn =
    unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> *mut c_void;
