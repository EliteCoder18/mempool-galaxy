use crossterm::{
    ExecutableCommand, cursor,
    style::{Color, Print, SetForegroundColor},
    terminal,
};
use std::io::{Result, Write, stdout};
pub struct Renderer;
impl Renderer {
    pub fn init() -> Result<()> {
        stdout().execute(terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        stdout().execute(cursor::Hide)?;
        Ok(())
    }
    pub fn cleanup() -> Result<()> {
        stdout().execute(cursor::Show)?;
        terminal::disable_raw_mode()?;
        stdout().execute(terminal::LeaveAlternateScreen)?;
        Ok(())
    }
    pub fn draw_frame() -> Result<()> {
        let (cols, rows) = terminal::size()?;
        let mut out = stdout();
        out.execute(terminal::Clear(terminal::ClearType::All))?;
        out.execute(cursor::MoveTo(cols / 2, rows / 2))?;
        out.execute(SetForegroundColor(Color::Yellow))?;
        out.execute(Print("*"))?;
        out.flush()?;
        Ok(())
    }
}
