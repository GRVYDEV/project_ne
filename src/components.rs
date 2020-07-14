use crate::graphics::{Region, SpriteBatch};
use hecs::World;
use nalgebra::{Vector2, Vector3};

use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;

use nphysics2d::object::{DefaultBodyHandle, DefaultBodySet, DefaultColliderSet};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};

use rand::Rng;
use std::collections::{HashSet, HashMap};

use std::time::Duration;
use tetra::graphics;

use tetra::graphics::{animation::Animation, DrawParams, Rectangle, Texture};
use tetra::Context;

use tetra::math::Vec2;

use crate::game::Game;

use tiled::Layer;
use luminance::context::GraphicsContext;
use crate::SCALE;
pub struct SpawnBounds {
    pub x: (f32, f32),
    pub y: (f32, f32),
}

pub struct Graphics{
    delta_time: Duration,
    keybuffer: HashSet<glfw::Key>
}

impl Graphics{
    pub fn new(keybuffer: &HashSet<glfw::Key>, delta_time: Duration) -> Graphics{
        Graphics{
            keybuffer: keybuffer.clone(),
            delta_time
        }
    }
    pub fn get_delta_time(&self) -> Duration{
        self.delta_time
    }
    pub fn get_keybuffer(&self) -> &HashSet<glfw::Key>{
        &self.keybuffer
    }
}

pub struct PhysicsWorld {
    pub mechanical_world: DefaultMechanicalWorld<f32>,
    pub geometrical_world: DefaultGeometricalWorld<f32>,
    pub bodies:  DefaultBodySet<f32>,
    pub colliders: DefaultColliderSet<f32>,
    pub force_generators: DefaultForceGeneratorSet<f32>,
    pub joint_constraints: DefaultJointConstraintSet<f32>,
}
pub struct LastDirection(pub Direction);
#[derive(Clone, Copy)]
pub struct Character(pub usize, pub usize);
#[derive(Clone, Copy)]
pub enum NPCState {
    Up,
    Down,
    Left,
    Right,
    Idle,
}
impl NPCState {
    pub fn random() -> NPCState {
        let mut rng = rand::thread_rng();

        let state = rng.gen_range(0, 30);
        match state {
            0 => NPCState::Up,
            1 => NPCState::Down,
            2 => NPCState::Left,
            3 => NPCState::Right,
            _ => NPCState::Idle,
        }
    }
    pub fn random_move() -> NPCState {
        let mut rng = rand::thread_rng();

        let state = rng.gen_range(0, 4);
        match state {
            0 => NPCState::Up,
            1 => NPCState::Down,
            2 => NPCState::Left,
            _ => NPCState::Right,
        }
    }
}
pub struct GameState {
    pub world: World,
    pub sprite_map: HashMap<u32, Sprite>,
    pub layers: Vec<Layer>,
    pub texture_map: HashMap<String, Texture>,
    pub mechanical_world: DefaultMechanicalWorld<f32>,
    pub geometrical_world: DefaultGeometricalWorld<f32>,
    pub body_set: DefaultBodySet<f32>,
    pub collider_set: DefaultColliderSet<f32>,
    pub constraint_set: DefaultJointConstraintSet<f32>,
    pub force_gen_set: DefaultForceGeneratorSet<f32>,
    pub characters: HashMap<usize, Texture>,
    pub npcs: HashMap<usize, Texture>,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub width: f32,
    pub height: f32,
    pub rect: Region,
    pub pos: Vector2<f32>,
    pub texture: String,
    pub collision_objects: Option<Vec<tiled::Object>>,
    //animation: Option<Animation>,
}
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
#[derive(Clone, PartialEq)]
pub enum DrawType {
    Character,
    Tile,
    NPC,
}
#[derive(Clone)]
pub struct CharacterDrawData {
    pub entity_animation: EntityAnimation,
    pub handle: DefaultBodyHandle,
    pub animation_data: AnimationData,
    pub character: Character,
    pub colliding: bool,
}
#[derive(Clone)]
pub struct TileDrawData {
    pub pos: Vec2<f32>,
    pub sprite: Sprite,
    pub rotation: f32,
}
#[derive(Clone)]
pub struct Draw {
    pub y: f32,
    pub draw_type: DrawType,
    pub player: Option<CharacterDrawData>,
    pub tile: Option<TileDrawData>,
}
#[allow(dead_code)]
impl Draw {
    pub fn draw(
        &self,
        characters: (&HashMap<usize, &str>, &HashMap<usize, &str>),
        body_set: &DefaultBodySet<f32>,
        batch: &mut SpriteBatch,
        z_val: f32,
    ) {
        match self.draw_type {
            DrawType::Tile => {
                // let tile = self.tile.as_ref().unwrap();
                // let texture = texture_map.get(&tile.sprite.texture).unwrap();
                // let position = tile.pos;
                // graphics::draw(
                //     ctx,
                //     texture,
                //     DrawParams::new()
                //         .position(Vec2::new(position.x + 16.0, position.y - 16.0))
                //         .origin(Vec2::new(8.0, 8.0))
                //         .scale(Vec2::new(SCALE, SCALE))
                //         //.clip(tile.sprite.rect)
                //         .rotation(tile.rotation.to_radians()),
                // );
                unimplemented!("Draw type not implemented yet")
            }
            DrawType::Character => {
                let player = self.player.as_ref().unwrap().clone();
                let entity_anim = player.entity_animation;
                let anim_data = player.animation_data;
                let character = player.character;
                let handle = player.handle;
                let anim = match entity_anim.direction {
                    Direction::Up => &anim_data.up,
                    Direction::Down => &anim_data.down,
                    Direction::Left => &anim_data.left,
                    Direction::Right => &anim_data.right,
                };

                let body = body_set.rigid_body(handle).unwrap();
                let pos = Vector3::new(
                    body.position().translation.x,
                    body.position().translation.y,
                    z_val,
                );

                batch.queue_sprite(
                    characters.0.get(&character.0).unwrap(),
                    pos,
                    anim.get_current_frame(),
                    0.0,
                )
            }
            DrawType::NPC => {
                let player = self.player.as_ref().unwrap().clone();
                let entity_anim = player.entity_animation;
                let anim_data = player.animation_data;
                let character = player.character;
                let handle = player.handle;
                let anim = match entity_anim.direction {
                    Direction::Up => &anim_data.up,
                    Direction::Down => &anim_data.down,
                    Direction::Left => &anim_data.left,
                    Direction::Right => &anim_data.right,
                };
                let body = body_set.rigid_body(handle).unwrap();
                let pos = Vector3::new(
                    body.position().translation.x,
                    body.position().translation.y,
                    z_val,
                );

                batch.queue_sprite(
                    characters.1.get(&character.0).unwrap(),
                    pos,
                    anim.get_current_frame(),
                    0.0,
                )
            }

            _ => unimplemented!("Invalid Draw Type"),
        }
    }
}
#[derive(PartialEq, Eq, Hash, Clone)]
pub enum AnimationKey {
    PlayerUp,
    PlayerDown,
    PlayerLeft,
    PlayerRight,
}
#[derive(Copy, Clone)]
pub struct EntityAnimation {
    pub direction: Direction,
}

#[derive(Clone, Debug)]
pub struct Anim {
    pub frames: Vec<Region>,
    pub frame_duration: Duration,
    pub time_elapsed: Duration,
    pub frame_index: usize,
}

impl Anim {
    pub fn new(frames: &[Region], frame_duration: Duration) -> Anim {
        Anim {
            frames: frames.to_vec(),
            frame_duration,
            time_elapsed: Duration::from_secs_f64(0.0),
            frame_index: 0,
        }
    }
    pub fn advance(&mut self, delta_time: Duration) {
        self.time_elapsed += delta_time;
        if self.time_elapsed > self.frame_duration {
            self.time_elapsed = Duration::from_secs_f64(0.0);
            self.frame_index += 1;
            if self.frame_index >= self.frames.len() {
                self.frame_index = 0;
            }
        }
    }

    pub fn get_current_frame(&self) -> Region {
        self.frames[self.frame_index]
    }
}
#[derive(Clone)]
pub struct AnimationData {
    pub left: Anim,
    pub right: Anim,
    pub up: Anim,
    pub down: Anim,
}

pub struct Player;
pub struct NPC;
