#[cfg(test)]
mod tests {
    use crate::cpu::Cpu;
    use crate::memory::{Memory, BasicMemory};

    #[test]
    fn test_lda_immediate() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        // LDA #$42
        memory.write(0x0000, 0xA9);
        memory.write(0x0001, 0x42);
        
        cpu.pc = 0x0000;
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.status.zero, false);
        assert_eq!(cpu.status.negative, false);
    }

    #[test]
    fn test_lda_zero_flag() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        // LDA #$00
        memory.write(0x0000, 0xA9);
        memory.write(0x0001, 0x00);
        
        cpu.pc = 0x0000;
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.status.zero, true);
        assert_eq!(cpu.status.negative, false);
    }

    #[test]
    fn test_lda_negative_flag() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        // LDA #$FF
        memory.write(0x0000, 0xA9);
        memory.write(0x0001, 0xFF);
        
        cpu.pc = 0x0000;
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.a, 0xFF);
        assert_eq!(cpu.status.zero, false);
        assert_eq!(cpu.status.negative, true);
    }

    #[test]
    fn test_tax() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.a = 0x42;
        cpu.pc = 0x0000;
        memory.write(0x0000, 0xAA); // TAX
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.x, 0x42);
    }

    #[test]
    fn test_sta_absolute() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.a = 0x42;
        cpu.pc = 0x0000;
        
        // STA $1234
        memory.write(0x0000, 0x8D);
        memory.write(0x0001, 0x34);
        memory.write(0x0002, 0x12);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(memory.read(0x1234), 0x42);
    }

    #[test]
    fn test_jmp_absolute() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.pc = 0x0000;
        
        // JMP $1234
        memory.write(0x0000, 0x4C);
        memory.write(0x0001, 0x34);
        memory.write(0x0002, 0x12);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.pc, 0x1234);
    }

    #[test]
    fn test_stack_operations() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.push(&mut memory, 0x42);
        let val = cpu.pop(&memory);
        
        assert_eq!(val, 0x42);
    }

    #[test]
    fn test_adc_no_carry() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.a = 0x10;
        cpu.pc = 0x0000;
        // ADC #$20
        memory.write(0x0000, 0x69);
        memory.write(0x0001, 0x20);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.a, 0x30);
        assert_eq!(cpu.status.carry, false);
        assert_eq!(cpu.status.zero, false);
        assert_eq!(cpu.status.negative, false);
    }

    #[test]
    fn test_adc_with_carry() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.a = 0xFF;
        cpu.status.carry = true;
        cpu.pc = 0x0000;
        // ADC #$01
        memory.write(0x0000, 0x69);
        memory.write(0x0001, 0x01);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.a, 0x01);
        assert_eq!(cpu.status.carry, true);
    }

    #[test]
    fn test_sbc() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.a = 0x30;
        cpu.status.carry = true; // No borrow
        cpu.pc = 0x0000;
        // SBC #$10
        memory.write(0x0000, 0xE9);
        memory.write(0x0001, 0x10);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.a, 0x20);
        assert_eq!(cpu.status.carry, true);
    }

    #[test]
    fn test_and() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.a = 0xFF;
        cpu.pc = 0x0000;
        // AND #$0F
        memory.write(0x0000, 0x29);
        memory.write(0x0001, 0x0F);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.a, 0x0F);
    }

    #[test]
    fn test_ora() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.a = 0xF0;
        cpu.pc = 0x0000;
        // ORA #$0F
        memory.write(0x0000, 0x09);
        memory.write(0x0001, 0x0F);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.a, 0xFF);
    }

    #[test]
    fn test_eor() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.a = 0xFF;
        cpu.pc = 0x0000;
        // EOR #$FF
        memory.write(0x0000, 0x49);
        memory.write(0x0001, 0xFF);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.status.zero, true);
    }

    #[test]
    fn test_cmp() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.a = 0x42;
        cpu.pc = 0x0000;
        // CMP #$42
        memory.write(0x0000, 0xC9);
        memory.write(0x0001, 0x42);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.status.zero, true);
        assert_eq!(cpu.status.carry, true);
    }

    #[test]
    fn test_inc() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        memory.write(0x0042, 0x10);
        cpu.pc = 0x0000;
        // INC $42
        memory.write(0x0000, 0xE6);
        memory.write(0x0001, 0x42);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(memory.read(0x0042), 0x11);
    }

    #[test]
    fn test_dec() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        memory.write(0x0042, 0x10);
        cpu.pc = 0x0000;
        // DEC $42
        memory.write(0x0000, 0xC6);
        memory.write(0x0001, 0x42);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(memory.read(0x0042), 0x0F);
    }

    #[test]
    fn test_asl() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.a = 0x42;
        cpu.pc = 0x0000;
        // ASL A
        memory.write(0x0000, 0x0A);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.a, 0x84);
        assert_eq!(cpu.status.carry, false);
        assert_eq!(cpu.status.negative, true);
    }

    #[test]
    fn test_lsr() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.a = 0x43;
        cpu.pc = 0x0000;
        // LSR A
        memory.write(0x0000, 0x4A);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.a, 0x21);
        assert_eq!(cpu.status.carry, true);
    }

    #[test]
    fn test_rol() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.a = 0x81;
        cpu.status.carry = true;
        cpu.pc = 0x0000;
        // ROL A
        memory.write(0x0000, 0x2A);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.a, 0x03);
        assert_eq!(cpu.status.carry, true);
    }

    #[test]
    fn test_ror() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.a = 0x81;
        cpu.status.carry = true;
        cpu.pc = 0x0000;
        // ROR A
        memory.write(0x0000, 0x6A);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.a, 0xC0);
        assert_eq!(cpu.status.carry, true);
    }

    #[test]
    fn test_bit() {
        let mut cpu = Cpu::new();
        let mut memory = BasicMemory::new();
        
        cpu.a = 0x0F;
        memory.write(0x0042, 0xC0); // Bit 7 and 6 set
        cpu.pc = 0x0000;
        // BIT $42
        memory.write(0x0000, 0x24);
        memory.write(0x0001, 0x42);
        
        cpu.step(&mut memory).unwrap();
        
        assert_eq!(cpu.status.zero, true); // No bits match
        assert_eq!(cpu.status.negative, true); // Bit 7 set
        assert_eq!(cpu.status.overflow, true); // Bit 6 set
    }
}
