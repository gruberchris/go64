/// CIA (Complex Interface Adapter) chip emulation
/// The C64 has two CIA chips: CIA1 ($DC00) and CIA2 ($DD00)
/// These handle keyboard, joystick, timers, and other I/O

pub struct Cia {
    pub pra: u8,  // Port Register A
    pub prb: u8,  // Port Register B
    pub ddra: u8, // Data Direction Register A
    pub ddrb: u8, // Data Direction Register B
    pub ta_lo: u8, // Timer A low
    pub ta_hi: u8, // Timer A high
    pub tb_lo: u8, // Timer B low
    pub tb_hi: u8, // Timer B high
    pub tod_10ths: u8,
    pub tod_sec: u8,
    pub tod_min: u8,
    pub tod_hr: u8,
    pub sdr: u8,  // Serial Data Register
    pub icr: u8,  // Interrupt Control Register (bit 7 = IR, bits 0-6 = interrupt sources)
    pub icr_mask: u8,  // Interrupt mask (which interrupts are enabled)
    pub cra: u8,  // Control Register A
    pub crb: u8,  // Control Register B
    
    // Internal state
    timer_a: u16,
    timer_b: u16,
    
    // Keyboard matrix (8x8 = 64 keys)
    // keyboard_matrix[row][col] is a counter: 0 = not pressed, >0 = pressed (frames remaining)
    keyboard_matrix: [[u8; 8]; 8],
}

impl Cia {
    pub fn new() -> Self {
        Self {
            pra: 0xFF,
            prb: 0xFF,
            ddra: 0,
            ddrb: 0,
            ta_lo: 0x25,  // Default timer value for 60Hz: $4025 = 16421 cycles
            ta_hi: 0x40,
            tb_lo: 0xFF,
            tb_hi: 0xFF,
            tod_10ths: 0,
            tod_sec: 0,
            tod_min: 0,
            tod_hr: 1,
            sdr: 0,
            icr: 0,
            icr_mask: 0,  // No interrupts enabled initially
            cra: 0x01,  // Timer A starts running on reset
            crb: 0,
            timer_a: 0x4025,  // 16421 cycles for 60Hz at 1MHz
            timer_b: 0xFFFF,
            keyboard_matrix: [[0; 8]; 8],  // No keys pressed initially
        }
    }
    
    pub fn read(&self, addr: u16) -> u8 {
        match addr & 0x0F {
            0x00 => self.pra,
            0x01 => {
                // Port B reads keyboard matrix based on Port A row selection
                // Each bit in PRA selects a row (active low)
                // Return column states (active low) in PRB
                let result = self.read_keyboard_columns();
                
                // Debug: Log when keyboard is being scanned
                if result != 0xFF {  // 0xFF means no keys pressed
                    use std::io::Write;
                    let _ = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/go64_keyboard.txt")
                        .and_then(|mut f| writeln!(f, "CIA read $DC01: PRA=${:02X} -> result=${:02X}", self.pra, result));
                }
                result
            },
            0x02 => self.ddra,
            0x03 => self.ddrb,
            0x04 => (self.timer_a & 0xFF) as u8,  // Timer A low byte (current value, not latch)
            0x05 => ((self.timer_a >> 8) & 0xFF) as u8,  // Timer A high byte (current value)
            0x06 => (self.timer_b & 0xFF) as u8,  // Timer B low byte (current value, not latch)
            0x07 => ((self.timer_b >> 8) & 0xFF) as u8,  // Timer B high byte (current value)
            0x08 => self.tod_10ths,
            0x09 => self.tod_sec,
            0x0A => self.tod_min,
            0x0B => self.tod_hr,
            0x0C => self.sdr,
            0x0D => {
                // Reading ICR returns current interrupts and clears them
                let result = self.icr;
                // Note: We can't clear here because we need mutable access
                // The caller should call clear_icr() after reading
                result
            }
            0x0E => self.cra,
            0x0F => self.crb,
            _ => 0,
        }
    }
    
    // pub fn clear_icr(&mut self) {
    //     // Reading ICR clears the interrupt flags (except bit 7 which indicates "interrupt occurred")
    //     self.icr &= 0x80; // Keep only bit 7
    // }
    
    // pub fn has_interrupt(&self) -> bool {
    //     // Check if any enabled interrupt has occurred
    //     // An interrupt fires if: (icr & icr_mask & 0x7F) != 0
    //     (self.icr & self.icr_mask & 0x7F) != 0
    // }
    
    pub fn write(&mut self, addr: u16, value: u8) {
        match addr & 0x0F {
            0x00 => self.pra = value,
            0x01 => self.prb = value,
            0x02 => self.ddra = value,
            0x03 => self.ddrb = value,
            0x04 => {
                self.ta_lo = value;
                self.timer_a = (self.timer_a & 0xFF00) | (value as u16);
            }
            0x05 => {
                self.ta_hi = value;
                self.timer_a = ((value as u16) << 8) | (self.timer_a & 0x00FF);
                // CIA Timer A configured (removed debug print to avoid TUI interference)
            }
            0x06 => {
                self.tb_lo = value;
                self.timer_b = (self.timer_b & 0xFF00) | (value as u16);
            }
            0x07 => {
                self.tb_hi = value;
                self.timer_b = ((value as u16) << 8) | (self.timer_b & 0x00FF);
            }
            0x08 => self.tod_10ths = value,
            0x09 => self.tod_sec = value,
            0x0A => self.tod_min = value,
            0x0B => self.tod_hr = value,
            0x0C => self.sdr = value,
            0x0D => {
                // ICR mask register write
                // Bit 7 = 1: SET interrupt mask bits
                // Bit 7 = 0: CLEAR interrupt mask bits
                use std::fs::OpenOptions;
                use std::io::Write;
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/go64_cia_debug.txt")
                    .ok();
                    
                if value & 0x80 != 0 {
                    // Set mask bits
                    self.icr_mask |= value & 0x7F;
                    if let Some(ref mut f) = file {
                        writeln!(f, "CIA ICR mask SET ${:02X} -> mask now ${:02X}", value & 0x7F, self.icr_mask).ok();
                    }
                } else {
                    // Clear mask bits
                    self.icr_mask &= !(value & 0x7F);
                    if let Some(ref mut f) = file {
                        writeln!(f, "CIA ICR mask CLEAR ${:02X} -> mask now ${:02X}", value & 0x7F, self.icr_mask).ok();
                    }
                }
            }
            0x0E => {
                self.cra = value;
                if value & 0x01 != 0 {
                    use std::fs::OpenOptions;
                    use std::io::Write;
                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/go64_cia_debug.txt")
                        .ok();
                    if let Some(ref mut f) = file {
                        writeln!(f, "CIA Timer A started! CRA=${:02X}, Timer=${:04X}", value, self.timer_a).ok();
                    }
                }
            }
            0x0F => {
                self.crb = value;
                // CIA Timer B control register (removed debug print)
            }
            _ => {}
        }
    }
    
    pub fn tick(&mut self, cycles: u8) -> bool {
        let mut irq = false;

        // Timer A handling
        if self.cra & 0x01 != 0 {
            // Timer A is running
            if self.timer_a > cycles as u16 {
                self.timer_a -= cycles as u16;
            } else {
                // Timer underflowed - reload from latch and set interrupt
                self.timer_a = ((self.ta_hi as u16) << 8) | (self.ta_lo as u16);
                self.icr |= 0x81; // Set bit 0 (timer A) and bit 7 (interrupt occurred)
                
                // Check if interrupt enabled
                if (self.icr_mask & 0x01) != 0 {
                    irq = true;
                }

                // Check if timer should stop (one-shot mode, bit 3 of CRA)
                if self.cra & 0x08 != 0 {
                    self.cra &= !0x01; // Stop timer
                }
            }
        }
        
        // Timer B handling
        if self.crb & 0x01 != 0 {
            // Timer B is running
            if self.timer_b > cycles as u16 {
                self.timer_b -= cycles as u16;
            } else {
                // Timer underflowed - reload from latch and set interrupt
                self.timer_b = ((self.tb_hi as u16) << 8) | (self.tb_lo as u16);
                self.icr |= 0x82; // Set bit 1 (timer B) and bit 7 (interrupt occurred)
                
                // Check if interrupt enabled
                if (self.icr_mask & 0x02) != 0 {
                    irq = true;
                }

                // Check if timer should stop (one-shot mode, bit 3 of CRB)
                if self.crb & 0x08 != 0 {
                    self.crb &= !0x01; // Stop timer
                }
            }
        }

        irq
    }
    
    // Keyboard matrix methods
    pub fn read_keyboard_columns(&self) -> u8 {
        // PRA selects rows (active low - 0 means selected)
        // Return PRB with columns (active low - 0 means key pressed)
        let mut result = 0xFF; // All keys up by default
        
        // Check each row
        for row in 0..8 {
            // If this row is selected (bit is 0 in PRA)
            if (self.pra & (1 << row)) == 0 {
                // Check each column in this row
                for col in 0..8 {
                    if self.keyboard_matrix[row][col] > 0 {
                        // Key is pressed - clear the bit (active low)
                        result &= !(1 << col);
                    }
                }
            }
        }
        
        result
    }
    
    pub fn set_key(&mut self, row: u8, col: u8, pressed: bool) {
        if row < 8 && col < 8 {
            if pressed {
                // Set persistence to 5 frames (approx 83ms) to ensure
                // KERNAL sees it even if input polling frequency < 60Hz or scan is missed
                self.keyboard_matrix[row as usize][col as usize] = 5;
            } else {
                self.keyboard_matrix[row as usize][col as usize] = 0;
            }
        }
    }
    
    pub fn decay_keyboard(&mut self) {
        for row in 0..8 {
            for col in 0..8 {
                if self.keyboard_matrix[row][col] > 0 {
                    self.keyboard_matrix[row][col] -= 1;
                }
            }
        }
    }
    
    // Legacy support for clear_keyboard, now just calls decay
    pub fn clear_keyboard(&mut self) {
        self.decay_keyboard();
    }
}
