use bevy::prelude::*;

pub trait LookAt2d {
    fn look_at_2d(&mut self, target: Vec3);
}

impl LookAt2d for Transform {
    fn look_at_2d(&mut self, target: Vec3) {
        let mut direction = target - self.translation;
        direction.z = 0.;
        direction = direction.normalize();
        self.rotation = Quat::from_rotation_z(direction.y.atan2(direction.x));
    }
}
