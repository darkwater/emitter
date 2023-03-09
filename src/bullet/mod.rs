use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{collision_groups, damageable::Damageable};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_particle_effect)
            .add_system(collide_bullets);
    }
}

#[derive(Component)]
pub struct Bullet {
    pub damage: f32,
}

#[derive(Component)]
pub struct BulletImpactEffect;

fn setup_particle_effect(mut effects: ResMut<Assets<EffectAsset>>, mut commands: Commands) {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(100.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(50.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(20.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(2.0, 0.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.10));
    size_gradient1.add_key(0.3, Vec2::splat(0.05));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let effect1 = effects.add(
        EffectAsset {
            name: "Bullet impact".to_string(),
            capacity: 32768,
            spawner: Spawner::once(50.0.into(), false),
            ..Default::default()
        }
        .init(InitPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 0.4,
            dimension: ShapeDimension::Volume,
        })
        .init(InitVelocitySphereModifier {
            center: Vec3::ZERO,
            // Give a bit of variation by randomizing the initial speed
            speed: Value::Uniform((1., 5.)),
        })
        .init(InitLifetimeModifier {
            // Give a bit of variation by randomizing the lifetime per particle
            lifetime: Value::Uniform((0.2, 0.8)),
        })
        .init(InitAgeModifier {
            // Give a bit of variation by randomizing the age per particle. This will control the
            // starting color and starting size of particles.
            age: Value::Uniform((0.0, 0.07)),
        })
        // .update(LinearDragModifier { drag: 5. })
        // .update(AccelModifier::constant(Vec3::new(0., -8., 0.)))
        .render(ColorOverLifetimeModifier { gradient: color_gradient1 })
        .render(SizeOverLifetimeModifier { gradient: size_gradient1 }),
    );

    commands.spawn((
        Name::new("Bullet impact effect"),
        ParticleEffectBundle {
            effect: ParticleEffect::new(effect1),
            transform: Transform::IDENTITY,
            ..Default::default()
        },
        BulletImpactEffect,
    ));
}

fn collide_bullets(
    query: Query<(Entity, &Bullet, &Transform, &Velocity), With<Bullet>>,
    mut targets: Query<&mut Damageable, Without<Bullet>>,
    context: Res<RapierContext>,
    mut commands: Commands,
    mut impact_effect: Query<
        (&mut ParticleEffect, &mut Transform),
        (With<BulletImpactEffect>, Without<Bullet>),
    >,
) {
    for (bullet_entity, bullet, transform, velocity) in query.iter() {
        let dir = velocity.linvel.normalize_or_zero();

        let Some((target_entity, intersection)) = context.cast_ray_and_get_normal(
            transform.translation - dir * 0.5,
            dir,
            1.,
            true,
            QueryFilter::default().groups(CollisionGroups {
                memberships: collision_groups::BULLET,
                filters: collision_groups::ALL
                    & !collision_groups::BULLET
                    & !collision_groups::PLAYER,
            }),
        ) else { continue };

        debug!("bullet hit entity: {:?}", bullet_entity);
        debug!("bullet hit intersection: {:?}", intersection);

        let (mut effect, mut effect_transform) = impact_effect.single_mut();
        effect_transform.translation = intersection.point;
        effect_transform.look_to(intersection.normal, Vec3::Z);
        effect.maybe_spawner().unwrap().reset();

        commands.entity(bullet_entity).despawn();

        if let Ok(mut damageable) = targets.get_mut(target_entity) {
            debug!("bullet hit target: {:?}", target_entity);
            debug!("bullet hit target health: {:?}", damageable.health);
            damageable.health -= bullet.damage;

            debug!("bullet hit target health: {:?}", damageable.health);
        }
    }
}
