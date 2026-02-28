//! Core functionality for the Nomad Web Engine.
//!
//! This crate provides foundational types and utilities used across
//! the entire web engine.

/// Core engine placeholder struct.
pub struct NomadCore {
    initialized: bool,
}

impl NomadCore {
    /// Creates a new instance of the core engine.
    pub fn new() -> Self {
        Self { initialized: true }
    }

    /// Returns whether the core is initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for NomadCore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_initialization() {
        let core = NomadCore::new();
        assert!(core.is_initialized());
    }
}
