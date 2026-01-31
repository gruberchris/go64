// Memory interface for C64

pub trait Memory {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
    
    // For VIC-II access
    // fn read_vic(&self, addr: u16) -> u8 {
    //    self.read(addr)
    // }
}

// C64 Memory Map:
// $0000-$0001: CPU I/O port
// $0002-$00FF: Zero page
// $0100-$01FF: Stack
// $0200-$9FFF: RAM (38KB)
// $A000-$BFFF: BASIC ROM (8KB) or RAM
// $C000-$CFFF: RAM (4KB)
// $D000-$DFFF: I/O or Character ROM or RAM (4KB)
// $E000-$FFFF: KERNAL ROM (8KB) or RAM

const RAM_SIZE: usize = 0x10000; // 64KB

pub struct C64Memory {
    ram: [u8; RAM_SIZE],
    basic_rom: Option<Vec<u8>>,    // $A000-$BFFF
    kernal_rom: Option<Vec<u8>>,   // $E000-$FFFF
    char_rom: Option<Vec<u8>>,     // $D000-$DFFF
    
    // Memory banking control
    port_0000: u8, // Data direction register
    port_0001: u8, // Data port (controls memory banking)
    
    // VIC-II chip reference
    pub vic: crate::vic::VicII,
    
    // CIA chips
    pub cia1: crate::cia::Cia, // $DC00-$DCFF
    pub cia2: crate::cia::Cia, // $DD00-$DDFF
}

impl C64Memory {
    pub fn new() -> Self {
        let mut mem = Self {
            ram: [0; RAM_SIZE],
            basic_rom: None,
            kernal_rom: None,
            char_rom: None,
            port_0000: 0xFF,
            port_0001: 0x37, // Default: BASIC+KERNAL visible, I/O visible
            vic: crate::vic::VicII::new(),
            cia1: crate::cia::Cia::new(),
            cia2: crate::cia::Cia::new(),
        };
        
        // Initialize default reset vector to point to $FCE2 (KERNAL cold start)
        mem.ram[0xFFFC] = 0xE2;
        mem.ram[0xFFFD] = 0xFC;
        
        mem
    }
    
    pub fn load_basic_rom(&mut self, data: Vec<u8>) {
        if data.len() == 0x2000 {
            self.basic_rom = Some(data);
        }
    }
    
    pub fn load_kernal_rom(&mut self, data: Vec<u8>) {
        if data.len() == 0x2000 {
            self.kernal_rom = Some(data);
        }
    }
    
    pub fn load_char_rom(&mut self, data: Vec<u8>) {
        if data.len() == 0x1000 {
            self.char_rom = Some(data);
        }
    }
    
    fn is_basic_visible(&self) -> bool {
        // BASIC ROM visible when bits 0 and 1 are both 1
        (self.port_0001 & 0x03) == 0x03
    }
    
    fn is_kernal_visible(&self) -> bool {
        // KERNAL ROM visible when bit 1 is 1
        (self.port_0001 & 0x02) != 0
    }
    
    fn is_io_visible(&self) -> bool {
        // I/O visible when bit 2 is 1 and bit 1 is 1
        (self.port_0001 & 0x07) >= 0x05
    }
    
    fn is_char_rom_visible(&self) -> bool {
        // CHAR ROM visible when bits 0-2 are specific values
        let bits = self.port_0001 & 0x07;
        bits == 0x01 || bits == 0x03
    }
    
    // /// Scan keyboard and update keyboard buffer (simulates KERNAL keyboard scanner)
    // /// This should be called ~60 times per second (during IRQ handling)
    // pub fn scan_keyboard(&mut self) {
    //     // Keyboard buffer is at $0277-$0280 (10 bytes)
    //     // $C6 = number of characters in buffer (NDX)
    //     let buffer_len = self.ram[0xC6] as usize;
    //     
    //     // Don't overflow buffer
    //     if buffer_len >= 10 {
    //         return;
    //     }
    //     
    //     // Scan all 8 rows of the keyboard matrix
    //     for row in 0..8 {
    //         // Set CIA1 Port A to scan this row (active low)
    //         let row_mask = !(1 << row);
    //         self.cia1.pra = row_mask;
    //         
    //         // Read CIA1 Port B to get column data
    //         let cols = self.cia1.read_keyboard_columns();
    //         
    //         // Check each column for a keypress
    //         for col in 0..8 {
    //             if (cols & (1 << col)) == 0 {
    //                 // Key is pressed (active low)
    //                 // Convert row/col to C64 key code
    //                 let key_code = self.matrix_to_keycode(row, col);
    //                 
    //                 if key_code == 0 {
    //                     continue; // Skip modifier keys and special keys for now
    //                 }
    //                 
    //                 // Check if this is a different key from last time
    //                 let last_key = self.ram[0xCB];
    //                 if key_code != last_key {
    //                     // New key pressed - add to buffer
    //                     self.ram[0xCB] = key_code;
    //                     
    //                     // Add to keyboard buffer
    //                     let buffer_start = 0x0277;
    //                     self.ram[buffer_start + buffer_len] = key_code;
    //                     self.ram[0xC6] = (buffer_len + 1) as u8;
    //                     
    //                     // Write to debug file (don't use println - it breaks TUI)
    //                     use std::fs::OpenOptions;
    //                     use std::io::Write;
    //                     if let Ok(mut file) = OpenOptions::new()
    //                         .create(true)
    //                         .append(true)
    //                         .open("/tmp/go64_kbd_debug.txt") {
    //                         writeln!(file, "Key pressed: row={}, col={}, code=${:02X} ({}), buffer_len={}", 
    //                                  row, col, key_code, key_code as char, buffer_len + 1).ok();
    //                     }
    //                     
    //                     return; // Only process one key per scan
    //                 }
    //             }
    //         }
    //     }
    //     
    //     // No keys pressed - reset last key
    //     self.ram[0xCB] = 0;
    // }
    
    // /// Convert keyboard matrix position to C64 PETSCII/scan code
    // pub fn matrix_to_keycode_direct(&self, row: u8, col: u8) -> Option<u8> {
    //     let code = self.matrix_to_keycode(row, col);
    //     if code == 0 { None } else { Some(code) }
    // }

    // fn matrix_to_keycode(&self, row: u8, col: u8) -> u8 {
    //     // C64 keyboard matrix to PETSCII mapping
    //     // This is a simplified version - real C64 has shift/control handling
    //     match (row, col) {
    //         // Row 0
    //         (0, 0) => 20,  // DEL
    //         (0, 1) => 13,  // RETURN
    //         (0, 2) => 29,  // Cursor Right
    //         (0, 3) => 0,   // F7 (special)
    //         (0, 4) => 0,   // F1 (special)
    //         (0, 5) => 0,   // F3 (special)
    //         (0, 6) => 0,   // F5 (special)
    //         (0, 7) => 17,  // Cursor Down
    //         
    //         // Row 1: 3, W, A, 4, Z, S, E, Shift
    //         (1, 0) => b'3',
    //         (1, 1) => b'W',
    //         (1, 2) => b'A',
    //         (1, 3) => b'4',
    //         (1, 4) => b'Z',
    //         (1, 5) => b'S',
    //         (1, 6) => b'E',
    //         (1, 7) => 0,  // Left Shift (modifier)
    //         
    //         // Row 2: 5, R, D, 6, C, F, T, X
    //         (2, 0) => b'5',
    //         (2, 1) => b'R',
    //         (2, 2) => b'D',
    //         (2, 3) => b'6',
    //         (2, 4) => b'C',
    //         (2, 5) => b'F',
    //         (2, 6) => b'T',
    //         (2, 7) => b'X',
    //         
    //         // Row 3: 7, Y, G, 8, B, H, U, V
    //         (3, 0) => b'7',
    //         (3, 1) => b'Y',
    //         (3, 2) => b'G',
    //         (3, 3) => b'8',
    //         (3, 4) => b'B',
    //         (3, 5) => b'H',
    //         (3, 6) => b'U',
    //         (3, 7) => b'V',
    //         
    //         // Row 4: 9, I, J, 0, M, K, O, N
    //         (4, 0) => b'9',
    //         (4, 1) => b'I',
    //         (4, 2) => b'J',
    //         (4, 3) => b'0',
    //         (4, 4) => b'M',
    //         (4, 5) => b'K',
    //         (4, 6) => b'O',
    //         (4, 7) => b'N',
    //         
    //         // Row 5: +, P, L, -, ., :, @, ,
    //         (5, 0) => b'+',
    //         (5, 1) => b'P',
    //         (5, 2) => b'L',
    //         (5, 3) => b'-',
    //         (5, 4) => b'.',
    //         (5, 5) => b':',
    //         (5, 6) => b'@',
    //         (5, 7) => b',',
    //         
    //         // Row 6: Pound, *, ;, HOME, Right Shift, =, Up Arrow, /
    //         (6, 0) => 0,   // Pound sign
    //         (6, 1) => b'*',
    //         (6, 2) => b';',
    //         (6, 3) => 19,  // HOME
    //         (6, 4) => 0,   // Right Shift (modifier)
    //         (6, 5) => b'=',
    //         (6, 6) => 145, // Cursor Up
    //         (6, 7) => b'/',
    //         
    //         // Row 7: 1, Left Arrow, CTRL, 2, SPACE, Commodore, Q, Run/Stop
    //         (7, 0) => b'1',
    //         (7, 1) => 157, // Cursor Left
    //         (7, 2) => 0,   // CTRL (modifier)
    //         (7, 3) => b'2',
    //         (7, 4) => b' ',
    //         (7, 5) => 0,   // Commodore key
    //         (7, 6) => b'Q',
    //         (7, 7) => 3,   // RUN/STOP
    //         
    //         _ => 0,
    //     }
    // }
}

impl Memory for C64Memory {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000 => self.port_0000,
            0x0001 => self.port_0001,
            
            // Zero page, stack, and low RAM
            0x0002..=0x9FFF => {
                // Debug: Track reads of keyboard buffer
                if addr == 0xC6 && self.ram[0xC6] > 0 {
                    use std::fs::OpenOptions;
                    use std::io::Write;
                    if let Ok(mut file) = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/go64_kbd_debug.txt") {
                        writeln!(file, "Reading keyboard buffer length $C6={}", self.ram[0xC6]).ok();
                    }
                }
                self.ram[addr as usize]
            },
            
            // BASIC ROM area
            0xA000..=0xBFFF => {
                if self.is_basic_visible() {
                    if let Some(ref rom) = self.basic_rom {
                        rom[(addr - 0xA000) as usize]
                    } else {
                        self.ram[addr as usize]
                    }
                } else {
                    self.ram[addr as usize]
                }
            }
            
            // High RAM
            0xC000..=0xCFFF => self.ram[addr as usize],
            
            // I/O or CHAR ROM area
            0xD000..=0xDFFF => {
                if self.is_io_visible() {
                    match addr {
                        // VIC-II registers: $D000-$D3FF (repeats every $0040 bytes)
                        0xD000..=0xD3FF => {
                            let reg = addr & 0x003F;
                            self.vic.read_register(reg)
                        }
                        // SID registers: $D400-$D7FF
                        0xD400..=0xD7FF => {
                            0 // SID not implemented yet
                        }
                        // Color RAM: $D800-$DBFF
                        0xD800..=0xDBFF => {
                            self.vic.read_color_ram(addr - 0xD800)
                        }
                        // CIA1: $DC00-$DCFF
                        0xDC00..=0xDCFF => {
                            self.cia1.read(addr)
                        }
                        // CIA2: $DD00-$DDFF
                        0xDD00..=0xDDFF => {
                            self.cia2.read(addr)
                        }
                        _ => self.ram[addr as usize]
                    }
                } else if self.is_char_rom_visible() {
                    if let Some(ref rom) = self.char_rom {
                        rom[(addr - 0xD000) as usize]
                    } else {
                        self.ram[addr as usize]
                    }
                } else {
                    self.ram[addr as usize]
                }
            }
            
            // KERNAL ROM area
            0xE000..=0xFFFF => {
                if self.is_kernal_visible() {
                    if let Some(ref rom) = self.kernal_rom {
                        rom[(addr - 0xE000) as usize]
                    } else {
                        self.ram[addr as usize]
                    }
                } else {
                    self.ram[addr as usize]
                }
            }
        }
    }
    
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000 => self.port_0000 = value,
            0x0001 => {
                self.port_0001 = value;
                // Note: Only bits controlled by port_0000 are writable
                // For simplicity, we allow all bits for now
            }
            
            // I/O area writes
            0xD000..=0xDFFF => {
                if self.is_io_visible() {
                    match addr {
                        // VIC-II registers: $D000-$D3FF
                        0xD000..=0xD3FF => {
                            let reg = addr & 0x003F;
                            self.vic.write_register(reg, value);
                            return;
                        }
                        // SID: $D400-$D7FF
                        0xD400..=0xD7FF => {
                            // SID not implemented yet
                            return;
                        }
                        // Color RAM: $D800-$DBFF (always writable even through I/O)
                        0xD800..=0xDBFF => {
                            self.vic.write_color_ram(addr - 0xD800, value);
                            return;
                        }
                        // CIA1: $DC00-$DCFF
                        0xDC00..=0xDCFF => {
                            use std::fs::OpenOptions;
                            use std::io::Write;
                            if addr == 0xDC0D || (addr >= 0xDC04 && addr <= 0xDC0F) {
                                let mut file = OpenOptions::new()
                                    .create(true)
                                    .append(true)
                                    .open("/tmp/go64_cia_write.txt")
                                    .ok();
                                if let Some(ref mut f) = file {
                                    writeln!(f, "Write ${:04X} = ${:02X}", addr, value).ok();
                                }
                            }
                            self.cia1.write(addr, value);
                            return;
                        }
                        // CIA2: $DD00-$DDFF
                        0xDD00..=0xDDFF => {
                            self.cia2.write(addr, value);
                            return;
                        }
                        _ => {}
                    }
                }
                // Fall through to RAM
                self.ram[addr as usize] = value;
            }
            
            // All other writes go to RAM (ROMs are not writable)
            _ => {
                // Debug: Log writes to screen memory
                if addr >= 0x0400 && addr < 0x0800 && value >= 0x20 && value < 0x80 {
                    use std::fs::OpenOptions;
                    use std::io::Write;
                    if let Ok(mut file) = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/go64_kbd_debug.txt") {
                        writeln!(file, "✍️  Screen write: addr=${:04X}, char=${:02X} ('{}')", 
                                 addr, value, value as char).ok();
                    }
                }
                self.ram[addr as usize] = value;
            }
        }
    }
}

// // Simple memory for testing
// pub struct BasicMemory {
//     ram: [u8; 0x10000], // 64KB
// }
// 
// impl BasicMemory {
//     pub fn new() -> Self {
//         Self {
//             ram: [0; 0x10000],
//         }
//     }
// }
// 
// impl Memory for BasicMemory {
//     fn read(&self, addr: u16) -> u8 {
//         self.ram[addr as usize]
//     }
// 
//     fn write(&mut self, addr: u16, value: u8) {
//         self.ram[addr as usize] = value;
//     }
// }
