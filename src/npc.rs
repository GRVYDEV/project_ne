use crate::components::{AnimationData, Character, Direction, EntityAnimation, NPCState, NPC};
use crate::player::PLAYER_SPEED;
use hecs::World;
use nalgebra::base::Vector2;
use nalgebra::geometry::Isometry2;

use ncollide2d::shape::{Cuboid, ShapeHandle};

use nphysics2d::material::{BasicMaterial, MaterialHandle};

use nphysics2d::object::{
    BodyPartHandle, BodyStatus, ColliderDesc, DefaultBodyHandle, DefaultBodySet,
    DefaultColliderSet, RigidBodyDesc,
};

use rand::Rng;

use tetra::Context;
pub fn npc_update(body_set: &mut DefaultBodySet<f32>, world: &mut World, ctx: &mut Context) {
    let delta_time = tetra::time::get_delta_time(ctx);
    // println!("{:?}", delta_time);
    let mut rng = rand::thread_rng();
    for (_id, (_npc, anim, handle, anim_data, state)) in &mut world.query::<(
        &NPC,
        &mut EntityAnimation,
        &DefaultBodyHandle,
        &mut AnimationData,
        &mut NPCState,
    )>() {
        *state = match rng.gen_range(0, 100) {
            1 => NPCState::random(),
            _ => *state,
        };
        let body = body_set.rigid_body_mut(*handle).unwrap();
        match state {
            NPCState::Down => {
                body.set_linear_velocity(Vector2::new(0.0, (PLAYER_SPEED * 0.75)));
                anim.direction = Direction::Down;
                anim_data.down.advance(delta_time);
            }
            NPCState::Up => {
                body.set_linear_velocity(Vector2::new(0.0, -(PLAYER_SPEED * 0.75)));
                anim.direction = Direction::Up;
                anim_data.up.advance(delta_time);
            }
            NPCState::Left => {
                body.set_linear_velocity(Vector2::new(-(PLAYER_SPEED * 0.75), 0.0));
                anim.direction = Direction::Left;
                anim_data.left.advance(delta_time);
            }
            NPCState::Right => {
                body.set_linear_velocity(Vector2::new((PLAYER_SPEED * 0.75), 0.0));
                anim.direction = Direction::Right;
                anim_data.right.advance(delta_time);
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
) {
    let mut rng = rand::thread_rng();
    for x in 0..count {
        let shape = ShapeHandle::new(Cuboid::new(Vector2::new(10.5, 10.0)));
        let player_pos = Isometry2::new(
            Vector2::new(rng.gen_range(100.0, 1600.0), rng.gen_range(100.0, 1600.0)),
            nalgebra::zero(),
        );
        let body = RigidBodyDesc::new()
            .position(player_pos)
            .gravity_enabled(false)
            .status(BodyStatus::Dynamic)
            .mass(1.2)
            .build();
        let handle = bodies.insert(body);

        let collider = ColliderDesc::new(shape)
            .material(MaterialHandle::new(BasicMaterial::new(0.0, 1.0)))
            .build(BodyPartHandle(handle, 0));

        colliders.insert(collider);
        let entity = world.spawn((
            NPC,
            EntityAnimation {
                direction: Direction::Down,
            },
            handle,
            Character(rng.gen_range(0, char_count + 1), char_count),
            anims.clone(),
            NPCState::Idle,
        ));
    }
}
