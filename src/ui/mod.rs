// Terminal UI using ratatui

use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use crossterm::{
    event::{self, Event, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use anyhow::Result;

use crate::vic::{C64Color, SCREEN_WIDTH, SCREEN_HEIGHT};

pub struct TerminalUI {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalUI {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        
        Ok(Self { terminal })
    }
    
    pub fn render<F>(&mut self, render_fn: F) -> Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        self.terminal.draw(render_fn)?;
        Ok(())
    }
    
    pub fn poll_event(&self) -> Result<Option<KeyEvent>> {
        // Use a zero timeout to make this non-blocking
        // The main loop manages frame timing via thread::sleep
        if event::poll(std::time::Duration::from_micros(0))? {
            if let Event::Key(key) = event::read()? {
                return Ok(Some(key));
            }
        }
        Ok(None)
    }
}

impl Drop for TerminalUI {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}

pub fn create_layout(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),     // Title bar
            Constraint::Min(1),        // Flexible middle area
            Constraint::Length(3),     // Status bar
        ])
        .split(area);
    
    // Center the C64 screen vertically within the middle chunk
    // Use a larger frame to accommodate the C64 border (approx 3 lines top/bottom)
    let display_height = SCREEN_HEIGHT as u16 + 6; // 25 + 6 = 31
    let vertical_center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(display_height),
            Constraint::Min(0),
        ])
        .split(chunks[1])[1];

    // Center the C64 screen horizontally
    // Use a larger frame to accommodate the C64 border (approx 5 chars left/right)
    let display_width = SCREEN_WIDTH as u16 + 10; // 40 + 10 = 50
    let screen_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(display_width),
            Constraint::Min(1),
        ])
        .split(vertical_center)[1];
    
    (chunks[0], screen_area, chunks[2])
}

pub fn create_simple_layout(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),        // Flexible middle area
            Constraint::Length(1),     // Minimal status line
        ])
        .split(area);
    
    // Center the C64 screen vertically
    let display_height = SCREEN_HEIGHT as u16 + 6; // 31
    let vertical_center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(display_height),
            Constraint::Min(0),
        ])
        .split(chunks[0])[1];

    // Center the C64 screen horizontally
    let display_width = SCREEN_WIDTH as u16 + 10; // 50
    let screen_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(display_width), 
            Constraint::Min(1),
        ])
        .split(vertical_center)[1];
    
    (screen_area, chunks[1])
}

pub fn render_simple_status(frame: &mut Frame, area: Rect) {
    let status = "F1: Debug | F5: Pause/Resume | PgUp: Restore | ESC: Quit";
    let paragraph = Paragraph::new(status)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

pub fn render_title_bar(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("go64 - Commodore 64 Emulator")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

pub fn render_c64_screen(
    frame: &mut Frame,
    area: Rect,
    vic: &crate::vic::VicII,
    memory: &dyn crate::memory::Memory,
) {
    // 1. Render the Border
    // The 'area' passed in is now the full frame (50x31) including the border.
    // We fill this entire block with the Border Color.
    let border_color = c64_color_to_ratatui(vic.get_border_color());
    let border_block = Block::default()
        .style(Style::default().bg(border_color));
    frame.render_widget(border_block, area);

    // 2. Calculate the Inner Screen Area (40x25) centered in the Border Area
    // The Layout was calculated to be SCREEN_WIDTH + 10 wide, SCREEN_HEIGHT + 6 high.
    // So we want to inset by 5 horizontally and 3 vertically.
    
    // Safety check to ensure we don't panic if area is too small
    let inner_x = area.x + (area.width.saturating_sub(SCREEN_WIDTH as u16)) / 2;
    let inner_y = area.y + (area.height.saturating_sub(SCREEN_HEIGHT as u16)) / 2;
    let inner_width = (SCREEN_WIDTH as u16).min(area.width);
    let inner_height = (SCREEN_HEIGHT as u16).min(area.height);
    
    let screen_rect = Rect::new(inner_x, inner_y, inner_width, inner_height);

    // 3. Render the Background (Main Screen)
    let bg_color = c64_color_to_ratatui(vic.get_background_color());
    let screen_block = Block::default()
        .style(Style::default().bg(bg_color));
    frame.render_widget(screen_block, screen_rect);
    
    // 4. Render screen content line by line
    let mut lines = Vec::new();
    for y in 0..SCREEN_HEIGHT.min(screen_rect.height as usize) {
        let mut line_spans = Vec::new();
        for x in 0..SCREEN_WIDTH.min(screen_rect.width as usize) {
            let (char_code, color) = vic.get_screen_char(memory, x, y);
            let ch = crate::vic::screen_code_to_char(char_code);
            let fg = c64_color_to_ratatui(C64Color::from_u8(color));
            line_spans.push(Span::styled(ch.to_string(), Style::default().fg(fg).bg(bg_color)));
        }
        lines.push(Line::from(line_spans));
    }
    
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, screen_rect);
}

pub fn render_status_bar(frame: &mut Frame, area: Rect, cpu: &crate::cpu::Cpu) {
    let status = format!(
        "PC:${:04X} A:${:02X} X:${:02X} Y:${:02X} SP:${:02X} Cyc:{} | F1:Hide | F5:Pause | PgUp:Rst | ESC:Quit",
        cpu.pc, cpu.a, cpu.x, cpu.y, cpu.sp, cpu.cycles
    );
    
    let paragraph = Paragraph::new(status)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

fn c64_color_to_ratatui(color: C64Color) -> Color {
    // Authentic Commodore 64 color palette RGB values
    match color {
        C64Color::Black => Color::Rgb(0, 0, 0),
        C64Color::White => Color::Rgb(255, 255, 255),
        C64Color::Red => Color::Rgb(136, 0, 0),
        C64Color::Cyan => Color::Rgb(170, 255, 238),
        C64Color::Purple => Color::Rgb(204, 68, 204),
        C64Color::Green => Color::Rgb(0, 204, 85),
        C64Color::Blue => Color::Rgb(53, 40, 121),           // Authentic Pepto Blue (Dark Purple-Blue)
        C64Color::Yellow => Color::Rgb(238, 238, 119),
        C64Color::Orange => Color::Rgb(221, 136, 85),
        C64Color::Brown => Color::Rgb(102, 68, 0),
        C64Color::LightRed => Color::Rgb(255, 119, 119),
        C64Color::DarkGrey => Color::Rgb(51, 51, 51),
        C64Color::Grey => Color::Rgb(119, 119, 119),
        C64Color::LightGreen => Color::Rgb(170, 255, 102),
        C64Color::LightBlue => Color::Rgb(108, 108, 255),   // Authentic Pepto Light Blue (Periwinkle)
        C64Color::LightGrey => Color::Rgb(187, 187, 187),
    }
}
