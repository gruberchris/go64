// Simple disassembler for 6502

use crate::memory::Memory;

pub fn disassemble(memory: &dyn Memory, addr: u16) -> (String, u16) {
    let opcode = memory.read(addr);
    
    match opcode {
        // LDA
        0xA9 => (format!("LDA #${:02X}", memory.read(addr + 1)), 2),
        0xA5 => (format!("LDA ${:02X}", memory.read(addr + 1)), 2),
        0xB5 => (format!("LDA ${:02X},X", memory.read(addr + 1)), 2),
        0xAD => (format!("LDA ${:02X}{:02X}", memory.read(addr + 2), memory.read(addr + 1)), 3),
        0xBD => (format!("LDA ${:02X}{:02X},X", memory.read(addr + 2), memory.read(addr + 1)), 3),
        0xB9 => (format!("LDA ${:02X}{:02X},Y", memory.read(addr + 2), memory.read(addr + 1)), 3),
        
        // STA
        0x85 => (format!("STA ${:02X}", memory.read(addr + 1)), 2),
        0x8D => (format!("STA ${:02X}{:02X}", memory.read(addr + 2), memory.read(addr + 1)), 3),
        
        // JMP
        0x4C => (format!("JMP ${:02X}{:02X}", memory.read(addr + 2), memory.read(addr + 1)), 3),
        0x6C => (format!("JMP (${:02X}{:02X})", memory.read(addr + 2), memory.read(addr + 1)), 3),
        
        // JSR/RTS
        0x20 => (format!("JSR ${:02X}{:02X}", memory.read(addr + 2), memory.read(addr + 1)), 3),
        0x60 => ("RTS".to_string(), 1),
        
        // Transfers
        0xAA => ("TAX".to_string(), 1),
        0xA8 => ("TAY".to_string(), 1),
        0x8A => ("TXA".to_string(), 1),
        0x98 => ("TYA".to_string(), 1),
        
        // Stack
        0x48 => ("PHA".to_string(), 1),
        0x68 => ("PLA".to_string(), 1),
        
        // Branches
        0x10 => (format!("BPL ${:02X}", memory.read(addr + 1)), 2),
        0x30 => (format!("BMI ${:02X}", memory.read(addr + 1)), 2),
        0x50 => (format!("BVC ${:02X}", memory.read(addr + 1)), 2),
        0x70 => (format!("BVS ${:02X}", memory.read(addr + 1)), 2),
        0x90 => (format!("BCC ${:02X}", memory.read(addr + 1)), 2),
        0xB0 => (format!("BCS ${:02X}", memory.read(addr + 1)), 2),
        0xD0 => (format!("BNE ${:02X}", memory.read(addr + 1)), 2),
        0xF0 => (format!("BEQ ${:02X}", memory.read(addr + 1)), 2),
        
        // Other common ones
        0xEA => ("NOP".to_string(), 1),
        0x00 => ("BRK".to_string(), 1),
        
        _ => (format!("??? ${:02X}", opcode), 1),
    }
}

pub fn disassemble_range(memory: &dyn Memory, start: u16, count: usize) {
    let mut addr = start;
    for _ in 0..count {
        let (instr, size) = disassemble(memory, addr);
        println!("${:04X}: {}", addr, instr);
        addr = addr.wrapping_add(size);
    }
}
