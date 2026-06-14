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
    pub fn draw_particles(&self, particles: &[mg_core::Particle])->Result<()>{
        let mut out = stdout();
        out.execute(terminal::Clear(terminal::ClearType::All))?;
        
        for p in particles{
            let x = p.pos.0 as u16;
            let y = p.pos.1 as u16;
            out.execute(cursor::MoveTo(x,y));
            out.execute(Print("*"));
        }
        out.flush()?;
        Ok(())
    }
}
