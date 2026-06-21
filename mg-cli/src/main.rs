use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use mg_core::MempoolState;
use mg_renderer::Renderer;
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(RwLock::new(MempoolState::new()));
    let renderer = Renderer;

    Renderer::init()?;

    {
        let (cols, rows) = crossterm::terminal::size()?;
        let mut w = state.write().unwrap();
        w.screen_size = (cols as f32, rows as f32);
    }

    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        mg_pipeline::run_websocket(state_clone).await;
    });

    loop {
        // check for 'q' or Ctrl-C without blocking
        if event::poll(std::time::Duration::from_millis(0))? {
            match event::read()?{
            Event::Key(key)=>{
                let quit = key.code == KeyCode::Char('q')
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL));
                if quit {
                    break;
                }
            }
            Event::Resize(cols,rows)=>{
                let mut w = state.write().unwrap();
                w.screen_size = (cols as f32, rows as f32);
            }
            _=>{}
        }}

        {
            let mut w = state.write().unwrap();
            mg_physics::update_physics(&mut w, 0.1);
        }

        {
            let r = state.read().unwrap();
            renderer
                .draw_particles(&r)
                .expect("draw failed");
        }

        tokio::time::sleep(std::time::Duration::from_millis(16)).await;
    }

    Renderer::cleanup()?;
    Ok(())
}
