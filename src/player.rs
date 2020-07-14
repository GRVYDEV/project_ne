use crate::components::*;
use hecs::{Entity, World};
use nalgebra::base::Vector2;

use crate::camera::*;
use luminance::context::GraphicsContext;
use nalgebra::geometry::Isometry2;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::object::{
    BodyPartHandle, BodyStatus, ColliderDesc, DefaultBodySet, DefaultColliderSet, RigidBodyDesc,
};

use tetra::Context;
use std::collections::HashSet;
use glfw::Key::{self, *};
pub const PLAYER_SPEED: f32 = 1.5 * 75.0;
//TODO: Make this return a result again
pub fn new_player<C>(
    ctx: &mut C,
    world: &mut World,
    char_count: usize,
    bodies: &mut DefaultBodySet<f32>,
    colliders: &mut DefaultColliderSet<f32>,
    anims: AnimationData,
    pos: &Vector2<f32>,
) where
    C: GraphicsContext,
{
    let mut camera = Camera::new(1600.0 , 900.0 );
    camera.set_position(Vector2::new(pos.x / 2.0, pos.y / -2.0));
   
    let player_shape = ShapeHandle::new(Cuboid::new(Vector2::new(5.25, 5.0)));

    let player_pos = Isometry2::new(*pos, nalgebra::zero());

    let player_body = RigidBodyDesc::new()
        .position(player_pos)
        .gravity_enabled(false)
        .status(BodyStatus::Dynamic)
        .mass(1.2)
        .build();
    let y = player_body.position().translation.y.clone();
    let player_handle = bodies.insert(player_body);

    let player_collider = ColliderDesc::new(player_shape).build(BodyPartHandle(player_handle, 0));

    colliders.insert(player_collider);

    let draw = Draw {
        draw_type: DrawType::Character,
        y,
        tile: None,
        player: Some(CharacterDrawData {
            animation_data: anims,
            entity_animation: EntityAnimation {
                direction: Direction::Down,
            },
            character: Character(0, char_count),
            handle: player_handle,
            colliding: false,
        }),
    };

    world.spawn((Player, draw, camera));
}

pub fn player_update(body_set: &mut DefaultBodySet<f32>,  world: &mut World, g_ctx: &Graphics)

{
    let delta_time = g_ctx.get_delta_time();
    let key_buffer: &HashSet<Key> = g_ctx.get_keybuffer();

    for (_id, (_camera, draw, _player)) in &mut world.query::<(&mut Camera, &mut Draw, &Player)>() {
        let player = draw.player.as_mut().unwrap();
        let handle = player.handle;
        let player_body = body_set.rigid_body_mut(handle).unwrap();

        if key_buffer.contains(&Key::LeftBracket) {
            if player.character.0 > 0 {
                player.character.0 = player.character.0 - 1;
            } else {
                player.character.0 = player.character.1;
            }
        }
        if key_buffer.contains(&Key::RightBracket) {
            if player.character.0 < player.character.1 {
                player.character.0 = player.character.0 + 1;
            } else {
                player.character.0 = 0;
            }
        }
        if key_buffer.contains(&Key::W) {
            player_body.set_linear_velocity(Vector2::new(0.0, -PLAYER_SPEED));
            player.entity_animation.direction = Direction::Up;
            player.animation_data.up.advance(delta_time);
        }
        if key_buffer.contains(&Key::S) {
            player_body.set_linear_velocity(Vector2::new(0.0, PLAYER_SPEED));
            player.entity_animation.direction = Direction::Down;
            player.animation_data.down.advance(delta_time);
        }
        if key_buffer.contains(&Key::D) {
            player_body.set_linear_velocity(Vector2::new(PLAYER_SPEED, 0.0));
            player.entity_animation.direction = Direction::Right;
            player.animation_data.right.advance(delta_time);
        }
        if key_buffer.contains(&Key::A) {
            player_body.set_linear_velocity(Vector2::new(-PLAYER_SPEED, 0.0));
            player.entity_animation.direction = Direction::Left;
            player.animation_data.left.advance(delta_time);
        }
    }
}
