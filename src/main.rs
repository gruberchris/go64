mod cpu;
mod memory;
mod vic;
mod sid;
mod cia;
mod io;
mod basic;
mod ui;
mod debugger;
mod keyboard;
mod storage;

use anyhow::Result;
use clap::Parser;
use crossterm::event::KeyCode;

#[derive(Parser, Debug)]
#[command(name = "go64")]
#[command(about = "Commodore 64 Emulator", long_about = None)]
struct Args {
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,
    
    /// Run without UI (for testing)
    #[arg(long)]
    no_ui: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.no_ui {
        run_headless(args.debug)?;
    } else {
        run_with_ui(args.debug)?;
    }
    
    Ok(())
}

fn run_headless(debug: bool) -> Result<()> {
    use memory::Memory;
    println!("go64 - Commodore 64 Emulator (Headless Mode)");
    println!("Initializing...");
    
    // Initialize storage
    storage::init()?;

    let mut cpu = cpu::Cpu::new();
    let mut memory = memory::C64Memory::new();
    let _vic = vic::VicII::new();

    // Load ROMs
    io::create_rom_directory_if_missing()?;
    match io::RomSet::load_from_directory("roms") {
        Ok(roms) => {
            println!("✅ ROMs loaded successfully!");
            memory.load_basic_rom(roms.basic);
            memory.load_kernal_rom(roms.kernal);
            memory.load_char_rom(roms.char_rom);
            
            // Reset CPU to start execution from KERNAL reset vector
            cpu.reset(&memory);
            println!("✅ CPU Reset. PC=${:04X}", cpu.pc);
        }
        Err(e) => {
            println!("⚠️  Could not load ROMs: {}", e);
            return Ok(());
        }
    }

    if debug {
        println!("CPU initialized: {:?}", cpu);
    }

    println!("Starting execution loop (Press Ctrl+C to stop)...");
    
            // Execute loop
    let mut cycles_total: u64 = 0;
    let start_time = std::time::Instant::now();
    let mut last_log = std::time::Instant::now();
    let mut _frame_count = 0;
    
    // Debug: Track if we're stuck
    let mut last_pc = 0;
    let mut stuck_count = 0;
    
    loop {
        // Execute one instruction
        match cpu.step(&mut memory) {
            Ok(cycles) => {
                cycles_total += cycles as u64;
                let irq1 = memory.cia1.tick(cycles);
                let irq2 = memory.cia2.tick(cycles);
                let irq_vic = memory.vic.tick(cycles);
                
                // Trigger IRQ if CIA requested it
                if irq1 || irq2 || irq_vic {
                    cpu.irq(&mut memory);
                }
            }
            Err(e) => {
                println!("CPU Error: {}", e);
                break;
            }
        }
        
        // Check for stuck loop
        if cpu.pc == last_pc {
            stuck_count += 1;
            if stuck_count == 1000 {
                println!("⚠️  STUCK at PC=${:04X} for >1000 instructions", cpu.pc);
                println!("   A=${:02X} X=${:02X} Y=${:02X} SP=${:02X} Status=${:02X}", 
                         cpu.a, cpu.x, cpu.y, cpu.sp, cpu.status.as_byte());
                
                // Disassemble a few bytes around PC
                print!("   Code: ");
                for i in 0..6 {
                    print!("{:02X} ", memory.read(cpu.pc.wrapping_add(i)));
                }
                println!("");
                
                // Check memory banking config at 0x0001
                println!("   Mem config: $0001=${:02X}", memory.read(0x0001));
                
                println!("   Mem config: $0001=${:02X}", memory.read(0x0001));
                println!("   VIC $D012: ${:02X}", memory.read(0xD012));
                
                // Check CIA interrupt state
                println!("   CIA1 ICR=${:02X} Mask=${:02X} TimerA=${:04X} Control=${:02X}", 
                         memory.cia1.icr, memory.cia1.icr_mask, 
                         ((memory.cia1.read(0xDC05) as u16) << 8) | memory.cia1.read(0xDC04) as u16,
                         memory.cia1.cra);
            }
        } else {
            stuck_count = 0;
            last_pc = cpu.pc;
        }
        
        // Log status every second
        if last_log.elapsed().as_secs() >= 1 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let mhz = (cycles_total as f64 / elapsed) / 1_000_000.0;
            print!("t={:.1}s | PC=${:04X} | Speed: {:.3} MHz | Cycles: {} | Code: ", 
                     elapsed, cpu.pc, mhz, cycles_total);
            
            // Print next 3 bytes
            for i in 0..3 {
                print!("{:02X} ", memory.read(cpu.pc.wrapping_add(i)));
            }
            println!("");
            
            last_log = std::time::Instant::now();
            
            // Exit after 5 seconds for testing
            if elapsed > 5.0 {
                println!("Test run complete.");
                break;
            }
        }
    }

    Ok(())
}

fn run_with_ui(_debug: bool) -> Result<()> {
    // Initialize storage
    storage::init()?;

    // Try to load ROMs
    io::create_rom_directory_if_missing()?;
    
    let roms_loaded = match io::RomSet::load_from_directory("roms") {
        Ok(_roms) => {
            println!("✅ ROMs loaded successfully!");
            use std::io::Write;
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open("/tmp/go64_keyboard.txt")
                .and_then(|mut f| writeln!(f, "🟢 STARTUP: roms_loaded = true"));
            true
        }
        Err(e) => {
            println!("⚠️  Could not load ROMs: {}", e);
            println!("Running in demo mode without ROMs...\n");
            use std::io::Write;
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open("/tmp/go64_keyboard.txt")
                .and_then(|mut f| writeln!(f, "🔴 STARTUP: roms_loaded = false, error: {}", e));
            false
        }
    };
    
    let mut cpu = cpu::Cpu::new();
    let mut memory = memory::C64Memory::new();
    
    // Our own cursor position for direct screen writes
    let _test_cursor_col: u16 = 0;
    let _test_cursor_row: u16 = 6;
    
    // Load ROMs if available
    if roms_loaded {
        let roms = io::RomSet::load_from_directory("roms")?;
        memory.load_basic_rom(roms.basic);
        memory.load_kernal_rom(roms.kernal);
        memory.load_char_rom(roms.char_rom);
        
        // STANDARD BOOT
        cpu.reset(&mut memory); // Vectors from $FFFC/$FFFD ($FCE2)
        println!("✅ System reset. Executing KERNAL boot sequence...");
        
        // Ensure CPU interrupts are enabled in our emulator struct so we don't block them artificially
        // (The 6502 starts with I flag set (disabled) after reset, which is correct)
    } else {
        // Demo mode: Write test message to screen
        use memory::Memory;
        let test_msg = b"    **** COMMODORE 64 BASIC V2 ****     64K RAM SYSTEM  38911 BASIC BYTES FREE  READY.";
        for (i, &ch) in test_msg.iter().enumerate() {
            if i < 40 * 25 {
                memory.write(0x0400 + i as u16, ch);
            }
        }
        
        // Add a cursor
        memory.write(0x0400 + 7 * 40, 0xA0);
    }
    
    let mut ui = ui::TerminalUI::new()?;
    let mut running_cpu = true;  // Enable CPU by default for standard boot
    let mut _frame_count = 0;
    let mut show_debug = false;  // Hide debug info by default, toggle with F1
    
    'mainloop: loop {
        // Render the screen
        ui.render(|frame| {
            if show_debug {
                let (title_area, screen_area, status_area) = ui::create_layout(frame.size());
                ui::render_title_bar(frame, title_area);
                use memory::Memory;
                ui::render_c64_screen(frame, screen_area, &memory.vic, &memory as &dyn Memory);
                ui::render_status_bar(frame, status_area, &cpu);
            } else {
                // Simple layout without debug info
                let (screen_area, status_area) = ui::create_simple_layout(frame.size());
                use memory::Memory;
                ui::render_c64_screen(frame, screen_area, &memory.vic, &memory as &dyn Memory);
                ui::render_simple_status(frame, status_area);
            }
        })?;
        
        // Handle input
        while let Some(key) = ui.poll_event()? {
            match key.code {
                KeyCode::Esc => {
                    // Quit the emulator
                    break 'mainloop;
                },
                KeyCode::F(1) => {
                    // Toggle debug view
                    show_debug = !show_debug;
                }
                KeyCode::F(5) => {
                    // Enable/toggle CPU execution
                    running_cpu = !running_cpu;
                    if running_cpu {
                        use std::fs::OpenOptions;
                        use std::io::Write;
                        if let Ok(mut file) = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("/tmp/go64_kbd_debug.txt") {
                            writeln!(file, "=== CPU execution enabled at PC=${:04X} ===", cpu.pc).ok();
                        }
                    }
                }
                KeyCode::PageUp => {
                    // RESTORE key simulation (NMI)
                    cpu.nmi(&mut memory);
                }
                KeyCode::Tab => {
                     // Explicitly handle Tab as Run/Stop for clarity, though map_key handles it too
                     // This ensures it gets registered if map_key is missed or we want debug logic
                     memory.cia1.set_key(7, 7, true); 
                }
                _ => {
                    // Map terminal key to C64 keyboard matrix
                    if let Some(positions) = keyboard::map_key(key.code) {
                        use std::io::Write;
                        let _ = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("/tmp/go64_keyboard.txt")
                            .and_then(|mut f| writeln!(f, "🎹 Key {:?} -> positions: {:?}", key.code, positions));
                        
                        // Set in CIA matrix - BASIC will read via our intercepted GETIN
                        for (row, col) in positions {
                            memory.cia1.set_key(row, col, true);
                        }
                    }
                }
            }
        }
        
        // Execute CPU cycles if ROMs are loaded
        if running_cpu {
            use memory::Memory;
            // C64 runs at ~985,248 Hz (PAL) ≈ 1MHz
            // At 60fps: 985248 / 60 ≈ 16,420 cycles per frame
            // Execute cycles and trigger IRQ once per frame
            const CYCLES_PER_FRAME: u64 = 16420;
            
            let cycles_this_frame = CYCLES_PER_FRAME;
            
            for _ in 0..cycles_this_frame {
                // Execute one CPU instruction
                match cpu.step(&mut memory) {
                    Ok(cycles) => {
                        // Tick CIA timers based on actual cycles executed
                        let irq1 = memory.cia1.tick(cycles);
                        let irq2 = memory.cia2.tick(cycles);
                        
                        // Tick VIC-II (raster beam)
                        let irq_vic = memory.vic.tick(cycles);
                        
                        // Connect CIA IRQs to CPU
                        if irq1 || irq2 || irq_vic {
                            cpu.irq(&mut memory);
                        }
                    },
                    Err(e) => {
                        // Hit unimplemented opcode or error
                        eprintln!("CPU Error: {} at PC=${:04X}", e, cpu.pc.wrapping_sub(1));
                        // Print some context
                        eprintln!("  A=${:02X} X=${:02X} Y=${:02X} SP=${:02X}", cpu.a, cpu.x, cpu.y, cpu.sp);
                        let prev_pc = cpu.pc.wrapping_sub(1);
                        eprintln!("  Memory at PC-1: ${:02X}", memory.read(prev_pc));
                        running_cpu = false;
                        break;
                    }
                }
            }
        }
        
        // Update jiffy clock once per frame (not using IRQs)
        // if roms_loaded { ... } logic removed until IRQs are working
        
        // Clear keyboard after each frame (keys only pressed for ~16ms)
        // This simulates key press/release and allows KERNAL to detect keypresses
        memory.cia1.clear_keyboard();
        
        // Slow down to ~60 FPS
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    
    Ok(())
}
