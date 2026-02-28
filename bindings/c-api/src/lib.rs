//! C API bindings for the Nomad Web Engine.
//!
//! Provides a C-compatible FFI interface to the Rust engine.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use nomad_core::Engine;

/// Opaque handle to the Nomad engine.
#[repr(C)]
pub struct NomadEngine {
    engine: Engine,
}

/// A byte buffer returned from the engine.
#[repr(C)]
pub struct ByteBuffer {
    pub data: *mut u8,
    pub len: usize,
}

impl ByteBuffer {
    fn from_vec(vec: Vec<u8>) -> Self {
        let mut vec = vec;
        let data = vec.as_mut_ptr();
        let len = vec.len();
        std::mem::forget(vec); // Prevent Rust from freeing the memory
        Self { data, len }
    }
}

/// Creates a new Nomad engine instance.
///
/// Returns a pointer to the engine or null on failure.
///
/// # Safety
///
/// The returned pointer must be freed with `nomad_engine_destroy`.
#[no_mangle]
pub extern "C" fn nomad_engine_create() -> *mut NomadEngine {
    match Engine::new() {
        Ok(engine) => {
            let nomad_engine = Box::new(NomadEngine { engine });
            Box::into_raw(nomad_engine)
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Destroys a Nomad engine instance.
///
/// # Safety
///
/// The pointer must have been created by `nomad_engine_create` and must not be used after this call.
#[no_mangle]
pub unsafe extern "C" fn nomad_engine_destroy(engine: *mut NomadEngine) {
    if !engine.is_null() {
        drop(Box::from_raw(engine));
    }
}

/// Loads a URL in the engine.
///
/// Returns 0 on success, non-zero on failure.
///
/// # Safety
///
/// The engine pointer must be a valid pointer returned by `nomad_engine_create`.
/// The url pointer must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn nomad_engine_load_url(
    engine: *mut NomadEngine,
    url: *const c_char,
) -> i32 {
    if engine.is_null() || url.is_null() {
        return -1;
    }

    let url_str = match CStr::from_ptr(url).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    match (*engine).engine.load_url(url_str) {
        Ok(_) => 0,
        Err(_) => -3,
    }
}

/// Navigates to a URL, resolving it relative to the current page if needed.
///
/// This function should be used for link clicks instead of `nomad_engine_load_url`,
/// as it properly handles relative URLs like "/path" or "page.html".
///
/// Returns 0 on success, non-zero on failure.
///
/// # Safety
///
/// The engine pointer must be a valid pointer returned by `nomad_engine_create`.
/// The url pointer must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn nomad_engine_navigate(
    engine: *mut NomadEngine,
    url: *const c_char,
) -> i32 {
    if engine.is_null() || url.is_null() {
        return -1;
    }

    let url_str = match CStr::from_ptr(url).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    match (*engine).engine.navigate(url_str) {
        Ok(_) => 0,
        Err(_) => -3,
    }
}

/// Ticks the engine (for future animations/updates).
///
/// # Safety
///
/// The engine pointer must be a valid pointer returned by `nomad_engine_create`.
#[no_mangle]
pub unsafe extern "C" fn nomad_engine_tick(engine: *mut NomadEngine) {
    // Tick is currently a no-op in Phase 4
    // Future: Animation updates, async task polling, etc.
    if !engine.is_null() {
        // Nothing to do for now
    }
}

/// Gets the display list from the engine as a byte buffer.
///
/// Returns a ByteBuffer with data and length. The caller must free the buffer
/// with `nomad_free_byte_buffer`.
///
/// # Safety
///
/// The engine pointer must be a valid pointer returned by `nomad_engine_create`.
#[no_mangle]
pub unsafe extern "C" fn nomad_engine_get_display_list(
    engine: *const NomadEngine,
) -> ByteBuffer {
    if engine.is_null() {
        return ByteBuffer {
            data: ptr::null_mut(),
            len: 0,
        };
    }

    match (*engine).engine.get_display_list_bytes() {
        Ok(bytes) => ByteBuffer::from_vec(bytes),
        Err(_) => ByteBuffer {
            data: ptr::null_mut(),
            len: 0,
        },
    }
}

/// Sets the viewport width for layout.
///
/// # Safety
///
/// The engine pointer must be a valid pointer returned by `nomad_engine_create`.
#[no_mangle]
pub unsafe extern "C" fn nomad_engine_set_viewport_width(
    engine: *mut NomadEngine,
    width: f32,
) {
    if !engine.is_null() {
        (*engine).engine.set_viewport_width(width);
    }
}

/// Submits a form with the given input values.
///
/// The inputs_json parameter should be a JSON string containing an array of [name, value] pairs:
/// Example: `[["q", "search term"], ["lang", "en"]]`
///
/// Returns 0 on success, non-zero on failure.
///
/// # Safety
///
/// The engine pointer must be a valid pointer returned by `nomad_engine_create`.
/// The inputs_json pointer must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn nomad_engine_submit_form(
    engine: *mut NomadEngine,
    form_index: usize,
    inputs_json: *const c_char,
) -> i32 {
    if engine.is_null() || inputs_json.is_null() {
        return -1;
    }

    let inputs_str = match CStr::from_ptr(inputs_json).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    // Parse JSON input
    let inputs: Vec<(String, String)> = match serde_json::from_str(inputs_str) {
        Ok(v) => v,
        Err(_) => return -3,
    };

    match (*engine).engine.submit_form(form_index, &inputs) {
        Ok(_) => 0,
        Err(_) => -4,
    }
}

/// Goes back in navigation history.
/// Returns 0 on success, non-zero on failure.
///
/// # Safety
///
/// The engine pointer must be a valid pointer returned by `nomad_engine_create`.
#[no_mangle]
pub unsafe extern "C" fn nomad_engine_go_back(engine: *mut NomadEngine) -> i32 {
    if engine.is_null() {
        return -1;
    }

    match (*engine).engine.go_back() {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

/// Goes forward in navigation history.
/// Returns 0 on success, non-zero on failure.
///
/// # Safety
///
/// The engine pointer must be a valid pointer returned by `nomad_engine_create`.
#[no_mangle]
pub unsafe extern "C" fn nomad_engine_go_forward(engine: *mut NomadEngine) -> i32 {
    if engine.is_null() {
        return -1;
    }

    match (*engine).engine.go_forward() {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

/// Checks if the engine can go back in history.
/// Returns 1 if can go back, 0 if cannot.
///
/// # Safety
///
/// The engine pointer must be a valid pointer returned by `nomad_engine_create`.
#[no_mangle]
pub unsafe extern "C" fn nomad_engine_can_go_back(engine: *const NomadEngine) -> i32 {
    if engine.is_null() {
        return 0;
    }

    if (*engine).engine.can_go_back() {
        1
    } else {
        0
    }
}

/// Checks if the engine can go forward in history.
/// Returns 1 if can go forward, 0 if cannot.
///
/// # Safety
///
/// The engine pointer must be a valid pointer returned by `nomad_engine_create`.
#[no_mangle]
pub unsafe extern "C" fn nomad_engine_can_go_forward(engine: *const NomadEngine) -> i32 {
    if engine.is_null() {
        return 0;
    }

    if (*engine).engine.can_go_forward() {
        1
    } else {
        0
    }
}

/// Frees a byte buffer returned by the engine.
///
/// # Safety
///
/// The buffer must have been returned by a Nomad engine function.
#[no_mangle]
pub unsafe extern "C" fn nomad_free_byte_buffer(buffer: ByteBuffer) {
    if !buffer.data.is_null() && buffer.len > 0 {
        drop(Vec::from_raw_parts(buffer.data, buffer.len, buffer.len));
    }
}

/// Frees a string returned by the engine.
///
/// # Safety
///
/// The pointer must have been returned by a Nomad engine function.
#[no_mangle]
pub unsafe extern "C" fn nomad_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_api_lifecycle() {
        unsafe {
            let engine = nomad_engine_create();
            assert!(!engine.is_null());
            nomad_engine_destroy(engine);
        }
    }

    #[test]
    fn test_null_engine() {
        unsafe {
            nomad_engine_destroy(ptr::null_mut());
            nomad_free_string(ptr::null_mut());
        }
    }

    #[test]
    fn test_tick() {
        unsafe {
            let engine = nomad_engine_create();
            assert!(!engine.is_null());
            nomad_engine_tick(engine);
            nomad_engine_destroy(engine);
        }
    }
}
