use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{collision_groups, damageable::Damageable, team::Team};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_particle_effect)
            .add_systems(Update, collide_bullets)
            .register_type::<Bullet>();
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Bullet {
    pub damage: f32,
}

#[derive(Component)]
pub struct BulletImpactEffect;

fn setup_particle_effect(mut effects: ResMut<Assets<EffectAsset>>, mut commands: Commands) {
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0., Vec4::new(100., 0., 0., 1.));
    color_gradient.add_key(0.1, Vec4::new(50., 0., 0., 1.));
    color_gradient.add_key(0.9, Vec4::new(20., 0., 0., 1.));
    color_gradient.add_key(1., Vec4::new(2., 0., 0., 0.));

    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0., Vec2::splat(0.1));
    size_gradient.add_key(0.3, Vec2::splat(0.05));
    size_gradient.add_key(1., Vec2::splat(0.));

    let writer = ExprWriter::default();

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.4).expr(),
        dimension: ShapeDimension::Volume,
    };

    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        // Give a bit of variation by randomizing the initial speed
        speed: writer.lit(1.).uniform(writer.lit(5.)).expr(),
    };

    let init_lifetime = SetAttributeModifier::new(
        Attribute::LIFETIME,
        writer.lit(0.2).uniform(writer.lit(0.8)).expr(),
    );

    let init_age =
        SetAttributeModifier::new(Attribute::AGE, writer.lit(0.).uniform(writer.lit(0.7)).expr());

    let effect = effects.add(
        EffectAsset::new(32768, Spawner::once(50.0.into(), false), writer.finish())
            .init(init_pos)
            .init(init_vel)
            .init(init_lifetime)
            .init(init_age)
            .render(ColorOverLifetimeModifier { gradient: color_gradient })
            .render(SizeOverLifetimeModifier {
                gradient: size_gradient,
                screen_space_size: false,
            }),
    );

    commands.spawn((
        Name::new("Bullet Impact Effect"),
        ParticleEffectBundle {
            effect: ParticleEffect::new(effect),
            transform: Transform::IDENTITY,
            ..Default::default()
        },
        BulletImpactEffect,
    ));
}

fn collide_bullets(
    query: Query<(Entity, &Bullet, &Transform, &Velocity, Option<&Team>)>,
    team_targets: Query<&Team>,
    mut damage_targets: Query<&mut Damageable>,
    mut velocity_targets: Query<(&GlobalTransform, &mut ExternalImpulse)>,
    context: Res<RapierContext>,
    mut commands: Commands,
    mut impact_effect: Query<
        (&mut ParticleEffect, &mut EffectSpawner, &mut Transform),
        (With<BulletImpactEffect>, Without<Bullet>),
    >,
) {
    for (bullet_entity, bullet, transform, velocity, bullet_team) in query.iter() {
        let dir = velocity.linvel.normalize_or_zero();

        let Some((target_entity, intersection)) = context.cast_ray_and_get_normal(
            transform.translation - dir * 0.5,
            dir,
            1.,
            true,
            QueryFilter::default().groups(CollisionGroups {
                memberships: collision_groups::BULLET,
                filters: collision_groups::ALL & !collision_groups::BULLET,
            }),
        ) else {
            continue;
        };

        debug!("bullet hit entity: {:?}", bullet_entity);
        debug!("bullet hit intersection: {:?}", intersection);

        if let (Some(bullet_team), Ok(target_team)) = (bullet_team, team_targets.get(target_entity))
        {
            if !bullet_team.can_damage(target_team) {
                continue;
            }
        }

        let (mut effect, mut spawner, mut effect_transform) = impact_effect.single_mut();
        effect_transform.translation = intersection.point;
        effect_transform.look_to(intersection.normal, Vec3::Z);
        spawner.reset();
        println!("spawning");

        commands.entity(bullet_entity).despawn();

        if let Ok(mut damageable) = damage_targets.get_mut(target_entity) {
            debug!("bullet hit target: {:?}", target_entity);
            debug!("bullet hit target health: {:?}", damageable.health);
            damageable.health -= bullet.damage;

            debug!("bullet hit target health: {:?}", damageable.health);
        }

        if let Ok((transform, mut impulse)) = velocity_targets.get_mut(target_entity) {
            *impulse += ExternalImpulse::at_point(
                intersection.normal * -2.,
                intersection.point,
                transform.translation(),
            );
        }
    }
}
