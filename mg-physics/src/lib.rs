use mg_core::{MempoolState, Particle};

pub fn update_physics(state: &mut MempoolState, dt: f32) {
    let center = (0.0, 0.0);
    let gravity_constant = 0.5;

    for p in state.particles.iter_mut() {
        let dx = center.0 - p.pos.0;
        let dy = center.1 - p.pos.1;
        let dist = (dx * dx + dy * dy).sqrt().max(1.0);

        let force = (p.fee_rate as f32 * gravity_constant) / dist;
        p.velocity.0 = (dx / dist) * force * dt;
        p.velocity.1 = (dy / dist) * force * dt;

        p.pos.0 += p.velocity.0;
        p.pos.1 += p.velocity.1;
    }
}
