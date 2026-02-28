//! C API bindings for the Nomad Web Engine.
//!
//! Provides a C-compatible FFI interface to the Rust engine.

use std::ptr;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use nomad_core::Engine;

/// Opaque handle to the Nomad engine.
#[repr(C)]
pub struct NomadEngine {
    engine: Engine,
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

/// Loads a URL and returns the extracted text.
///
/// Returns a pointer to a null-terminated C string containing the text,
/// or null on failure. The caller must free the returned string with
/// `nomad_free_string`.
///
/// # Safety
///
/// The engine pointer must be a valid pointer returned by `nomad_engine_create`.
/// The url pointer must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn nomad_engine_load_url(
    engine: *const NomadEngine,
    url: *const c_char,
) -> *mut c_char {
    if engine.is_null() || url.is_null() {
        return ptr::null_mut();
    }

    let url_str = match CStr::from_ptr(url).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    match (*engine).engine.load_url(url_str) {
        Ok(text) => match CString::new(text) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => ptr::null_mut(),
        },
        Err(_) => ptr::null_mut(),
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
}
