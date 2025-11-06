// Stub LSP client - to be fully implemented in Phase 5
use anyhow::Result;

pub struct LspClient;

impl LspClient {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl Default for LspClient {
    fn default() -> Self {
        Self
    }
}

