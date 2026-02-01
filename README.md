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
*   **F9**: Toggle Debug Overlay (CPU registers, PC, cycles)
*   **PageUp**: `RESTORE` key (triggers NMI).
*   **Tab**: `RUN/STOP` key. (Hold `Tab` + Press `PageUp` for Soft Reset/Restore).
*   **F10**: Toggle CPU execution (pause/resume)
*   **Typing**: Maps your PC keyboard to the C64 keyboard matrix.

## Emulation Status

| System | Status | Details |
| :--- | :--- | :--- |
| **CPU** | ✅ Working | Full MOS 6502 instruction set (unofficial opcodes not yet supported). |
| **Memory** | ✅ Working | Complete 64KB RAM + ROM Banking (BASIC/KERNAL/IO switching). |
| **VIC-II** | ⚠️ Text Only | Authentic PAL color palette. No Sprites or Bitmaps (see Limitations). |
| **CIA** | ⚠️ Partial | Timers A/B, IRQs, and Keyboard Matrix implemented. No Serial Bus (IEC). |
| **SID** | ❌ Not Planned | No sound support (see Limitations). |
| **Storage** | ✅ Working | **Device 8** (Disk) mapped to `~/.go64/1541/`. Tape (Device 1) not supported. |

## Limitations & Technical Constraints

This emulator is built as a **Terminal User Interface (TUI)** application. This design choice imposes specific limitations compared to graphical emulators like VICE:

### 1. Graphics (VIC-II)
*   **Text Mode Only:** The emulator renders into a grid of characters. It cannot natively display the C64's pixel-perfect hardware **Sprites**, smooth scrolling, or high-resolution **Bitmap Modes** (320x200).
*   **Result:** Games relying on sprites or bitmapped graphics will execute logically (CPU instructions run correctly), but the visuals will not appear on screen. Text adventures and BASIC programs work perfectly.

### 2. Sound (SID)
*   **No Audio:** The MOS 6581 SID chip is a complex analog/digital synthesizer. Accurate emulation requires cycle-exact synchronization between the 1MHz CPU and host audio buffers, plus complex waveform mathematics.
*   **Reason:** Implementing audio in a TUI environment introduces significant complexity (threading, ring buffers, latency management) that falls outside the scope of this project's goal: a lightweight, terminal-based emulator.

## Storage (Virtual 1541)

The emulator provides High-Level Emulation (HLE) of a 1541 Disk Drive on **Device 8**.

*   **Filesystem Location:** `~/.go64/1541/`
*   **Supported Commands:**
    *   `LOAD "$",8` - List directory
    *   `LOAD "FILENAME",8` - Load a program
    *   `SAVE "FILENAME",8` - Save a program
*   **Tape (Device 1):** Not supported (returns `DEVICE NOT PRESENT` error).

**Note:** C64 filenames are automatically sanitized to work on your host OS:
*   Special characters (`/`, `\`, `:`, `*`, `?`, etc.) are replaced with `_`.
*   `.prg` extension is automatically appended if missing.

## Debugging

The emulator includes a built-in debug overlay for inspecting the internal state of the 6502 CPU and emulator.

### Using the Debug Overlay
Press **F9** at any time to toggle the debug overlay. This will:
1.  Add a status bar to the bottom of the screen.
2.  Display real-time CPU register values (PC, A, X, Y, SP) and cycle count.

### Debug Controls
*   **F9**: Toggle the debug overlay on/off.
*   **F10**: **Pause/Resume execution**. Use this to freeze the emulator state for inspection.
*   **PageUp**: **RESTORE** (NMI).
*   **Tab**: **RUN/STOP**.
    *   **Soft Reset:** Hold `Tab` (Run/Stop) and press `PageUp` (Restore) to reset the computer (clear screen, reset colors) without rebooting.
*   **ESC**: Quit the emulator.

### Typical Debugging Workflow
1.  **Freeze State**: Press `F5` to pause execution.
2.  **Inspect Registers**: Check the **PC** (Program Counter) to see where execution has stopped.
    *   If PC is stuck in a tight loop (e.g., waiting for a bit to change), you might see the address barely changing.
    *   Check **A/X/Y** registers to verify logic values.
3.  **Reset/Restore**: If the system is unresponsive, press `PgUp` (NMI) to attempt a soft reset via the KERNAL.

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

## Writing Assembly Code

The C64 did not come with a built-in assembler or monitor. To run machine code, users typically wrote BASIC loaders that `POKE`d values directly into memory. You can do the same here!

### Example: Instant Purple Border (Machine Code)

This BASIC program writes a tiny machine code routine to memory address 49152 (`$C000`) that changes the border color to purple (Color 5) and returns.

```basic
10 REM === HAND ASSEMBLED CODE ===
20 FOR A = 49152 TO 49158
30 READ B : POKE A,B
40 NEXT A
50 PRINT "CODE LOADED. TYPE SYS 49152 TO RUN"
60 END
100 DATA 169, 5      :REM LDA #5   (Load Accumulator with color 5/Purple)
110 DATA 141, 32, 208:REM STA $D020(Store Accumulator into Border Color)
120 DATA 96          :REM RTS      (Return To Basic)
```

**To Run:**
1.  Type `RUN` (This POKEs the code into memory).
2.  Type `SYS 49152` (This executes the machine code).
    *   *Result:* The border will turn purple instantly.

### Using a Machine Language Monitor

For serious assembly development, C64 programmers used "Monitor" programs. You can do this too:

1.  **Download** a C64 monitor program (e.g., **SuperMon 64** or **Micromon**) as a `.prg` file.
2.  **Place** the file in your emulator's disk folder: `~/.go64/1541/`.
3.  **Load** it using the emulator:
    ```basic
    LOAD "SUPERMON",8,1
    ```
4.  **Start** the monitor (often with a `SYS` command provided by the tool's documentation, e.g., `SYS 32768`).

This gives you a native environment to inspect memory, disassemble code, and write assembly instructions directly.

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

This project is licensed under the **GNU General Public License v3.0** - see the [LICENSE](LICENSE) file for details.
