use bevy::{
    prelude::{shape::Circle, *},
    render::camera::ScalingMode,
    sprite::Mesh2dHandle,
};

#[derive(Debug, Clone, Default)]
pub struct ParticleLifePlugin {
    pub initial_particles: Vec<Particle>,
    pub colors: Vec<Color>,
    pub color_attractions: ColorAttractions,
    pub attraction_radius: AttractionRadius,
}

impl Plugin for ParticleLifePlugin {
    fn build(&self, app: &mut App) {
        app.world
            .spawn_batch(self.initial_particles.iter().copied());

        app.insert_resource(self.color_attractions.clone())
            .insert_resource(self.attraction_radius);

        app.add_startup_system(setup_camera);

        app.insert_resource(ParticleColors(self.colors.clone()))
            .init_resource::<ColorHandles>()
            .add_startup_system(setup_color_materials);

        app.init_resource::<MeshHandle>()
            .add_startup_system(setup_mesh);

        app.add_startup_system(
            setup_mesh_and_color
                .after(setup_color_materials)
                .after(setup_mesh),
        );

        app.add_system(update_position)
            .add_system(update_velocity)
            .add_system(update_transform)
            .add_system(update_material);
    }
}

#[derive(Debug, Clone, Copy, Bundle)]
pub struct Particle {
    pub position: Position,
    pub velocity: Velocity,
    pub color: ColorId,
}

#[derive(Debug, Clone, Copy, Default, Component)]
pub struct Position(pub Vec2);

#[derive(Debug, Clone, Copy, Default, Component)]
pub struct Velocity(pub Vec2);

#[derive(Debug, Clone, Copy, Component)]
pub struct ColorId(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct Attraction(pub f32);

#[derive(Debug, Clone, Copy, Default, Resource)]
pub struct AttractionRadius {
    pub rmin: f32,
    pub rmax: f32,
}

/// Particles with the `i`th color are attracted by particles with the `j`th color by
/// `self.0[i][j]`.
#[derive(Debug, Clone, Resource, Default)]
pub struct ColorAttractions(pub Vec<Vec<Attraction>>);

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            left: -1.0,
            right: 1.0,
            bottom: -1.0,
            top: 1.0,
            scaling_mode: ScalingMode::None,
            ..Default::default()
        },
        ..Default::default()
    });
}

#[derive(Debug, Clone, Default, Resource)]
struct ParticleColors(Vec<Color>);

#[derive(Debug, Clone, Default, Resource)]
struct ColorHandles(Vec<Handle<ColorMaterial>>);

fn setup_color_materials(
    colors: Res<ParticleColors>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut handles: ResMut<ColorHandles>,
) {
    for &color in &colors.0 {
        handles.0.push(materials.add(ColorMaterial::from(color)));
    }
}

#[derive(Debug, Clone, Default, Resource)]
struct MeshHandle(Mesh2dHandle);

fn setup_mesh(mut meshes: ResMut<Assets<Mesh>>, mut handle: ResMut<MeshHandle>) {
    let circle = Circle {
        radius: 1.0,
        vertices: 8,
    };
    handle.0 = meshes.add(circle.into()).into();
}

fn setup_mesh_and_color(
    mut commands: Commands,
    mesh: Res<MeshHandle>,
    materials: Res<ColorHandles>,
    query: Query<(&ColorId, Entity)>,
) {
    for (&color, entity) in query.iter() {
        commands.entity(entity).insert(ColorMesh2dBundle {
            mesh: mesh.0.clone(),
            material: materials.0[color.0].clone(),
            ..Default::default()
        });
    }
}

fn update_position(time: Res<Time>, mut query: Query<(&mut Position, &Velocity)>) {
    let delta = time.delta_seconds();
    for (mut position, velocity) in &mut query {
        position.0 += delta * velocity.0;
        position.0.x = position.0.x - 2.0 * f32::round(position.0.x / 2.0);
        position.0.y = position.0.y - 2.0 * f32::round(position.0.y / 2.0);
    }
}

fn update_velocity(
    time: Res<Time>,
    attraction_radius: Res<AttractionRadius>,
    color_attractions: Res<ColorAttractions>,
    mut query: Query<(&mut Velocity, &Position, &ColorId, Entity)>,
) {
    let delta = time.delta_seconds();
    let AttractionRadius { rmin, rmax } = *attraction_radius;
    let color_attractions = &*color_attractions;

    let mut particle_pairs = query.iter_combinations_mut();
    while let Some([query_a, query_b]) = particle_pairs.fetch_next() {
        let (mut velocity_a, position_a, &color_a, entity_a) = query_a;
        let (mut velocity_b, position_b, &color_b, entity_b) = query_b;

        // Don't attract/repell an entity from itself
        if entity_a == entity_b {
            continue;
        }

        let distance = toroidal_distance(position_a, position_b).max(0.01);
        let (attraction_a_by_b, attraction_b_by_a) =
            attraction_factor(distance, color_a, color_b, color_attractions, rmin, rmax);

        let a_to_b_direction = toroidal_difference(position_a, position_b)
            .try_normalize()
            .unwrap_or(Vec2 { x: 1.0, y: 0.0 });

        velocity_a.0 += delta * attraction_a_by_b.0 * a_to_b_direction;
        velocity_b.0 -= delta * attraction_b_by_a.0 * a_to_b_direction;
    }
}

fn toroidal_distance(position_a: &Position, position_b: &Position) -> f32 {
    let diff = toroidal_difference(position_a, position_b);
    diff.length()
}

/// A to B
fn toroidal_difference(base: &Position, tip: &Position) -> Vec2 {
    let mut dir = tip.0 - base.0;
    if dir.x.abs() > 1.0 {
        dir.x = dir.x - 2.0;
    }
    if dir.y.abs() > 1.0 {
        dir.y = dir.y - 2.0;
    }
    dir
}

/// Calculates how much a particle A is attracted to a particle B. Negative values represent
/// equivalent repulsion.
///
/// Given the distance `d` between the two particles, this attraction factor `F` is calculated as
/// follows:
///
/// - If `d <= rmin`, `F < 0` to make the particles repell. `F = d / rmin - 1`: at `d = 0`, the
/// particles repell with a force of `1` and at `d = rmin`, their velocity stays fixed.
///
/// - If `rmin <= d <= rmax`, the attraction factor is calculated using the appropriate entry in
/// `color_attractions`: `F = 0` at `d = rmin` at `d = rmax`, and peaks halfway.
///
/// - If `d > rmax`, `F = 0`.
///
/// The first return value indicates how particle A is attracted by particle B, the second the
/// opposite.
fn attraction_factor(
    distance: f32,
    color_a: ColorId,
    color_b: ColorId,
    color_attractions: &ColorAttractions,
    rmin: f32,
    rmax: f32,
) -> (Attraction, Attraction) {
    if distance <= rmin {
        let attraction_factor = distance / rmin - 1.0;
        (Attraction(attraction_factor), Attraction(attraction_factor))
    } else if distance <= rmax {
        let peak_attraction_a_by_b = color_attractions.0[color_a.0][color_b.0];
        let peak_attraction_b_by_a = color_attractions.0[color_b.0][color_a.0];

        let peak_distance = (rmin + rmax) / 2.0;
        let distance_scalar = (distance - peak_distance).abs();
        (
            Attraction(distance_scalar * peak_attraction_a_by_b.0),
            Attraction(distance_scalar * peak_attraction_b_by_a.0),
        )
    } else {
        (Attraction(0.0), Attraction(0.0))
    }
}

fn update_transform(
    mut commands: Commands,
    mut query: Query<(Option<&mut Transform>, &Position, Entity), Changed<Position>>,
) {
    for (transform, position, entity) in query.iter_mut() {
        let new_transform = Transform::from_translation(Vec3::new(position.0.x, position.0.y, 0.0))
            .with_scale(Vec3::splat(0.01));
        if let Some(mut transform) = transform {
            *transform = new_transform;
        } else {
            commands.entity(entity).insert(new_transform);
        }
    }
}

fn update_material(
    mut commands: Commands,
    handles: Res<ColorHandles>,
    mut query: Query<(Option<&mut Handle<ColorMaterial>>, &ColorId, Entity), Changed<ColorId>>,
) {
    for (material, color, entity) in query.iter_mut() {
        let new_material = handles.0[color.0].clone();
        if let Some(mut material) = material {
            *material = new_material;
        } else {
            commands.entity(entity).insert(new_material);
        }
    }
}
