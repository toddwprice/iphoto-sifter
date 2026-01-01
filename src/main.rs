use clap::Parser;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::process;

use iphoto_sifter::{
    compare, photo, photos,
    report::Report,
    Result,
};

#[derive(Parser)]
#[command(name = "iphoto-sifter")]
#[command(author = "Photo Library Sifter Contributors")]
#[command(version = "0.1.0")]
#[command(about = "Sync photos from directories into Photos.app libraries", long_about = None)]
struct Cli {
    /// Source directory to scan for photos
    #[arg(short, long, value_name = "DIR")]
    source: PathBuf,

    /// Path to Photos.app library (.photoslibrary)
    #[arg(short, long, value_name = "LIBRARY")]
    library: PathBuf,

    /// Dry run - show what would be added without making changes
    #[arg(short = 'n', long, default_value_t = true)]
    dry_run: bool,

    /// Actually commit changes (disables dry-run)
    #[arg(short, long)]
    commit: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Scan source directory recursively
    #[arg(short = 'R', long, default_value_t = true)]
    recursive: bool,

    /// Export report to JSON file
    #[arg(short = 'r', long, value_name = "FILE")]
    report: Option<PathBuf>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Determine if we're in dry-run mode
    let dry_run = !cli.commit;

    // Print banner
    println!("{}", "=".repeat(60).bright_cyan());
    println!("{}", "Photo Library Sifter".bright_cyan().bold());
    println!("{}", "=".repeat(60).bright_cyan());
    println!();

    if dry_run {
        println!("{}", "Running in DRY RUN mode. Use --commit to apply changes.".yellow());
        println!();
    }

    // Validate inputs
    if !cli.source.exists() {
        eprintln!("{} Source directory does not exist: {}",
            "Error:".red().bold(),
            cli.source.display()
        );
        process::exit(1);
    }

    if !cli.library.exists() {
        eprintln!("{} Library does not exist: {}",
            "Error:".red().bold(),
            cli.library.display()
        );
        process::exit(1);
    }

    // Open the Photos library
    println!("{} Opening Photos library...", "•".bright_cyan());
    let library = photos::PhotosLibrary::open(&cli.library)?;

    // Check if library is in use
    if library.is_in_use() && !dry_run {
        println!("{}",
            "Warning: The Photos library appears to be in use.".yellow()
        );
        println!("{}",
            "Please close Photos.app before running with --commit.".yellow()
        );
        println!();
    }

    // Scan source directory
    println!("{} Scanning source directory: {}",
        "•".bright_cyan(),
        cli.source.display()
    );

    let source_photos = {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
        );
        pb.set_message("Scanning and hashing photos...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let photos = photo::scan_directory(&cli.source, cli.recursive)?;

        pb.finish_with_message(format!("Found {} photos", photos.len()));
        photos
    };

    println!("  Found {} photos", source_photos.len().to_string().green());
    println!();

    // Read library assets
    println!("{} Reading Photos library database...", "•".bright_cyan());
    let conn = photos::open_database(&cli.library)?;
    let assets = photos::read_all_assets(&conn)?;
    println!("  Library contains {} photos", assets.len().to_string().green());
    println!();

    // Build hash map of library photos
    println!("{} Building hash map of library photos...", "•".bright_cyan());
    let library_hash_map = {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
        );
        pb.set_message("Hashing library photos...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let hash_map = photos::build_hash_map(&cli.library, &assets)?;

        pb.finish_with_message(format!("Hashed {} photos", hash_map.len()));
        hash_map
    };
    println!();

    // Compare
    println!("{} Comparing photos...", "•".bright_cyan());
    let comparison = compare::compare_photos(source_photos, &library_hash_map);
    println!();

    // Create report
    let mut report = Report::new(&comparison, dry_run);

    // Add photos if not in dry-run mode
    if !dry_run && !comparison.missing_in_library.is_empty() {
        println!("{} Creating database backup...", "•".bright_cyan());
        let backup_path = photos::backup_database(&cli.library)?;
        println!("  Backup created: {}", backup_path.display().to_string().dimmed());
        println!();

        println!("{} Adding photos to library...", "•".bright_cyan());
        let pb = ProgressBar::new(comparison.missing_in_library.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-")
        );

        for photo in &comparison.missing_in_library {
            pb.set_message(format!("{}", photo.path.file_name().unwrap().to_string_lossy()));

            match photos::add_photo_to_library(&cli.library, photo, dry_run) {
                Ok(result) => {
                    report.add_success(&result);
                }
                Err(e) => {
                    report.add_failure(&photo.path, e.to_string());
                }
            }

            pb.inc(1);
        }

        pb.finish_with_message("Done");
        println!();
    }

    // Print report
    report.print_summary();

    if cli.verbose {
        report.print_details(cli.verbose);
    }

    // Export JSON report if requested
    if let Some(report_path) = cli.report {
        println!("\n{} Exporting report to {}...",
            "•".bright_cyan(),
            report_path.display()
        );

        let json = report.to_json()
            .map_err(|e| iphoto_sifter::SifterError::PhotoProcessing(e.to_string()))?;

        std::fs::write(&report_path, json)?;
        println!("  Report exported successfully");
    }

    Ok(())
}
