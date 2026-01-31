# go64 - Commodore 64 Emulator

A Commodore 64 emulator written in Rust with a terminal-based UI.

## Status

**Phase 4 Complete!** - ROM Loading & CPU Execution! ✅

### Completed
- ✅ **Phase 1 - 6502 CPU Core** (all 56 opcodes, 21 tests passing)
- ✅ **Phase 2 - Memory System** (C64 memory map with ROM banking)
- ✅ **Phase 3 - VIC-II & Terminal UI** (40x25 screen, authentic colors, centered layout)
- ✅ **Phase 4 - ROM Loading & Execution**
  - ROM file loading from `roms/` directory
  - Automatic ROM validation (correct sizes)
  - CPU execution loop (~1000 cycles/frame)
  - RESET vector support
  - Demo mode (works without ROMs)
  - Helpful error messages and instructions

### Current State
The emulator is **feature-complete for booting C64 BASIC**! It just needs the ROM files.

**What works RIGHT NOW:**
- Full 6502 CPU emulation
- Complete C64 memory system
- Terminal UI with authentic C64 colors
- ROM loading infrastructure
- CPU execution at ~60fps

**What's needed to boot BASIC:**
- Place C64 ROM files in `roms/` directory (see `roms/README.md`)
- ROMs will be loaded automatically on startup
- CPU will execute KERNAL bootup sequence
- BASIC should display and run

### Next - Phase 5: Keyboard Input & KERNAL I/O
- CIA chip emulation (keyboard matrix)
- KERNAL I/O hooks
- Character input/output
- Type into BASIC!

### Planned
- Memory system with C64 memory map
- VIC-II graphics chip (text mode)
- Keyboard input
- BASIC ROM integration
- Terminal UI with ratatui
- Full instruction set
- C64 ROM integration
- And much more...

## Building

```bash
cargo build
```

## Running

### Without ROMs (Demo Mode)
```bash
cargo run
```

Shows a static BASIC screen. Press ESC to quit.

### With C64 ROMs (Full Emulation)

1. **Get ROM files** - See `roms/README.md` for instructions
2. **Place ROMs in `roms/` directory**:
   - `roms/basic.rom` (8KB)
   - `roms/kernal.rom` (8KB)
   - `roms/char.rom` (4KB)
3. **Run**:
```bash
cargo run
```

The emulator will:
- Load ROMs automatically
- Reset the 6502 CPU
- Execute the KERNAL boot sequence
- Display the actual C64 BASIC screen

### Controls
- `ESC` - Quit emulator
- `F5` - Reset CPU

## Testing

```bash
cargo test
```

## Project Goals

- Authentic C64 experience capable of running actual C64 games and code
- Type BASIC programs and run them
- Execute 6502 machine code via POKE/SYS
- Modern conveniences (fast loading, no tape delay simulation)
- Terminal-based UI using ratatui

## Architecture

Single Rust crate with modular design:
- `cpu/` - 6502 CPU emulation
- `memory/` - Memory management and banking
- `vic/` - VIC-II graphics chip
- `sid/` - SID sound chip
- `io/` - Keyboard and I/O
- `basic/` - BASIC interpreter integration
- `ui/` - Terminal UI
- `debugger/` - Development tools

## License

TBD
