// I/O system and ROM loading

use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

pub struct RomSet {
    pub basic: Vec<u8>,
    pub kernal: Vec<u8>,
    pub char_rom: Vec<u8>,
}

impl RomSet {
    pub fn load_from_directory(rom_dir: &str) -> Result<Self> {
        let basic_path = Path::new(rom_dir).join("basic.rom");
        let kernal_path = Path::new(rom_dir).join("kernal.rom");
        let char_path = Path::new(rom_dir).join("char.rom");
        
        let basic = fs::read(&basic_path)
            .context(format!("Failed to load BASIC ROM from {:?}", basic_path))?;
        let kernal = fs::read(&kernal_path)
            .context(format!("Failed to load KERNAL ROM from {:?}", kernal_path))?;
        let char_rom = fs::read(&char_path)
            .context(format!("Failed to load Character ROM from {:?}", char_path))?;
        
        // Validate sizes
        if basic.len() != 0x2000 {
            anyhow::bail!("BASIC ROM must be 8KB (0x2000 bytes), got {} bytes", basic.len());
        }
        if kernal.len() != 0x2000 {
            anyhow::bail!("KERNAL ROM must be 8KB (0x2000 bytes), got {} bytes", kernal.len());
        }
        if char_rom.len() != 0x1000 {
            anyhow::bail!("Character ROM must be 4KB (0x1000 bytes), got {} bytes", char_rom.len());
        }
        
        Ok(Self {
            basic,
            kernal,
            char_rom,
        })
    }
}

pub fn create_rom_directory_if_missing() -> Result<()> {
    let rom_dir = Path::new("roms");
    if !rom_dir.exists() {
        fs::create_dir(rom_dir)?;
        println!("Created 'roms' directory.");
        println!("\nTo run the emulator, you need C64 ROM files:");
        println!("  - roms/basic.rom  (8KB - BASIC interpreter)");
        println!("  - roms/kernal.rom (8KB - Operating system)");
        println!("  - roms/char.rom   (4KB - Character set)");
        println!("\nYou can extract these from:");
        println!("  1. VICE emulator installation");
        println!("  2. Download from: https://www.zimmers.net/anonftp/pub/cbm/firmware/computers/c64/");
        println!("\nFiles needed:");
        println!("  - basic.901226-01.bin  -> rename to basic.rom");
        println!("  - kernal.901227-03.bin -> rename to kernal.rom");
        println!("  - characters.901225-01.bin -> rename to char.rom");
    }
    Ok(())
}
