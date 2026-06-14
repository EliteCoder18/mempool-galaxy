use std::sync::{Arc, RwLock};
use mg_core::MempoolState;
use mg_renderer::Renderer;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(RwLock::new(MempoolState::new()));
    let renderer = Renderer;
    Renderer::init()?;

    for _ in 0..60 {
        {
            let mut w_state = state.write().unwrap();
            mg_physics::update_physics(&mut w_state, 0.1);
        }

        {
            let r_state = state.read().unwrap();
            renderer.draw_particles(&r_state.particles)?;
        }
        tokio::time::sleep(std::time::Duration::from_millis(16)).await;
    }
    Renderer::cleanup()?;
    println!("Renderer test Complete.");
    Ok(())
}
