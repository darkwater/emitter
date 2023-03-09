use bevy::prelude::*;

// pub fn circle(radius: f32, segments: usize) -> Vec<(Vec3, Vec3)> {
//     let mut vertices = Vec::new();
//     let mut prev = Vec3::new(radius, 0., 0.);
//     for i in 1..=segments {
//         let angle = (i as f32 / segments as f32) * PI * 2.;
//         let x = angle.cos() * radius;
//         let y = angle.sin() * radius;
//         let next = Vec3::new(x, y, 0.);
//         vertices.push((prev, next));
//         prev = next;
//     }
//     vertices
// }

pub fn arc(
    radius: f32,
    start: f32,
    end: f32,
    segments: usize,
    complete: bool,
) -> Vec<(Vec3, Vec3)> {
    let mut vertices = Vec::new();
    let first = if complete { end } else { start };
    let mut prev = Vec3::new(first.cos() * radius, first.sin() * radius, 0.);
    for i in (if complete { 0 } else { 1 })..=segments {
        let angle = start + (i as f32 / segments as f32) * (end - start);
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
        let next = Vec3::new(x, y, 0.);
        vertices.push((prev, next));
        prev = next;
    }
    vertices
}
