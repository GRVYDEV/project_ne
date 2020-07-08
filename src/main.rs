use hecs::{Entity, World};
use nalgebra::base::{Unit, Vector2};
use nalgebra::geometry::{Isometry2, Point2, Translation2};
use nalgebra::Vector3;
use ncollide2d::pipeline::{
    object, CollisionGroups, CollisionWorld, ContactEvent, GeometricQueryType, ProximityEvent,
};
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Ball, ConvexPolygon, Cuboid, Plane, Polyline, ShapeHandle};
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::math::{Force, ForceType, Velocity};
use nphysics2d::object::{
    BodyPartHandle, BodyStatus, Collider, ColliderDesc, DefaultBodyHandle, DefaultBodySet,
    DefaultColliderSet, RigidBody, RigidBodyDesc,
};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use tetra::graphics::animation::Animation;
use tetra::graphics::{self, Camera, Color, DrawParams, Rectangle, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, Event, State};
use tiled::parse;
use tiled::Layer;
use tiled::ObjectShape;

const WINDOW_WIDTH: f32 = 1600.0;
const WINDOW_HEIGHT: f32 = 900.0;

const SCALE: f32 = 2.0;

const CHAR_HEIGHT: f32 = 32.0;
const CHAR_WIDTH: f32 = 19.0;

const PLAYER_SPEED: f32 = 3.0 * 75.0;

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
    (
        "chest-sheet",
        include_bytes!("../resources/map/tilesets/chest-sheet.png"),
    ),
];

struct LastDirection(Direction);
struct Character(usize, usize);

struct GameState {
    world: World,
    sprite_map: HashMap<u32, Sprite>,
    layers: Vec<Layer>,
    texture_map: HashMap<String, Texture>,
    player_anim_map: HashMap<AnimationKey, Animation>,
    mechanical_world: DefaultMechanicalWorld<f32>,
    geometrical_world: DefaultGeometricalWorld<f32>,
    body_set: DefaultBodySet<f32>,
    collider_set: DefaultColliderSet<f32>,
    constraint_set: DefaultJointConstraintSet<f32>,
    force_gen_set: DefaultForceGeneratorSet<f32>,
    characters: HashMap<usize, Texture>,
}
#[derive(Debug)]
struct Sprite {
    width: f32,
    height: f32,
    rect: Rectangle,
    pos: Vec2<f32>,
    texture: String,
    collision_objects: Option<Vec<tiled::Object>>,
    //animation: Option<Animation>,
}
#[derive(PartialEq, Eq, Hash)]
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

struct Player;

// x width for char = 75
// y height for char = 144

// down = x: 0.0 y: 0.0
// right = x: 0.0, y: 36.0
// left = x: 0.0, y: 72.0
// up = x: 0.0, y: 108.
fn get_layer_size(lyr: tiled::Layer) -> Vec2<u32> {
    let mut size_y = 0;
    let mut size_x = 0;
    for (y, row) in lyr.tiles.iter().enumerate().clone() {
        for (x, &tile) in row.iter().enumerate() {
            if tile.gid == 0 {
                continue;
            }
            size_x += 1;
        }
        size_y += 1;
    }
    return Vec2::new(size_x, size_y);
}

fn create_map_bounds(
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
fn create_physics_world(
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

                        //println!("Points: {:?}", points);
                    } //ConvexPolygon::try_new(points).unwrap()
                    let shape = ShapeHandle::new(Cuboid::new(Vector2::new(width, height)));

                    let shape_pos = Isometry2::new(
                        Vector2::new((x as f32 * 32.0) + 12.0, y as f32 * 32.0),
                        nalgebra::zero(),
                    );
                    let mut translator: (f32, f32) = (0.0, 0.0);
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
fn draw_layer(
    lyr: tiled::Layer,
    texture_map: &HashMap<String, Texture>,
    sprite_map: &HashMap<u32, Sprite>,
    ctx: &mut Context,
) {
    for (y, row) in lyr.tiles.iter().enumerate().clone() {
        for (x, &tile) in row.iter().enumerate() {
            if tile.gid == 0 {
                continue;
            }

            let gid = tile.gid;
            let sprite = sprite_map.get(&gid).unwrap();
            let mut scale = 1.0;
            let mut origin = Vec2::new(8.0, 8.0);
            if sprite.width == 16.0 && sprite.height == 16.0 {
                scale = 2.0;
                origin.x = 8.0;
                origin.y = 8.0;
            }

            let rect: Rectangle;
            let texture = texture_map.get(&sprite.texture).unwrap();
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
                    .position(Vec2::new(
                        (x as f32 * 32.0) + 16.0,
                        (y as f32 * 32.0) + 16.0,
                    ))
                    .origin(origin)
                    .scale(Vec2::new(SCALE, SCALE))
                    .clip(sprite.rect)
                    .rotation(rotation.to_radians()),
            );
        }
    }
}

fn new_player(
    ctx: &mut Context,
    world: &mut World,
    body: DefaultBodyHandle,
    char_count: usize,
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
    )))
}

fn player_update(
    body_set: &mut DefaultBodySet<f32>,
    ctx: &mut Context,
    world: &mut World,
    anim_map: &mut HashMap<AnimationKey, Animation>,
) {
    for (_id, (camera, anim, _player, handle, character)) in &mut world.query::<(
        &mut Camera,
        &mut EntityAnimation,
        &Player,
        &DefaultBodyHandle,
        &mut Character,
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
            anim_map
                .get_mut(&AnimationKey::PlayerUp)
                .unwrap()
                .advance(ctx);
        }
        if input::is_key_down(ctx, Key::S) {
            player_body.set_linear_velocity(Vector2::new(0.0, PLAYER_SPEED));
            anim.direction = Direction::Down;
            anim_map
                .get_mut(&AnimationKey::PlayerDown)
                .unwrap()
                .advance(ctx);
        }
        if input::is_key_down(ctx, Key::D) {
            player_body.set_linear_velocity(Vector2::new(PLAYER_SPEED, 0.0));
            anim.direction = Direction::Right;
            anim_map
                .get_mut(&AnimationKey::PlayerRight)
                .unwrap()
                .advance(ctx);
        }
        if input::is_key_down(ctx, Key::A) {
            player_body.set_linear_velocity(Vector2::new(-PLAYER_SPEED, 0.0));
            anim.direction = Direction::Left;
            anim_map
                .get_mut(&AnimationKey::PlayerLeft)
                .unwrap()
                .advance(ctx);
        }
    }
}

// fn spaw_npcs(count: u32, colliders: &mut DefaultColliderSet, bodies: &mut)

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let mut world = World::new();
        let mut file_to_texture = HashMap::new();
        for (k, v) in TILESETS {
            file_to_texture
                .entry(k.to_string())
                .or_insert(Texture::from_file_data(ctx, v)?);
        }
        let player_sheets = [
            Texture::from_file_data(ctx, include_bytes!("../resources/Wizard-Sheet.png"))?,
            Texture::from_file_data(ctx, include_bytes!("../resources/Viking-Sheet.png"))?,
            Texture::from_file_data(ctx, include_bytes!("../resources/Fire-Man-Sheet.png"))?,
        ];

        let mut character_map = HashMap::new();
        for x in 0..player_sheets.len() {
            character_map.insert(x, player_sheets.get(x).unwrap().clone());
        }
        let up = Animation::new(
            player_sheets[0].clone(),
            Rectangle::row(0.0, 96.0, CHAR_WIDTH, CHAR_HEIGHT)
                .take(3)
                .collect(),
            Duration::from_secs_f64(ANIM_SPEED),
        );
        let down = Animation::new(
            player_sheets[0].clone(),
            Rectangle::row(0.0, 0.0, CHAR_WIDTH, CHAR_HEIGHT)
                .take(3)
                .collect(),
            Duration::from_secs_f64(ANIM_SPEED),
        );
        let left = Animation::new(
            player_sheets[0].clone(),
            Rectangle::row(0.0, 32.0, CHAR_WIDTH, CHAR_HEIGHT)
                .take(3)
                .collect(),
            Duration::from_secs_f64(ANIM_SPEED),
        );
        let right = Animation::new(
            player_sheets[0].clone(),
            Rectangle::row(0.0, 64.0, CHAR_WIDTH, CHAR_HEIGHT)
                .take(3)
                .collect(),
            Duration::from_secs_f64(ANIM_SPEED),
        );

        let mut anims = HashMap::new();
        anims.insert(AnimationKey::PlayerUp, up);
        anims.insert(AnimationKey::PlayerDown, down);
        anims.insert(AnimationKey::PlayerLeft, left);
        anims.insert(AnimationKey::PlayerRight, right);

        let tiled_data = parse(&include_bytes!("../resources/map/map3.tmx")[..]).unwrap();
        //fs::write("bar.json", format!("{:#?}", tiled_data)).unwrap();
        let tilesets = tiled_data.tilesets;
        let mut tile_sprites: HashMap<u32, Sprite> = HashMap::new();
        let mut gid = tilesets[0].first_gid as u32;
        for x in 0..tilesets.len() {
            let map_tileset = tilesets[x].clone();
            let tile_width = map_tileset.tile_width as i32;
            let tile_height = map_tileset.tile_height as i32;
            let tileset_width = &map_tileset.images[0].width;
            let tileset_height = &map_tileset.images[0].height;
            let tileset_sprite_columns = tileset_width / tile_width as i32;
            let tileset_sprite_rows = tileset_height / tile_height as i32;
            let mut object_map: HashMap<u32, Vec<tiled::Object>> = HashMap::new();
            let mut id_to_rect: HashMap<u32, Rectangle> = HashMap::new();
            let mut anim_map: HashMap<u32, Animation> = HashMap::new();
            for tile in map_tileset.tiles {
                if tile.objectgroup.is_some() {
                    object_map.insert(tile.id, tile.objectgroup.unwrap().objects);
                }
                // if tile.animation.is_some() && id_to_rect.is_empty() {
                //     let mut id = 0;
                //     for x in 0..tileset_sprite_rows {
                //         for y in 0..tileset_sprite_columns {
                //             let sprite_w = tile_width as f32;
                //             let sprite_h = tile_height as f32;
                //             let pos_x = (x * tile_width) as f32;
                //             let pos_y = (y * tile_height) as f32;
                //             Rectangle::new(pos_y, pos_x, sprite_w, sprite_h)
                //             tile_sprites.entry(gid).or_insert(sprite);
                //             id += 1;
                //         }
                //     }
                // }
            }
            let mut id = 0;
            for x in 0..tileset_sprite_rows {
                for y in 0..tileset_sprite_columns {
                    let sprite_w = tile_width as f32;
                    let sprite_h = tile_height as f32;
                    let pos_x = (x * tile_width) as f32;
                    let pos_y = (y * tile_height) as f32;
                    let objects = object_map.remove(&id).clone();
                    let sprite = Sprite {
                        width: sprite_w,
                        height: sprite_h,
                        rect: Rectangle::new(pos_y, pos_x, sprite_w, sprite_h),
                        pos: Vec2::new(pos_x, pos_y),
                        texture: map_tileset.name.clone(),
                        collision_objects: objects,
                    };

                    tile_sprites.entry(gid).or_insert(sprite);
                    gid += 1;
                    id += 1;
                }
            }
        }

        let player_shape = ShapeHandle::new(Cuboid::new(Vector2::new(10.5, 10.0)));

        let player_pos = Isometry2::new(Vector2::new(800.0, 800.0), nalgebra::zero());

        let geometrical_world: DefaultGeometricalWorld<f32> = DefaultGeometricalWorld::new();
        let mechanical_world: DefaultMechanicalWorld<f32> =
            DefaultMechanicalWorld::new(Vector2::new(0.0, 0.0));
        let mut bodies = DefaultBodySet::new();
        let mut colliders = DefaultColliderSet::new();
        let joint_constraints: DefaultJointConstraintSet<f32> = DefaultJointConstraintSet::new();
        let force_generators: DefaultForceGeneratorSet<f32> = DefaultForceGeneratorSet::new();
        let layers = tiled_data.layers;
        create_map_bounds(&layers[0], &mut colliders, &mut bodies);
        let player_body = RigidBodyDesc::new()
            .position(player_pos)
            .gravity_enabled(false)
            .status(BodyStatus::Dynamic)
            .mass(1.2)
            .build();
        let player_handle = bodies.insert(player_body);

        new_player(
            ctx,
            &mut world,
            player_handle.clone(),
            character_map.len() - 1,
        )
        .expect("Failed to create Player");

        let player_collider =
            ColliderDesc::new(player_shape).build(BodyPartHandle(player_handle, 0));

        colliders.insert(player_collider);

        //fs::write("sprite.txt", format!("{:#?}", tile_sprites)).unwrap();

        create_physics_world(&layers, &tile_sprites, &mut colliders, &mut bodies);

        Ok(GameState {
            characters: character_map,
            world,
            player_anim_map: anims,
            sprite_map: tile_sprites,
            layers: layers,
            texture_map: file_to_texture,
            mechanical_world: mechanical_world,
            geometrical_world: geometrical_world,
            body_set: bodies,
            collider_set: colliders,
            force_gen_set: force_generators,
            constraint_set: joint_constraints,
        })
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        //&self.texture.set_current_frame_index(1);
        for (_id, (camera, _player, character)) in self
            .world
            .query::<(&Camera, &Player, &Character)>()
            .iter()
            .take(1)
        {
            graphics::set_transform_matrix(ctx, camera.as_matrix());
        }
        graphics::clear(ctx, Color::rgb(0.0, 0.0, 0.0));
        for (_id, (_camera, anim, _player, handle, last_direction, character)) in
            &mut self.world.query::<(
                &Camera,
                &EntityAnimation,
                &Player,
                &DefaultBodyHandle,
                &mut LastDirection,
                &Character,
            )>()
        {
            let mut layers = self.layers.clone();
            let bg_layer: tiled::Layer = layers.remove(0);
            let bg_layer_2: tiled::Layer = layers.remove(0);
            draw_layer(bg_layer.clone(), &self.texture_map, &self.sprite_map, ctx);
            let key = match anim.direction {
                Direction::Up => AnimationKey::PlayerUp,
                Direction::Down => AnimationKey::PlayerDown,
                Direction::Left => AnimationKey::PlayerLeft,
                Direction::Right => AnimationKey::PlayerRight,
            };
            let animation = self.player_anim_map.get_mut(&key).unwrap();
            animation.set_texture(self.characters.get(&character.0).unwrap().clone());
            let player_body = self.body_set.rigid_body(*handle).unwrap();
            let player_pos = Vec2::new(
                player_body.position().translation.vector.x,
                player_body.position().translation.vector.y,
            );
            if key == AnimationKey::PlayerDown {
                if last_direction.0 == Direction::Up {
                    draw_layer(bg_layer_2.clone(), &self.texture_map, &self.sprite_map, ctx);
                    graphics::draw(
                        ctx,
                        animation,
                        DrawParams::new()
                            .position(player_pos)
                            .origin(Vec2::new(9.5, 27.0))
                            .scale(Vec2::new(SCALE, SCALE)),
                    );
                    for x in layers {
                        draw_layer(x.clone(), &self.texture_map, &self.sprite_map, ctx);
                    }
                } else {
                    graphics::draw(
                        ctx,
                        animation,
                        DrawParams::new()
                            .position(player_pos)
                            .origin(Vec2::new(9.5, 27.0))
                            .scale(Vec2::new(SCALE, SCALE)),
                    );
                    draw_layer(bg_layer_2.clone(), &self.texture_map, &self.sprite_map, ctx);
                    for x in layers {
                        draw_layer(x, &self.texture_map, &self.sprite_map, ctx);
                    }
                }
                last_direction.0 = Direction::Down;
            } else if key == AnimationKey::PlayerUp {
                if last_direction.0 == Direction::Down {
                    graphics::draw(
                        ctx,
                        animation,
                        DrawParams::new()
                            .position(player_pos)
                            .origin(Vec2::new(9.5, 27.0))
                            .scale(Vec2::new(SCALE, SCALE)),
                    );
                    draw_layer(bg_layer_2.clone(), &self.texture_map, &self.sprite_map, ctx);
                    for x in layers {
                        draw_layer(x, &self.texture_map, &self.sprite_map, ctx);
                    }
                } else {
                    draw_layer(bg_layer_2.clone(), &self.texture_map, &self.sprite_map, ctx);
                    graphics::draw(
                        ctx,
                        animation,
                        DrawParams::new()
                            .position(player_pos)
                            .origin(Vec2::new(9.5, 27.0))
                            .scale(Vec2::new(SCALE, SCALE)),
                    );
                    for x in layers {
                        draw_layer(x.clone(), &self.texture_map, &self.sprite_map, ctx);
                    }
                }
                last_direction.0 = Direction::Up;
            } else if key == AnimationKey::PlayerLeft || key == AnimationKey::PlayerRight {
                if last_direction.0 == Direction::Down {
                    graphics::draw(
                        ctx,
                        animation,
                        DrawParams::new()
                            .position(player_pos)
                            .origin(Vec2::new(9.5, 27.0))
                            .scale(Vec2::new(SCALE, SCALE)),
                    );
                    draw_layer(bg_layer_2.clone(), &self.texture_map, &self.sprite_map, ctx);
                    for x in layers {
                        draw_layer(x, &self.texture_map, &self.sprite_map, ctx);
                    }
                } else if last_direction.0 == Direction::Up {
                    draw_layer(bg_layer_2.clone(), &self.texture_map, &self.sprite_map, ctx);
                    graphics::draw(
                        ctx,
                        animation,
                        DrawParams::new()
                            .position(player_pos)
                            .origin(Vec2::new(9.5, 27.0))
                            .scale(Vec2::new(SCALE, SCALE)),
                    );
                    for x in layers {
                        draw_layer(x.clone(), &self.texture_map, &self.sprite_map, ctx);
                    }
                }
            }
        }

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        player_update(
            &mut self.body_set,
            ctx,
            &mut self.world,
            &mut self.player_anim_map,
        );
        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.body_set,
            &mut self.collider_set,
            &mut self.constraint_set,
            &mut self.force_gen_set,
        );

        for (_id, (camera, _player, handle)) in
            &mut self
                .world
                .query::<(&mut Camera, &Player, &DefaultBodyHandle)>()
        {
            let player_body = self.body_set.rigid_body_mut(*handle).unwrap();
            player_body.set_linear_velocity(Vector2::new(0.0, 0.0));
            camera.position = Vec2::new(
                player_body.position().translation.vector.x,
                player_body.position().translation.vector.y,
            );
            camera.update();
        }

        Ok(())
    }

    fn event(&mut self, _ctx: &mut Context, event: Event) -> tetra::Result {
        if let Event::Resized { width, height } = event {
            for (_id, camera) in self.world.query::<&mut Camera>().iter().take(1) {
                camera.set_viewport_size(width as f32, height as f32);
                camera.update();
            }
        }
        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Neon", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .resizable(true)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
