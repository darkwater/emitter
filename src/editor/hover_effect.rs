use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use super::CursorHoveringEntity;

#[derive(Component)]
pub struct HoverEffect;

#[derive(Component)]
pub struct HoverEffectSpawner;

pub fn set_hover_effect(
    hovering_entity: Res<CursorHoveringEntity>,
    target: Query<&Transform, (With<HoverEffect>, Without<HoverEffectSpawner>)>,
    mut particle_spawner: Query<(&mut Transform, &mut ParticleEffect), With<HoverEffectSpawner>>,
) {
    let (mut transform, mut spawner) = particle_spawner.single_mut();
    let Some(spawner) = spawner.maybe_spawner() else { return };

    let Some(entity) = hovering_entity.entity else {
        spawner.set_active(false);
        return;
    };

    if let Ok(target) = target.get(entity) {
        transform.translation = target.translation;
        spawner.set_active(true);
    } else {
        spawner.set_active(false);
    }
}

pub fn setup_effect(mut effects: ResMut<Assets<EffectAsset>>, mut commands: Commands) {
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(100.0, 100.0, 100.0, 1.0));
    color_gradient.add_key(0.1, Vec4::new(50.0, 50.0, 50.0, 1.0));
    color_gradient.add_key(0.9, Vec4::new(20.0, 20.0, 20.0, 1.0));
    color_gradient.add_key(1.0, Vec4::new(2.0, 2.0, 2.0, 0.0));

    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0.0, Vec2::splat(0.20));
    size_gradient.add_key(0.3, Vec2::splat(0.05));
    size_gradient.add_key(1.0, Vec2::splat(0.0));

    let effect = effects.add(
        EffectAsset {
            name: "Hover Effect".to_string(),
            capacity: 32768,
            spawner: Spawner::rate(20.0.into()).with_active(true),
            ..Default::default()
        }
        .init(InitPositionSphereModifier {
            center: Vec3::ZERO,
            // axis: Vec3::Z,
            radius: 0.1,
            dimension: ShapeDimension::Volume,
        })
        // .init(InitVelocityTangentModifier {
        //     origin: Vec3::ZERO,
        //     axis: Vec3::Z,
        //     speed: Value::Uniform((0., 1.)),
        // })
        .init(InitVelocitySphereModifier {
            center: Vec3::ZERO,
            // Give a bit of variation by randomizing the initial speed
            speed: Value::Uniform((0., 1.)),
        })
        .init(InitLifetimeModifier {
            // Give a bit of variation by randomizing the lifetime per particle
            lifetime: Value::Uniform((0.1, 1.4)),
        })
        // .init(InitAgeModifier {
        //     // Give a bit of variation by randomizing the age per particle. This will control the
        //     // starting color and starting size of particles.
        //     age: Value::Uniform((0.0, 0.07)),
        // })
        // .update(LinearDragModifier { drag: 5. })
        // .update(AccelModifier::constant(Vec3::new(0., -8., 0.)))
        .render(ColorOverLifetimeModifier { gradient: color_gradient })
        .render(SizeOverLifetimeModifier { gradient: size_gradient }),
    );

    commands.spawn((
        ParticleEffectBundle {
            effect: ParticleEffect::new(effect),
            transform: Transform::IDENTITY,
            ..Default::default()
        },
        HoverEffectSpawner,
    ));
}
