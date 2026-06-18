use mg_core::{MempoolState, Particle};

pub fn update_physics(state: &mut MempoolState, dt: f32) {
    let center = (state.screen_size.0 / 2.0, state.screen_size.1 / 2.0);
    let gravity_constant = 0.5;
    let friction = 0.95;

    state.particles.retain(|p|{
        let dx = center.0-p.pos.0;
        let dy = center.1-p.pos.1;
        let dist = (dx*dx+dy*dy).sqrt();
        dist>2.0
    });

    for p in state.particles.iter_mut() {
        let dx = center.0 - p.pos.0;
        let dy = center.1 - p.pos.1;
        let dist = (dx * dx + dy * dy).sqrt().max(1.0);
        //inward Gravity
        let force = (p.fee_rate as f32 * gravity_constant) / dist;
        p.velocity.0 += (dx / dist) * force * dt;
        p.velocity.1 += (dy / dist) * force * dt;
        //Tangential Swirl
        let swirl_force = force * 0.5;
        p.velocity.0 += (-dy / dist) * swirl_force * dt;
        p.velocity.1 += (-dx / dist) * swirl_force * dt;

        p.velocity.0 *= friction;
        p.velocity.1 *= friction;

        p.pos.0 += p.velocity.0;
        p.pos.1 += p.velocity.1;
    }
}
