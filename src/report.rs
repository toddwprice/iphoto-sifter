use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::compare::ComparisonResult;
use crate::photos::AddResult;

/// Summary report of the sifting operation
#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub total_scanned: usize,
    pub already_in_library: usize,
    pub to_add: usize,
    pub successfully_added: usize,
    pub failed_to_add: usize,
    pub duplicate_groups: usize,
    pub dry_run: bool,
    pub added_photos: Vec<AddedPhotoReport>,
    pub failed_photos: Vec<FailedPhotoReport>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddedPhotoReport {
    pub source_path: String,
    pub library_path: String,
    pub uuid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailedPhotoReport {
    pub source_path: String,
    pub error: String,
}

impl Report {
    /// Create a new report from comparison results
    pub fn new(comparison: &ComparisonResult, dry_run: bool) -> Self {
        Self {
            total_scanned: comparison.total_scanned(),
            already_in_library: comparison.already_present(),
            to_add: comparison.to_add(),
            successfully_added: 0,
            failed_to_add: 0,
            duplicate_groups: comparison.duplicate_groups(),
            dry_run,
            added_photos: Vec::new(),
            failed_photos: Vec::new(),
        }
    }

    /// Add a successful addition to the report
    pub fn add_success(&mut self, result: &AddResult) {
        self.successfully_added += 1;
        self.added_photos.push(AddedPhotoReport {
            source_path: result.source_path.display().to_string(),
            library_path: result.library_path.display().to_string(),
            uuid: result.uuid.clone(),
        });
    }

    /// Add a failed addition to the report
    pub fn add_failure(&mut self, source_path: &PathBuf, error: String) {
        self.failed_to_add += 1;
        self.failed_photos.push(FailedPhotoReport {
            source_path: source_path.display().to_string(),
            error,
        });
    }

    /// Print a human-readable summary to stdout
    pub fn print_summary(&self) {
        println!("\n{}", "=".repeat(60).bright_cyan());
        println!("{}", "Photo Library Sifter - Summary".bright_cyan().bold());
        println!("{}", "=".repeat(60).bright_cyan());

        if self.dry_run {
            println!("\n{}", "DRY RUN MODE - No changes were made".yellow().bold());
        }

        println!("\n{}", "Scan Results:".bold());
        println!("  Total photos scanned: {}", self.total_scanned);
        println!("  Already in library:   {}", self.already_in_library.to_string().green());
        println!("  To add:               {}", self.to_add.to_string().yellow());

        if self.duplicate_groups > 0 {
            println!(
                "  Duplicate groups:     {} {}",
                self.duplicate_groups.to_string().magenta(),
                "(only one copy of each will be added)".dimmed()
            );
        }

        if !self.dry_run {
            println!("\n{}", "Addition Results:".bold());
            println!("  Successfully added:   {}", self.successfully_added.to_string().green());

            if self.failed_to_add > 0 {
                println!("  Failed to add:        {}", self.failed_to_add.to_string().red());
            }
        }

        println!("\n{}", "=".repeat(60).bright_cyan());
    }

    /// Print detailed information about added photos
    pub fn print_details(&self, verbose: bool) {
        if self.added_photos.is_empty() && self.failed_photos.is_empty() {
            return;
        }

        if !self.added_photos.is_empty() {
            println!("\n{}", "Added Photos:".green().bold());
            for photo in &self.added_photos {
                if verbose {
                    println!("  {} → {}",
                        photo.source_path.bright_white(),
                        photo.library_path.dimmed()
                    );
                    println!("    UUID: {}", photo.uuid.dimmed());
                } else {
                    println!("  ✓ {}", photo.source_path.green());
                }
            }
        }

        if !self.failed_photos.is_empty() {
            println!("\n{}", "Failed Photos:".red().bold());
            for photo in &self.failed_photos {
                println!("  ✗ {}", photo.source_path.red());
                println!("    Error: {}", photo.error.dimmed());
            }
        }
    }

    /// Export the report as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
