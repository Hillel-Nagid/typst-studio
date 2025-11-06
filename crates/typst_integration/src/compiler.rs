// Stub compiler implementation - to be fully implemented in Phase 6
use anyhow::Result;
use std::path::Path;

pub struct TypstCompiler;

impl TypstCompiler {
    pub fn new() -> Self {
        Self
    }

    pub async fn compile(&self, _path: &Path) -> Result<Vec<u8>> {
        // Placeholder - returns empty PDF
        Ok(Vec::new())
    }
}

impl Default for TypstCompiler {
    fn default() -> Self {
        Self::new()
    }
}

