# Photo Library Sifter - Implementation Plan

## Project Overview
A Rust CLI application that scans a directory for photos, compares them against a Photos.app library, and adds any missing photos to the library.

## Goals
1. Hash all photos in a specified directory
2. Hash all photos in a Photos.app library
3. Identify photos not in the Photos library
4. Add missing photos to the Photos library
5. Provide detailed reporting on additions

## Technical Background

### Photos.app Library Structure
Photos.app libraries (.photoslibrary) are package directories with this structure:
```
MyLibrary.photoslibrary/
├── database/
│   ├── Photos.sqlite           # Main database (primary metadata)
│   ├── Photos.sqlite-shm       # Shared memory file
│   ├── Photos.sqlite-wal       # Write-ahead log
│   ├── search/                 # Search index
│   └── ...
├── originals/                  # Original photos organized by UUID
│   └── [UUID-based paths]/
│       └── IMG_1234.jpg
├── resources/                  # Derivatives, thumbnails, edited versions
│   ├── derivatives/
│   ├── media/
│   └── renders/
├── private/                    # Private cache data
└── scopes/                     # Shared albums and iCloud data
```

**Important Notes:**
- Photos.app replaced iPhoto in 2015 (OS X Yosemite)
- Uses SQLite database (Photos.sqlite) for all metadata
- Stores originals in UUID-based directory structure
- Supports all modern formats: JPEG, PNG, HEIC, RAW (CR2, NEF, ARW, etc.), Live Photos

### Hash Algorithm
- **SHA-256**: Industry standard, good balance of speed and collision resistance
- Will hash the actual image file content, not metadata

## Architecture

### Module Structure
```
iphoto-sifter/
├── src/
│   ├── main.rs              # CLI entry point and argument parsing
│   ├── lib.rs               # Library root
│   ├── error.rs             # Error types
│   ├── photo/
│   │   ├── mod.rs           # Photo module
│   │   ├── hasher.rs        # Photo hashing functionality
│   │   └── scanner.rs       # Directory scanning
│   ├── photos/
│   │   ├── mod.rs           # Photos.app module
│   │   ├── library.rs       # Library structure and parsing
│   │   ├── database.rs      # SQLite database access
│   │   └── writer.rs        # Add photos to library
│   ├── compare.rs           # Comparison logic
│   └── report.rs            # Reporting functionality
├── Cargo.toml
└── tests/
    └── integration_tests.rs
```

## Required Rust Crates

### Core Dependencies
1. **clap** (v4) - CLI argument parsing with derive macros
2. **sha2** - SHA-256 hashing implementation
3. **walkdir** - Recursive directory traversal
4. **anyhow** - Error handling
5. **thiserror** - Custom error types

### Photos.app Library Parsing
6. **rusqlite** - SQLite database access for Photos.sqlite
7. **serde** - Serialization/deserialization
8. **serde_json** - JSON handling for reports

### Additional Utilities
10. **chrono** - Date/time handling
11. **image** - Image format validation (optional, for file type detection)
12. **indicatif** - Progress bars for better UX
13. **colored** - Colored terminal output for reports

## Implementation Plan

### Phase 1: Project Setup
- [ ] Initialize Cargo project
- [ ] Set up Cargo.toml with dependencies
- [ ] Create module structure
- [ ] Set up basic CLI with clap
- [ ] Add error types

### Phase 2: Photo Hashing & Scanning
- [ ] Implement photo file detection (jpg, jpeg, png, heic, etc.)
- [ ] Implement SHA-256 hashing for files
- [ ] Implement recursive directory scanner
- [ ] Create PhotoInfo struct (path, hash, metadata)
- [ ] Add progress indicators

### Phase 3: iPhoto Library Reading
- [ ] Implement iPhoto library detection (version detection)
- [ ] Implement XML parser for older iPhoto libraries
- [ ] Implement SQLite parser for newer iPhoto libraries
- [ ] Create unified library representation
- [ ] Extract existing photo hashes from library
- [ ] Map library photos to their Masters/ locations

### Phase 4: Comparison Logic
- [ ] Implement hash-based comparison
- [ ] Identify photos in directory but not in library
- [ ] Handle duplicate detection (same hash, different path)
- [ ] Create diff report structure

### Phase 5: Adding Photos to iPhoto
**Note: This is the most complex part**
- [ ] Research iPhoto's import mechanism
- [ ] Implement file copying to Masters/ directory
- [ ] Update AlbumData.xml (for XML-based libraries)
- [ ] Update SQLite database (for DB-based libraries)
- [ ] Handle metadata preservation (EXIF, timestamps)
- [ ] Create backup mechanism before modifications

### Phase 6: Reporting
- [ ] Implement summary report (photos scanned, found, added)
- [ ] Create detailed report with file paths
- [ ] Add JSON export option
- [ ] Implement dry-run mode (show what would be added)
- [ ] Add verbose logging option

### Phase 7: Testing & Polish
- [ ] Unit tests for hashing
- [ ] Unit tests for parsing
- [ ] Integration tests with sample libraries
- [ ] Error handling improvements
- [ ] Documentation (README, --help)
- [ ] Performance optimization

## CLI Interface Design

```bash
# Basic usage
iphoto-sifter --source /path/to/photos --library /path/to/Library.photolibrary

# With options
iphoto-sifter \
  --source /path/to/photos \
  --library /path/to/Library.photolibrary \
  --dry-run \              # Show what would be added without making changes
  --verbose \              # Detailed logging
  --report report.json     # Export report to JSON

# Short form
iphoto-sifter -s /photos -l /Library.photolibrary -n
```

### Expected Arguments
- `--source, -s`: Source directory to scan for photos
- `--library, -l`: Path to iPhoto library
- `--dry-run, -n`: Preview mode, don't make changes
- `--verbose, -v`: Verbose output
- `--report, -r`: Export report to file (JSON)
- `--recursive, -R`: Scan source directory recursively (default: true)

## Technical Challenges & Solutions

### Challenge 1: iPhoto Library Modification
**Problem**: Modifying iPhoto libraries is risky - corruption could lose user data.

**Solutions**:
1. **Create backup** before any modifications
2. **Dry-run mode** as default, require explicit `--commit` flag
3. **Validate library** before and after changes
4. **Atomic operations** - all changes succeed or all fail
5. Consider **warning users** to close iPhoto before running

### Challenge 2: iPhoto Version Detection
**Problem**: Different iPhoto versions use different formats.

**Solution**:
1. Check for AlbumData.xml (older versions)
2. Check for Database/apdb/ directory (newer versions)
3. Parse version info from library metadata
4. Support both formats with abstraction layer

### Challenge 3: Photo Format Support
**Problem**: Many photo formats exist (JPEG, PNG, HEIC, RAW, etc.)

**Solution**:
1. Define supported formats list
2. Use file extension detection
3. Optionally validate with `image` crate
4. Make format list configurable

### Challenge 4: Performance
**Problem**: Hashing large photo collections is slow.

**Solutions**:
1. Use parallel hashing with rayon
2. Add progress indicators
3. Consider caching hashes
4. Stream file reading for large files

### Challenge 5: Duplicate Handling
**Problem**: Same photo might exist in multiple locations.

**Solution**:
1. Hash-based deduplication
2. Report duplicates to user
3. Let user decide how to handle (skip, all locations, etc.)

## Alternative Approach: AppleScript Integration

**Note**: Instead of directly modifying iPhoto library files, we could:
1. Use AppleScript to automate iPhoto's import function
2. Safer and uses official APIs
3. Requires iPhoto to be running
4. May be slower but more reliable

This could be a Phase 2 feature or alternative implementation mode.

## Success Criteria

1. ✅ Can successfully scan directories and hash photos
2. ✅ Can read and parse iPhoto libraries (both formats)
3. ✅ Can identify missing photos accurately
4. ✅ Can add photos to library without corruption
5. ✅ Provides clear, actionable reports
6. ✅ Handles errors gracefully with helpful messages
7. ✅ Includes dry-run mode for safety
8. ✅ Performance: Can handle 10,000+ photos efficiently

## Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Library corruption | High | Medium | Backups, dry-run default, validation |
| iPhoto version incompatibility | Medium | Medium | Support both XML and SQLite formats |
| Poor performance | Low | Low | Parallel processing, progress indicators |
| Missing edge cases | Medium | High | Comprehensive testing, dry-run mode |

## Timeline Estimate

- Phase 1: Project Setup - **Quick**
- Phase 2: Photo Hashing - **Moderate**
- Phase 3: iPhoto Reading - **Complex** (requires format research)
- Phase 4: Comparison - **Quick**
- Phase 5: Adding Photos - **Very Complex** (requires careful implementation)
- Phase 6: Reporting - **Moderate**
- Phase 7: Testing - **Moderate**

## Next Steps

1. Get user approval on this plan
2. Confirm which iPhoto version(s) to target
3. Decide on library modification approach (direct vs AppleScript)
4. Begin Phase 1 implementation

## Open Questions for User

1. **iPhoto version**: Which iPhoto version(s) should we prioritize?
2. **Safety vs Speed**: Should dry-run be the default mode?
3. **Photo formats**: Which formats should be supported? (JPEG, PNG, HEIC, RAW?)
4. **Modification approach**: Direct library modification or AppleScript?
5. **Photos.app**: Should we also support modern Photos.app libraries?
