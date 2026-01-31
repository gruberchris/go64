// VIC-II chip emulation (text mode)

// C64 colors (PETSCII color palette)
#[derive(Debug, Clone, Copy)]
pub enum C64Color {
    Black = 0,
    White = 1,
    Red = 2,
    Cyan = 3,
    Purple = 4,
    Green = 5,
    Blue = 6,
    Yellow = 7,
    Orange = 8,
    Brown = 9,
    LightRed = 10,
    DarkGrey = 11,
    Grey = 12,
    LightGreen = 13,
    LightBlue = 14,
    LightGrey = 15,
}

impl C64Color {
    pub fn from_u8(value: u8) -> Self {
        match value & 0x0F {
            0 => C64Color::Black,
            1 => C64Color::White,
            2 => C64Color::Red,
            3 => C64Color::Cyan,
            4 => C64Color::Purple,
            5 => C64Color::Green,
            6 => C64Color::Blue,
            7 => C64Color::Yellow,
            8 => C64Color::Orange,
            9 => C64Color::Brown,
            10 => C64Color::LightRed,
            11 => C64Color::DarkGrey,
            12 => C64Color::Grey,
            13 => C64Color::LightGreen,
            14 => C64Color::LightBlue,
            _ => C64Color::LightGrey,
        }
    }
}

pub const SCREEN_WIDTH: usize = 40;
pub const SCREEN_HEIGHT: usize = 25;

pub struct VicII {
    // Screen memory ($0400-$07E7 default)
    screen_base: u16,
    
    // Color RAM ($D800-$DBE7)
    color_ram: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
    
    // Border and background colors
    border_color: u8,
    background_color: u8,
    
    // VIC registers (simplified for text mode)
    registers: [u8; 64],
    
    // Internal timing
    cycle_count: u16,
    raster_line: u16,
}

impl VicII {
    pub fn new() -> Self {
        Self {
            screen_base: 0x0400,
            color_ram: [C64Color::LightBlue as u8; SCREEN_WIDTH * SCREEN_HEIGHT],
            border_color: C64Color::LightBlue as u8,
            background_color: C64Color::Blue as u8,
            registers: [0; 64],
            cycle_count: 0,
            raster_line: 0,
        }
    }
    
    pub fn read_register(&self, addr: u16) -> u8 {
        let reg = (addr & 0x3F) as usize;
        match reg {
            0x11 => {
                let val = self.registers[0x11] & 0x7F; // Clear bit 7 (stored IRQ bit)
                let raster_high = if self.raster_line > 255 { 0x80 } else { 0 };
                val | raster_high
            }
            0x12 => (self.raster_line & 0xFF) as u8,
            _ => self.registers[reg]
        }
    }
    
    pub fn write_register(&mut self, addr: u16, value: u8) {
        let reg = (addr & 0x3F) as usize;
        self.registers[reg] = value;
        
        match reg {
            0x20 => {
                self.border_color = value & 0x0F;
                // Border color updated (removed debug print)
            }
            0x21 => {
                self.background_color = value & 0x0F;
                // Background color updated (removed debug print)
            }
            _ => {}
        }
    }
    
    // Simulate VIC-II timing (raster beam)
    pub fn tick(&mut self, cycles: u8) -> bool {
        // C64 PAL: 312 lines, 63 cycles/line
        // Raster line at $D012 (bits 0-7) and $D011 (bit 7)
        self.cycle_count += cycles as u16;
        
        if self.cycle_count >= 63 {
            self.cycle_count -= 63;
            
            // Increment raster line
            self.raster_line += 1;
            if self.raster_line >= 312 {
                self.raster_line = 0;
            }
            
            // Check for Raster IRQ
            // IRQ condition: raster_line == irq_raster_line
            let irq_raster_line = (self.registers[0x12] as u16) | 
                                 (if (self.registers[0x11] & 0x80) != 0 { 0x100 } else { 0 });
            
            if self.raster_line == irq_raster_line {
                // Set Raster IRQ flag (Bit 0 of $D019)
                self.registers[0x19] |= 0x01;
                
                // If Raster IRQ Enabled ($D01A Bit 0), signal interrupt
                if (self.registers[0x1A] & 0x01) != 0 {
                    return true;
                }
            }
        }
        
        false
    }

    pub fn read_color_ram(&self, offset: u16) -> u8 {
        if (offset as usize) < self.color_ram.len() {
            self.color_ram[offset as usize] & 0x0F
        } else {
            0
        }
    }
    
    pub fn write_color_ram(&mut self, offset: u16, value: u8) {
        if (offset as usize) < self.color_ram.len() {
            self.color_ram[offset as usize] = value & 0x0F;
        }
    }
    
    pub fn get_screen_char(&self, memory: &dyn crate::memory::Memory, x: usize, y: usize) -> (u8, u8) {
        if x >= SCREEN_WIDTH || y >= SCREEN_HEIGHT {
            return (0x20, C64Color::LightBlue as u8); // Space character
        }
        
        let offset = y * SCREEN_WIDTH + x;
        let char_code = memory.read(self.screen_base + offset as u16);
        let color = self.color_ram[offset];
        
        (char_code, color)
    }
    
    pub fn get_border_color(&self) -> C64Color {
        C64Color::from_u8(self.border_color)
    }
    
    pub fn get_background_color(&self) -> C64Color {
        C64Color::from_u8(self.background_color)
    }
}

// Convert C64 Screen Code to ASCII/Unicode char
pub fn screen_code_to_char(code: u8) -> char {
    match code {
        0 => '@',
        1..=26 => (b'A' + (code - 1)) as char,
        27 => '[',
        28 => '£',
        29 => ']',
        30 => '↑',
        31 => '←',
        32 => ' ',
        33 => '!',
        34 => '"',
        35 => '#',
        36 => '$',
        37 => '%',
        38 => '&',
        39 => '\'',
        40 => '(',
        41 => ')',
        42 => '*',
        43 => '+',
        44 => ',',
        45 => '-',
        46 => '.',
        47 => '/',
        48..=57 => (b'0' + (code - 48)) as char,
        58 => ':',
        59 => ';',
        60 => '<',
        61 => '=',
        62 => '>',
        63 => '?',
        65 => '♠',
        66 => '│', // Vertical bar
        67 => '─', // Horizontal bar
        68 => '─',
        69 => '─',
        70 => '─',
        71 => '│',
        72 => '│',
        73 => '╯', // Curved corner or similar
        74 => '╮',
        75 => '╰',
        76 => '╭',
        77 => '╲', // Diagonal
        78 => '╱',
        79 => '╳',
        80 => '●', // Circle
        81 => '●',
        82 => '○',
        83 => '♥',
        84 => '─',
        85 => '╭',
        86 => '╳',
        87 => '○',
        88 => '♣',
        89 => '─',
        90 => '♦',
        91 => '+',
        92 => '│',
        93 => '│',
        94 => 'π',
        95 => '◥', // Triangle?
        160 => '█', // Shift+Space / Reverse Space (Cursor)
        // Reversed/Inverse characters (recurse to base char)
        128..=255 => screen_code_to_char(code & 0x7F), 
        // Default fallback
        _ => '▒',
    }
}

// // PETSCII to ASCII conversion (simplified)
// pub fn petscii_to_char(petscii: u8) -> char {
//     match petscii {
//         0x00..=0x1F => ' ', // Control characters as space
//         0x20..=0x3F => (petscii as char), // Standard ASCII
//         0x40 => '@',
//         0x41..=0x5A => (petscii as char), // A-Z
//         0x5B => '[',
//         0x5C => '£', // Pound sign
//         0x5D => ']',
//         0x5E => '↑', // Up arrow
//         0x5F => '←', // Left arrow
//         0x60 => '─', // Horizontal line
//         0x61..=0x7A => ((petscii - 0x20) as char), // a-z (lowercase in C64 is uppercase+32)
//         0x7B => '+',
//         0x7C => '|',
//         0x7D => '}',
//         0x7E => '~',
//         0x7F => ' ',
//         0x80..=0xFF => {
//             // Inverted/graphics characters - use ASCII approximations
//             if petscii == 0xA0 {
//                 '█' // Cursor - solid block
//             } else if petscii >= 0xA0 && petscii <= 0xBF {
//                 ((petscii - 0x80) as char)
//             } else {
//                 '▒' // Use block for graphics chars
//             }
//         }
//     }
// }
