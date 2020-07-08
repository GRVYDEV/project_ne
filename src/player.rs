use crate::components::*;
use hecs::{Entity, World};
use nalgebra::base::Vector2;

use nphysics2d::object::{DefaultBodyHandle, DefaultBodySet};

use tetra::graphics::{Camera};
use tetra::input::{self, Key};

use tetra::Context;

pub const PLAYER_SPEED: f32 = 3.0 * 75.0;

pub fn new_player(
    ctx: &mut Context,
    world: &mut World,
    body: DefaultBodyHandle,
    char_count: usize,
    anims: AnimationData,
) -> tetra::Result<Entity> {
    let camera = Camera::with_window_size(ctx);

    Ok(world.spawn((
        Player,
        EntityAnimation {
            direction: Direction::Down,
        },
        camera,
        body,
        LastDirection(Direction::Down),
        Character(0, char_count),
        anims,
    )))
}

pub fn player_update(body_set: &mut DefaultBodySet<f32>, ctx: &mut Context, world: &mut World) {
    let delta_time = tetra::time::get_delta_time(ctx);

    for (_id, (_camera, anim, _player, handle, character, anim_data)) in &mut world.query::<(
        &mut Camera,
        &mut EntityAnimation,
        &Player,
        &DefaultBodyHandle,
        &mut Character,
        &mut AnimationData,
    )>() {
        let player_body = body_set.rigid_body_mut(*handle).unwrap();
        if input::is_key_pressed(ctx, Key::LeftBracket) {
            if character.0 > 0 {
                character.0 = character.0 - 1;
            } else {
                character.0 = character.1;
            }
        }
        if input::is_key_pressed(ctx, Key::RightBracket) {
            if character.0 < character.1 {
                character.0 = character.0 + 1;
            } else {
                character.0 = 0;
            }
        }
        if input::is_key_down(ctx, Key::W) {
            player_body.set_linear_velocity(Vector2::new(0.0, -PLAYER_SPEED));
            anim.direction = Direction::Up;
            anim_data.up.advance(delta_time);
        }
        if input::is_key_down(ctx, Key::S) {
            player_body.set_linear_velocity(Vector2::new(0.0, PLAYER_SPEED));
            anim.direction = Direction::Down;
            anim_data.down.advance(delta_time);
        }
        if input::is_key_down(ctx, Key::D) {
            player_body.set_linear_velocity(Vector2::new(PLAYER_SPEED, 0.0));
            anim.direction = Direction::Right;
            anim_data.right.advance(delta_time);
        }
        if input::is_key_down(ctx, Key::A) {
            player_body.set_linear_velocity(Vector2::new(-PLAYER_SPEED, 0.0));
            anim.direction = Direction::Left;
            anim_data.left.advance(delta_time);
        }
    }
}
