use bevy::prelude::*;
use particle_life::*;
use rand::Rng;

use std::iter;

fn main() {
    let mut app = App::new();

    let window = WindowDescriptor {
        title: "Particle Life".to_string(),
        width: 750.0,
        height: 750.0,
        resizable: false,
        ..Default::default()
    };
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        window,
        ..Default::default()
    }));

    app.add_plugin(init_particle_life());

    app.run();
}

fn init_particle_life() -> ParticleLifePlugin {
    let colors = vec![
        Color::RED,
        Color::GREEN,
        Color::BLUE,
        Color::YELLOW,
        Color::PINK,
        Color::CYAN,
    ];

    let mut rng = rand::thread_rng();

    // let initial_particles = iter::repeat_with(|| Particle {
    //     position: Position(Vec2::new(
    //         rng.gen_range(-1.0..1.0),
    //         rng.gen_range(-1.0..1.0),
    //     )),
    //     velocity: Default::default(),
    //     color: ColorId(rng.gen_range(0..6)),
    // })
    // .take(1000)
    // .collect();

    const PARTICLES_PER_COLOR: usize = 200;

    let red_particles: Vec<_> = iter::repeat_with(|| Particle {
        position: Position(Vec2::new(
            rng.gen_range(-1.0..-0.75),
            rng.gen_range(-0.25..0.0),
        )),
        velocity: Default::default(),
        color: ColorId(0),
    })
    .take(PARTICLES_PER_COLOR)
    .collect();

    let green_particles: Vec<_> = iter::repeat_with(|| Particle {
        position: Position(Vec2::new(
            rng.gen_range(-0.75..-0.5),
            rng.gen_range(-0.25..0.0),
        )),
        velocity: Default::default(),
        color: ColorId(1),
    })
    .take(PARTICLES_PER_COLOR)
    .collect();

    let blue_particles: Vec<_> = iter::repeat_with(|| Particle {
        position: Position(Vec2::new(
            rng.gen_range(-0.5..-0.25),
            rng.gen_range(-0.25..0.0),
        )),
        velocity: Default::default(),
        color: ColorId(2),
    })
    .take(PARTICLES_PER_COLOR)
    .collect();

    let yellow_particles: Vec<_> = iter::repeat_with(|| Particle {
        position: Position(Vec2::new(
            rng.gen_range(-0.25..0.0),
            rng.gen_range(-0.25..0.0),
        )),
        velocity: Default::default(),
        color: ColorId(3),
    })
    .take(PARTICLES_PER_COLOR)
    .collect();

    let pink_particles: Vec<_> = iter::repeat_with(|| Particle {
        position: Position(Vec2::new(
            rng.gen_range(0.0..0.25),
            rng.gen_range(-0.25..0.0),
        )),
        velocity: Default::default(),
        color: ColorId(4),
    })
    .take(PARTICLES_PER_COLOR)
    .collect();

    let cyan_particles = iter::repeat_with(|| Particle {
        position: Position(Vec2::new(
            rng.gen_range(0.25..0.5),
            rng.gen_range(-0.25..0.0),
        )),
        velocity: Default::default(),
        color: ColorId(5),
    })
    .take(PARTICLES_PER_COLOR);

    let initial_particles = red_particles
        .into_iter()
        .chain(green_particles)
        .chain(blue_particles)
        .chain(yellow_particles)
        .chain(pink_particles)
        .chain(cyan_particles)
        .collect();

    const SELF_ATTRACTION: f32 = 0.3;
    const PREVIOUS_ATTRACTION: f32 = -0.001;
    const NEXT_ATTRACTION: f32 = 0.002;
    const OTHER_ATTRACTION: f32 = -0.05;

    let color_attractions = vec![
        vec![
            Attraction(SELF_ATTRACTION),
            Attraction(NEXT_ATTRACTION),
            Attraction(OTHER_ATTRACTION),
            Attraction(OTHER_ATTRACTION),
            Attraction(OTHER_ATTRACTION),
            Attraction(PREVIOUS_ATTRACTION),
        ],
        vec![
            Attraction(PREVIOUS_ATTRACTION),
            Attraction(SELF_ATTRACTION),
            Attraction(NEXT_ATTRACTION),
            Attraction(OTHER_ATTRACTION),
            Attraction(OTHER_ATTRACTION),
            Attraction(OTHER_ATTRACTION),
        ],
        vec![
            Attraction(OTHER_ATTRACTION),
            Attraction(PREVIOUS_ATTRACTION),
            Attraction(SELF_ATTRACTION),
            Attraction(NEXT_ATTRACTION),
            Attraction(OTHER_ATTRACTION),
            Attraction(OTHER_ATTRACTION),
        ],
        vec![
            Attraction(OTHER_ATTRACTION),
            Attraction(OTHER_ATTRACTION),
            Attraction(PREVIOUS_ATTRACTION),
            Attraction(SELF_ATTRACTION),
            Attraction(NEXT_ATTRACTION),
            Attraction(OTHER_ATTRACTION),
        ],
        vec![
            Attraction(OTHER_ATTRACTION),
            Attraction(OTHER_ATTRACTION),
            Attraction(OTHER_ATTRACTION),
            Attraction(PREVIOUS_ATTRACTION),
            Attraction(SELF_ATTRACTION),
            Attraction(NEXT_ATTRACTION),
        ],
        vec![
            Attraction(NEXT_ATTRACTION),
            Attraction(OTHER_ATTRACTION),
            Attraction(OTHER_ATTRACTION),
            Attraction(OTHER_ATTRACTION),
            Attraction(PREVIOUS_ATTRACTION),
            Attraction(SELF_ATTRACTION),
        ],
    ];

    ParticleLifePlugin {
        initial_particles,
        colors,
        color_attractions: ColorAttractions(color_attractions),
        attraction_radius: AttractionRadius {
            rmin: 0.04,
            rmax: 0.4,
        },
    }
}
