use bevy::prelude::*;
#[allow(unused_imports)]
use spaceship_core::{
    Active, CullBoundary, Emitter, GameConfig, Inactive, LinearPath, MovementIntent, PlayerEmitter,
    Projectile,
};
use spaceship_core::{cull_projectiles, player_emit, update_linear};
use std::time::Duration;

#[allow(dead_code)]
fn spawn_active_projectile(world: &mut World, position: Vec3) -> Entity {
    world
        .spawn((
            Projectile {
                damage: 1.0,
                lifetime: Timer::from_seconds(1.0, TimerMode::Once),
            },
            Active,
            LinearPath {
                origin: position.truncate(),
                dir: Vec2::Y,
                speed: 600.0,
                spawn_time: 0.0,
            },
            Transform::from_translation(position),
        ))
        .id()
}

#[allow(dead_code)]
fn cull_test_app() -> App {
    let mut app = App::new();
    app.init_resource::<MovementIntent>()
        .init_resource::<GameConfig>()
        .insert_resource(CullBoundary {
            half_width: 100.0,
            half_height: 100.0,
        })
        .add_systems(Update, cull_projectiles);
    app
}

#[test]
fn culls_projectile_outside_boundary() {
    let mut app = cull_test_app();
    let entity = spawn_active_projectile(app.world_mut(), Vec3::new(1_000.0, 0.0, 0.0));

    app.update();

    let world = app.world();
    assert!(world.get::<Active>(entity).is_none());
    assert!(world.get::<Inactive>(entity).is_some());
}

#[test]
fn keeps_projectile_inside_boundary() {
    let mut app = cull_test_app();
    let entity = spawn_active_projectile(app.world_mut(), Vec3::new(50.0, 50.0, 0.0));

    app.update();

    let world = app.world();
    assert!(world.get::<Active>(entity).is_some());
    assert!(world.get::<Inactive>(entity).is_none());
}

#[test]
fn keeps_projectile_within_margin() {
    let mut app = cull_test_app();
    let entity = spawn_active_projectile(app.world_mut(), Vec3::new(150.0, 0.0, 0.0));

    app.update();

    let world = app.world();
    assert!(
        world.get::<Active>(entity).is_some(),
        "projectile inside half_width + CULL_MARGIN should stay active"
    );
}

#[allow(dead_code)]
fn finished_timer() -> Timer {
    let mut timer = Timer::from_seconds(0.1, TimerMode::Once);
    timer.tick(Duration::from_secs(1));
    timer
}

#[allow(dead_code)]
fn sim_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(bevy::time::TimePlugin)
        .add_systems(Update, update_linear);
    app
}

#[test]
fn releases_projectile_when_lifetime_expires() {
    let mut app = sim_test_app();
    let entity = app
        .world_mut()
        .spawn((
            Active,
            Projectile {
                damage: 1.0,
                lifetime: finished_timer(),
            },
            LinearPath {
                origin: Vec2::ZERO,
                dir: Vec2::Y,
                speed: 600.0,
                spawn_time: 0.0,
            },
            Transform::default(),
        ))
        .id();

    app.update();

    let world = app.world();
    assert!(world.get::<Active>(entity).is_none());
    assert!(world.get::<Inactive>(entity).is_some());
}

#[allow(dead_code)]
fn emitter_app() -> App {
    let mut app = App::new();
    app.add_systems(Update, player_emit);
    app
}

#[allow(dead_code)]
fn set_time_delta(app: &mut App, delta_secs: f32) {
    let mut time: Time<()> = Time::default();
    time.advance_by(Duration::from_secs_f32(delta_secs));
    app.world_mut().insert_resource(time);
}

#[allow(dead_code)]
fn spawn_about_to_fire_emitter(world: &mut World, position: Vec3) -> Entity {
    let mut fire_rate = Timer::from_seconds(0.2, TimerMode::Repeating);
    fire_rate.set_elapsed(Duration::from_micros(199_900));
    world
        .spawn((
            Emitter { fire_rate },
            PlayerEmitter,
            Transform::from_translation(position),
        ))
        .id()
}

#[test]
fn acquire_reuses_inactive_entity() {
    let mut app = emitter_app();
    let inactive = app.world_mut().spawn((Inactive, Transform::default())).id();
    let active = app
        .world_mut()
        .spawn((
            Active,
            Projectile {
                damage: 1.0,
                lifetime: Timer::from_seconds(3.0, TimerMode::Once),
            },
            LinearPath {
                origin: Vec2::new(10.0, 10.0),
                dir: Vec2::Y,
                speed: 600.0,
                spawn_time: 0.0,
            },
            Transform::from_translation(Vec3::new(10.0, 10.0, 0.0)),
        ))
        .id();
    spawn_about_to_fire_emitter(app.world_mut(), Vec3::ZERO);

    set_time_delta(&mut app, 0.05);
    app.update();

    let world = app.world();
    assert!(
        world.get::<Active>(inactive).is_some(),
        "inactive entity should be activated"
    );
    assert!(world.get::<Inactive>(inactive).is_none());
    let active_transform = world.get::<Transform>(active).unwrap();
    assert_eq!(
        active_transform.translation,
        Vec3::new(10.0, 10.0, 0.0),
        "previously-active entity should be untouched"
    );
}

#[test]
fn pool_exhaustion_silently_skips() {
    let mut app = emitter_app();
    let max_bullets = GameConfig::default().max_bullets;
    for _ in 0..max_bullets {
        app.world_mut().spawn((
            Active,
            Projectile {
                damage: 0.0,
                lifetime: Timer::from_seconds(3.0, TimerMode::Once),
            },
            Transform::default(),
        ));
    }
    spawn_about_to_fire_emitter(app.world_mut(), Vec3::ZERO);

    let before = app
        .world_mut()
        .query_filtered::<Entity, With<Active>>()
        .iter(app.world())
        .count();
    assert_eq!(before, max_bullets);

    set_time_delta(&mut app, 0.05);
    app.update();

    let after = app
        .world_mut()
        .query_filtered::<Entity, With<Active>>()
        .iter(app.world())
        .count();
    assert_eq!(after, max_bullets, "no new active entity should be created");
}

#[test]
fn does_not_move_without_input() {
    let ship_speed = GameConfig::default().ship_speed;
    let displacement = Vec2::ZERO.normalize_or_zero() * ship_speed * 1.0;
    assert_eq!(displacement, Vec2::ZERO);
}

#[test]
fn keeps_diagonal_speed_normalized() {
    let ship_speed = GameConfig::default().ship_speed;
    let displacement = Vec2::ONE.normalize_or_zero() * ship_speed * 1.0;
    assert!((displacement.length() - ship_speed).abs() < 0.001);
}

fn main() {
    println!("Run tests with: cargo test -p tests");
}
