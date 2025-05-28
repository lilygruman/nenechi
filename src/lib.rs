use std::f32::consts::{FRAC_PI_2, FRAC_PI_6};

use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    asset::Assets,
    color::Color,
    core_pipeline::core_3d::Camera3d,
    ecs::{
        component::Component,
        error::Result,
        query::{Added, Changed, With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    input::gamepad::{Gamepad, GamepadAxis, GamepadButton},
    math::{
        Dir3, FloatExt, Quat, Vec3,
        primitives::{Cuboid, Sphere},
    },
    pbr::{DirectionalLight, MeshMaterial3d, StandardMaterial},
    render::{
        camera::{Camera, PerspectiveProjection, Projection},
        mesh::{Mesh, Mesh3d},
        view::Visibility,
    },
    time::Time,
    transform::components::{GlobalTransform, Transform},
};

use avian3d::{
    PhysicsPlugins,
    collision::collider::Collider,
    dynamics::rigid_body::{
        AngularVelocity, CoefficientCombine, LinearVelocity, Restitution, RigidBody,
    },
};

pub fn plugin(app: &mut App) {
    app.add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, maze_attitude)
        .add_systems(Update, spawn_ball)
        .add_systems(Update, reset_ball)
        .add_systems(Update, adjust_camera);
}

#[derive(Component)]
struct Ball;

const THICKNESS: f32 = 0.1;
const WIDTH: f32 = 2.;
const WALL_Y: f32 = WIDTH / 2.;
const WALL_RESTITUTION: f32 = 1.;

#[derive(Component)]
struct BallStart;

#[derive(Component)]
struct Maze;

const BALL_RADIUS: f32 = 0.8;
const BALL_START_ELEVATION: f32 = (THICKNESS / 2.) + BALL_RADIUS;
fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) -> Result {
    #[expect(unused)]
    const FULL_Z_ROW: &str = ".-.-.-.-.-.-.";
    #[expect(unused)]
    const FULL_X_ROW: &str = "| | | | | | |";
    #[expect(unused)]
    const EMPTY_X_ROW: &str = "|           |";

    const SAMPLE_MAZE: &[&str] = &[
        ".-.-.-.-.-.-.",
        "|x  |       |",
        ".-. .-.-. .-.",
        "|           |",
        ". .-. .-.-. .",
        "|   | |     |",
        ".-.-. .-.-.-.",
        "| |   |     |",
        ". . .-.-. . .",
        "|   |     | |",
        ". .-. .-.-. .",
        "|       |   |",
        ".-.-.-.-. .-.",
    ];

    debug_assert_eq!(
        SAMPLE_MAZE.len() % 2,
        1,
        "Maze should have odd number of rows because it should have Z walls on both ends"
    );
    let z_offset = (SAMPLE_MAZE.len() / 2) as f32;

    let maze_rotation = Quat::from_axis_angle(Vec3::X, FRAC_PI_2);
    let maze_transform = Transform::from_rotation(maze_rotation);

    let maze_material =
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgb(0.1, 0.1, 0.1))));

    let x_wall_collider = Collider::cuboid(THICKNESS, WIDTH, WIDTH);
    let x_wall_mesh = Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(THICKNESS, WIDTH, WIDTH))));
    let z_wall_collider = Collider::cuboid(WIDTH, WIDTH, THICKNESS);
    let z_wall_mesh = Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(WIDTH, WIDTH, THICKNESS))));
    let wall_restitution = Restitution::new(WALL_RESTITUTION);

    commands
        .spawn((
            RigidBody::Kinematic,
            Maze,
            maze_transform,
            Visibility::default(),
        ))
        .with_children(|spawner| {
            for (zi, row) in SAMPLE_MAZE.iter().enumerate() {
                let zw = zi as f32 - z_offset;
                let x_offset = (row.len() / 2) as f32;

                for (xi, c) in row.chars().enumerate() {
                    let xw = xi as f32 - x_offset;
                    match c {
                        '|' => {
                            spawner.spawn((
                                x_wall_collider.clone(),
                                wall_restitution,
                                Transform::from_xyz(xw, WALL_Y, zw),
                                maze_material.clone(),
                                x_wall_mesh.clone(),
                            ));
                        }
                        'x' => {
                            spawner.spawn((
                                Transform::from_xyz(xw, BALL_START_ELEVATION, zw),
                                BallStart,
                            ));
                        }
                        '-' => {
                            spawner.spawn((
                                z_wall_collider.clone(),
                                wall_restitution,
                                Transform::from_xyz(xw, WALL_Y, zw),
                                maze_material.clone(),
                                z_wall_mesh.clone(),
                            ));
                        }
                        _ => (),
                    }
                }
            }
            let floor_width = SAMPLE_MAZE.len() as f32;
            spawner.spawn((
                Collider::cuboid(floor_width, THICKNESS, floor_width),
                Restitution::new(0.).with_combine_rule(CoefficientCombine::Min),
                maze_material,
                Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(
                    floor_width,
                    THICKNESS,
                    floor_width,
                )))),
            ));
            spawner.spawn((
                Collider::cuboid(floor_width, THICKNESS, floor_width),
                Restitution::new(0.).with_combine_rule(CoefficientCombine::Min),
                Transform::from_translation(Vec3::Y * WIDTH),
            ));
        });

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(maze_transform.up() * 30.).looking_at(Vec3::ZERO, Dir3::Y),
    ));

    // Lighting
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, FRAC_PI_6)),
    ));

    Ok(())
}

fn spawn_ball(
    mut commands: Commands,
    start: Query<&GlobalTransform, Added<BallStart>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) -> Result {
    for start in start {
        // Ball
        commands.spawn((
            Ball,
            RigidBody::Dynamic,
            Collider::sphere(BALL_RADIUS),
            Restitution::new(0.1),
            start.compute_transform(),
            MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgb(0.4, 0.1, 0.1)))),
            Mesh3d(meshes.add(Sphere::new(BALL_RADIUS))),
        ));
    }
    Ok(())
}

const ANALOG_THRESHOLD: f32 = 0.1;

fn adjust_camera(
    gamepads: Query<&Gamepad>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera>>,
    time: Res<Time>,
) -> Result {
    for gp in gamepads {
        let (mut transform, projection) = camera.single_mut()?;
        let analog = gp.analog();

        // Dolly translation
        const CAMERA_CLOSEST: f32 = 10.;
        const CAMERA_FURTHEST: f32 = 100.;
        const MAX_DOLLY_SPEED_PER_SEC: f32 = 50.;
        let dolly = analog.get(GamepadButton::RightTrigger2).unwrap_or_default()
            - analog.get(GamepadButton::LeftTrigger2).unwrap_or_default();
        if dolly.abs() > ANALOG_THRESHOLD {
            let forward = transform.forward();
            let distance = transform.translation.length();
            if is_in_bounds(dolly, distance, CAMERA_FURTHEST, CAMERA_CLOSEST) {
                transform.translation +=
                    forward * dolly * time.delta_secs() * MAX_DOLLY_SPEED_PER_SEC
            }
        }

        if let Projection::Perspective(PerspectiveProjection { fov, .. }) = projection.into_inner()
        {
            const SCALE_PER_SECOND: f32 = 1.3;
            let left = gp.pressed(GamepadButton::LeftTrigger);
            let right = gp.pressed(GamepadButton::RightTrigger);
            if left && right {
                // Press both to reset
                *fov = 1.;
            } else if right {
                *fov /= SCALE_PER_SECOND.powf(time.delta_secs());
            } else if left {
                *fov *= SCALE_PER_SECOND.powf(time.delta_secs());
            }
        }
    }
    Ok(())
}

fn maze_attitude(
    gamepads: Query<&Gamepad, Changed<Gamepad>>,
    mut maze: Query<(&mut AngularVelocity, &Transform), With<Maze>>,
) -> Result {
    const MAX_ATTITUDE_DELTA_RAD_PER_SEC: f32 = FRAC_PI_2;
    for gp in gamepads {
        if let Some(rotation) = gp.analog().get(GamepadAxis::LeftStickX) {
            let (mut angular, transform) = maze.single_mut()?;
            *angular = AngularVelocity(if rotation.abs() > ANALOG_THRESHOLD {
                -transform.up() * rotation * MAX_ATTITUDE_DELTA_RAD_PER_SEC
            } else {
                Vec3::ZERO
            });
        }
    }
    Ok(())
}

fn is_in_bounds(delta: f32, current: f32, min: f32, max: f32) -> bool {
    (delta.is_sign_positive() && f32::inverse_lerp(max, min, current).is_sign_positive())
        || (delta.is_sign_negative() && f32::inverse_lerp(min, max, current).is_sign_positive())
}

fn reset_ball(
    gamepads: Query<&Gamepad, Changed<Gamepad>>,
    mut ball: Query<(&mut Transform, &mut LinearVelocity), With<Ball>>,
    start: Query<&GlobalTransform, (With<BallStart>, Without<Ball>)>,
) -> Result {
    for gp in gamepads {
        if gp.just_pressed(GamepadButton::Start) {
            let (mut transform, mut velocity) = ball.single_mut()?;
            transform.translation = start.single()?.translation();
            *velocity = LinearVelocity::ZERO;
        }
    }
    Ok(())
}
