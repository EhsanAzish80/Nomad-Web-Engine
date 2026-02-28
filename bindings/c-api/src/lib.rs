//! C API bindings for the Nomad Web Engine.
//!
//! Provides a C-compatible FFI interface to the Rust engine.

use std::ptr;

use nomad_core::NomadCore;

/// Opaque handle to the Nomad engine.
#[repr(C)]
pub struct NomadEngine {
    core: NomadCore,
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
    let engine = Box::new(NomadEngine {
        core: NomadCore::new(),
    });
    Box::into_raw(engine)
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

/// Checks if the engine is initialized.
///
/// # Safety
///
/// The pointer must be a valid pointer returned by `nomad_engine_create`.
#[no_mangle]
pub unsafe extern "C" fn nomad_engine_is_initialized(engine: *const NomadEngine) -> bool {
    if engine.is_null() {
        return false;
    }
    (*engine).core.is_initialized()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_api_lifecycle() {
        unsafe {
            let engine = nomad_engine_create();
            assert!(!engine.is_null());
            assert!(nomad_engine_is_initialized(engine));
            nomad_engine_destroy(engine);
        }
    }

    #[test]
    fn test_null_engine() {
        unsafe {
            assert!(!nomad_engine_is_initialized(ptr::null()));
            nomad_engine_destroy(ptr::null_mut());
        }
    }
}
