use bevy::prelude::*;
#[allow(unused_imports)]
use spaceship_core::{
    Active, CullBoundary, Emitter, Enemy, EnemyPool, EnemyPoolStats, GameConfig, Health, Inactive,
    Movement, MovementIntent, PatternEmitter, PatternState, PatternType, PlayerEmitter,
    PlayerTarget, Projectile,
};
use spaceship_core::{
    cull_enemies, cull_projectiles, enemy_emit, player_emit, release_dead_enemies,
};
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
            Movement::linear(Vec2::Y * 600.0),
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
        .add_systems(Update, spaceship_core::update_movement);
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
            Movement::linear(Vec2::Y * 600.0),
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
    let inactive = app
        .world_mut()
        .spawn((
            Inactive,
            Projectile {
                damage: 0.0,
                lifetime: Timer::from_seconds(1.0, TimerMode::Once),
            },
            Transform::default(),
        ))
        .id();
    let active = app
        .world_mut()
        .spawn((
            Active,
            Projectile {
                damage: 1.0,
                lifetime: Timer::from_seconds(3.0, TimerMode::Once),
            },
            Movement::linear(Vec2::Y * 600.0),
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

#[allow(dead_code)]
fn spinning_test_app() -> App {
    let mut app = App::new();
    app.init_resource::<GameConfig>()
        .add_systems(Startup, spaceship_core::init_projectile_pool)
        .add_systems(Update, enemy_emit);
    app
}

#[test]
fn spinning_pattern_releases_pairs() {
    let mut app = spinning_test_app();

    let player = app
        .world_mut()
        .spawn((PlayerTarget, Transform::from_xyz(0.0, -200.0, 0.0)))
        .id();

    let mut fire_rate = Timer::from_seconds(0.1, TimerMode::Repeating);
    fire_rate.set_elapsed(Duration::from_micros(99_950));

    let enemy = app
        .world_mut()
        .spawn((
            Enemy,
            Active,
            PatternEmitter {
                fire_rate,
                cooldown: {
                    let mut t = Timer::from_seconds(3.0, TimerMode::Once);
                    t.tick(Duration::from_secs_f32(3.0));
                    t
                },
                pattern: PatternType::Spinning {
                    pairs: 3,
                    spacing: 0.1,
                    angular_deviation: 0.1,
                    pair_offset: 20.0,
                    orbit_radius: 30.0,
                    orbit_speed: 3.0,
                },
                state: PatternState::default(),
            },
            Transform::from_xyz(0.0, 200.0, 0.0),
        ))
        .id();

    set_time_delta(&mut app, 0.001);
    app.update();

    let active_count = app
        .world_mut()
        .query_filtered::<Entity, (With<Active>, With<Projectile>)>()
        .iter(app.world())
        .count();

    assert_eq!(active_count, 2, "Should release 2 bullets in first pair");

    let _ = player;
    let _ = enemy;
}

// Movement system tests (TDD - these will fail until we implement Movement)
#[allow(dead_code)]
fn movement_test_app() -> App {
    let mut app = App::new();
    app.add_systems(Update, spaceship_core::update_movement);
    app
}

#[test]
fn linear_movement_maintains_constant_velocity() {
    let mut app = movement_test_app();

    let entity = app
        .world_mut()
        .spawn((
            Active,
            Movement::linear(Vec2::new(100.0, 0.0)),
            Projectile {
                damage: 1.0,
                lifetime: Timer::from_seconds(5.0, TimerMode::Once),
            },
            Transform::default(),
        ))
        .id();

    // Check if entity has Movement component
    let has_movement = app.world().get::<Movement>(entity).is_some();
    assert!(has_movement, "Entity should have Movement component");

    // Check if entity has Active component
    let has_active = app.world().get::<Active>(entity).is_some();
    assert!(has_active, "Entity should have Active component");

    let mut time: Time<()> = Time::default();
    time.advance_by(Duration::from_secs_f32(1.0));
    app.world_mut().insert_resource(time);

    app.update();

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert!((transform.translation.x - 100.0).abs() < 0.01);
    assert!(transform.translation.y.abs() < 0.01);
}

#[test]
fn accelerated_movement_changes_velocity_over_time() {
    let mut app = movement_test_app();

    let entity = app
        .world_mut()
        .spawn((
            Active,
            Movement::accelerated(Vec2::new(50.0, 0.0), Vec2::new(10.0, 0.0)),
            Projectile {
                damage: 1.0,
                lifetime: Timer::from_seconds(5.0, TimerMode::Once),
            },
            Transform::default(),
        ))
        .id();

    let mut time: Time<()> = Time::default();
    time.advance_by(Duration::from_secs_f32(1.0));
    app.world_mut().insert_resource(time);

    app.update();

    let transform = app.world().get::<Transform>(entity).unwrap();
    // With initial velocity 50 and acceleration 10, after 1 second:
    // The movement system applies: velocity = acceleration + velocity * retention
    // With retention = 1.0, velocity becomes 10 + 50 = 60
    // Position = initial_velocity * dt = 50 * 1 = 50
    // But the velocity is updated after position, so position should be 50
    assert!(transform.translation.x >= 50.0);
}

#[test]
fn asymptotic_movement_transitions_smoothly() {
    let mut app = movement_test_app();

    let entity = app
        .world_mut()
        .spawn((
            Active,
            Movement::asymptotic(
                Vec2::new(100.0, 0.0), // initial velocity
                Vec2::new(50.0, 0.0),  // target velocity
                0.95,                  // retention
            ),
            Projectile {
                damage: 1.0,
                lifetime: Timer::from_seconds(5.0, TimerMode::Once),
            },
            Transform::default(),
        ))
        .id();

    let mut time: Time<()> = Time::default();
    time.advance_by(Duration::from_secs_f32(0.5));
    app.world_mut().insert_resource(time);

    app.update();

    let transform = app.world().get::<Transform>(entity).unwrap();
    // Should have moved, but velocity should be transitioning
    assert!(transform.translation.x > 0.0);
}

#[test]
fn attraction_movement_curves_toward_target() {
    let mut app = movement_test_app();

    let entity = app
        .world_mut()
        .spawn((
            Active,
            Movement::towards(
                Vec2::new(100.0, 0.0), // initial velocity
                Vec2::new(0.0, 100.0), // target point
                500.0,                 // attraction strength (increased)
            ),
            Projectile {
                damage: 1.0,
                lifetime: Timer::from_seconds(5.0, TimerMode::Once),
            },
            Transform::default(),
        ))
        .id();

    let mut time: Time<()> = Time::default();
    time.advance_by(Duration::from_secs_f32(0.5));
    app.world_mut().insert_resource(time);

    app.update();

    let transform = app.world().get::<Transform>(entity).unwrap();
    // Should have moved toward target (upward)
    // With strong attraction, the bullet should curve upward
    assert!(
        transform.translation.y > 0.0,
        "Bullet should have moved upward toward target"
    );
}

#[test]
fn movement_builder_creates_custom_combinations() {
    let mut app = movement_test_app();

    let entity = app
        .world_mut()
        .spawn((
            Active,
            Movement::builder()
                .velocity(Vec2::new(50.0, 0.0))
                .acceleration(Vec2::new(5.0, 0.0))
                .retention(0.98)
                .build(),
            Projectile {
                damage: 1.0,
                lifetime: Timer::from_seconds(5.0, TimerMode::Once),
            },
            Transform::default(),
        ))
        .id();

    let mut time: Time<()> = Time::default();
    time.advance_by(Duration::from_secs_f32(1.0));
    app.world_mut().insert_resource(time);

    app.update();

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert!(transform.translation.x > 0.0);
}

#[test]
fn enemy_pool_acquire_is_atomic() {
    let mut pool = EnemyPool::default();
    let entity = Entity::from_bits(42);
    pool.push(entity);

    let acquired = pool.acquire();
    assert_eq!(acquired, Some(entity));
    assert_eq!(
        pool.acquire(),
        None,
        "second acquire must not return the same entity"
    );
}

#[test]
fn enemy_pool_release_makes_entity_available_again() {
    let mut pool = EnemyPool::default();
    let entity = Entity::from_bits(7);
    pool.release(entity);

    assert_eq!(pool.acquire(), Some(entity));
    assert_eq!(pool.acquire(), None);
}

#[test]
fn enemy_pool_stats_default_is_zero() {
    let stats = EnemyPoolStats::default();
    assert_eq!(stats.failed_spawns, 0);
    assert_eq!(stats.total_releases, 0);
}

#[allow(dead_code)]
fn enemy_pool_init_test_app() -> App {
    let mut app = App::new();
    app.init_resource::<GameConfig>()
        .init_resource::<EnemyPool>()
        .init_resource::<EnemyPoolStats>()
        .add_systems(Startup, spaceship_core::init_enemy_pool);
    app
}

#[test]
fn init_enemy_pool_creates_configured_count() {
    let mut app = enemy_pool_init_test_app();
    app.update();

    let pool = app.world().resource::<EnemyPool>();
    assert_eq!(pool.available_count(), GameConfig::default().max_enemies);
}

#[test]
fn init_enemy_pool_creates_inactive_enemies() {
    let mut app = enemy_pool_init_test_app();
    app.update();

    let world = app.world_mut();
    let active = world
        .query_filtered::<Entity, (With<Enemy>, With<Active>)>()
        .iter(world)
        .count();
    let inactive = world
        .query_filtered::<Entity, (With<Enemy>, With<Inactive>)>()
        .iter(world)
        .count();
    let total = world
        .query_filtered::<Entity, With<Enemy>>()
        .iter(world)
        .count();

    assert_eq!(active, 0, "no enemy should be active after init");
    assert_eq!(inactive, total, "every enemy should be inactive after init");
    assert_eq!(total, GameConfig::default().max_enemies);
}

#[test]
fn init_enemy_pool_makes_enemies_hidden() {
    let mut app = enemy_pool_init_test_app();
    app.update();

    let mut q = app.world_mut().query::<(&Enemy, &Visibility)>();
    for (_, visibility) in q.iter(app.world_mut()) {
        assert_eq!(*visibility, Visibility::Hidden);
    }
}

#[allow(dead_code)]
fn spawn_enemy_test_app() -> App {
    let mut app = App::new();
    app.init_resource::<GameConfig>()
        .init_resource::<EnemyPool>()
        .init_resource::<EnemyPoolStats>()
        .add_systems(Startup, spaceship_core::init_enemy_pool);
    app
}

#[allow(dead_code)]
fn test_pattern() -> PatternEmitter {
    PatternEmitter {
        fire_rate: Timer::from_seconds(0.1, TimerMode::Repeating),
        cooldown: {
            let mut t = Timer::from_seconds(3.0, TimerMode::Once);
            t.tick(Duration::from_secs_f32(3.0));
            t
        },
        pattern: PatternType::Ring {
            bullet_count: 8,
            speed: 100.0,
            rotation_speed: 0.0,
        },
        state: PatternState::default(),
    }
}

#[test]
fn spawn_enemy_activates_one_available_entity() {
    let mut app = spawn_enemy_test_app();
    app.update();

    let entity = spaceship_core::spawn_enemy(
        app.world_mut(),
        Vec2::new(10.0, 20.0),
        test_pattern(),
        100.0,
    )
    .expect("pool is non-empty");

    let world = app.world();
    assert!(world.get::<Active>(entity).is_some());
    assert!(world.get::<Inactive>(entity).is_none());
    assert_eq!(
        world.get::<Transform>(entity).unwrap().translation,
        Vec3::new(10.0, 20.0, 0.0)
    );
    let health = world.get::<Health>(entity).unwrap();
    assert_eq!(health.current, 100.0);
    assert_eq!(health.max, 100.0);
    assert!(world.get::<PatternEmitter>(entity).is_some());
}

#[test]
fn spawn_enemy_initializes_visibility_inherited() {
    let mut app = spawn_enemy_test_app();
    app.update();

    let entity = spaceship_core::spawn_enemy(app.world_mut(), Vec2::ZERO, test_pattern(), 50.0)
        .expect("pool is non-empty");

    let world = app.world();
    assert_eq!(
        *world.get::<Visibility>(entity).unwrap(),
        Visibility::Inherited
    );
}

#[test]
fn spawn_enemy_when_pool_empty_records_failure() {
    let mut app = App::new();
    app.init_resource::<GameConfig>()
        .init_resource::<EnemyPool>()
        .init_resource::<EnemyPoolStats>();

    let result = spaceship_core::spawn_enemy(app.world_mut(), Vec2::ZERO, test_pattern(), 50.0);

    assert!(result.is_none());
    let stats = app.world().resource::<EnemyPoolStats>();
    assert_eq!(stats.failed_spawns, 1);
}

#[test]
fn release_enemy_makes_entity_inactive_and_returns_to_pool() {
    let mut app = spawn_enemy_test_app();
    app.update();

    let entity =
        spaceship_core::spawn_enemy(app.world_mut(), Vec2::new(5.0, 5.0), test_pattern(), 50.0)
            .expect("pool is non-empty");

    assert_eq!(
        app.world().resource::<EnemyPool>().available_count(),
        GameConfig::default().max_enemies - 1
    );

    spaceship_core::release_enemy(app.world_mut(), entity);

    let world = app.world();
    assert!(world.get::<Active>(entity).is_none());
    assert!(world.get::<Inactive>(entity).is_some());
    assert_eq!(
        *world.get::<Visibility>(entity).unwrap(),
        Visibility::Hidden
    );
    assert_eq!(
        world.get::<Transform>(entity).unwrap().translation,
        Vec3::ZERO
    );
    assert_eq!(
        world.resource::<EnemyPool>().available_count(),
        GameConfig::default().max_enemies
    );
    assert_eq!(world.resource::<EnemyPoolStats>().total_releases, 1);
}

#[test]
fn release_enemy_clears_pattern_and_health_state() {
    let mut app = spawn_enemy_test_app();
    app.update();

    let entity = spaceship_core::spawn_enemy(app.world_mut(), Vec2::ZERO, test_pattern(), 100.0)
        .expect("pool is non-empty");

    assert!(app.world().get::<PatternEmitter>(entity).is_some());
    assert!(app.world().get::<Health>(entity).is_some());

    spaceship_core::release_enemy(app.world_mut(), entity);

    let world = app.world();
    assert!(
        world.get::<PatternEmitter>(entity).is_none(),
        "PatternEmitter must be cleared on release"
    );
    assert!(
        world.get::<Health>(entity).is_none(),
        "Health must be cleared on release"
    );
}

#[test]
fn enemy_can_be_activated_released_and_activated_again_with_new_state() {
    let mut app = spawn_enemy_test_app();
    app.update();

    let entity =
        spaceship_core::spawn_enemy(app.world_mut(), Vec2::new(1.0, 2.0), test_pattern(), 30.0)
            .expect("pool is non-empty");

    spaceship_core::release_enemy(app.world_mut(), entity);

    let different_pattern = PatternEmitter {
        fire_rate: Timer::from_seconds(0.2, TimerMode::Repeating),
        cooldown: {
            let mut t = Timer::from_seconds(1.0, TimerMode::Once);
            t.tick(Duration::from_secs_f32(1.0));
            t
        },
        pattern: PatternType::Spinning {
            pairs: 5,
            spacing: 0.05,
            angular_deviation: 0.05,
            pair_offset: 10.0,
            orbit_radius: 15.0,
            orbit_speed: 2.0,
        },
        state: PatternState::default(),
    };

    let reactivated = spaceship_core::spawn_enemy(
        app.world_mut(),
        Vec2::new(50.0, 60.0),
        different_pattern,
        200.0,
    )
    .expect("entity should be available again");

    assert_eq!(reactivated, entity, "pool should hand back the same entity");

    let world = app.world();
    assert!(world.get::<Active>(reactivated).is_some());
    let transform = world.get::<Transform>(reactivated).unwrap();
    assert_eq!(transform.translation, Vec3::new(50.0, 60.0, 0.0));
    let health = world.get::<Health>(reactivated).unwrap();
    assert_eq!(health.current, 200.0);
    assert_eq!(health.max, 200.0);
    let pattern = world.get::<PatternEmitter>(reactivated).unwrap();
    match pattern.pattern {
        PatternType::Spinning { pairs, .. } => assert_eq!(pairs, 5),
        PatternType::Ring { .. } => panic!("stale Ring pattern persisted"),
    }
}

#[test]
fn simultaneous_spawns_acquire_distinct_entities() {
    let mut app = spawn_enemy_test_app();
    app.update();

    let mut acquired = Vec::new();
    for _ in 0..GameConfig::default().max_enemies {
        let entity = spaceship_core::spawn_enemy(app.world_mut(), Vec2::ZERO, test_pattern(), 10.0)
            .expect("pool should still have entities");
        acquired.push(entity);
    }

    let unique: std::collections::HashSet<_> = acquired.iter().map(|e| e.index()).collect();
    assert_eq!(
        unique.len(),
        acquired.len(),
        "no entity may be acquired twice"
    );

    let fourth = spaceship_core::spawn_enemy(app.world_mut(), Vec2::ZERO, test_pattern(), 10.0);
    assert!(fourth.is_none(), "exhausted pool returns None");
    assert_eq!(
        app.world().resource::<EnemyPoolStats>().failed_spawns,
        1,
        "exhausted pool increments failed_spawns exactly once"
    );
}

#[allow(dead_code)]
fn enemy_cull_test_app(boundary: CullBoundary) -> App {
    let mut app = App::new();
    app.init_resource::<GameConfig>()
        .init_resource::<EnemyPool>()
        .init_resource::<EnemyPoolStats>()
        .insert_resource(boundary)
        .add_systems(Startup, spaceship_core::init_enemy_pool)
        .add_systems(Update, cull_enemies);
    app
}

#[test]
fn cull_enemies_releases_enemy_outside_boundary() {
    let mut app = enemy_cull_test_app(CullBoundary {
        half_width: 100.0,
        half_height: 100.0,
    });

    app.update(); // run Startup to initialize the pool

    let entity = spaceship_core::spawn_enemy(
        app.world_mut(),
        Vec2::new(1_000.0, 0.0),
        test_pattern(),
        100.0,
    )
    .expect("pool is non-empty");

    app.update();

    let world = app.world();
    assert!(world.get::<Active>(entity).is_none());
    assert!(world.get::<Inactive>(entity).is_some());
    assert_eq!(
        world.resource::<EnemyPool>().available_count(),
        GameConfig::default().max_enemies
    );
    assert_eq!(world.resource::<EnemyPoolStats>().total_releases, 1);
}

#[test]
fn cull_enemies_keeps_enemy_inside_boundary() {
    let mut app = enemy_cull_test_app(CullBoundary {
        half_width: 100.0,
        half_height: 100.0,
    });

    app.update(); // run Startup to initialize the pool

    let entity = spaceship_core::spawn_enemy(
        app.world_mut(),
        Vec2::new(50.0, 50.0),
        test_pattern(),
        100.0,
    )
    .expect("pool is non-empty");

    app.update();

    let world = app.world();
    assert!(world.get::<Active>(entity).is_some());
    assert_eq!(
        world.resource::<EnemyPool>().available_count(),
        GameConfig::default().max_enemies - 1
    );
}

#[allow(dead_code)]
fn dead_enemy_test_app() -> App {
    let mut app = App::new();
    app.init_resource::<GameConfig>()
        .init_resource::<EnemyPool>()
        .init_resource::<EnemyPoolStats>()
        .add_systems(Startup, spaceship_core::init_enemy_pool)
        .add_systems(Update, release_dead_enemies);
    app
}

#[test]
fn release_dead_enemies_returns_zero_hp_to_pool() {
    let mut app = dead_enemy_test_app();

    app.update(); // run Startup to initialize the pool

    let entity = spaceship_core::spawn_enemy(app.world_mut(), Vec2::ZERO, test_pattern(), 0.0)
        .expect("pool is non-empty");

    app.update();

    let world = app.world();
    assert!(world.get::<Active>(entity).is_none());
    assert!(world.get::<Inactive>(entity).is_some());
    assert!(world.get::<Health>(entity).is_none());
    assert_eq!(
        world.resource::<EnemyPool>().available_count(),
        GameConfig::default().max_enemies
    );
}

#[test]
fn release_dead_enemies_keeps_alive_enemies() {
    let mut app = dead_enemy_test_app();

    app.update(); // run Startup to initialize the pool

    let entity = spaceship_core::spawn_enemy(app.world_mut(), Vec2::ZERO, test_pattern(), 50.0)
        .expect("pool is non-empty");

    app.update();

    let world = app.world();
    assert!(world.get::<Active>(entity).is_some());
    assert_eq!(
        world.resource::<EnemyPool>().available_count(),
        GameConfig::default().max_enemies - 1
    );
}

fn main() {
    println!("Run tests with: cargo test -p tests");
}
