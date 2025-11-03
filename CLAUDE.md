# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based JETS (JSON Event Trace Streaming) trace viewer and tools suite. JETS is a streaming JSON format for hardware execution traces that captures complete execution pipelines as hierarchical tree structures with precise clock timestamps.

The project contains:
- **jets-gui**: Interactive GUI trace viewer built with egui/eframe
- **jets-tracegen**: Synthetic RISC-V SoC trace generator for testing
- **rjets library**: Core parsing, writing, and visualization components

## Build Commands

```bash
# Build all binaries and library
cargo build

# Build with optimizations (recommended for large traces)
cargo build --release

# Build specific binaries
cargo build --bin jets-gui
cargo build --bin jets-tracegen

# Run tests
cargo test

# Run integration tests
cargo test --test integration_test

# Run GUI viewer
cargo run --bin jets-gui [trace_file.jets]

# Run trace generator
cargo run --bin jets-tracegen -- [OPTIONS]

# Generate example trace
cargo run --bin jets-tracegen -- -num_instr 1000 -out example.jets
```

## JETS Format

JETS (JSON Event Trace Streaming) is a JSON Lines format where each line represents one of:
- **header**: Metadata (version, hardware info) - must be first line
- **record**: Hierarchical trace record with start timestamp
- **record_end**: Marks completion with end timestamp
- **annotation**: Non-timed metadata for records
- **event**: Timed operation/state change
- **footer**: Summary statistics - optional, must be last line

Key constraints:
- Records must appear before their children/annotations/events
- Clock values must be monotonically increasing
- Parent records must appear before child records

See JETS.md for complete format specification.

## Architecture

### Trait-Based Abstraction Layer

The codebase uses Rust traits to support multiple trace formats through a unified API:

```
src/traits.rs          - Core trait definitions
src/parser.rs          - JETS format implementation (JetsTraceReader)
src/virtual_reader.rs  - Virtual/synthetic trace implementation
src/pipetrace_reader.rs - Pipetrace format implementation
src/writer.rs          - JETS format writer with Brotli compression
```

**Key Traits:**
- `TraceReader` - Opens and parses trace files
- `TraceData` - Provides access to trace records and metadata
- `TraceMetadata` - Header/footer information
- `TraceRecord<'data>` - Individual record with children/events
- `TraceEvent` - Timed event within a record
- `AttributeAccessor` - Ordered attribute access (preserves insertion order)

### GUI Application Structure (src/jets-gui.rs)

The GUI is organized into modular components:

```
app/                   - Application coordination and state management
  ├─ app_state.rs      - Centralized AppState struct
  ├─ application_coordinator.rs - File loading, error handling, interaction
  ├─ theme_coordinator.rs       - Theme persistence and application
  └─ settings_coordinator.rs    - Settings loading/saving

domain/                - Core business logic (pure functions)
  ├─ tree_operations.rs    - Tree traversal, filtering
  ├─ viewport_operations.rs - Viewport calculations
  └─ visibility.rs         - Visibility state management

presentation/          - Visual styling and color mapping
  └─ color_mapping.rs  - Record-to-color mapping

cache/                 - Performance optimization
  └─ tree_cache.rs     - Tree computation caching

io/                    - File loading and trace generation
  ├─ async_loader.rs   - Asynchronous file loading
  └─ file_loader.rs    - Synchronous file operations

state/                 - State management
  ├─ trace_state.rs    - Loaded trace data
  ├─ tree_state.rs     - Tree UI state (expand/collapse)
  ├─ selection.rs      - Selection state
  ├─ viewport.rs       - Timeline viewport state
  ├─ layout_state.rs   - Panel layout state
  └─ theme_state.rs    - Theme state

ui/                    - UI panel rendering
  ├─ panel_manager.rs  - Panel layout orchestration
  ├─ tree_panel.rs     - Left hierarchical tree view
  ├─ timeline_panel.rs - Right timeline visualization
  ├─ details_panel.rs  - Bottom details view
  ├─ header.rs         - Top menu bar
  └─ status_bar.rs     - Bottom status bar

rendering/             - Low-level rendering
  ├─ tree_renderer.rs     - Tree node rendering
  ├─ timeline_renderer.rs - Timeline bar rendering
  └─ time_axis_renderer.rs - Time axis ticks

utils/                 - Utility functions
  ├─ formatting.rs     - Time/number formatting
  └─ geometry.rs       - Geometric calculations
```

**Important architectural principles:**
- Domain logic is separated from presentation (domain/ vs presentation/)
- State is centralized in AppState and managed by coordinators
- Coordinators handle complex interactions (ApplicationCoordinator, ThemeCoordinator)
- UI panels delegate to domain functions for calculations

### String Interning

The codebase uses `Arc<str>` for string sharing to reduce memory usage when parsing large traces. See `src/string_intern.rs` for the `StringInterner` utility that deduplicates strings during parsing.

### Virtual Scrolling

The tree panel uses virtual scrolling for performance with large traces:
- Only visible rows are rendered
- Scroll position determines visible range
- See `src/ui/virtual_scrolling.rs` and `src/ui/virtual_scroll_manager.rs`

## Trace Generator (src/tracegen.rs)

Generates synthetic RISC-V SoC traces for testing and benchmarking.

**Command-line options:**
```
-num_clt <N>         Number of clusters (default: 1)
-num_core <N>        Cores per cluster (default: 1)
-num_threads <N>     Threads per core (default: 1)
-num_instr <N> [M]   Instruction count or range (default: 100)
-out <FILE>          Output file path (default: trace.jets)
-h, -help, --help    Show help message
```

**Pipeline stages:** F1, F2, D, RN, DS, IS, RR, EX, M (memory ops only), WB, C

**Monotonicity:** Uses buffer-and-sort approach to ensure clock monotonicity despite pipelined execution. All items for each thread are buffered, sorted by clock, then emitted.

See GENERATOR.md for complete implementation details.

## File Compression

The TraceWriter automatically enables Brotli compression for files ending in `.br`:
- `trace.jets` - uncompressed
- `trace.jets.br` - compressed with Brotli (quality 6)
- Typical compression: 60-70% size reduction

The parser transparently handles both compressed and uncompressed files.

## Testing

Integration tests in `tests/integration_test.rs` demonstrate:
- Writing traces with TraceWriter
- Reading traces with trait-based API
- Verifying hierarchical structure
- Testing multiple trace formats

When adding new features:
- Add unit tests in the same file as the implementation
- Add integration tests for end-to-end workflows
- Test with both small and large synthetic traces
