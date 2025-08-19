// Pattern scanning using patternscan crate
use patternscan::scan_first_match;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::{debug_println, QAGAME, QAGAME_DLLENTRY};

#[derive(Debug, Clone)]
pub struct MapRegion {
    pub start: usize,
    pub end: usize,
    pub perms: String,
    pub path: Option<String>,
}

// Parse /proc/self/maps to get memory regions
pub fn parse_maps() -> Result<Vec<MapRegion>, std::io::Error> {
    let file = File::open("/proc/self/maps")?;
    let mut regions = Vec::new();

    for line in BufReader::new(file).lines() {
        let line = line?;
        let mut parts = line.split_whitespace();

        let range = parts.next().unwrap_or("");
        let perms = parts.next().unwrap_or("").to_string();
        let _ = parts.next(); // offset
        let _ = parts.next(); // dev
        let _ = parts.next(); // inode
        let path = parts.next().map(|s| s.to_string());

        if let Some((start_str, end_str)) = range.split_once('-') {
            if let (Ok(start), Ok(end)) = (
                usize::from_str_radix(start_str, 16),
                usize::from_str_radix(end_str, 16),
            ) {
                regions.push(MapRegion {
                    start,
                    end,
                    perms,
                    path,
                });
            }
        }
    }

    Ok(regions)
}

// Get executable regions excluding our own library
pub fn get_executable_regions() -> Result<Vec<MapRegion>, std::io::Error> {
    let regions = parse_maps()?;
    const SELF_SUBSTR: &str = "minqlxtism"; // substring of our .so name

    Ok(regions
        .into_iter()
        .filter(|r| r.perms.starts_with("r-x")) // readable + executable
        .filter(|r| {
            r.path
                .as_deref()
                .map(|p| !p.contains(SELF_SUBSTR))
                .unwrap_or(true)
        })
        .collect())
}

// Find pattern using patternscan crate in executable memory regions
pub unsafe fn find_pattern(pattern_str: &str) -> Option<usize> {
    let regions = match get_executable_regions() {
        Ok(r) => r,
        Err(_) => return None,
    };

    for region in regions {
        let size = region.end - region.start;
        let bytes = unsafe { std::slice::from_raw_parts(region.start as *const u8, size) };

        if let Ok(Some(offset)) = scan_first_match(bytes, pattern_str) {
            return Some(region.start + offset);
        }
    }

    None
}

// Find pattern in specific module (for VM scanning)
pub unsafe fn find_pattern_in_module(
    module_base: usize,
    module_size: usize,
    pattern_str: &str,
) -> Option<usize> {
    let bytes = unsafe { std::slice::from_raw_parts(module_base as *const u8, module_size) };

    if let Ok(Some(offset)) = scan_first_match(bytes, pattern_str) {
        Some(module_base + offset)
    } else {
        None
    }
}

// Convert C-style byte pattern to hex string format for patternscan
pub fn convert_c_pattern_to_hex(c_pattern: &str, mask: &str) -> String {
    let mut result = String::new();
    let mut i = 0;
    let mut mask_index = 0;
    
    let chars: Vec<char> = c_pattern.chars().collect();
    
    while i < chars.len() {
        if chars[i] == '\\' && i + 3 < chars.len() && chars[i + 1] == 'x' {
            // Extract hex byte (e.g., \x41 -> 41)
            let hex_str: String = chars[i + 2..i + 4].iter().collect();
            
            if mask_index < mask.len() {
                let mask_char = mask.chars().nth(mask_index).unwrap_or('X');
                
                if !result.is_empty() {
                    result.push(' ');
                }
                
                if mask_char == 'X' {
                    result.push_str(&hex_str.to_uppercase());
                } else {
                    result.push_str("??");
                }
            }
            
            i += 4; // Skip \xNN
            mask_index += 1;
        } else {
            i += 1;
        }
    }
    
    result
}

// Legacy pattern format conversion (from patterns.h format to hex string)
pub fn convert_legacy_pattern(pattern_bytes: &[u8], mask: &str) -> String {
    let mut result = String::new();

    for (i, &byte) in pattern_bytes.iter().enumerate() {
        if i < mask.len() {
            let mask_char = mask.chars().nth(i).unwrap_or('X');
            if mask_char == 'X' {
                if !result.is_empty() {
                    result.push(' ');
                }
                result.push_str(&format!("{:02X}", byte));
            } else {
                if !result.is_empty() {
                    result.push(' ');
                }
                result.push_str("??");
            }
        }
    }

    result
}

// Find qagame module in memory maps
pub fn find_qagame_module() -> Option<(usize, usize)> {
    let regions = match parse_maps() {
        Ok(r) => r,
        Err(_) => {
            eprintln!("ERROR: Unable to parse /proc/self/maps");
            return None;
        }
    };

    // Look for qagame module
    #[cfg(target_arch = "x86_64")]
    const QAGAME_NAME: &str = "qagamex64.so";
    #[cfg(target_arch = "x86")]
    const QAGAME_NAME: &str = "qagamei386.so";

    for region in regions {
        if let Some(ref path) = region.path {
            if path.contains(QAGAME_NAME) && region.perms.starts_with("r-x") {
                // Found executable segment of qagame
                let size = region.end - region.start;
                return Some((region.start, size));
            }
        }
    }

    None
}

// Search for VM functions in qagame module
pub unsafe fn search_vm_functions() -> bool {
    use crate::quake::patterns;

    // First, find the qagame module
    let (base, size) = match find_qagame_module() {
        Some((b, s)) => (b, s),
        None => {
            debug_println!("ERROR: Unable to find qagame module");
            return false;
        }
    };

    crate::quake::QAGAME_MODULE_BASE = base;
    crate::quake::QAGAME_MODULE_SIZE = size;
    debug_println!(
        "Found qagame module: base={:p}, size={:#x}",
        base as *const (),
        size
    );

    let mut failed = false;

    // ClientConnect
    if let Some(addr) = find_pattern_in_module(base, size, patterns::CLIENTCONNECT) {
        crate::quake::CLIENTCONNECT = Some(std::mem::transmute(addr));
        debug_println!("ClientConnect: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find ClientConnect");
        failed = true;
    }

    // ClientSpawn
    if let Some(addr) = find_pattern_in_module(base, size, patterns::CLIENTSPAWN) {
        crate::quake::CLIENTSPAWN = Some(std::mem::transmute(addr));
        debug_println!("ClientSpawn: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find ClientSpawn");
        failed = true;
    }

    // G_Damage
    if let Some(addr) = find_pattern_in_module(base, size, patterns::G_DAMAGE) {
        crate::quake::G_DAMAGE = Some(std::mem::transmute(addr));
        debug_println!("G_Damage: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find G_Damage");
        failed = true;
    }

    // Touch_Item
    if let Some(addr) = find_pattern_in_module(base, size, patterns::TOUCH_ITEM) {
        crate::quake::TOUCH_ITEM = Some(std::mem::transmute(addr));
        debug_println!("Touch_Item: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find Touch_Item");
        failed = true;
    }

    // Cmd_CallVote_f
    if let Some(addr) = find_pattern_in_module(base, size, patterns::CMD_CALLVOTE_F) {
        crate::quake::CMD_CALLVOTE_F = Some(std::mem::transmute(addr));
        debug_println!("Cmd_CallVote_f: {:p}", addr as *const ());
    } else {
        debug_println!("ERROR: Unable to find Cmd_CallVote_f");
        failed = true;
    }

    if failed {
        debug_println!("Failed to find some VM functions");
        return false;
    }

    debug_println!("Successfully found all VM functions");
    true
}
