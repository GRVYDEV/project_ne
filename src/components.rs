use hecs::World;

use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;

use nphysics2d::object::{DefaultBodySet, DefaultColliderSet};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};

use rand::Rng;
use std::collections::HashMap;

use std::time::Duration;

use tetra::graphics::{Rectangle, Texture};

use tetra::math::Vec2;

use tiled::Layer;

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

        let state = rng.gen_range(0, 5);
        match state {
            0 => NPCState::Up,
            1 => NPCState::Down,
            2 => NPCState::Left,
            3 => NPCState::Right,
            _ => NPCState::Idle,
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
}
#[derive(Debug)]
pub struct Sprite {
    pub width: f32,
    pub height: f32,
    pub rect: Rectangle,
    pub pos: Vec2<f32>,
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
    pub frames: Vec<Rectangle>,
    pub frame_duration: Duration,
    pub time_elapsed: Duration,
    pub frame_index: usize,
}

impl Anim {
    pub fn new(frames: &[Rectangle], frame_duration: Duration) -> Anim {
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
