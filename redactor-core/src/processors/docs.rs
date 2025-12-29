use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use log::{warn, debug, error};

use crate::processors::pdf::PdfEngine;


// Temp directory for document conversions
pub struct DocConverter {
    temp_dir: Option<TempDir>,
}

impl DocConverter {
    /// Create a temporary directory for document conversions
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        Ok(Self {
            temp_dir: Some(temp_dir),
        })
    }

    /// Convert a .docx or .doc file to PDF in the temporary directory
    /// Returns the path to the generated PDF file
    pub fn convert_to_pdf(&self, input_path: &str) -> Result<PathBuf> {
        let input_path = Path::new(input_path);

        // Validate input file
        if !input_path.exists() {
            return Err(anyhow::anyhow!("Input file not found: {}", input_path.display()));
        }

        let temp_path = self
            .temp_dir
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Temporary directory not available"))?
            .path();

        // Try soffice first (should be in PATH now)
        let mut output = Command::new("soffice")
            .arg("--headless")
            .arg("--convert-to")
            .arg("pdf")
            .arg("--outdir")
            .arg(temp_path)
            .arg(input_path)
            .output();

        // If soffice fails to execute, try full path
        if output.is_err() {
            warn!("soffice not found in PATH, trying full path...");
            output = Command::new("C:\\Program Files\\LibreOffice\\program\\soffice.exe")
                .arg("--headless")
                .arg("--convert-to")
                .arg("pdf")
                .arg("--outdir")
                .arg(temp_path)
                .arg(input_path)
                .output();
        }

        let output = output.context("Failed to execute LibreOffice. Ensure it's installed at C:\\Program Files\\LibreOffice or in your PATH")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            error!("LibreOffice stderr: {}", stderr);
            error!("LibreOffice stdout: {}", stdout);
            return Err(anyhow::anyhow!(
                "LibreOffice conversion failed.\nStderr: {}\nStdout: {}",
                stderr, stdout
            ));
        }

        // Construct the expected PDF output path
        let pdf_filename = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

        let pdf_path = temp_path.join(format!("{}.pdf", pdf_filename));

        if !pdf_path.exists() {
            error!("Expected PDF at: {}", pdf_path.display());
            debug!("Temp directory contents:");
            if let Ok(entries) = std::fs::read_dir(temp_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        debug!("  - {}", entry.path().display());
                    }
                }
            }
            return Err(anyhow::anyhow!(
                "Conversion appeared to succeed but PDF file not found at: {}",
                pdf_path.display()
            ));
        }

        Ok(pdf_path)
    }

    /// Get the temporary directory path
    pub fn temp_dir_path(&self) -> Result<&Path> {
        self.temp_dir
            .as_ref()
            .map(|t| t.path())
            .ok_or_else(|| anyhow::anyhow!("Temporary directory not available"))
    }
}

impl Default for DocConverter {
    fn default() -> Self {
        // In real scenarios, you might want to handle this Result more carefully
        Self::new().expect("Failed to create default DocConverter")
    }
}



/// High-level document processor that handles document conversion and PDF operations
pub struct DocumentProcessor;

impl DocumentProcessor {
    /// Convert a document (.docx, .doc) to PDF and return a PdfEngine ready for use
    pub fn process_document(input_path: &str) -> Result<(PdfEngine, DocConverter)> {
        let converter = DocConverter::new()?;
        let pdf_path = converter.convert_to_pdf(input_path)?;

        let mut pdf_engine = PdfEngine::new();
        pdf_engine.load_file(pdf_path.to_str().unwrap())?;

        Ok((pdf_engine, converter))
    }

    /// Convert a document to PDF and return the path (without creating a PdfEngine)
    pub fn convert_document(input_path: &str) -> Result<PathBuf> {
        let converter = DocConverter::new()?;
        converter.convert_to_pdf(input_path)
    }
}

