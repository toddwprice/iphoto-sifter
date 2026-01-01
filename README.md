# Photo Library Sifter

A Rust CLI tool that scans directories for photos and syncs them into your Photos.app library.

## Features

- 🔍 **Smart Scanning**: Recursively scans directories for photos
- 🔐 **Hash-Based Deduplication**: Uses SHA-256 hashing to avoid duplicates
- 📸 **Format Support**: Supports JPEG, PNG, HEIC, RAW formats (CR2, NEF, ARW, DNG, etc.), and videos
- 🛡️ **Safe by Default**: Dry-run mode prevents accidental changes
- 📊 **Detailed Reports**: View what will be added before committing
- 💾 **Automatic Backups**: Creates database backups before modifications
- 🎨 **Beautiful Output**: Colored terminal output with progress bars

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/toddwprice/iphoto-sifter.git
cd iphoto-sifter

# Build the release binary
cargo build --release

# The binary will be at target/release/iphoto-sifter
```

### Install via Cargo

```bash
cargo install --path .
```

## Usage

### Basic Usage

**Dry Run (Default - Safe Mode)**

The tool runs in dry-run mode by default, showing you what would be added without making changes:

```bash
iphoto-sifter --source /path/to/photos --library ~/Pictures/Photos.photoslibrary
```

**Commit Changes**

To actually add photos to your library, use the `--commit` flag:

```bash
iphoto-sifter --source /path/to/photos --library ~/Pictures/Photos.photoslibrary --commit
```

### CLI Options

```
Options:
  -s, --source <DIR>          Source directory to scan for photos
  -l, --library <LIBRARY>     Path to Photos.app library (.photoslibrary)
  -n, --dry-run              Dry run mode (default: true)
  -c, --commit               Actually commit changes (disables dry-run)
  -v, --verbose              Verbose output with detailed file paths
  -R, --recursive            Scan source directory recursively (default: true)
  -r, --report <FILE>        Export report to JSON file
  -h, --help                 Print help
  -V, --version              Print version
```

### Examples

**1. Preview what will be added (dry-run)**

```bash
iphoto-sifter -s ~/Downloads/vacation-photos -l ~/Pictures/Photos.photoslibrary
```

**2. Add photos with verbose output**

```bash
iphoto-sifter -s ~/Downloads/vacation-photos -l ~/Pictures/Photos.photoslibrary --commit --verbose
```

**3. Scan only the top-level directory (non-recursive)**

```bash
iphoto-sifter -s ~/Pictures/imports -l ~/Pictures/Photos.photoslibrary --recursive false --commit
```

**4. Export results to JSON**

```bash
iphoto-sifter -s ~/Downloads/photos -l ~/Pictures/Photos.photoslibrary --commit -r report.json
```

## How It Works

1. **Scan Source**: Recursively scans the source directory for supported photo formats
2. **Hash Photos**: Calculates SHA-256 hash of each photo file
3. **Read Library**: Opens the Photos.app SQLite database and reads existing assets
4. **Hash Library**: Hashes all photos in the library for comparison
5. **Compare**: Identifies photos in source that don't exist in library (by hash)
6. **Add Photos** (if `--commit`):
   - Creates a backup of the Photos.sqlite database
   - Copies new photos to the library's `originals/` directory
   - Updates the database with new asset entries
7. **Report**: Displays summary and detailed results

## Supported Formats

### Photos
- JPEG (.jpg, .jpeg)
- PNG (.png)
- HEIC/HEIF (.heic, .heif)
- TIFF (.tiff, .tif)
- GIF (.gif)
- BMP (.bmp)

### RAW Formats
- Canon (.cr2)
- Nikon (.nef)
- Sony (.arw)
- Adobe DNG (.dng)
- Olympus (.orf)
- Panasonic (.rw2)
- Pentax (.pef)
- Samsung (.srw)

### Videos
- MOV (.mov)
- MP4 (.mp4)
- M4V (.m4v)

## Important Safety Notes

⚠️ **CRITICAL: Close Photos.app before using `--commit`**

The tool directly modifies the Photos.app SQLite database. Having Photos.app open while running this tool can cause:
- Database corruption
- Data loss
- Undefined behavior

### Safety Features

1. **Dry-run by default**: The tool will NOT make changes unless you use `--commit`
2. **Automatic backups**: Creates `Photos.sqlite.backup_TIMESTAMP` before modifications
3. **Library validation**: Checks if Photos.app is currently using the library
4. **Hash-based deduplication**: Never adds duplicate photos (same content)

### Best Practices

1. Always run with dry-run first to preview changes
2. Close Photos.app before running with `--commit`
3. Keep backups of your Photos library (Time Machine, etc.)
4. Test on a copy of your library first if you're unsure

## Output Example

```
============================================================
Photo Library Sifter
============================================================

Running in DRY RUN mode. Use --commit to apply changes.

• Opening Photos library...
• Scanning source directory: /Users/me/Downloads/photos
  Found 42 photos

• Reading Photos library database...
  Library contains 1,523 photos

• Building hash map of library photos...

• Comparing photos...

============================================================
Photo Library Sifter - Summary
============================================================

DRY RUN MODE - No changes were made

Scan Results:
  Total photos scanned: 42
  Already in library:   35
  To add:               7
  Duplicate groups:     2 (only one copy of each will be added)

============================================================
```

## Architecture

The project is organized into several modules:

- **photo**: Photo scanning and hashing
  - `scanner.rs`: Directory traversal and file discovery
  - `hasher.rs`: SHA-256 hash calculation
- **photos**: Photos.app library management
  - `library.rs`: Library structure and validation
  - `database.rs`: SQLite database access
  - `writer.rs`: Adding photos to the library
- **compare**: Hash-based comparison logic
- **report**: Terminal and JSON reporting
- **error**: Error types and handling

## Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Check code
cargo check
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_is_photo_file
```

## Limitations & Known Issues

1. **Database Schema**: The Photos.app database schema is complex and not fully documented. This tool implements a simplified approach that works for basic photo imports.

2. **Metadata**: Currently preserves file creation/modification times but doesn't extract or preserve all EXIF metadata like Photos.app's native import.

3. **Live Photos**: Live Photos (.HEIC + .MOV pairs) are treated as separate files.

4. **Edited Photos**: Does not handle edited versions/derivatives - only adds originals.

5. **iCloud**: Not tested with iCloud Photo Library enabled. Use with caution.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

**USE AT YOUR OWN RISK**

This tool directly modifies the Photos.app library database. While it includes safety features like backups and validation, there is always a risk when modifying system databases. The authors are not responsible for any data loss or corruption.

**Recommendations:**
- Always maintain backups of your Photos library
- Test on a copy of your library first
- Close Photos.app before running with `--commit`
- Start with small test runs before processing large photo collections

## Troubleshooting

### "Library appears to be in use"

Close Photos.app completely before running with `--commit`.

### "Not a valid Photos library"

Ensure you're pointing to a `.photoslibrary` directory, not a folder of photos. The library should contain `database/Photos.sqlite` and `originals/` directories.

### "Database error"

This usually means Photos.app is open or the database is locked. Close all Photos.app instances and try again.

### Performance Issues

For very large libraries (100,000+ photos), the initial hash calculation may take a while. Consider:
- Running on a subset first
- Using a more powerful machine
- Being patient - hashing is CPU-intensive but only done once per run

## Acknowledgments

Built with:
- [clap](https://github.com/clap-rs/clap) - CLI argument parsing
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite database access
- [sha2](https://github.com/RustCrypto/hashes) - SHA-256 hashing
- [walkdir](https://github.com/BurntSushi/walkdir) - Directory traversal
- [indicatif](https://github.com/console-rs/indicatif) - Progress bars
- [colored](https://github.com/mackwic/colored) - Terminal colors

## See Also

- [PLAN.md](PLAN.md) - Detailed implementation plan and architecture
