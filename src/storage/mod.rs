use std::fs;
use std::path::PathBuf;
use std::io::Write;
use anyhow::Result;

/// Directory where virtual 1541 disks are stored
const STORAGE_DIR: &str = ".go64/1541";

/// Initialize storage subsystem
pub fn init() -> Result<()> {
    let storage_path = get_storage_path()?;
    if !storage_path.exists() {
        fs::create_dir_all(&storage_path)?;
    }
    Ok(())
}

/// Get the full path to the storage directory
fn get_storage_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home_dir.join(STORAGE_DIR))
}

/// Save a PRG file (2-byte load address + data)
pub fn save_prg(filename: &[u8], start_addr: u16, data: &[u8]) -> Result<()> {
    // Sanitize filename
    let safe_name = sanitize_filename(filename);
    
    // Construct full path
    let mut path = get_storage_path()?;
    path.push(safe_name);
    
    // Create PRG file format: [Low Addr] [High Addr] [Data...]
    let mut file_content = Vec::with_capacity(2 + data.len());
    file_content.push((start_addr & 0xFF) as u8);
    file_content.push(((start_addr >> 8) & 0xFF) as u8);
    file_content.extend_from_slice(data);
    
    // Write to disk
    let mut file = fs::File::create(path)?;
    file.write_all(&file_content)?;
    
    Ok(())
}

/// Load a PRG file
/// Returns (start_address, data)
pub fn load_prg(filename: &[u8]) -> Result<(u16, Vec<u8>)> {
    // Sanitize filename
    let safe_name = sanitize_filename(filename);
    
    // Construct full path
    let mut path = get_storage_path()?;
    path.push(safe_name);
    
    // Read file
    let content = fs::read(path)?;
    
    if content.len() < 2 {
        return Err(anyhow::anyhow!("File too short to be a valid PRG"));
    }
    
    // Parse header
    let start_addr = (content[0] as u16) | ((content[1] as u16) << 8);
    let data = content[2..].to_vec();
    
    Ok((start_addr, data))
}

/// Generate a C64 directory listing as a BASIC program
/// Returns (load_address, data)
pub fn list_directory() -> Result<(u16, Vec<u8>)> {
    let mut data = Vec::new();
    let start_addr = 0x0801; // Standard BASIC start address
    
    // Helper to write a BASIC line
    // Returns the address of the NEXT line
    let mut current_addr = start_addr;
    
    let mut write_line = |line_num: u16, text: &str| {
        // Calculate line size: 2 (next) + 2 (num) + text.len() + 1 (null)
        let line_size = 2 + 2 + text.len() + 1;
        let next_addr = current_addr + line_size as u16;
        
        // Next Line Pointer
        data.push((next_addr & 0xFF) as u8);
        data.push(((next_addr >> 8) & 0xFF) as u8);
        
        // Line Number (used as block count in directory listings)
        data.push((line_num & 0xFF) as u8);
        data.push(((line_num >> 8) & 0xFF) as u8);
        
        // Text
        data.extend_from_slice(text.as_bytes());
        data.push(0); // Null terminator
        
        current_addr = next_addr;
    };

    // Header: Line 0
    // Simplified header to avoid special characters that might render poorly in terminal
    write_line(0, "\"FLOPPY DISK\"     ID 2A");
    
    // Read directory
    let path = get_storage_path()?;
    if path.exists() {
        // Collect entries to sort them
        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.to_lowercase().ends_with(".prg") {
                                let len = entry.metadata().map(|m| m.len()).unwrap_or(0);
                                entries.push((name.to_string(), len));
                            }
                        }
                    }
                }
            }
        }
        
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        
        for (name, size) in entries {
            // Calculate blocks (approx 254 bytes per block)
            let blocks = (size + 253) / 254;
            
            // Format name: remove .prg, quote it, pad to align "PRG"
            let disp_name = &name[0..name.len()-4];
            let upper_name = disp_name.to_uppercase();
            
            // Pad with spaces to make it look nice (max 16 chars for name usually)
            // "NAME"            PRG
            let mut line_text = format!("\"{}\"", upper_name);
            while line_text.len() < 18 {
                line_text.push(' ');
            }
            line_text.push_str("PRG");
            
            write_line(blocks as u16, &line_text);
        }
    }
    
    // Footer: Line <Free Blocks>
    write_line(664, "BLOCKS FREE.");
    
    // End of Program (2 null bytes)
    data.push(0);
    data.push(0);
    
    Ok((start_addr, data))
}

/// Sanitize C64 filename to be safe for host OS
/// Replaces reserved characters with '_' and trims whitespace
pub fn sanitize_filename(petscii: &[u8]) -> String {
    let mut name = String::new();
    
    for &byte in petscii {
        let ch = byte as char;
        // Check for reserved chars on Windows/Unix
        // / \ : * ? " < > |
        match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => name.push('_'),
            _ => {
                // Only allow printable ASCII
                if byte >= 32 && byte <= 126 {
                    name.push(ch);
                } else {
                    name.push('_');
                }
            }
        }
    }
    
    // Trim whitespace
    let trimmed = name.trim().to_string();
    
    // Ensure not empty
    let mut final_name = if trimmed.is_empty() {
        "UNNAMED".to_string()
    } else {
        trimmed
    };
    
    // Ensure extension
    if !final_name.to_lowercase().ends_with(".prg") {
        final_name.push_str(".prg");
    }
    
    final_name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        // Basic case
        assert_eq!(sanitize_filename(b"TEST"), "TEST.prg");
        
        // With extension
        assert_eq!(sanitize_filename(b"GAME.PRG"), "GAME.PRG");
        
        // Reserved chars
        assert_eq!(sanitize_filename(b"TEST/FILE"), "TEST_FILE.prg");
        assert_eq!(sanitize_filename(b"TEST:FILE"), "TEST_FILE.prg");
        
        // Whitespace
        assert_eq!(sanitize_filename(b"  TEST  "), "TEST.prg");
        
        // Empty
        assert_eq!(sanitize_filename(b""), "UNNAMED.prg");
        
        // Non-printable
        assert_eq!(sanitize_filename(&[0, 1, 65, 66]), "__AB.prg");
    }
}
