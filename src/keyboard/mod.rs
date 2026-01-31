/// C64 Keyboard Matrix Mapping
/// 
/// The C64 keyboard is an 8x8 matrix connected to CIA1
/// Port A (PRA) selects rows, Port B (PRB) reads columns
/// Both are active low (0 = selected/pressed)

use crossterm::event::KeyCode;

/// C64 keyboard matrix position (row, column)
pub type MatrixPosition = (u8, u8);

/// Map terminal KeyCode to C64 keyboard matrix position
pub fn map_key(key: KeyCode) -> Option<Vec<MatrixPosition>> {
    // Returns Vec because some keys need multiple matrix positions (e.g., shifted keys)
    // Format: (row, col)
    
    match key {
        // Row 0: DEL, RETURN, →, F7, F1, F3, F5, ↓
        KeyCode::Backspace => Some(vec![(0, 0)]),  // DEL (backspace on modern keyboard)
        KeyCode::Enter => Some(vec![(0, 1)]),      // RETURN
        KeyCode::Right => Some(vec![(0, 2)]),      // →
        KeyCode::F(7) => Some(vec![(0, 3)]),       // F7
        KeyCode::F(1) => Some(vec![(0, 4)]),       // F1
        KeyCode::F(3) => Some(vec![(0, 5)]),       // F3
        KeyCode::F(5) => Some(vec![(0, 6)]),       // F5
        KeyCode::Down => Some(vec![(0, 7)]),       // ↓
        
        // Row 1: 3#, W, A, 4$, Z, S, E, Left Shift
        KeyCode::Char('3') => Some(vec![(1, 0)]),
        KeyCode::Char('#') => Some(vec![(1, 7), (1, 0)]),  // SHIFT + 3
        KeyCode::Char('w') | KeyCode::Char('W') => Some(vec![(1, 1)]),
        KeyCode::Char('a') | KeyCode::Char('A') => Some(vec![(1, 2)]),
        KeyCode::Char('4') => Some(vec![(1, 3)]),
        KeyCode::Char('$') => Some(vec![(1, 7), (1, 3)]),  // SHIFT + 4
        KeyCode::Char('z') | KeyCode::Char('Z') => Some(vec![(1, 4)]),
        KeyCode::Char('s') | KeyCode::Char('S') => Some(vec![(1, 5)]),
        KeyCode::Char('e') | KeyCode::Char('E') => Some(vec![(1, 6)]),
        
        // Row 2: 5%, R, D, 6&, C, F, T, X
        KeyCode::Char('5') => Some(vec![(2, 0)]),
        KeyCode::Char('%') => Some(vec![(1, 7), (2, 0)]),  // SHIFT + 5
        KeyCode::Char('r') | KeyCode::Char('R') => Some(vec![(2, 1)]),
        KeyCode::Char('d') | KeyCode::Char('D') => Some(vec![(2, 2)]),
        KeyCode::Char('6') => Some(vec![(2, 3)]),
        KeyCode::Char('&') => Some(vec![(1, 7), (2, 3)]),  // SHIFT + 6
        KeyCode::Char('c') | KeyCode::Char('C') => Some(vec![(2, 4)]),
        KeyCode::Char('f') | KeyCode::Char('F') => Some(vec![(2, 5)]),
        KeyCode::Char('t') | KeyCode::Char('T') => Some(vec![(2, 6)]),
        KeyCode::Char('x') | KeyCode::Char('X') => Some(vec![(2, 7)]),
        
        // Row 3: 7', Y, G, 8(, B, H, U, V
        KeyCode::Char('7') => Some(vec![(3, 0)]),
        KeyCode::Char('\'') => Some(vec![(1, 7), (3, 0)]),  // SHIFT + 7 (apostrophe)
        KeyCode::Char('y') | KeyCode::Char('Y') => Some(vec![(3, 1)]),
        KeyCode::Char('g') | KeyCode::Char('G') => Some(vec![(3, 2)]),
        KeyCode::Char('8') => Some(vec![(3, 3)]),
        KeyCode::Char('(') => Some(vec![(1, 7), (3, 3)]),  // SHIFT + 8
        KeyCode::Char('b') | KeyCode::Char('B') => Some(vec![(3, 4)]),
        KeyCode::Char('h') | KeyCode::Char('H') => Some(vec![(3, 5)]),
        KeyCode::Char('u') | KeyCode::Char('U') => Some(vec![(3, 6)]),
        KeyCode::Char('v') | KeyCode::Char('V') => Some(vec![(3, 7)]),
        
        // Row 4: 9), I, J, 0, M, K, O, N
        KeyCode::Char('9') => Some(vec![(4, 0)]),
        KeyCode::Char(')') => Some(vec![(1, 7), (4, 0)]),  // SHIFT + 9
        KeyCode::Char('i') | KeyCode::Char('I') => Some(vec![(4, 1)]),
        KeyCode::Char('j') | KeyCode::Char('J') => Some(vec![(4, 2)]),
        KeyCode::Char('0') => Some(vec![(4, 3)]),
        KeyCode::Char('m') | KeyCode::Char('M') => Some(vec![(4, 4)]),
        KeyCode::Char('k') | KeyCode::Char('K') => Some(vec![(4, 5)]),
        KeyCode::Char('o') | KeyCode::Char('O') => Some(vec![(4, 6)]),
        KeyCode::Char('n') | KeyCode::Char('N') => Some(vec![(4, 7)]),
        
        // Row 5: +, P, L, -, ., :, @, ,
        KeyCode::Char('+') => Some(vec![(5, 0)]),
        KeyCode::Char('p') | KeyCode::Char('P') => Some(vec![(5, 1)]),
        KeyCode::Char('l') | KeyCode::Char('L') => Some(vec![(5, 2)]),
        KeyCode::Char('-') => Some(vec![(5, 3)]),
        KeyCode::Char('.') => Some(vec![(5, 4)]),
        KeyCode::Char('>') => Some(vec![(1, 7), (5, 4)]),  // SHIFT + .
        KeyCode::Char(':') => Some(vec![(5, 5)]),
        KeyCode::Char('[') => Some(vec![(1, 7), (5, 5)]),  // SHIFT + : = [ on C64
        KeyCode::Char('@') => Some(vec![(5, 6)]),
        KeyCode::Char(',') => Some(vec![(5, 7)]),
        KeyCode::Char('<') => Some(vec![(1, 7), (5, 7)]),  // SHIFT + ,
        
        // Row 6: £ (pound), *, ;, Home, Right Shift, =, ↑, /
        KeyCode::Char('*') => Some(vec![(6, 1)]),
        KeyCode::Char(';') => Some(vec![(6, 2)]),
        KeyCode::Char(']') => Some(vec![(1, 7), (6, 2)]),  // SHIFT + ; = ] on C64
        KeyCode::Home => Some(vec![(6, 3)]),
        KeyCode::Char('=') => Some(vec![(6, 5)]),
        KeyCode::Up => Some(vec![(6, 6)]),                 // ↑
        KeyCode::Char('/') => Some(vec![(6, 7)]),
        KeyCode::Char('?') => Some(vec![(1, 7), (6, 7)]),  // SHIFT + /
        
        // Row 7: 1!, ←, Ctrl, 2", SPACE, Commodore, Q, Run/Stop
        KeyCode::Char('1') => Some(vec![(7, 0)]),
        KeyCode::Char('!') => Some(vec![(1, 7), (7, 0)]),  // SHIFT + 1
        KeyCode::Left => Some(vec![(7, 1)]),               // ← (cursor left)
        KeyCode::Char('2') => Some(vec![(7, 3)]),
        KeyCode::Char('"') => Some(vec![(1, 7), (7, 3)]),  // SHIFT + 2
        KeyCode::Char(' ') => Some(vec![(7, 4)]),          // SPACE
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(vec![(7, 6)]),
        KeyCode::Tab => Some(vec![(7, 7)]),                // Run/Stop
        
        _ => None,
    }
}

    // /// Check if a key is uppercase (needs shift if it's a letter)
    // pub fn needs_shift(key: KeyCode) -> bool {
    //     match key {
    //         KeyCode::Char(c) => c.is_uppercase(),
    //         _ => false,
    //     }
    // }
