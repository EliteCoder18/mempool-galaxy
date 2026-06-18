use crossterm::{
    ExecutableCommand, cursor,
    style::{Color, Print, SetForegroundColor, ResetColor},
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
        out.execute(Print("●"))?;
        out.flush()?;
        Ok(())
    }

    pub fn draw_particles(
        &self,
        particles: &[mg_core::Particle],
        status: &mg_core::ConnectionStatus,
    ) -> Result<()> {
        let (cols, rows) = terminal::size()?;
        let mut out = stdout();
        out.execute(terminal::Clear(terminal::ClearType::All))?;

        if particles.is_empty() {
            let (msg, color) = match status {
                mg_core::ConnectionStatus::Connecting => ("connecting to mempool...", Color::DarkYellow),
                mg_core::ConnectionStatus::Failed    => ("connection failed — check network", Color::Red),
                mg_core::ConnectionStatus::Connected => ("waiting for transactions...", Color::DarkGreen),
            };
            let x = cols.saturating_sub(msg.len() as u16) / 2;
            let y = rows / 2;
            out.execute(cursor::MoveTo(x, y))?;
            out.execute(SetForegroundColor(color))?;
            out.execute(Print(msg))?;
            out.execute(ResetColor)?;
            out.flush()?;
            return Ok(());
        }

        for p in particles {
            let x = p.pos.0.clamp(0.0, (cols - 1) as f32) as u16;
            let y = p.pos.1.clamp(0.0, (rows - 1) as f32) as u16;

            let (symbol, color) = if p.fee_rate > 30.0 {
                ("O", Color::Red)
            } else if p.fee_rate > 10.0 {
                ("o", Color::Yellow)
            } else {
                (".", Color::DarkGrey)
            };
            out.execute(cursor::MoveTo(x, y))?;
            out.execute(SetForegroundColor(color))?;
            out.execute(Print(symbol))?;
        }

        out.execute(ResetColor)?;
        out.flush()?;
        Ok(())
    }
}
