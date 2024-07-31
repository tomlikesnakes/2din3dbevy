use bevy::math::prelude::*;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

const SPRITE_SIZE: f32 = 192.0;
const SPRITE_COLS: usize = 5;
const SPRITE_ROWS: usize = 5;
const TOTAL_FRAMES: usize = SPRITE_COLS * SPRITE_ROWS;

#[derive(Component)]
struct WaterSkill {
    animation_timer: Timer,
    lifetime: Timer,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct MainCamera;

#[derive(Resource)]
struct SkillSpriteSheet {
    texture: Handle<Image>,
    atlas_layout: Handle<TextureAtlasLayout>,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct SkillMaterial {
    #[uniform(0)]
    frame: Vec4,
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
}

impl Material for SkillMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/skill_material.wgsl".into()
    }
}

#[derive(Resource)]
struct SkillMaterialHandle(Handle<SkillMaterial>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spawn_skill,
                animate_skills,
                despawn_skills,
                camera_controls,
                player_movement,
                debug_skill_info,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut skill_materials: ResMut<Assets<SkillMaterial>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Set up the camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MainCamera,
    ));

    // Add a light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Create a plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Plane3d::new(Vec3::Y, Vec2::splat(10.0)))),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    // Create the player
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0))),
            material: materials.add(Color::rgb(0.8, 0.2, 0.3)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Player,
    ));

    // Create an enemy
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0))),
            material: materials.add(Color::rgb(0.2, 0.3, 0.8)),
            transform: Transform::from_xyz(5.0, 0.5, 5.0),
            ..default()
        },
        Enemy,
    ));

    // Set up the skill sprite sheet
    let texture_handle: Handle<Image> = asset_server.load("water.png");
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(SPRITE_SIZE as u32, SPRITE_SIZE as u32),
        SPRITE_COLS as u32,
        SPRITE_ROWS as u32,
        None,
        None,
    );
    let atlas_layout_handle = texture_atlas_layouts.add(layout);

    commands.insert_resource(SkillSpriteSheet {
        texture: texture_handle.clone(),
        atlas_layout: atlas_layout_handle,
    });

    // Create the skill material
    let skill_material = skill_materials.add(SkillMaterial {
        frame: Vec4::new(0.0, 0.0, 1.0 / SPRITE_COLS as f32, 1.0 / SPRITE_ROWS as f32),
        texture: texture_handle,
    });

    commands.insert_resource(SkillMaterialHandle(skill_material));
}

fn spawn_skill(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    skill_material: Res<SkillMaterialHandle>,
    query: Query<&Transform, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(player_transform) = query.get_single() {
            let spawn_position = player_transform.translation + Vec3::new(1.0, 1.0, 0.0);

            let quad_handle = meshes.add(Mesh::from(Rectangle::new(1.0, 1.0)));

            commands.spawn((
                MaterialMeshBundle {
                    mesh: quad_handle,
                    material: skill_material.0.clone(),
                    transform: Transform::from_translation(spawn_position)
                        .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2))
                        .with_scale(Vec3::splat(0.5)),
                    ..default()
                },
                WaterSkill {
                    animation_timer: Timer::from_seconds(0.05, TimerMode::Repeating),
                    lifetime: Timer::from_seconds(3.0, TimerMode::Once),
                },
            ));
            println!("Skill spawned at {:?}", spawn_position);
        }
    }
}

fn animate_skills(
    time: Res<Time>,
    mut query: Query<&mut WaterSkill>,
    mut skill_materials: ResMut<Assets<SkillMaterial>>,
    skill_material_handle: Res<SkillMaterialHandle>,
) {
    if let Some(material) = skill_materials.get_mut(&skill_material_handle.0) {
        for mut skill in query.iter_mut() {
            skill.animation_timer.tick(time.delta());
            if skill.animation_timer.just_finished() {
                let frame_index = (material.frame.x * SPRITE_COLS as f32) as usize;
                let next_frame = (frame_index + 1) % TOTAL_FRAMES;
                if next_frame == 0 {
                    material.frame.x = 1.0 / SPRITE_COLS as f32;
                    material.frame.y = 0.0;
                } else {
                    material.frame.x = (next_frame % SPRITE_COLS) as f32 / SPRITE_COLS as f32;
                    material.frame.y = (next_frame / SPRITE_COLS) as f32 / SPRITE_ROWS as f32;
                }
            }
        }
    }
}

fn despawn_skills(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut WaterSkill)>,
) {
    for (entity, mut skill) in query.iter_mut() {
        skill.lifetime.tick(time.delta());
        if skill.lifetime.finished() {
            commands.entity(entity).despawn();
            println!("Skill despawned");
        }
    }
}

fn camera_controls(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<MainCamera>>,
) {
    if let Ok(mut transform) = query.get_single_mut() {
        let mut movement = Vec3::ZERO;
        let mut rotation = Vec3::ZERO;
        let speed = 5.0;
        let rotate_speed = 1.0;

        if keyboard_input.pressed(KeyCode::KeyW) {
            movement.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            movement.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            movement.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            movement.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyQ) {
            movement.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            movement.y += 1.0;
        }

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            rotation.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            rotation.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            rotation.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            rotation.x -= 1.0;
        }

        transform.translation += movement * speed * time.delta_seconds();
        transform.rotate_x(rotation.x * rotate_speed * time.delta_seconds());
        transform.rotate_y(rotation.y * rotate_speed * time.delta_seconds());
    }
}

fn player_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    if let Ok(mut transform) = query.get_single_mut() {
        let mut movement = Vec3::ZERO;
        let speed = 3.0;

        if keyboard_input.pressed(KeyCode::KeyI) {
            movement.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyK) {
            movement.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyJ) {
            movement.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyL) {
            movement.x += 1.0;
        }

        transform.translation += movement * speed * time.delta_seconds();
    }
}

fn debug_skill_info(query: Query<(&Transform, &TextureAtlas), With<WaterSkill>>) {
    for (transform, atlas) in query.iter() {
        println!(
            "Skill position: {:?}, Current frame: {}",
            transform.translation, atlas.index
        );
    }
}
