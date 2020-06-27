use std::time::Duration;

extern crate tiled;
use std::fs;
use std::io::prelude::*;
use std::io::Read;
use std::path::Path;
use tetra::graphics::animation::Animation;
use tetra::graphics::{self, Color, DrawParams, Drawable, Rectangle, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};
use tiled::parse;

const WINDOW_HEIGHT: f32 = 1600.0;
const WINDOW_WIDTH: f32 = 900.0;

const CHAR_HEIGHT: f32 = 36.0;
const CHAR_WIDTH: f32 = 25.0;

const PLAYER_SPEED: f32 = 2.0;

const ANIM_SPEED: f64 = 0.3;

struct GameState {
    player: Player,
    bg: Texture,
    fg: Texture
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Player {
    up: Animation,
    down: Animation,
    left: Animation,
    right: Animation,
    position: Vec2<f32>,
    direction: Direction,
}

// x width for char = 75
// y height for char = 144

// down = x: 0.0 y: 0.0
// right = x: 0.0, y: 36.0
// left = x: 0.0, y: 72.0
// up = x: 0.0, y: 108.
impl Player {
    fn new(ctx: &mut Context, texture: &Texture) -> tetra::Result<Player> {
        let position = Vec2::new(
            (WINDOW_HEIGHT / 2.0) - ((CHAR_HEIGHT / 2.0) * 2.5),
            (WINDOW_WIDTH / 2.0) - ((CHAR_WIDTH / 2.0) * 2.5),
        );

        let up = Animation::new(
            texture.clone(),
            Rectangle::row(0.0, 108.0, CHAR_WIDTH, CHAR_HEIGHT)
                .take(3)
                .collect(),
            Duration::from_secs_f64(ANIM_SPEED),
        );
        let down = Animation::new(
            texture.clone(),
            Rectangle::row(0.0, 0.0, CHAR_WIDTH, CHAR_HEIGHT)
                .take(3)
                .collect(),
            Duration::from_secs_f64(ANIM_SPEED),
        );
        let left = Animation::new(
            texture.clone(),
            Rectangle::row(0.0, 36.0, CHAR_WIDTH, CHAR_HEIGHT)
                .take(3)
                .collect(),
            Duration::from_secs_f64(ANIM_SPEED),
        );
        let right = Animation::new(
            texture.clone(),
            Rectangle::row(0.0, 72.0, CHAR_WIDTH, CHAR_HEIGHT)
                .take(3)
                .collect(),
            Duration::from_secs_f64(ANIM_SPEED),
        );

        Ok(Player {
            up,
            down,
            left,
            right,
            position,
            direction: Direction::Down,
        })
    }

    fn update(&mut self, ctx: &mut Context) {
        if input::is_key_down(ctx, Key::W) {
            self.position.y -= PLAYER_SPEED;
            self.direction = Direction::Up;
            self.up.advance(ctx);
        }
        if input::is_key_down(ctx, Key::S) {
            self.position.y += PLAYER_SPEED;
            self.direction = Direction::Down;
            self.down.advance(ctx);
        }
        if input::is_key_down(ctx, Key::D) {
            self.position.x += PLAYER_SPEED;
            self.direction = Direction::Right;
            self.right.advance(ctx);
        }
        if input::is_key_down(ctx, Key::A) {
            self.position.x -= PLAYER_SPEED;
            self.direction = Direction::Left;
            self.left.advance(ctx);
        }
    }
}

impl Drawable for Player {
    fn draw<P>(&self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>,
    {
        let anim = match self.direction {
            Direction::Up => &self.up,
            Direction::Down => &self.down,
            Direction::Left => &self.left,
            Direction::Right => &self.right,
        };
        graphics::draw(
            ctx,
            anim,
            DrawParams::new()
                .position(self.position)
                .origin(Vec2::new(0.0, 0.0))
                .scale(Vec2::new(2.5, 2.5)),
        )
    }
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let texture = Texture::from_file_data(ctx, include_bytes!("../resources/chara5.png"))?;
        let bg = Texture::from_file_data(ctx, include_bytes!("../resources/map/map2bg.png"))?;
        let fg = Texture::from_file_data(ctx, include_bytes!("../resources/map/map2trees.png"))?;
        Ok(GameState {
            player: Player::new(ctx, &texture)?,
            bg,
            fg
        })
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        //&self.texture.set_current_frame_index(1);
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        graphics::draw(
            ctx,
            &self.bg,
            DrawParams::new()
                .position(Vec2::new(0.0, 0.0))
                .origin(Vec2::new(0.0, 0.0))
                .scale(Vec2::new(2.5, 2.5)),
        );
        graphics::draw(ctx, &self.player, DrawParams::default());
        graphics::draw(
            ctx,
            &self.fg,
            DrawParams::new()
                .position(Vec2::new(0.0, 0.0))
                .origin(Vec2::new(0.0, 0.0))
                .scale(Vec2::new(2.5, 2.5)),
        );
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.player.update(ctx);

        Ok(())
    }
}

fn main() -> tetra::Result {
    let tiled_data = tiled::parse(&include_bytes!("../resources/map/map2.tmx")[..]).unwrap();
    // #[derive(Debug)]
    // println!("{:#?}", tiled_data);
    //fs::write("foo.txt", format!("{:#?}", tiled_data)).unwrap();
    //let mut file = File::create("output.txt").unwrap();
    //file.write_all(tiled_data);
    ContextBuilder::new("Neon", WINDOW_HEIGHT as i32, WINDOW_WIDTH as i32)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
