/// lumen-tui — Raw terminal bindings mínimo para LÚMEN
/// Solo expone operaciones de terminal que requieren Rust.
/// TODO el TUI lógico (ventanas, tablas, layout) vive en stdlib/tui.nv
use crossterm::{
    cursor::{MoveTo, Show, Hide},
    event::{self, Event, KeyCode, KeyEventKind, MouseEventKind},
    execute,
    style::{SetForegroundColor, SetBackgroundColor, Color as CColor, ResetColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, size},
    QueueableCommand,
};
use std::io::{stdout, Write};

pub fn raw_mode(on: bool) -> Result<(), String> {
    if on {
        enable_raw_mode().map_err(|e| e.to_string())?;
        execute!(stdout(), EnterAlternateScreen, Hide).map_err(|e| e.to_string())
    } else {
        execute!(stdout(), Show, LeaveAlternateScreen).map_err(|e| e.to_string())?;
        disable_raw_mode().map_err(|e| e.to_string())
    }
}

pub fn terminal_size() -> Result<(u16, u16), String> {
    size().map_err(|e| e.to_string())
}

pub fn write_at(text: &str, x: u16, y: u16) -> Result<(), String> {
    let mut out = stdout();
    out.queue(MoveTo(x, y)).map_err(|e| e.to_string())?;
    out.queue(crossterm::style::Print(text)).map_err(|e| e.to_string())?;
    out.flush().map_err(|e| e.to_string())
}

pub fn clear_screen() -> Result<(), String> {
    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).map_err(|e| e.to_string())
}

pub fn set_color(fg: &str, bg: &str) -> Result<(), String> {
    let fg_c = parse_color(fg).unwrap_or(CColor::Reset);
    let bg_c = parse_color(bg).unwrap_or(CColor::Reset);
    execute!(stdout(), SetForegroundColor(fg_c), SetBackgroundColor(bg_c)).map_err(|e| e.to_string())
}

pub fn reset_color() -> Result<(), String> {
    execute!(stdout(), ResetColor).map_err(|e| e.to_string())
}

fn parse_color(s: &str) -> Option<CColor> {
    match s.to_lowercase().as_str() {
        "negro" | "black" => Some(CColor::Black),
        "rojo" | "red" => Some(CColor::DarkRed),
        "verde" | "green" => Some(CColor::DarkGreen),
        "amarillo" | "yellow" => Some(CColor::DarkYellow),
        "azul" | "blue" => Some(CColor::DarkBlue),
        "magenta" | "purple" => Some(CColor::DarkMagenta),
        "cyan" => Some(CColor::DarkCyan),
        "blanco" | "white" | "gris" | "gray" => Some(CColor::Grey),
        "rojob" | "lightred" => Some(CColor::Red),
        "verdeb" | "lightgreen" => Some(CColor::Green),
        "amarillob" | "lightyellow" => Some(CColor::Yellow),
        "azulb" | "lightblue" => Some(CColor::Blue),
        "magentalb" | "lightmagenta" => Some(CColor::Magenta),
        "cyanb" | "lightcyan" => Some(CColor::Cyan),
        "reset" => Some(CColor::Reset),
        _ => None,
    }
}

pub fn read_event() -> Result<String, String> {
    match event::read().map_err(|e| e.to_string())? {
        Event::Key(k) if k.kind == KeyEventKind::Press => Ok(match k.code {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Enter => "enter".into(),
            KeyCode::Esc => "esc".into(),
            KeyCode::Backspace => "backspace".into(),
            KeyCode::Tab => "tab".into(),
            KeyCode::Up => "up".into(),
            KeyCode::Down => "down".into(),
            KeyCode::Left => "left".into(),
            KeyCode::Right => "right".into(),
            KeyCode::Home => "home".into(),
            KeyCode::End => "end".into(),
            KeyCode::PageUp => "pageup".into(),
            KeyCode::PageDown => "pagedown".into(),
            KeyCode::F(n) => format!("f{}", n),
            _ => "?".into(),
        }),
        Event::Mouse(m) => match m.kind {
            MouseEventKind::Down(btn) => Ok(format!("m{}", match btn {
                crossterm::event::MouseButton::Left => "left",
                crossterm::event::MouseButton::Right => "right",
                crossterm::event::MouseButton::Middle => "mid",
            })),
            _ => Ok("".into()),
        },
        Event::Resize(w, h) => Ok(format!("rs_{}_{}", w, h)),
        _ => Ok("".into()),
    }
}
