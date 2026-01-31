// 6502 instruction set implementation

use crate::cpu::{Cpu, StatusFlags};
use crate::memory::Memory;
use super::addressing::AddressingMode;
use anyhow::{Result, bail};

pub fn execute(cpu: &mut Cpu, memory: &mut dyn Memory, opcode: u8) -> Result<u8> {
    match opcode {
        // LDA - Load Accumulator
        0xA9 => lda(cpu, memory, AddressingMode::Immediate),
        0xA5 => lda(cpu, memory, AddressingMode::ZeroPage),
        0xB5 => lda(cpu, memory, AddressingMode::ZeroPageX),
        0xAD => lda(cpu, memory, AddressingMode::Absolute),
        0xBD => lda(cpu, memory, AddressingMode::AbsoluteX),
        0xB9 => lda(cpu, memory, AddressingMode::AbsoluteY),
        0xA1 => lda(cpu, memory, AddressingMode::IndirectX),
        0xB1 => lda(cpu, memory, AddressingMode::IndirectY),

        // LDX - Load X Register
        0xA2 => ldx(cpu, memory, AddressingMode::Immediate),
        0xA6 => ldx(cpu, memory, AddressingMode::ZeroPage),
        0xB6 => ldx(cpu, memory, AddressingMode::ZeroPageY),
        0xAE => ldx(cpu, memory, AddressingMode::Absolute),
        0xBE => ldx(cpu, memory, AddressingMode::AbsoluteY),

        // LDY - Load Y Register
        0xA0 => ldy(cpu, memory, AddressingMode::Immediate),
        0xA4 => ldy(cpu, memory, AddressingMode::ZeroPage),
        0xB4 => ldy(cpu, memory, AddressingMode::ZeroPageX),
        0xAC => ldy(cpu, memory, AddressingMode::Absolute),
        0xBC => ldy(cpu, memory, AddressingMode::AbsoluteX),

        // STA - Store Accumulator
        0x85 => sta(cpu, memory, AddressingMode::ZeroPage),
        0x95 => sta(cpu, memory, AddressingMode::ZeroPageX),
        0x8D => sta(cpu, memory, AddressingMode::Absolute),
        0x9D => sta(cpu, memory, AddressingMode::AbsoluteX),
        0x99 => sta(cpu, memory, AddressingMode::AbsoluteY),
        0x81 => sta(cpu, memory, AddressingMode::IndirectX),
        0x91 => sta(cpu, memory, AddressingMode::IndirectY),

        // STX - Store X Register
        0x86 => stx(cpu, memory, AddressingMode::ZeroPage),
        0x96 => stx(cpu, memory, AddressingMode::ZeroPageY),
        0x8E => stx(cpu, memory, AddressingMode::Absolute),

        // STY - Store Y Register
        0x84 => sty(cpu, memory, AddressingMode::ZeroPage),
        0x94 => sty(cpu, memory, AddressingMode::ZeroPageX),
        0x8C => sty(cpu, memory, AddressingMode::Absolute),

        // TAX, TAY, TXA, TYA
        0xAA => { cpu.x = cpu.a; cpu.status.update_zero_negative(cpu.x); Ok(2) }
        0xA8 => { cpu.y = cpu.a; cpu.status.update_zero_negative(cpu.y); Ok(2) }
        0x8A => { cpu.a = cpu.x; cpu.status.update_zero_negative(cpu.a); Ok(2) }
        0x98 => { cpu.a = cpu.y; cpu.status.update_zero_negative(cpu.a); Ok(2) }

        // TSX, TXS
        0xBA => { cpu.x = cpu.sp; cpu.status.update_zero_negative(cpu.x); Ok(2) }
        0x9A => { cpu.sp = cpu.x; Ok(2) }

        // PHA, PLA, PHP, PLP
        0x48 => { let a = cpu.a; cpu.push(memory, a); Ok(3) }
        0x68 => { let val = cpu.pop(memory); cpu.a = val; cpu.status.update_zero_negative(cpu.a); Ok(4) }
        0x08 => { let status = cpu.status.as_byte() | 0x30; cpu.push(memory, status); Ok(3) }
        0x28 => { let val = cpu.pop(memory); cpu.status = StatusFlags::from_byte(val); cpu.status.unused = true; Ok(4) }

        // JMP
        0x4C => {
            let result = cpu.get_operand_address(memory, AddressingMode::Absolute);
            cpu.pc = result.address;
            Ok(3)
        }
        0x6C => {
            let result = cpu.get_operand_address(memory, AddressingMode::Indirect);
            cpu.pc = result.address;
            Ok(5)
        }

        // JSR
        0x20 => {
            let target = {
                let lo = memory.read(cpu.pc) as u16;
                let hi = memory.read(cpu.pc.wrapping_add(1)) as u16;
                (hi << 8) | lo
            };
            let return_addr = cpu.pc.wrapping_add(1);
            cpu.push_word(memory, return_addr);
            cpu.pc = target;
            Ok(6)
        }

        // RTS
        0x60 => {
            let addr = cpu.pop_word(memory);
            cpu.pc = addr.wrapping_add(1);
            Ok(6)
        }

        // RTI
        0x40 => {
            let status = cpu.pop(memory);
            cpu.status = StatusFlags::from_byte(status);
            cpu.status.unused = true;
            cpu.pc = cpu.pop_word(memory);
            Ok(6)
        }

        // BRK
        0x00 => {
            cpu.pc = cpu.pc.wrapping_add(1);
            cpu.push_word(memory, cpu.pc);
            cpu.status.break_flag = true;
            let status = cpu.status.as_byte() | 0x10;
            cpu.push(memory, status);
            cpu.status.interrupt = true;
            let lo = memory.read(0xFFFE) as u16;
            let hi = memory.read(0xFFFF) as u16;
            cpu.pc = (hi << 8) | lo;
            Ok(7)
        }

        // NOP
        0xEA => Ok(2),

        // Branches
        0x10 => branch(cpu, memory, !cpu.status.negative),
        0x30 => branch(cpu, memory, cpu.status.negative),
        0x50 => branch(cpu, memory, !cpu.status.overflow),
        0x70 => branch(cpu, memory, cpu.status.overflow),
        0x90 => branch(cpu, memory, !cpu.status.carry),
        0xB0 => branch(cpu, memory, cpu.status.carry),
        0xD0 => branch(cpu, memory, !cpu.status.zero),
        0xF0 => branch(cpu, memory, cpu.status.zero),

        // Flag instructions
        0x18 => { cpu.status.carry = false; Ok(2) }
        0x38 => { cpu.status.carry = true; Ok(2) }
        0x58 => { cpu.status.interrupt = false; Ok(2) }
        0x78 => { cpu.status.interrupt = true; Ok(2) }
        0xB8 => { cpu.status.overflow = false; Ok(2) }
        0xD8 => { cpu.status.decimal = false; Ok(2) }
        0xF8 => { cpu.status.decimal = true; Ok(2) }

        // INX, INY, DEX, DEY
        0xE8 => { cpu.x = cpu.x.wrapping_add(1); cpu.status.update_zero_negative(cpu.x); Ok(2) }
        0xC8 => { cpu.y = cpu.y.wrapping_add(1); cpu.status.update_zero_negative(cpu.y); Ok(2) }
        0xCA => { cpu.x = cpu.x.wrapping_sub(1); cpu.status.update_zero_negative(cpu.x); Ok(2) }
        0x88 => { cpu.y = cpu.y.wrapping_sub(1); cpu.status.update_zero_negative(cpu.y); Ok(2) }

        // ADC - Add with Carry
        0x69 => adc(cpu, memory, AddressingMode::Immediate),
        0x65 => adc(cpu, memory, AddressingMode::ZeroPage),
        0x75 => adc(cpu, memory, AddressingMode::ZeroPageX),
        0x6D => adc(cpu, memory, AddressingMode::Absolute),
        0x7D => adc(cpu, memory, AddressingMode::AbsoluteX),
        0x79 => adc(cpu, memory, AddressingMode::AbsoluteY),
        0x61 => adc(cpu, memory, AddressingMode::IndirectX),
        0x71 => adc(cpu, memory, AddressingMode::IndirectY),

        // SBC - Subtract with Carry
        0xE9 => sbc(cpu, memory, AddressingMode::Immediate),
        0xE5 => sbc(cpu, memory, AddressingMode::ZeroPage),
        0xF5 => sbc(cpu, memory, AddressingMode::ZeroPageX),
        0xED => sbc(cpu, memory, AddressingMode::Absolute),
        0xFD => sbc(cpu, memory, AddressingMode::AbsoluteX),
        0xF9 => sbc(cpu, memory, AddressingMode::AbsoluteY),
        0xE1 => sbc(cpu, memory, AddressingMode::IndirectX),
        0xF1 => sbc(cpu, memory, AddressingMode::IndirectY),

        // AND - Logical AND
        0x29 => and(cpu, memory, AddressingMode::Immediate),
        0x25 => and(cpu, memory, AddressingMode::ZeroPage),
        0x35 => and(cpu, memory, AddressingMode::ZeroPageX),
        0x2D => and(cpu, memory, AddressingMode::Absolute),
        0x3D => and(cpu, memory, AddressingMode::AbsoluteX),
        0x39 => and(cpu, memory, AddressingMode::AbsoluteY),
        0x21 => and(cpu, memory, AddressingMode::IndirectX),
        0x31 => and(cpu, memory, AddressingMode::IndirectY),

        // ORA - Logical OR
        0x09 => ora(cpu, memory, AddressingMode::Immediate),
        0x05 => ora(cpu, memory, AddressingMode::ZeroPage),
        0x15 => ora(cpu, memory, AddressingMode::ZeroPageX),
        0x0D => ora(cpu, memory, AddressingMode::Absolute),
        0x1D => ora(cpu, memory, AddressingMode::AbsoluteX),
        0x19 => ora(cpu, memory, AddressingMode::AbsoluteY),
        0x01 => ora(cpu, memory, AddressingMode::IndirectX),
        0x11 => ora(cpu, memory, AddressingMode::IndirectY),

        // EOR - Logical XOR
        0x49 => eor(cpu, memory, AddressingMode::Immediate),
        0x45 => eor(cpu, memory, AddressingMode::ZeroPage),
        0x55 => eor(cpu, memory, AddressingMode::ZeroPageX),
        0x4D => eor(cpu, memory, AddressingMode::Absolute),
        0x5D => eor(cpu, memory, AddressingMode::AbsoluteX),
        0x59 => eor(cpu, memory, AddressingMode::AbsoluteY),
        0x41 => eor(cpu, memory, AddressingMode::IndirectX),
        0x51 => eor(cpu, memory, AddressingMode::IndirectY),

        // CMP - Compare Accumulator
        0xC9 => cmp(cpu, memory, AddressingMode::Immediate),
        0xC5 => cmp(cpu, memory, AddressingMode::ZeroPage),
        0xD5 => cmp(cpu, memory, AddressingMode::ZeroPageX),
        0xCD => cmp(cpu, memory, AddressingMode::Absolute),
        0xDD => cmp(cpu, memory, AddressingMode::AbsoluteX),
        0xD9 => cmp(cpu, memory, AddressingMode::AbsoluteY),
        0xC1 => cmp(cpu, memory, AddressingMode::IndirectX),
        0xD1 => cmp(cpu, memory, AddressingMode::IndirectY),

        // CPX - Compare X Register
        0xE0 => cpx(cpu, memory, AddressingMode::Immediate),
        0xE4 => cpx(cpu, memory, AddressingMode::ZeroPage),
        0xEC => cpx(cpu, memory, AddressingMode::Absolute),

        // CPY - Compare Y Register
        0xC0 => cpy(cpu, memory, AddressingMode::Immediate),
        0xC4 => cpy(cpu, memory, AddressingMode::ZeroPage),
        0xCC => cpy(cpu, memory, AddressingMode::Absolute),

        // INC - Increment Memory
        0xE6 => inc(cpu, memory, AddressingMode::ZeroPage),
        0xF6 => inc(cpu, memory, AddressingMode::ZeroPageX),
        0xEE => inc(cpu, memory, AddressingMode::Absolute),
        0xFE => inc(cpu, memory, AddressingMode::AbsoluteX),

        // DEC - Decrement Memory
        0xC6 => dec(cpu, memory, AddressingMode::ZeroPage),
        0xD6 => dec(cpu, memory, AddressingMode::ZeroPageX),
        0xCE => dec(cpu, memory, AddressingMode::Absolute),
        0xDE => dec(cpu, memory, AddressingMode::AbsoluteX),

        // ASL - Arithmetic Shift Left
        0x0A => asl_acc(cpu),
        0x06 => asl(cpu, memory, AddressingMode::ZeroPage),
        0x16 => asl(cpu, memory, AddressingMode::ZeroPageX),
        0x0E => asl(cpu, memory, AddressingMode::Absolute),
        0x1E => asl(cpu, memory, AddressingMode::AbsoluteX),

        // LSR - Logical Shift Right
        0x4A => lsr_acc(cpu),
        0x46 => lsr(cpu, memory, AddressingMode::ZeroPage),
        0x56 => lsr(cpu, memory, AddressingMode::ZeroPageX),
        0x4E => lsr(cpu, memory, AddressingMode::Absolute),
        0x5E => lsr(cpu, memory, AddressingMode::AbsoluteX),

        // ROL - Rotate Left
        0x2A => rol_acc(cpu),
        0x26 => rol(cpu, memory, AddressingMode::ZeroPage),
        0x36 => rol(cpu, memory, AddressingMode::ZeroPageX),
        0x2E => rol(cpu, memory, AddressingMode::Absolute),
        0x3E => rol(cpu, memory, AddressingMode::AbsoluteX),

        // ROR - Rotate Right
        0x6A => ror_acc(cpu),
        0x66 => ror(cpu, memory, AddressingMode::ZeroPage),
        0x76 => ror(cpu, memory, AddressingMode::ZeroPageX),
        0x6E => ror(cpu, memory, AddressingMode::Absolute),
        0x7E => ror(cpu, memory, AddressingMode::AbsoluteX),

        // BIT - Test Bits
        0x24 => bit(cpu, memory, AddressingMode::ZeroPage),
        0x2C => bit(cpu, memory, AddressingMode::Absolute),

        _ => bail!("Unimplemented opcode: 0x{:02X} at PC: 0x{:04X}", opcode, cpu.pc.wrapping_sub(1)),
    }
}

fn lda(cpu: &mut Cpu, memory: &dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    cpu.a = memory.read(result.address);
    cpu.status.update_zero_negative(cpu.a);
    Ok(match mode {
        AddressingMode::Immediate => 2,
        AddressingMode::ZeroPage => 3,
        AddressingMode::ZeroPageX => 4,
        AddressingMode::Absolute => 4,
        AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => 4 + result.page_crossed as u8,
        AddressingMode::IndirectX => 6,
        AddressingMode::IndirectY => 5 + result.page_crossed as u8,
        _ => 0,
    })
}

fn ldx(cpu: &mut Cpu, memory: &dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    cpu.x = memory.read(result.address);
    cpu.status.update_zero_negative(cpu.x);
    Ok(match mode {
        AddressingMode::Immediate => 2,
        AddressingMode::ZeroPage => 3,
        AddressingMode::ZeroPageY => 4,
        AddressingMode::Absolute => 4,
        AddressingMode::AbsoluteY => 4 + result.page_crossed as u8,
        _ => 0,
    })
}

fn ldy(cpu: &mut Cpu, memory: &dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    cpu.y = memory.read(result.address);
    cpu.status.update_zero_negative(cpu.y);
    Ok(match mode {
        AddressingMode::Immediate => 2,
        AddressingMode::ZeroPage => 3,
        AddressingMode::ZeroPageX => 4,
        AddressingMode::Absolute => 4,
        AddressingMode::AbsoluteX => 4 + result.page_crossed as u8,
        _ => 0,
    })
}

fn sta(cpu: &mut Cpu, memory: &mut dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    memory.write(result.address, cpu.a);
    Ok(match mode {
        AddressingMode::ZeroPage => 3,
        AddressingMode::ZeroPageX => 4,
        AddressingMode::Absolute => 4,
        AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => 5,
        AddressingMode::IndirectX => 6,
        AddressingMode::IndirectY => 6,
        _ => 0,
    })
}

fn stx(cpu: &mut Cpu, memory: &mut dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    memory.write(result.address, cpu.x);
    Ok(match mode {
        AddressingMode::ZeroPage => 3,
        AddressingMode::ZeroPageY => 4,
        AddressingMode::Absolute => 4,
        _ => 0,
    })
}

fn sty(cpu: &mut Cpu, memory: &mut dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    memory.write(result.address, cpu.y);
    Ok(match mode {
        AddressingMode::ZeroPage => 3,
        AddressingMode::ZeroPageX => 4,
        AddressingMode::Absolute => 4,
        _ => 0,
    })
}

fn branch(cpu: &mut Cpu, memory: &dyn Memory, condition: bool) -> Result<u8> {
    let result = cpu.get_operand_address(memory, AddressingMode::Relative);
    if condition {
        cpu.pc = result.address;
        Ok(3 + result.page_crossed as u8)
    } else {
        Ok(2)
    }
}

// ADC - Add with Carry
fn adc(cpu: &mut Cpu, memory: &dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    let value = memory.read(result.address);
    let carry = cpu.status.carry as u16;
    let sum = cpu.a as u16 + value as u16 + carry;
    
    // Check for overflow: both operands same sign, result different sign
    let overflow = ((cpu.a ^ value) & 0x80) == 0 && ((cpu.a ^ sum as u8) & 0x80) != 0;
    
    cpu.a = sum as u8;
    cpu.status.carry = sum > 0xFF;
    cpu.status.overflow = overflow;
    cpu.status.update_zero_negative(cpu.a);
    
    Ok(match mode {
        AddressingMode::Immediate => 2,
        AddressingMode::ZeroPage => 3,
        AddressingMode::ZeroPageX => 4,
        AddressingMode::Absolute => 4,
        AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => 4 + result.page_crossed as u8,
        AddressingMode::IndirectX => 6,
        AddressingMode::IndirectY => 5 + result.page_crossed as u8,
        _ => 0,
    })
}

// SBC - Subtract with Carry
fn sbc(cpu: &mut Cpu, memory: &dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    let value = memory.read(result.address);
    let carry = if cpu.status.carry { 1 } else { 0 };
    let diff = (cpu.a as i16) - (value as i16) - (1 - carry);
    
    // Check for overflow
    let overflow = ((cpu.a ^ value) & 0x80) != 0 && ((cpu.a ^ diff as u8) & 0x80) != 0;
    
    cpu.a = diff as u8;
    cpu.status.carry = diff >= 0;
    cpu.status.overflow = overflow;
    cpu.status.update_zero_negative(cpu.a);
    
    Ok(match mode {
        AddressingMode::Immediate => 2,
        AddressingMode::ZeroPage => 3,
        AddressingMode::ZeroPageX => 4,
        AddressingMode::Absolute => 4,
        AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => 4 + result.page_crossed as u8,
        AddressingMode::IndirectX => 6,
        AddressingMode::IndirectY => 5 + result.page_crossed as u8,
        _ => 0,
    })
}

// AND - Logical AND
fn and(cpu: &mut Cpu, memory: &dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    cpu.a &= memory.read(result.address);
    cpu.status.update_zero_negative(cpu.a);
    Ok(match mode {
        AddressingMode::Immediate => 2,
        AddressingMode::ZeroPage => 3,
        AddressingMode::ZeroPageX => 4,
        AddressingMode::Absolute => 4,
        AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => 4 + result.page_crossed as u8,
        AddressingMode::IndirectX => 6,
        AddressingMode::IndirectY => 5 + result.page_crossed as u8,
        _ => 0,
    })
}

// ORA - Logical OR
fn ora(cpu: &mut Cpu, memory: &dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    cpu.a |= memory.read(result.address);
    cpu.status.update_zero_negative(cpu.a);
    Ok(match mode {
        AddressingMode::Immediate => 2,
        AddressingMode::ZeroPage => 3,
        AddressingMode::ZeroPageX => 4,
        AddressingMode::Absolute => 4,
        AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => 4 + result.page_crossed as u8,
        AddressingMode::IndirectX => 6,
        AddressingMode::IndirectY => 5 + result.page_crossed as u8,
        _ => 0,
    })
}

// EOR - Logical XOR
fn eor(cpu: &mut Cpu, memory: &dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    cpu.a ^= memory.read(result.address);
    cpu.status.update_zero_negative(cpu.a);
    Ok(match mode {
        AddressingMode::Immediate => 2,
        AddressingMode::ZeroPage => 3,
        AddressingMode::ZeroPageX => 4,
        AddressingMode::Absolute => 4,
        AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => 4 + result.page_crossed as u8,
        AddressingMode::IndirectX => 6,
        AddressingMode::IndirectY => 5 + result.page_crossed as u8,
        _ => 0,
    })
}

// CMP - Compare Accumulator
fn cmp(cpu: &mut Cpu, memory: &dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    let value = memory.read(result.address);
    let diff = cpu.a.wrapping_sub(value);
    cpu.status.carry = cpu.a >= value;
    cpu.status.update_zero_negative(diff);
    Ok(match mode {
        AddressingMode::Immediate => 2,
        AddressingMode::ZeroPage => 3,
        AddressingMode::ZeroPageX => 4,
        AddressingMode::Absolute => 4,
        AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => 4 + result.page_crossed as u8,
        AddressingMode::IndirectX => 6,
        AddressingMode::IndirectY => 5 + result.page_crossed as u8,
        _ => 0,
    })
}

// CPX - Compare X Register
fn cpx(cpu: &mut Cpu, memory: &dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    let value = memory.read(result.address);
    let diff = cpu.x.wrapping_sub(value);
    cpu.status.carry = cpu.x >= value;
    cpu.status.update_zero_negative(diff);
    Ok(match mode {
        AddressingMode::Immediate => 2,
        AddressingMode::ZeroPage => 3,
        AddressingMode::Absolute => 4,
        _ => 0,
    })
}

// CPY - Compare Y Register
fn cpy(cpu: &mut Cpu, memory: &dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    let value = memory.read(result.address);
    let diff = cpu.y.wrapping_sub(value);
    cpu.status.carry = cpu.y >= value;
    cpu.status.update_zero_negative(diff);
    Ok(match mode {
        AddressingMode::Immediate => 2,
        AddressingMode::ZeroPage => 3,
        AddressingMode::Absolute => 4,
        _ => 0,
    })
}

// INC - Increment Memory
fn inc(cpu: &mut Cpu, memory: &mut dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    let value = memory.read(result.address).wrapping_add(1);
    memory.write(result.address, value);
    cpu.status.update_zero_negative(value);
    Ok(match mode {
        AddressingMode::ZeroPage => 5,
        AddressingMode::ZeroPageX => 6,
        AddressingMode::Absolute => 6,
        AddressingMode::AbsoluteX => 7,
        _ => 0,
    })
}

// DEC - Decrement Memory
fn dec(cpu: &mut Cpu, memory: &mut dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    let value = memory.read(result.address).wrapping_sub(1);
    memory.write(result.address, value);
    cpu.status.update_zero_negative(value);
    Ok(match mode {
        AddressingMode::ZeroPage => 5,
        AddressingMode::ZeroPageX => 6,
        AddressingMode::Absolute => 6,
        AddressingMode::AbsoluteX => 7,
        _ => 0,
    })
}

// ASL - Arithmetic Shift Left (Accumulator)
fn asl_acc(cpu: &mut Cpu) -> Result<u8> {
    cpu.status.carry = (cpu.a & 0x80) != 0;
    cpu.a <<= 1;
    cpu.status.update_zero_negative(cpu.a);
    Ok(2)
}

// ASL - Arithmetic Shift Left (Memory)
fn asl(cpu: &mut Cpu, memory: &mut dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    let value = memory.read(result.address);
    cpu.status.carry = (value & 0x80) != 0;
    let new_value = value << 1;
    memory.write(result.address, new_value);
    cpu.status.update_zero_negative(new_value);
    Ok(match mode {
        AddressingMode::ZeroPage => 5,
        AddressingMode::ZeroPageX => 6,
        AddressingMode::Absolute => 6,
        AddressingMode::AbsoluteX => 7,
        _ => 0,
    })
}

// LSR - Logical Shift Right (Accumulator)
fn lsr_acc(cpu: &mut Cpu) -> Result<u8> {
    cpu.status.carry = (cpu.a & 0x01) != 0;
    cpu.a >>= 1;
    cpu.status.update_zero_negative(cpu.a);
    Ok(2)
}

// LSR - Logical Shift Right (Memory)
fn lsr(cpu: &mut Cpu, memory: &mut dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    let value = memory.read(result.address);
    cpu.status.carry = (value & 0x01) != 0;
    let new_value = value >> 1;
    memory.write(result.address, new_value);
    cpu.status.update_zero_negative(new_value);
    Ok(match mode {
        AddressingMode::ZeroPage => 5,
        AddressingMode::ZeroPageX => 6,
        AddressingMode::Absolute => 6,
        AddressingMode::AbsoluteX => 7,
        _ => 0,
    })
}

// ROL - Rotate Left (Accumulator)
fn rol_acc(cpu: &mut Cpu) -> Result<u8> {
    let old_carry = cpu.status.carry as u8;
    cpu.status.carry = (cpu.a & 0x80) != 0;
    cpu.a = (cpu.a << 1) | old_carry;
    cpu.status.update_zero_negative(cpu.a);
    Ok(2)
}

// ROL - Rotate Left (Memory)
fn rol(cpu: &mut Cpu, memory: &mut dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    let value = memory.read(result.address);
    let old_carry = cpu.status.carry as u8;
    cpu.status.carry = (value & 0x80) != 0;
    let new_value = (value << 1) | old_carry;
    memory.write(result.address, new_value);
    cpu.status.update_zero_negative(new_value);
    Ok(match mode {
        AddressingMode::ZeroPage => 5,
        AddressingMode::ZeroPageX => 6,
        AddressingMode::Absolute => 6,
        AddressingMode::AbsoluteX => 7,
        _ => 0,
    })
}

// ROR - Rotate Right (Accumulator)
fn ror_acc(cpu: &mut Cpu) -> Result<u8> {
    let old_carry = cpu.status.carry as u8;
    cpu.status.carry = (cpu.a & 0x01) != 0;
    cpu.a = (cpu.a >> 1) | (old_carry << 7);
    cpu.status.update_zero_negative(cpu.a);
    Ok(2)
}

// ROR - Rotate Right (Memory)
fn ror(cpu: &mut Cpu, memory: &mut dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    let value = memory.read(result.address);
    let old_carry = cpu.status.carry as u8;
    cpu.status.carry = (value & 0x01) != 0;
    let new_value = (value >> 1) | (old_carry << 7);
    memory.write(result.address, new_value);
    cpu.status.update_zero_negative(new_value);
    Ok(match mode {
        AddressingMode::ZeroPage => 5,
        AddressingMode::ZeroPageX => 6,
        AddressingMode::Absolute => 6,
        AddressingMode::AbsoluteX => 7,
        _ => 0,
    })
}

// BIT - Test Bits
fn bit(cpu: &mut Cpu, memory: &dyn Memory, mode: AddressingMode) -> Result<u8> {
    let result = cpu.get_operand_address(memory, mode);
    let value = memory.read(result.address);
    cpu.status.zero = (cpu.a & value) == 0;
    cpu.status.negative = (value & 0x80) != 0;
    cpu.status.overflow = (value & 0x40) != 0;
    Ok(match mode {
        AddressingMode::ZeroPage => 3,
        AddressingMode::Absolute => 4,
        _ => 0,
    })
}
