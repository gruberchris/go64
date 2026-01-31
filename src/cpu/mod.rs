// 6502 CPU Emulator

pub mod opcodes;
pub mod addressing;
mod tests;

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusFlags {
    pub carry: bool,        // C
    pub zero: bool,         // Z
    pub interrupt: bool,    // I (interrupt disable)
    pub decimal: bool,      // D (decimal mode, not used in C64)
    pub break_flag: bool,   // B
    pub unused: bool,       // Always 1
    pub overflow: bool,     // V
    pub negative: bool,     // N
}

impl StatusFlags {
    pub fn new() -> Self {
        Self {
            carry: false,
            zero: false,
            interrupt: true, // Start with interrupts disabled
            decimal: false,
            break_flag: false,
            unused: true,
            overflow: false,
            negative: false,
        }
    }

    pub fn as_byte(&self) -> u8 {
        (self.carry as u8)
            | ((self.zero as u8) << 1)
            | ((self.interrupt as u8) << 2)
            | ((self.decimal as u8) << 3)
            | ((self.break_flag as u8) << 4)
            | ((self.unused as u8) << 5)
            | ((self.overflow as u8) << 6)
            | ((self.negative as u8) << 7)
    }

    pub fn from_byte(byte: u8) -> Self {
        Self {
            carry: (byte & 0x01) != 0,
            zero: (byte & 0x02) != 0,
            interrupt: (byte & 0x04) != 0,
            decimal: (byte & 0x08) != 0,
            break_flag: (byte & 0x10) != 0,
            unused: true, // Always 1
            overflow: (byte & 0x40) != 0,
            negative: (byte & 0x80) != 0,
        }
    }

    pub fn update_zero_negative(&mut self, value: u8) {
        self.zero = value == 0;
        self.negative = (value & 0x80) != 0;
    }
}

#[derive(Debug)]
pub struct Cpu {
    pub a: u8,      // Accumulator
    pub x: u8,      // X register
    pub y: u8,      // Y register
    pub pc: u16,    // Program counter
    pub sp: u8,     // Stack pointer (points to $0100 + sp)
    pub status: StatusFlags,
    pub cycles: u64, // Total cycles executed
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0xFD, // Stack starts at $01FD
            status: StatusFlags::new(),
            cycles: 0,
        }
    }

    pub fn reset(&mut self, memory: &dyn crate::memory::Memory) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.status = StatusFlags::new();
        
        // Read reset vector from $FFFC-$FFFD
        let lo = memory.read(0xFFFC) as u16;
        let hi = memory.read(0xFFFD) as u16;
        self.pc = (hi << 8) | lo;
        
        self.cycles = 0;
    }

    pub fn step(&mut self, memory: &mut dyn crate::memory::Memory) -> Result<u8> {
        let opcode = memory.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        
        let cycles = opcodes::execute(self, memory, opcode)?;
        self.cycles += cycles as u64;
        
        Ok(cycles)
    }
    
    // IRQ interrupt request
    pub fn irq(&mut self, memory: &mut dyn crate::memory::Memory) {
        // Only trigger if interrupts are enabled
        if !self.status.interrupt {
            // Debug: log IRQ (temporary for debugging)
            // use std::io::Write;
            // let _ = std::fs::OpenOptions::new()
            //    .create(true)
            //    .append(true)
            //    .open("/tmp/go64_irq.txt")
            //    .and_then(|mut f| writeln!(f, "IRQ at PC=${:04X}", self.pc));

            // Push PC and status to stack
            let pc_hi = (self.pc >> 8) as u8;
            let pc_lo = (self.pc & 0xFF) as u8;
            self.push(memory, pc_hi);
            self.push(memory, pc_lo);
            
            // B flag is clear (bit 4), bit 5 is always 1
            self.push(memory, self.status.as_byte() & !0x10); 
            
            // Set interrupt disable flag to prevent recursive IRQs
            self.status.interrupt = true;
            
            // Jump to IRQ vector
            let lo = memory.read(0xFFFE) as u16;
            let hi = memory.read(0xFFFF) as u16;
            self.pc = (hi << 8) | lo;
            
            self.cycles += 7; // IRQ takes 7 cycles
        }
    }
    
    // NMI non-maskable interrupt
    pub fn nmi(&mut self, memory: &mut dyn crate::memory::Memory) {
        // NMI cannot be disabled
        // Push PC and status to stack
        let pc_hi = (self.pc >> 8) as u8;
        let pc_lo = (self.pc & 0xFF) as u8;
        self.push(memory, pc_hi);
        self.push(memory, pc_lo);
        self.push(memory, self.status.as_byte() | 0x20); // B flag clear for NMI
        
        // Set interrupt disable flag
        self.status.interrupt = true;
        
        // Jump to NMI vector
        let lo = memory.read(0xFFFA) as u16;
        let hi = memory.read(0xFFFB) as u16;
        self.pc = (hi << 8) | lo;
        
        self.cycles += 7; // NMI takes 7 cycles
    }
    
    // Check if PC is about to call a KERNAL routine we want to intercept
    pub fn check_kernal_call(&self) -> Option<u16> {
        match self.pc {
            0xFFE4 => Some(0xFFE4), // GETIN
            0xFFD2 => Some(0xFFD2), // CHROUT
            _ => None
        }
    }

    // Stack operations
    pub fn push(&mut self, memory: &mut dyn crate::memory::Memory, value: u8) {
        let addr = 0x0100 | (self.sp as u16);
        memory.write(addr, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn pop(&mut self, memory: &dyn crate::memory::Memory) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let addr = 0x0100 | (self.sp as u16);
        memory.read(addr)
    }

    pub fn push_word(&mut self, memory: &mut dyn crate::memory::Memory, value: u16) {
        self.push(memory, (value >> 8) as u8); // High byte first
        self.push(memory, (value & 0xFF) as u8); // Low byte
    }

    pub fn pop_word(&mut self, memory: &dyn crate::memory::Memory) -> u16 {
        let lo = self.pop(memory) as u16;
        let hi = self.pop(memory) as u16;
        (hi << 8) | lo
    }
}
