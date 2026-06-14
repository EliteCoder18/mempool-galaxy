use mg_renderer::Renderer;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Renderer::init()?;
    for _ in 0..60 {
        {
            let mut state = state.write().unwrap();
            mg - physics::update_physics(&mut state, 0.1);
        }

        {
            let state = state.read().unwrap();
            Renderer.draw_particles(&state.particles)?;
        }

        Renderer::draw_frame()?;
        tokio::time::sleep(std::time::Duration::from_millis(16)).await;
    }
    Renderer::cleanup()?;
    println!("Renderer test Complete.");
    Ok(())
}
