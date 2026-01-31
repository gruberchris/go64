// Addressing modes for 6502

use crate::cpu::Cpu;
use crate::memory::Memory;

#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    // Implied,
    // Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Relative,
}

pub struct AddressResult {
    pub address: u16,
    pub page_crossed: bool,
}

impl Cpu {
    pub fn get_operand_address(&mut self, memory: &dyn Memory, mode: AddressingMode) -> AddressResult {
        match mode {
            AddressingMode::Immediate => {
                let addr = self.pc;
                self.pc = self.pc.wrapping_add(1);
                AddressResult { address: addr, page_crossed: false }
            }
            AddressingMode::ZeroPage => {
                let addr = memory.read(self.pc) as u16;
                self.pc = self.pc.wrapping_add(1);
                AddressResult { address: addr, page_crossed: false }
            }
            AddressingMode::ZeroPageX => {
                let base = memory.read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let addr = base.wrapping_add(self.x) as u16;
                AddressResult { address: addr, page_crossed: false }
            }
            AddressingMode::ZeroPageY => {
                let base = memory.read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let addr = base.wrapping_add(self.y) as u16;
                AddressResult { address: addr, page_crossed: false }
            }
            AddressingMode::Absolute => {
                let lo = memory.read(self.pc) as u16;
                let hi = memory.read(self.pc.wrapping_add(1)) as u16;
                self.pc = self.pc.wrapping_add(2);
                AddressResult { address: (hi << 8) | lo, page_crossed: false }
            }
            AddressingMode::AbsoluteX => {
                let lo = memory.read(self.pc) as u16;
                let hi = memory.read(self.pc.wrapping_add(1)) as u16;
                self.pc = self.pc.wrapping_add(2);
                let base = (hi << 8) | lo;
                let addr = base.wrapping_add(self.x as u16);
                let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
                AddressResult { address: addr, page_crossed }
            }
            AddressingMode::AbsoluteY => {
                let lo = memory.read(self.pc) as u16;
                let hi = memory.read(self.pc.wrapping_add(1)) as u16;
                self.pc = self.pc.wrapping_add(2);
                let base = (hi << 8) | lo;
                let addr = base.wrapping_add(self.y as u16);
                let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
                AddressResult { address: addr, page_crossed }
            }
            AddressingMode::Indirect => {
                let ptr_lo = memory.read(self.pc) as u16;
                let ptr_hi = memory.read(self.pc.wrapping_add(1)) as u16;
                self.pc = self.pc.wrapping_add(2);
                let ptr = (ptr_hi << 8) | ptr_lo;
                
                // 6502 bug: if ptr is at page boundary, high byte wraps within page
                let lo = memory.read(ptr) as u16;
                let hi_addr = if (ptr & 0xFF) == 0xFF {
                    ptr & 0xFF00
                } else {
                    ptr.wrapping_add(1)
                };
                let hi = memory.read(hi_addr) as u16;
                AddressResult { address: (hi << 8) | lo, page_crossed: false }
            }
            AddressingMode::IndirectX => {
                let base = memory.read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let ptr = base.wrapping_add(self.x);
                let lo = memory.read(ptr as u16) as u16;
                let hi = memory.read(ptr.wrapping_add(1) as u16) as u16;
                AddressResult { address: (hi << 8) | lo, page_crossed: false }
            }
            AddressingMode::IndirectY => {
                let ptr = memory.read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let lo = memory.read(ptr as u16) as u16;
                let hi = memory.read(ptr.wrapping_add(1) as u16) as u16;
                let base = (hi << 8) | lo;
                let addr = base.wrapping_add(self.y as u16);
                let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
                AddressResult { address: addr, page_crossed }
            }
            AddressingMode::Relative => {
                let offset = memory.read(self.pc) as i8;
                self.pc = self.pc.wrapping_add(1);
                let addr = self.pc.wrapping_add(offset as u16);
                let page_crossed = (self.pc & 0xFF00) != (addr & 0xFF00);
                AddressResult { address: addr, page_crossed }
            }
        }
    }
}
