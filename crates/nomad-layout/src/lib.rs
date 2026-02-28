//! Layout engine for the Nomad Web Engine.
//!
//! Computes layout and positions elements based on CSS box model.

use nomad_core::NomadCore;
use nomad_style::StyleSheet;

/// Layout engine placeholder struct.
pub struct LayoutEngine {
    _core: NomadCore,
}

impl LayoutEngine {
    /// Creates a new layout engine.
    pub fn new(core: NomadCore) -> Self {
        Self { _core: core }
    }

    /// Computes layout (placeholder).
    pub fn compute_layout(&self, _stylesheet: &StyleSheet) -> Result<LayoutTree, String> {
        Ok(LayoutTree::new())
    }
}

/// Layout tree representation.
pub struct LayoutTree {
    box_count: usize,
}

impl LayoutTree {
    /// Creates an empty layout tree.
    pub fn new() -> Self {
        Self { box_count: 0 }
    }

    /// Returns the box count.
    pub fn box_count(&self) -> usize {
        self.box_count
    }
}

impl Default for LayoutTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_engine_creation() {
        let core = NomadCore::new();
        let engine = LayoutEngine::new(core);
        let stylesheet = StyleSheet::new();
        let tree = engine.compute_layout(&stylesheet).unwrap();
        assert_eq!(tree.box_count(), 0);
    }
}
