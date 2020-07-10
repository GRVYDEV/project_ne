use crate::components::{
    AnimationData, Character, CharacterDrawData, Direction, Draw, DrawType, EntityAnimation,
    NPCState, SpawnBounds, NPC,
};
use crate::player::PLAYER_SPEED;
use hecs::World;
use nalgebra::base::Vector2;
use nalgebra::geometry::Isometry2;

use ncollide2d::shape::{Cuboid, ShapeHandle};

use nphysics2d::material::{BasicMaterial, MaterialHandle};

use nphysics2d::object::{
    BodyPartHandle, BodyStatus, ColliderDesc, DefaultBodySet, DefaultColliderSet, RigidBodyDesc,
};

use rand::Rng;

use tetra::Context;
pub fn npc_update(body_set: &mut DefaultBodySet<f32>, world: &mut World, ctx: &mut Context) {
    let delta_time = tetra::time::get_delta_time(ctx);
    // println!("{:?}", delta_time);
    let mut rng = rand::thread_rng();
    for (_id, (_npc, draw, state)) in &mut world.query::<(&NPC, &mut Draw, &mut NPCState)>() {
        let mut player = draw.player.as_mut().unwrap();

        if player.colliding {
            *state = NPCState::random_move();
            player.colliding = false;
        } else {
            *state = match rng.gen_range(0, 50) {
                1 => NPCState::random(),
                _ => *state,
            };
        }

        let body = body_set.rigid_body_mut(player.handle).unwrap();
        match state {
            NPCState::Down => {
                body.set_linear_velocity(Vector2::new(0.0, (PLAYER_SPEED * 0.75)));
                player.entity_animation.direction = Direction::Down;
                player.animation_data.down.advance(delta_time);
            }
            NPCState::Up => {
                body.set_linear_velocity(Vector2::new(0.0, -(PLAYER_SPEED * 0.75)));
                player.entity_animation.direction = Direction::Up;
                player.animation_data.up.advance(delta_time);
            }
            NPCState::Left => {
                body.set_linear_velocity(Vector2::new(-(PLAYER_SPEED * 0.75), 0.0));
                player.entity_animation.direction = Direction::Left;
                player.animation_data.left.advance(delta_time);
            }
            NPCState::Right => {
                body.set_linear_velocity(Vector2::new((PLAYER_SPEED * 0.75), 0.0));
                player.entity_animation.direction = Direction::Right;
                player.animation_data.right.advance(delta_time);
            }
            NPCState::Idle => {
                body.set_linear_velocity(Vector2::new(0.0, 0.0));
            }
        }
    }
}

pub fn spawn_npcs(
    count: u32,
    colliders: &mut DefaultColliderSet<f32>,
    bodies: &mut DefaultBodySet<f32>,
    world: &mut World,
    char_count: usize,
    anims: AnimationData,
    bounds: &SpawnBounds,
) {
    let mut rng = rand::thread_rng();
    for x in 0..count {
        let shape = ShapeHandle::new(Cuboid::new(Vector2::new(5.25, 5.0)));
        let player_pos = Isometry2::new(
            Vector2::new(
                rng.gen_range(bounds.x.0, bounds.x.1),
                rng.gen_range(bounds.y.0, bounds.y.1),
            ),
            nalgebra::zero(),
        );
        let body = RigidBodyDesc::new()
            .position(player_pos)
            .gravity_enabled(false)
            .status(BodyStatus::Dynamic)
            .mass(1.2)
            .build();

        let y = body.position().translation.y;
        let handle = bodies.insert(body);

        let collider = ColliderDesc::new(shape)
            .material(MaterialHandle::new(BasicMaterial::new(0.0, 1.0)))
            .build(BodyPartHandle(handle, 0));

        colliders.insert(collider);
        let draw = Draw {
            draw_type: DrawType::NPC,
            y,
            tile: None,
            player: Some(CharacterDrawData {
                animation_data: anims.clone(),
                entity_animation: EntityAnimation {
                    direction: Direction::Down,
                },
                character: Character(rng.gen_range(0, char_count), char_count),
                handle: handle,
                colliding: false,
            }),
        };
        let entity = world.spawn((NPC, draw, NPCState::Idle));
    }
}
