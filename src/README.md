# FreeCrime

FreeCrime is written in Rust using Bevy.

This file document the architecture and design choices.

## General guidelines

### Error Handling

All library operations return a `resources::Result<T>`, using a custom `Error`
enum that distinguishes between:

- `Io`: Standard filesystem or buffer access errors.
- `Parse`: Logic errors (version mismatch, invalid headers, trailing bytes).
- `Other`: Uncategorized runtime failures.

## Components

### Resources

The library follows a strict separation between **file formats** (how data is
stored on disk) and **domain models** (how data is represented in memory):

 - `src/resources/types/`: Contains engine-agnostic domain models (`Map`,
   `Style`, `Mission`, `Font`, etc.). These structures are designed to be used
   by the game engine and are independent of the original binary layouts.
 - `src/resources/parsers/`: Contains format-specific logic (e.g., `cmp.rs`,
   `gry.rs`). These parsers are responsible for reading raw bytes and populating
   the domain models.

The objective is to allow supporting different sources for data (GTA1, GTA2,
FreeCrime custom resources) while keeping the dirty hacks required by the
original game confined to the related parsers.

To ensure the integrity of the reverse-engineering work and detect logic errors
early, all parsers operate in **Strict Mode**:

 - **Version Validation**: Parsers verify version codes (e.g., `331` for CMP)
   and fail on unsupported versions.
 - **Trailing Byte Checks**: Parsers ensure that the entire file (or section) is
   consumed according to the header's expected sizes. Any remaining data
   triggers a `ParseError`.
 - **Bounds Checking**: Offsets (common in CMP and GRY formats) are validated
   against the actual buffer sizes during resolution.

## CLI Tool

A command-line tool is provided in `src/main.rs` to inspect and extract original game files.

### Usage

```bash
# Show summary of a file
cargo run -- info gamedata/gta/NYC.CMP

# Extract content (text from FXT, BMPs from FON/GRY)
cargo run -- extract gamedata/gta/ENGLISH.FXT english.txt

# Generate a map overview BMP
cargo run -- overview gamedata/gta/NYC.CMP

# Interactive 3D map viewer (Bevy)
cargo run -- display gamedata/gta/NYC.CMP gamedata/gta/STYLE001.GRY
```

Supported extraction:
- **FXT**: Decrypts and saves to a plain text file.
- **FON**: Extracts all glyphs as 32-bit BMP images.
- **GRY**: Extracts block faces as 32-bit BMP images.
- **SDT**: Exports sound index metadata as a CSV file.

The `overview` command generates a static top-down overview of the map as a `map_overview.bmp` file.

The `display` command launches an interactive 3D viewer using Bevy.
- **Controls**:
  - **WASD**: Move horizontally
  - **Space/Shift**: Move Up/Down
  - **Q/E/Up/Down/Left/Right**: Rotate camera
