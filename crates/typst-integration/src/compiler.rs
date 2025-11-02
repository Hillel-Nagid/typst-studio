//! Typst compilation service

use crate::diagnostics::{ Diagnostic, DiagnosticList };
use crate::world::SystemWorld;
use crate::{ Result, TypstError };
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Request for compilation
#[derive(Debug, Clone)]
pub struct CompileRequest {
    /// Root directory of the project
    pub root: PathBuf,
    /// Main file to compile
    pub main_file: PathBuf,
    /// Request ID for tracking
    pub id: u64,
}

/// Result of compilation
#[derive(Debug, Clone)]
pub struct CompileResult {
    /// Request ID
    pub id: u64,
    /// Whether compilation succeeded
    pub success: bool,
    /// Diagnostics from compilation
    pub diagnostics: DiagnosticList,
    /// Compiled document (if successful)
    pub document: Option<PathBuf>, // Would be Document in real impl
}

/// Typst compiler service
pub struct Compiler {
    request_tx: mpsc::Sender<CompileRequest>,
    result_rx: mpsc::Receiver<CompileResult>,
}

impl Compiler {
    /// Create a new compiler service
    pub fn new() -> Self {
        let (request_tx, mut request_rx) = mpsc::channel::<CompileRequest>(10);
        let (result_tx, result_rx) = mpsc::channel::<CompileResult>(10);

        tokio::spawn(async move {
            while let Some(request) = request_rx.recv().await {
                let result = Self::compile_internal(request).await;
                let _ = result_tx.send(result).await;
            }
        });

        Self {
            request_tx,
            result_rx,
        }
    }

    /// Submit a compilation request
    pub async fn compile(&self, request: CompileRequest) -> Result<()> {
        self.request_tx
            .send(request).await
            .map_err(|e| TypstError::CompilationFailed(e.to_string()))
    }

    /// Receive a compilation result
    pub async fn receive_result(&mut self) -> Option<CompileResult> {
        self.result_rx.recv().await
    }

    /// Internal compilation implementation
    async fn compile_internal(request: CompileRequest) -> CompileResult {
        let mut diagnostics = DiagnosticList::new();
        let world = match SystemWorld::new(request.root.clone(), request.main_file.clone()) {
            Ok(w) => w,
            Err(e) => {
                diagnostics.add(Diagnostic::error(format!("Failed to create world: {}", e)));
                return CompileResult {
                    id: request.id,
                    success: false,
                    diagnostics,
                    document: None,
                };
            }
        };

        let result = typst::compile(&world);

        for warning in &result.warnings {
            diagnostics.add(Diagnostic::warning(format!("{:?}", warning)));
        }

        match result.output {
            Ok(_document) => {
                // In a real implementation, we'd save or return the document
                CompileResult {
                    id: request.id,
                    success: true,
                    diagnostics,
                    document: Some(request.main_file),
                }
            }
            Err(errors) => {
                // Convert Typst errors to diagnostics
                for error in errors {
                    diagnostics.add(Diagnostic::error(format!("{:?}", error)));
                }
                CompileResult {
                    id: request.id,
                    success: false,
                    diagnostics,
                    document: None,
                }
            }
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compiler_creation() {
        let _compiler = Compiler::new();
        // Compiler created successfully
    }
}
