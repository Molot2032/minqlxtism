// Utility functions converted from misc.c
use crate::common::PInt;
use libc::{rand, size_t, RAND_MAX};
use std::os::raw::{c_char, c_int, c_void};

// Permission flags for module parsing
pub const PG_READ: i32 = 1;
pub const PG_WRITE: i32 = 2;
pub const PG_EXECUTE: i32 = 4;
pub const PG_PRIVATE: i32 = 8;
pub const PG_SHARED: i32 = 16;

// Module info structure
#[repr(C)]
pub struct ModuleInfo {
    pub name: [c_char; 512],
    pub path: [c_char; 4096],
    pub entries: c_int,
    pub permissions: [c_int; 128],
    pub address_start: [PInt; 128],
    pub address_end: [PInt; 128],
}

// Bit manipulation for player flags
pub fn get_pending_player(players: &mut u64) -> i32 {
    if *players == 0 {
        return -1;
    }

    for id in 0..64 {
        let flag = *players & (1u64 << id);
        *players &= !flag;
        if flag != 0 {
            return id;
        }
    }

    -1
}

pub fn set_pending_player(players: &mut u64, client_id: i32) {
    *players |= 1u64 << client_id;
}

// Random number generation
pub fn random_float() -> f32 {
    unsafe { (rand() as f32) / (RAND_MAX as f32) }
}

pub fn random_float_with_negative() -> f32 {
    unsafe { (rand() as f32) / ((RAND_MAX / 2) as f32) - 1.0 }
}

// Pattern searching functions (will be replaced by patternscan crate later)
pub unsafe fn pattern_search(
    address: *mut c_void,
    length: size_t,
    pattern: *const c_char,
    mask: *const c_char,
) -> *mut c_void {
    let addr_bytes = address as *const u8;
    let pattern_bytes = pattern as *const u8;
    let mask_str = std::ffi::CStr::from_ptr(mask);
    let mask_bytes = mask_str.to_bytes();

    for i in 0..length {
        let mut found = true;
        for j in 0..mask_bytes.len() {
            if mask_bytes[j] == b'X' {
                if *pattern_bytes.add(j) != *addr_bytes.add(i + j) {
                    found = false;
                    break;
                }
            }
        }

        if found {
            return (addr_bytes.add(i)) as *mut c_void;
        }
    }

    std::ptr::null_mut()
}

pub unsafe fn pattern_search_module(
    module: *mut ModuleInfo,
    pattern: *const c_char,
    mask: *const c_char,
) -> *mut c_void {
    let module_ref = &*module;

    for i in 0..module_ref.entries as usize {
        if module_ref.permissions[i] & PG_READ == 0 {
            continue;
        }

        let size = module_ref.address_end[i] - module_ref.address_start[i];
        let result = pattern_search(
            module_ref.address_start[i] as *mut c_void,
            size as size_t,
            pattern,
            mask,
        );

        if !result.is_null() {
            return result;
        }
    }

    std::ptr::null_mut()
}

// Stub for GetModuleInfo - will need to be implemented for Linux /proc/self/maps parsing
extern "C" {
    pub fn get_module_info(module_info: *mut ModuleInfo) -> c_int;
}
