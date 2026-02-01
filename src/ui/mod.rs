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
    let vertical_center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(SCREEN_HEIGHT as u16 + 2),
            Constraint::Min(0),
        ])
        .split(chunks[1])[1];

    // Center the C64 screen horizontally
    let screen_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(SCREEN_WIDTH as u16 + 2), // +2 for borders
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
    let vertical_center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(SCREEN_HEIGHT as u16 + 2),
            Constraint::Min(0),
        ])
        .split(chunks[0])[1];

    // Center the C64 screen horizontally
    let screen_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(SCREEN_WIDTH as u16 + 2), // +2 for borders
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
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(c64_color_to_ratatui(vic.get_border_color())))
        .style(Style::default().bg(c64_color_to_ratatui(vic.get_background_color())));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    // Render screen content line by line
    let mut lines = Vec::new();
    for y in 0..SCREEN_HEIGHT.min(inner.height as usize) {
        let mut line_spans = Vec::new();
        for x in 0..SCREEN_WIDTH.min(inner.width as usize) {
            let (char_code, color) = vic.get_screen_char(memory, x, y);
            let ch = crate::vic::screen_code_to_char(char_code);
            let fg = c64_color_to_ratatui(C64Color::from_u8(color));
            line_spans.push(Span::styled(ch.to_string(), Style::default().fg(fg)));
        }
        lines.push(Line::from(line_spans));
    }
    
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
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
        C64Color::Blue => Color::Rgb(0, 0, 170),           // Dark blue for background
        C64Color::Yellow => Color::Rgb(238, 238, 119),
        C64Color::Orange => Color::Rgb(221, 136, 85),
        C64Color::Brown => Color::Rgb(102, 68, 0),
        C64Color::LightRed => Color::Rgb(255, 119, 119),
        C64Color::DarkGrey => Color::Rgb(51, 51, 51),
        C64Color::Grey => Color::Rgb(119, 119, 119),
        C64Color::LightGreen => Color::Rgb(170, 255, 102),
        C64Color::LightBlue => Color::Rgb(0, 136, 255),   // Bright blue for text
        C64Color::LightGrey => Color::Rgb(187, 187, 187),
    }
}
