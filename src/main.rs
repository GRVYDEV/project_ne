use hecs::{ Entity, World};
use std::collections::HashMap;
use std::time::Duration;
use tetra::graphics::animation::Animation;
use tetra::graphics::{self, Camera, Color, DrawParams, Rectangle, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, Event, State};
use tiled::parse;
use tiled::Layer;

const WINDOW_WIDTH: f32 = 1600.0;
const WINDOW_HEIGHT: f32 = 900.0;

const CHAR_HEIGHT: f32 = 36.0;
const CHAR_WIDTH: f32 = 25.0;

const PLAYER_SPEED: f32 = 3.0;

const ANIM_SPEED: f64 = 0.2;

const TILESETS: &[(&str, &[u8])] = &[
    (
        "terrain_2",
        include_bytes!("../resources/map/tilesets/terrain_2.png"),
    ),
    (
        "outdoors",
        include_bytes!("../resources/map/tilesets/outside.png"),
    ),
];

struct GameState {
    player: Entity,
    world: World,
    sprite_map: HashMap<u32, Sprite>,
    layers: Vec<Layer>,
    texture_map: HashMap<String, Texture>,
    player_anim_map: HashMap<AnimationKey, Animation>,
}
#[derive(Debug)]
struct Sprite {
    rect: Rectangle,
    pos: Vec2<f32>,
    texture: String,
}
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
#[derive(PartialEq, Eq, Hash)]
pub enum AnimationKey {
    PlayerUp,
    PlayerDown,
    PlayerLeft,
    PlayerRight,
}
struct EntityAnimation {
    direction: Direction,
}

struct PlayerCamera(Camera);

struct Player;

// x width for char = 75
// y height for char = 144

// down = x: 0.0 y: 0.0
// right = x: 0.0, y: 36.0
// left = x: 0.0, y: 72.0
// up = x: 0.0, y: 108.

fn new_player(ctx: &mut Context, world: &mut World) -> tetra::Result<Entity> {
    let position = Vec2::new(
        (WINDOW_WIDTH / 2.0) - ((CHAR_HEIGHT / 2.0) * 2.0),
        (WINDOW_HEIGHT / 2.0) - ((CHAR_WIDTH / 2.0) * 2.0),
    );
    let mut camera = Camera::with_window_size(ctx);
    camera.position = position;

    Ok(world.spawn((
        Player,
        EntityAnimation {
            direction: Direction::Down,
        },
        PlayerCamera(camera),
        position,
    )))
}

fn player_update(
    ctx: &mut Context,
    world: &mut World,
    anim_map: &mut HashMap<AnimationKey, Animation>,
) {
    for (_id, (pos, camera, anim, _player)) in &mut world.query::<(
        &mut Vec2<f32>,
        &mut PlayerCamera,
        &mut EntityAnimation,
        &Player,
    )>() {
        if input::is_key_down(ctx, Key::W) {
            pos.y -= PLAYER_SPEED;
            camera.0.position.y -= PLAYER_SPEED;
            anim.direction = Direction::Up;
            anim_map
                .get_mut(&AnimationKey::PlayerUp)
                .unwrap()
                .advance(ctx);
        }
        if input::is_key_down(ctx, Key::S) {
            pos.y += PLAYER_SPEED;
            camera.0.position.y += PLAYER_SPEED;
            anim.direction = Direction::Down;
            anim_map
                .get_mut(&AnimationKey::PlayerDown)
                .unwrap()
                .advance(ctx);
        }
        if input::is_key_down(ctx, Key::D) {
            pos.x += PLAYER_SPEED;
            camera.0.position.x += PLAYER_SPEED;
            anim.direction = Direction::Right;
            anim_map
                .get_mut(&AnimationKey::PlayerRight)
                .unwrap()
                .advance(ctx);
        }
        if input::is_key_down(ctx, Key::A) {
            pos.x -= PLAYER_SPEED;
            camera.0.position.x -= PLAYER_SPEED;
            anim.direction = Direction::Left;
            anim_map
                .get_mut(&AnimationKey::PlayerLeft)
                .unwrap()
                .advance(ctx);
        }
        camera.0.update();
    }
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let mut world = World::new();
        let mut file_to_texture = HashMap::new();
        for (k, v) in TILESETS {
            file_to_texture
                .entry(k.to_string())
                .or_insert(Texture::from_file_data(ctx, v)?);
        }
        let player_sheet = Texture::from_file_data(ctx, include_bytes!("../resources/chara5.png"))?;
        let up = Animation::new(
            player_sheet.clone(),
            Rectangle::row(0.0, 108.0, CHAR_WIDTH, CHAR_HEIGHT)
                .take(3)
                .collect(),
            Duration::from_secs_f64(ANIM_SPEED),
        );
        let down = Animation::new(
            player_sheet.clone(),
            Rectangle::row(0.0, 0.0, CHAR_WIDTH, CHAR_HEIGHT)
                .take(3)
                .collect(),
            Duration::from_secs_f64(ANIM_SPEED),
        );
        let left = Animation::new(
            player_sheet.clone(),
            Rectangle::row(0.0, 36.0, CHAR_WIDTH, CHAR_HEIGHT)
                .take(3)
                .collect(),
            Duration::from_secs_f64(ANIM_SPEED),
        );
        let right = Animation::new(
            player_sheet.clone(),
            Rectangle::row(0.0, 72.0, CHAR_WIDTH, CHAR_HEIGHT)
                .take(3)
                .collect(),
            Duration::from_secs_f64(ANIM_SPEED),
        );

        let mut anims = HashMap::new();
        anims.insert(AnimationKey::PlayerUp, up);
        anims.insert(AnimationKey::PlayerDown, down);
        anims.insert(AnimationKey::PlayerLeft, left);
        anims.insert(AnimationKey::PlayerRight, right);
        //println!("{:?}", file_to_texture.get(&"terrain").unwrap());

        let tiled_data = parse(&include_bytes!("../resources/map/map2.tmx")[..]).unwrap();

        let tilesets = tiled_data.tilesets;
        let mut tile_sprites: HashMap<u32, Sprite> = HashMap::new();
        let mut gid = tilesets[0].first_gid as u32;
        for x in 0..tilesets.len() {
            let map_tileset = &tilesets[x];
            let tile_width = map_tileset.tile_width as i32;
            let tile_height = map_tileset.tile_height as i32;
            let tileset_width = &map_tileset.images[0].width;
            let tileset_height = &map_tileset.images[0].height;
            let tileset_sprite_columns = tileset_width / tile_width as i32;
            let tileset_sprite_rows = tileset_height / tile_height as i32;
            for x in 0..tileset_sprite_rows {
                for y in 0..tileset_sprite_columns {
                    let sprite_w = tile_width as f32;
                    let sprite_h = tile_height as f32;
                    let pos_x = (x * tile_width) as f32;
                    let pos_y = (y * tile_height) as f32;

                    let sprite = Sprite {
                        rect: Rectangle::new(pos_y, pos_x, sprite_w, sprite_h),
                        pos: Vec2::new(pos_x, pos_y),
                        texture: map_tileset.name.clone(),
                    };

                    tile_sprites.entry(gid).or_insert(sprite);
                    gid += 1;
                }
            }
        }
        //fs::write("sprite.txt", format!("{:#?}", tile_sprites)).unwrap();

        Ok(GameState {
            player: new_player(ctx, &mut world)?,
            world,
            player_anim_map: anims,
            sprite_map: tile_sprites,
            layers: tiled_data.layers,
            texture_map: file_to_texture,
        })
    }
}

fn draw_layer(lyr: tiled::Layer, ste: &mut GameState, ctx: &mut Context) {
    for (y, row) in lyr.tiles.iter().enumerate().clone() {
        for (x, &tile) in row.iter().enumerate() {
            if tile.gid == 0 {
                continue;
            }

            let gid = tile.gid;
            let sprite = ste.sprite_map.get(&gid).unwrap();
            //println!("{:?}", &sprite.texture);
            let texture = ste.texture_map.get(&sprite.texture).unwrap();
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
            graphics::draw(
                ctx,
                texture,
                DrawParams::new()
                    .position(Vec2::new(x as f32 * 32.0, y as f32 * 32.0))
                    .origin(Vec2::new(8.0, 8.0))
                    .scale(Vec2::new(2.0, 2.0))
                    .clip(sprite.rect)
                    .rotation(rotation.to_radians()),
            );
        }
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        //&self.texture.set_current_frame_index(1);
        for (_id, camera) in self.world.query::<&PlayerCamera>().iter().take(1) {
            graphics::set_transform_matrix(ctx, camera.0.as_matrix());
        }
        graphics::clear(ctx, Color::rgb(0.0, 0.0, 0.0));
        let mut layers = self.layers.clone();
        let bg_layer: tiled::Layer = layers.remove(0);

        draw_layer(bg_layer.clone(), self, ctx);

        for (_id, (pos, _camera, anim, _player)) in
            &mut self
                .world
                .query::<(&Vec2<f32>, &PlayerCamera, &EntityAnimation, &Player)>()
        {
            let key = match anim.direction {
                Direction::Up => AnimationKey::PlayerUp,
                Direction::Down => AnimationKey::PlayerDown,
                Direction::Left => AnimationKey::PlayerLeft,
                Direction::Right => AnimationKey::PlayerRight,
            };
            let animation = self.player_anim_map.get(&key).unwrap();
            graphics::draw(
                ctx,
                animation,
                DrawParams::new()
                    .position(pos.clone())
                    .origin(Vec2::new(18.0, 12.5))
                    .scale(Vec2::new(2.0, 2.0)),
            );
        }

        //graphics::draw(ctx, &self.player, DrawParams::default());

        for x in layers {
            draw_layer(x, self, ctx);
        }

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        player_update(ctx, &mut self.world, &mut self.player_anim_map);

        Ok(())
    }

    fn event(&mut self, _ctx: &mut Context, event: Event) -> tetra::Result {
        if let Event::Resized { width, height } = event {
            for (_id, camera) in self.world.query::<&mut PlayerCamera>().iter().take(1) {
                camera.0.set_viewport_size(width as f32, height as f32);
                camera.0.update();
            }
        }
        Ok(())
    }
}

fn main() -> tetra::Result {
    // #[derive(Debug)]
    // println!("{:#?}", tiled_data);
    //fs::write("foo.txt", format!("{:#?}", tiled_data)).unwrap();
    ContextBuilder::new("Neon", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .resizable(true)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
