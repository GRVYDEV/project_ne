use crate::components::Sprite;

use nalgebra::base::Vector2;
use nalgebra::geometry::Isometry2;

use ncollide2d::pipeline::CollisionGroups;

use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::object::{
    BodyPartHandle, BodyStatus, ColliderDesc, DefaultBodySet, DefaultColliderSet, RigidBodyDesc,
};

use std::collections::HashMap;

use tiled::ObjectShape;

pub fn create_map_bounds(
    lyr: &tiled::Layer,
    colliders: &mut DefaultColliderSet<f32>,
    bodies: &mut DefaultBodySet<f32>,
) {
    let mut tile_group = CollisionGroups::new();
    tile_group.set_membership(&[3]);
    tile_group.set_whitelist(&[1]);
    let y_max = lyr.tiles.len() - 1;
    for (y, row) in lyr.tiles.iter().enumerate().clone() {
        let x_max = row.len() - 1;
        for (x, &tile) in row.iter().enumerate() {
            let shape = ShapeHandle::new(Cuboid::new(Vector2::repeat(16.0 - 0.01)));
            if tile.gid == 0 {
                let shape_pos = Isometry2::new(
                    Vector2::new((x as f32 * 32.0) + 16.0, (y as f32 * 32.0) + 16.0),
                    nalgebra::zero(),
                );
                let world_body = RigidBodyDesc::new()
                    .position(shape_pos)
                    .gravity_enabled(false)
                    .status(BodyStatus::Static)
                    .build();

                let world_body_handle = bodies.insert(world_body);

                let world_body_collider =
                    ColliderDesc::new(shape.clone()).build(BodyPartHandle(world_body_handle, 0));

                colliders.insert(world_body_collider);
                continue;
            }
            if y == 0 {
                let shape_pos = Isometry2::new(
                    Vector2::new((x as f32 * 32.0) + 16.0, (y as f32 * 32.0) - 16.0),
                    nalgebra::zero(),
                );
                let world_body = RigidBodyDesc::new()
                    .position(shape_pos)
                    .gravity_enabled(false)
                    .status(BodyStatus::Static)
                    .build();

                let world_body_handle = bodies.insert(world_body);

                let world_body_collider =
                    ColliderDesc::new(shape.clone()).build(BodyPartHandle(world_body_handle, 0));

                colliders.insert(world_body_collider);
            } else if y == y_max {
                let shape_pos = Isometry2::new(
                    Vector2::new((x as f32 * 32.0) + 16.0, ((y as f32 + 1.0) * 32.0) + 16.0),
                    nalgebra::zero(),
                );
                let world_body = RigidBodyDesc::new()
                    .position(shape_pos)
                    .gravity_enabled(false)
                    .status(BodyStatus::Static)
                    .build();

                let world_body_handle = bodies.insert(world_body);

                let world_body_collider =
                    ColliderDesc::new(shape.clone()).build(BodyPartHandle(world_body_handle, 0));

                colliders.insert(world_body_collider);
            }

            if x == 0 {
                let shape_pos = Isometry2::new(
                    Vector2::new((x as f32 * 32.0) - 16.0, (y as f32 * 32.0) + 16.0),
                    nalgebra::zero(),
                );
                let world_body = RigidBodyDesc::new()
                    .position(shape_pos)
                    .gravity_enabled(false)
                    .status(BodyStatus::Static)
                    .build();

                let world_body_handle = bodies.insert(world_body);

                let world_body_collider =
                    ColliderDesc::new(shape.clone()).build(BodyPartHandle(world_body_handle, 0));

                colliders.insert(world_body_collider);
            } else if x == x_max {
                let shape_pos = Isometry2::new(
                    Vector2::new(((x as f32 + 1.0) * 32.0) + 16.0, (y as f32 * 32.0) + 16.0),
                    nalgebra::zero(),
                );
                let world_body = RigidBodyDesc::new()
                    .position(shape_pos)
                    .gravity_enabled(false)
                    .status(BodyStatus::Static)
                    .build();

                let world_body_handle = bodies.insert(world_body);

                let world_body_collider =
                    ColliderDesc::new(shape.clone()).build(BodyPartHandle(world_body_handle, 0));

                colliders.insert(world_body_collider);
            }
        }
    }
}
pub fn create_physics_world(
    lyrs: &Vec<tiled::Layer>,
    sprite_map: &HashMap<u32, Sprite>,
    colliders: &mut DefaultColliderSet<f32>,
    bodies: &mut DefaultBodySet<f32>,
) {
    let mut tile_group = CollisionGroups::new();
    tile_group.set_membership(&[3]);
    tile_group.set_whitelist(&[1]);
    for lyr in lyrs {
        for (y, row) in lyr.tiles.iter().enumerate().clone() {
            for (x, &tile) in row.iter().enumerate() {
                if tile.gid == 0 {
                    continue;
                }
                let gid = tile.gid;
                let sprite = sprite_map.get(&gid).unwrap();
                let mut rotation: f32 = 0.0;
                if tile.flip_h {
                    rotation += 180.0;
                }
                if tile.flip_d {
                    rotation -= 90.0;
                }
                if tile.flip_v {
                    rotation -= 0.0;
                }
                if sprite.collision_objects.is_some() {
                    let objs = sprite.collision_objects.as_ref().unwrap();
                    let obj = &objs[0];
                    let dimensions: Option<(&f32, &f32)> = match &obj.shape {
                        ObjectShape::Rect { width, height } => Some((width, height)),
                        _ => None,
                    };
                    let mut height: f32 = 0.0;
                    let mut width: f32 = 0.0;

                    if dimensions.is_some() {
                        width = dimensions.unwrap().0.clone();
                        height = dimensions.unwrap().1.clone();
                    }
                    let shape = ShapeHandle::new(Cuboid::new(Vector2::new(width, height)));
                    let mut translator: (f32, f32);
                    match rotation {
                        0.0 => translator = (obj.x, obj.y),
                        90.0 => translator = (obj.y, obj.x),
                        -90.0 => translator = (obj.y, -obj.x),
                        180.0 => translator = (-obj.x, obj.y),
                        _ => panic!("Invalid Rotation: {:?}", rotation),
                    }
                    if sprite.width == 32.0 && sprite.height == 32.0 {
                        translator.0 = translator.0 + translator.0;
                        translator.1 = translator.1 + translator.1;
                    }
                    let world_body = RigidBodyDesc::new()
                        .translation(Vector2::new(
                            ((x as f32 * 32.0) + translator.0) + 16.0,
                            ((y as f32 * 32.0) + translator.1) + 16.0,
                        ))
                        .rotation(nalgebra::zero())
                        .gravity_enabled(false)
                        .status(BodyStatus::Static)
                        .build();

                    let world_body_handle = bodies.insert(world_body);

                    let world_body_collider = ColliderDesc::new(shape)
                        .rotation(rotation.to_radians())
                        .build(BodyPartHandle(world_body_handle, 0));

                    colliders.insert(world_body_collider);
                }
            }
        }
    }
}
