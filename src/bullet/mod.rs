use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{collision_groups, damageable::Damageable};

#[derive(Component)]
pub struct Bullet {
    pub damage: f32,
}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(collide_bullets);
    }
}

fn collide_bullets(
    query: Query<(Entity, &Bullet, &Transform, &Velocity), With<Bullet>>,
    mut targets: Query<&mut Damageable, Without<Bullet>>,
    context: Res<RapierContext>,
    mut commands: Commands,
    mut impact_effect: Query<(&mut ParticleEffect, &mut Transform), Without<Bullet>>,
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
