use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct ZLocked {
    pub angular: bool,
}

pub struct ZLockPlugin;

impl Plugin for ZLockPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(zlock_transform).add_system(zlock_velocity);
    }
}

fn zlock_transform(mut query: Query<(&mut Transform, &ZLocked)>) {
    for (mut transform, zlock) in query.iter_mut() {
        transform.translation.z = 0.;

        if zlock.angular {
            transform.rotation.x = 0.;
            transform.rotation.y = 0.;
        }
    }
}

fn zlock_velocity(mut query: Query<(&mut Velocity, &ZLocked)>) {
    for (mut velocity, zlock) in query.iter_mut() {
        velocity.linvel.z = 0.;

        if zlock.angular {
            velocity.angvel.x = 0.;
            velocity.angvel.y = 0.;
        }
    }
}
