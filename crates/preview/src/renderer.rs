// Stub PDF renderer - to be fully implemented in Phase 7
use anyhow::Result;

pub struct PdfRenderer;

impl PdfRenderer {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn load_document(&mut self, _data: &[u8]) -> Result<()> {
        // Placeholder
        Ok(())
    }
}

impl Default for PdfRenderer {
    fn default() -> Self {
        Self
    }
}
