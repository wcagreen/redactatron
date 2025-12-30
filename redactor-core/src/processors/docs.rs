use anyhow::{Context, Result};
use log::{debug, error, info};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

use crate::processors::pdf::PdfEngine;

// Temp directory for document conversions
pub struct DocConverter {
    temp_dir: Option<TempDir>,
}

impl DocConverter {
    /// Create a temporary directory for document conversions
    pub fn new() -> Result<Self> {
        debug!("Initializing DocConverter with new temporary directory");
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        info!(
            "DocConverter temporary directory created at: {}",
            temp_dir.path().display()
        );
        Ok(Self {
            temp_dir: Some(temp_dir),
        })
    }

    /// Convert a .docx or .doc file to PDF in the temporary directory
    /// Returns the path to the generated PDF file
    pub fn convert_to_pdf(&self, input_path: &str) -> Result<PathBuf> {
        let input_path = Path::new(input_path);
        info!(
            "Starting document to PDF conversion: {}",
            input_path.display()
        );

        // Validate input file
        if !input_path.exists() {
            error!("Input file not found: {}", input_path.display());
            return Err(anyhow::anyhow!(
                "Input file not found: {}",
                input_path.display()
            ));
        }

        let temp_path = self
            .temp_dir
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Temporary directory not available"))?
            .path();

        // Construct the output PDF path
        let pdf_filename = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
        let output_pdf_path = temp_path.join(format!("{}.pdf", pdf_filename));

        // Use Pandoc to convert document to PDF
        debug!(
            "Invoking Pandoc with command: pandoc {} -o {} --pdf-engine=pdflatex",
            input_path.display(),
            output_pdf_path.display()
        );
        let output = Command::new("pandoc")
            .arg(input_path)
            .arg("-o")
            .arg(&output_pdf_path)
            .arg("--pdf-engine=pdflatex")
            .output();

        let output = output.context("Failed to execute Pandoc. Ensure Pandoc is installed and in your system PATH. Download from https://pandoc.org/installing.html")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            error!("Pandoc conversion failed with status: {}", output.status);
            error!("Pandoc stderr: {}", stderr);
            error!("Pandoc stdout: {}", stdout);
            return Err(anyhow::anyhow!(
                "Pandoc conversion failed.\nStderr: {}\nStdout: {}",
                stderr,
                stdout
            ));
        }
        debug!("Pandoc conversion completed successfully");

        // Verify the PDF was created
        if !output_pdf_path.exists() {
            error!("Expected PDF at: {}", output_pdf_path.display());
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
                output_pdf_path.display()
            ));
        }

        info!(
            "Document successfully converted to PDF: {}",
            output_pdf_path.display()
        );
        Ok(output_pdf_path)
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
