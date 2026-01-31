# go64 - Commodore 64 Emulator

A Commodore 64 emulator written in Rust with a terminal-based UI (TUI). It emulates the MOS 6502 CPU, VIC-II video chip, and CIA timers to run the original C64 KERNAL and BASIC V2 operating system in your terminal.

## Getting Started

### Prerequisites
1.  **Rust Toolchain**: Install via [rustup.rs](https://rustup.rs).
2.  **C64 ROM Files**: You must provide the original Commodore 64 ROMs (BASIC, KERNAL, and CHAR).

### Setup

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/yourusername/go64.git
    cd go64
    ```

2.  **Install ROMs:**
    Create a `roms/` directory in the project root and copy the following files into it:
    *   `basic.rom` (8KB) - The BASIC V2 Interpreter
    *   `kernal.rom` (8KB) - The C64 Operating System
    *   `char.rom` (4KB) - The Character Set Generator

    *Note: These files can be extracted from other emulators like VICE or downloaded from C64 preservation sites.*

3.  **Run the Emulator:**
    ```bash
    cargo run --release
    ```

## Controls

*   **ESC**: Quit the emulator
*   **F1**: Toggle Debug Overlay (CPU registers, PC, cycles)
*   **PageUp**: `RESTORE` key (triggers NMI for soft reset)
*   **F5**: Toggle CPU execution (pause/resume)
*   **Typing**: Maps your PC keyboard to the C64 keyboard matrix.

## Emulation Status

| System | Status | Details |
| :--- | :--- | :--- |
| **CPU** | ✅ Working | Full MOS 6502 instruction set (unofficial opcodes not yet supported). |
| **Memory** | ✅ Working | Complete 64KB RAM + ROM Banking (BASIC/KERNAL/IO switching). |
| **VIC-II** | ⚠️ Partial | **Text Mode only**. Authentic PAL color palette. No Sprites or Bitmaps. |
| **CIA** | ⚠️ Partial | Timers A/B, IRQs, and Keyboard Matrix implemented. No Serial Bus (IEC). |
| **SID** | ❌ Missing | No sound support yet. |
| **Storage** | ❌ Missing | No disk drive (1541) or tape emulation. Programs live in RAM only. |

## Example: Testing Colors

You can test the emulator's functionality by typing this BASIC program to cycle border and background colors:

```basic
10 REM === COLOR & TEXT TEST ===
20 PRINT CHR$(147)
30 FOR I = 0 TO 15
40 POKE 53280, I
50 POKE 53281, 15-I
60 POKE 646, I
70 PRINT "COLOR TEST " I
80 FOR J = 0 TO 200 : NEXT J
90 NEXT I
100 POKE 53280, 14 : POKE 53281, 6 : POKE 646, 14
110 PRINT "TEST COMPLETE."
120 END
```

## Development

### Building
To build the project executable without running it (useful for checking compilation errors):

```bash
cargo build --release
```

*Note: The `--release` flag is highly recommended for performance. The emulator relies on being able to execute ~1 million cycles per second, which debug builds may struggle to maintain.*

### Running Unit Tests
The project includes a comprehensive suite of unit tests, particularly for the CPU instruction set. To run them:

```bash
cargo test
```

To run only the CPU tests:

```bash
cargo test cpu
```

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
