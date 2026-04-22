use serde::{Deserialize, Serialize};

#[cfg(target_os = "macos")]
use libloading::{Library, Symbol};

#[cfg(target_os = "macos")]
use std::{
    ffi::{c_char, c_void, CStr, CString},
    sync::OnceLock,
};

#[cfg(target_os = "macos")]
use core_foundation_sys::{
    array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef},
    base::{CFRelease, CFTypeRef},
    dictionary::{CFDictionaryGetValue, CFDictionaryRef},
    number::{CFNumberGetValue, CFNumberRef, kCFNumberSInt64Type},
    string::{
        kCFStringEncodingUTF8, CFStringCreateWithCString, CFStringGetCString, CFStringRef,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceInfo {
    pub id: u64,
    pub index: usize,
    pub uuid: String,
    pub display_id: String,
    pub active: bool,
}

#[cfg(target_os = "macos")]
type CGSConnectionID = i32;

#[cfg(target_os = "macos")]
type FnCGSConnection = unsafe extern "C" fn() -> CGSConnectionID;

#[cfg(target_os = "macos")]
type FnCGSGetActiveSpace = unsafe extern "C" fn(CGSConnectionID) -> u64;

#[cfg(target_os = "macos")]
type FnCGSCopyManagedDisplaySpaces = unsafe extern "C" fn(CGSConnectionID) -> *const c_void;

#[cfg(target_os = "macos")]
type FnCGSManagedDisplaySetCurrentSpace =
    unsafe extern "C" fn(CGSConnectionID, *const c_void, u64);

#[cfg(target_os = "macos")]
static SKYLIGHT: OnceLock<Library> = OnceLock::new();

#[cfg(target_os = "macos")]
fn skylight() -> &'static Library {
    SKYLIGHT.get_or_init(|| unsafe {
        Library::new(
            "/System/Library/PrivateFrameworks/SkyLight.framework/SkyLight",
        )
        .expect("Failed to load SkyLight.framework")
    })
}

#[cfg(target_os = "macos")]
fn default_connection() -> CGSConnectionID {
    unsafe {
        if let Ok(f) = skylight().get::<FnCGSConnection>(b"CGSMainConnectionID\0") {
            return f();
        }
        let f: Symbol<FnCGSConnection> = skylight()
            .get(b"CGSDefaultConnection\0")
            .expect("CGSDefaultConnection not found");
        f()
    }
}

#[cfg(target_os = "macos")]
fn get_active_space_id() -> u64 {
    let cid = default_connection();
    unsafe {
        let f: Symbol<FnCGSGetActiveSpace> = skylight()
            .get(b"CGSGetActiveSpace\0")
            .expect("CGSGetActiveSpace not found");
        f(cid)
    }
}

#[cfg(target_os = "macos")]
unsafe fn make_cf_string(s: &str) -> CFStringRef {
    let cstr = CString::new(s).unwrap();
    CFStringCreateWithCString(std::ptr::null(), cstr.as_ptr(), kCFStringEncodingUTF8)
}

#[cfg(target_os = "macos")]
unsafe fn cfstring_to_rust(s: CFStringRef) -> Option<String> {
    if s.is_null() {
        return None;
    }
    let mut buf = [0u8; 512];
    let ok = CFStringGetCString(
        s,
        buf.as_mut_ptr() as *mut c_char,
        buf.len() as isize,
        kCFStringEncodingUTF8,
    );
    if ok != 0 {
        CStr::from_ptr(buf.as_ptr() as *const c_char)
            .to_str()
            .ok()
            .map(|s| s.to_owned())
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
unsafe fn cfnumber_to_i64(num: CFNumberRef) -> Option<i64> {
    if num.is_null() {
        return None;
    }
    let mut value: i64 = 0;
    let ok = CFNumberGetValue(
        num,
        kCFNumberSInt64Type,
        &mut value as *mut i64 as *mut c_void,
    );
    if ok != 0 {
        Some(value)
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
unsafe fn dict_get_string(dict: CFDictionaryRef, key: &str) -> Option<String> {
    let key_cf = make_cf_string(key);
    let val = CFDictionaryGetValue(dict, key_cf as *const c_void) as CFStringRef;
    CFRelease(key_cf as CFTypeRef);
    cfstring_to_rust(val)
}

#[cfg(target_os = "macos")]
unsafe fn dict_get_i64(dict: CFDictionaryRef, key: &str) -> Option<i64> {
    let key_cf = make_cf_string(key);
    let val = CFDictionaryGetValue(dict, key_cf as *const c_void) as CFNumberRef;
    CFRelease(key_cf as CFTypeRef);
    cfnumber_to_i64(val)
}

#[cfg(not(target_os = "macos"))]
pub fn list_spaces() -> Vec<SpaceInfo> {
    vec![]
}

#[cfg(target_os = "macos")]
pub fn list_spaces() -> Vec<SpaceInfo> {
    let cid = default_connection();
    let active_id = get_active_space_id();

    let displays_arr: *const c_void = unsafe {
        let f: Symbol<FnCGSCopyManagedDisplaySpaces> = skylight()
            .get(b"CGSCopyManagedDisplaySpaces\0")
            .expect("CGSCopyManagedDisplaySpaces not found");
        f(cid)
    };

    if displays_arr.is_null() {
        return vec![];
    }

    let mut result = Vec::new();
    let mut global_index: usize = 1;

    unsafe {
        let displays_arr = displays_arr as CFArrayRef;
        let display_count = CFArrayGetCount(displays_arr);

        for i in 0..display_count {
            let display_dict =
                CFArrayGetValueAtIndex(displays_arr, i) as CFDictionaryRef;
            if display_dict.is_null() {
                continue;
            }

            let display_id = dict_get_string(display_dict, "Display Identifier")
                .unwrap_or_else(|| "Main".to_string());

            let spaces_key = make_cf_string("Spaces");
            let spaces_arr =
                CFDictionaryGetValue(display_dict, spaces_key as *const c_void)
                    as CFArrayRef;
            CFRelease(spaces_key as CFTypeRef);

            if spaces_arr.is_null() {
                continue;
            }

            let space_count = CFArrayGetCount(spaces_arr);
            for j in 0..space_count {
                let space_dict =
                    CFArrayGetValueAtIndex(spaces_arr, j) as CFDictionaryRef;
                if space_dict.is_null() {
                    continue;
                }

                // Skip fullscreen/special spaces (type != 0 means non-user space)
                let space_type = dict_get_i64(space_dict, "type").unwrap_or(0);
                if space_type != 0 {
                    continue;
                }

                let space_id =
                    dict_get_i64(space_dict, "id64").unwrap_or(0) as u64;
                let uuid =
                    dict_get_string(space_dict, "uuid").unwrap_or_default();

                result.push(SpaceInfo {
                    id: space_id,
                    index: global_index,
                    uuid,
                    display_id: display_id.clone(),
                    active: space_id == active_id,
                });
                global_index += 1;
            }
        }

        // CFRelease the outer array (Create Rule — we own this)
        CFRelease(displays_arr as CFTypeRef);
    }

    result
}

#[cfg(not(target_os = "macos"))]
pub fn switch_to_space(_space_id: u64, _display_id: &str) -> Result<(), String> {
    Err("Only supported on macOS".to_string())
}

#[cfg(target_os = "macos")]
pub fn switch_to_space(space_id: u64, display_id: &str) -> Result<(), String> {
    let cid = default_connection();
    unsafe {
        let display_cf = make_cf_string(display_id);
        let f: Symbol<FnCGSManagedDisplaySetCurrentSpace> = skylight()
            .get(b"CGSManagedDisplaySetCurrentSpace\0")
            .map_err(|e| format!("symbol not found: {e}"))?;
        f(cid, display_cf as *const c_void, space_id);
        CFRelease(display_cf as CFTypeRef);
    }
    Ok(())
}
