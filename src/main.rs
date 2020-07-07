use hecs::{Entity, World};
use nalgebra::base::{Unit, Vector2};
use nalgebra::geometry::{Isometry2, Point2, Translation2};
use nalgebra::Vector3;
use ncollide2d::pipeline::{
    object, CollisionGroups, CollisionWorld, ContactEvent, GeometricQueryType, ProximityEvent,
};
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Ball, ConvexPolygon, Cuboid, Plane, ShapeHandle};
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
#[derive(Debug)]
struct CanMove {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

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
}
#[derive(Debug)]
struct Sprite {
    rect: Rectangle,
    pos: Vec2<f32>,
    texture: String,
    collision_objects: Option<Vec<tiled::Object>>,
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

struct Player;

#[derive(Clone, PartialEq)]
pub enum CollisionType {
    Player,
    World,
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone)]
struct CollisionObjectData {
    pub name: &'static str,
    pub collision_type: CollisionType,
}

impl CollisionObjectData {
    pub fn new(name: &'static str, col_type: CollisionType) -> CollisionObjectData {
        CollisionObjectData {
            name: name,
            collision_type: col_type,
        }
    }
}

fn handle_proximity_event(
    world: &CollisionWorld<f32, CollisionObjectData>,
    event: &ProximityEvent<object::CollisionObjectSlabHandle>,
) {
    let area_name;
    let co1 = world.collision_object(event.collider1).unwrap();
    let co2 = world.collision_object(event.collider2).unwrap();

    if co1.data().collision_type == CollisionType::World {
        area_name = co1.data().name;
    } else {
        area_name = co2.data().name;
    }

    if event.new_status == Proximity::Intersecting {
        println!("The player is in the area");
    } else if event.new_status == Proximity::Disjoint {
        println!("The player is not in the area");
    }
}

fn handle_contact_event(
    world: &CollisionWorld<f32, CollisionObjectData>,
    event: &ContactEvent<object::CollisionObjectSlabHandle>,
    ecs_world: &mut World,
) {
    if let &ContactEvent::Started(collider1, collider2) = event {
        if world.contact_pair(collider1, collider2, true).is_some() {
            let pair = world.contact_pair(collider1, collider2, true).unwrap();
            let contact = pair.3.deepest_contact().unwrap();
            let co1 = world.collision_object(collider1).unwrap();
            let co2 = world.collision_object(collider2).unwrap();
            for (_id, (_player, can_move)) in
                ecs_world.query::<(&Player, &mut CanMove)>().iter().take(1)
            {
                if co1.data().collision_type == CollisionType::Player {
                    println!("Player is Co1");
                    let normal = contact.contact.normal.into_inner().data;
                    //println!("Contact World1 {:?}, World 2 {:?}", contact.contact.world1, contact.contact.world2);
                    let normals = [normal[0] as f32, normal[1] as f32];
                    let player_contact = [
                        contact.contact.world1.coords.data[0],
                        contact.contact.world1.coords.data[1],
                    ];
                    println!("Normal {:?}", normal);
                    let player_pos = [
                        co1.position().translation.vector.data[0],
                        co1.position().translation.vector.data[1],
                    ];
                    let local_frame = [
                        (player_contact[0] / player_pos[0]) as f32,
                        (player_contact[1] / player_pos[1]) as f32,
                    ];
                    let pos = [(local_frame[0] - 8.0).abs(), (local_frame[1] - 8.0).abs()];
                    let neg = [
                        (local_frame[0] - (-8.0)).abs(),
                        (local_frame[1] - (-8.0)).abs(),
                    ];
                    println!("Pos: {:?} Neg: {:?}", pos, neg);
                    println!("Local Frame {:?}", local_frame);
                    if normals[0] <= -0.1 && normal[1] <= 0.0 {
                        can_move.left = false
                    } else if normals[0] > 0.0 && normal[1] >= 0.0 {
                        can_move.right = false;
                    } else if normals[0] <= -0.0 && normal[1] <= -0.0 {
                        can_move.up = false;
                    } else if normals[0] <= 0.0 && normal[1] >= 0.0 {
                        can_move.down = false;
                    }
                // println!(
                //     "Handling event...{} Can i move? {}",
                //     co2.data().name,
                //     format!("{:#?}", can_move)
                // );
                } else if co2.data().collision_type == CollisionType::Player {
                    println!("Player is Co2");
                    let normal = (-contact.contact.normal).into_inner().data;
                    //println!("Contact World1 {:?}, World 2 {:?}", contact.contact.world1, contact.contact.world2);
                    let normals = [normal[0] as f32, normal[1] as f32];
                    println!("Normal {:?}", normals);
                    let player_contact = [
                        contact.contact.world2.coords.data[0],
                        contact.contact.world2.coords.data[1],
                    ];
                    let player_pos = [
                        co2.position().translation.vector.data[0],
                        co2.position().translation.vector.data[1],
                    ];
                    let local_frame = [
                        player_contact[0] / player_pos[0],
                        player_contact[1] / player_pos[1],
                    ];
                    println!(
                        "Player Contact: {:?} Player Pos: {:?}",
                        player_contact, player_pos
                    );
                    println!("Local Frame {:?}", local_frame);
                    if normals[0] <= -0.0 && normal[1] <= 0.0 {
                        can_move.left = false
                    } else if normals[0] > 0.0 && normal[1] >= 0.0 {
                        can_move.right = false;
                    } else if normals[0] <= -0.0 && normal[1] <= -0.0 {
                        can_move.up = false;
                    } else if normals[0] <= 0.0 && normal[1] >= 0.0 {
                        can_move.down = false;
                    }
                    // println!(
                    //     "Handling event...{} Can i move? {}",
                    //     co1.data().name,
                    //     format!("{:#?}", can_move)
                    // );
                }
            }
        }
    }
    if let &ContactEvent::Stopped(collider1, collider2) = event {
        if world.contact_pair(collider1, collider2, true).is_some() {
            let pair = world.contact_pair(collider1, collider2, false).unwrap();
            let contact = pair.3.deepest_contact().unwrap();
            let co1 = world.collision_object(collider1).unwrap();
            let co2 = world.collision_object(collider2).unwrap();
            for (_id, (_player, can_move)) in
                ecs_world.query::<(&Player, &mut CanMove)>().iter().take(1)
            {
                if co1.data().collision_type == CollisionType::Player {
                    let normal = contact.contact.normal.into_inner().data;
                    //println!("Stop Contact World1 {:?}, World 2 {:?}", contact.contact.world1, contact.contact.world2);
                    let normals = [normal[0] as f32, normal[1] as f32];
                    if normals[0] <= -0.0 && normal[1] <= 0.0 {
                        can_move.left = true
                    } else if normals[0] > 0.0 && normal[1] >= 0.0 {
                        can_move.right = true;
                    } else if normals[0] <= -0.0 && normal[1] <= -0.0 {
                        can_move.up = true;
                    } else if normals[0] <= 0.0 && normal[1] >= 0.0 {
                        can_move.down = true;
                    }
                // println!(
                //     "Handling event...{} Can i move? {}",
                //     co2.data().name,
                //     format!("{:#?}", can_move)
                // );
                } else if co2.data().collision_type == CollisionType::Player {
                    let normal = (-contact.contact.normal).into_inner().data;
                    //println!("Stop Contact World1 {:?}, World 2 {:?}", contact.contact.world1, contact.contact.world2);
                    let normals = [normal[0] as f32, normal[1] as f32];
                    if normals[0] <= -0.0 && normal[1] <= 0.0 {
                        can_move.left = true
                    } else if normals[0] > 0.0 && normal[1] >= 0.0 {
                        can_move.right = true;
                    } else if normals[0] <= -0.0 && normal[1] <= -0.0 {
                        can_move.up = true;
                    } else if normals[0] <= 0.0 && normal[1] >= 0.0 {
                        can_move.down = true;
                    }

                    // println!(
                    //     "Handling event...{} Can i move? {}",
                    //     co1.data().name,
                    //     format!("{:#?}", can_move)
                    // );
                }
            }
        } else {
            for (_id, (_player, can_move)) in
                ecs_world.query::<(&Player, &mut CanMove)>().iter().take(1)
            {
                if !can_move.up {
                    can_move.up = true
                }
                if !can_move.down {
                    can_move.down = true
                }
                if !can_move.left {
                    can_move.left = true
                }
                if !can_move.right {
                    can_move.right = true
                }
            }
        }
    }
}

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
fn create_physics_world(
    lyrs: &Vec<tiled::Layer>,
    sprite_map: &HashMap<u32, Sprite>,
    colliders: &mut DefaultColliderSet<f32>,
    bodies: &mut DefaultBodySet<f32>,
) {
    let mut tile_group = CollisionGroups::new();
    tile_group.set_membership(&[3]);
    tile_group.set_whitelist(&[1]);
    let shape_data = CollisionObjectData::new("world obj", CollisionType::World);
    let contacts_query = GeometricQueryType::Contacts(0.0, 0.0);
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
                    let mut points: Vec<Point2<f32>> = Vec::new();
                    let pts: Option<&Vec<_>> = match &obj.shape {
                        ObjectShape::Polygon { points } => Some(points),
                        _ => None,
                    };

                    if pts.is_some() {
                        for point in pts.unwrap() {
                            points.push(Point2::new(point.0, point.1))
                        }
                    } //ConvexPolygon::try_new(points).unwrap()
                    let shape = ShapeHandle::new(Cuboid::new(Vector2::new(8.0, 8.0)));
                    let shape_pos =
                        Isometry2::new(Vector2::new(x as f32  * 32.0, y as f32  * 32.0), rotation);
                    let world_body = RigidBodyDesc::new()
                        .position(shape_pos)
                        .gravity_enabled(false)
                        .status(BodyStatus::Static)
                        .build();

                    let world_body_handle = bodies.insert(world_body);

                    let world_body_collider =
                        ColliderDesc::new(shape).build(BodyPartHandle(world_body_handle, 0));

                    colliders.insert(world_body_collider);
                }
                //println!("{:?}", &sprite.texture);
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
                    .position(Vec2::new(x as f32 * 32.0, y as f32 * 32.0))
                    .origin(Vec2::new(8.0, 8.0))
                    .scale(Vec2::new(SCALE, SCALE))
                    .clip(sprite.rect)
                    .rotation(rotation.to_radians()),
            );
            //println!("{:?}", &sprite.texture);
        }
    }
}

fn new_player(
    ctx: &mut Context,
    world: &mut World,
    body: DefaultBodyHandle,
) -> tetra::Result<Entity> {
    let camera = Camera::with_window_size(ctx);

    Ok(world.spawn((
        Player,
        EntityAnimation {
            direction: Direction::Down,
        },
        camera,
        body,
        CanMove {
            up: true,
            down: true,
            left: true,
            right: true,
        },
    )))
}

fn player_update(
    body_set: &mut DefaultBodySet<f32>,
    ctx: &mut Context,
    world: &mut World,
    anim_map: &mut HashMap<AnimationKey, Animation>,
) {
    for (_id, (camera, anim, _player, can_move, handle)) in &mut world.query::<(
        &mut Camera,
        &mut EntityAnimation,
        &Player,
        &CanMove,
        &DefaultBodyHandle,
    )>() {
        let player_body = body_set.rigid_body_mut(*handle).unwrap();
        if input::is_key_down(ctx, Key::W) {
            player_body.set_linear_velocity(Vector2::new(0.0, -PLAYER_SPEED * 100.0));
            anim.direction = Direction::Up;
            anim_map
                .get_mut(&AnimationKey::PlayerUp)
                .unwrap()
                .advance(ctx);
        }
        if input::is_key_down(ctx, Key::S) {
            player_body.set_linear_velocity(Vector2::new(0.0, PLAYER_SPEED * 100.0));
            anim.direction = Direction::Down;
            anim_map
                .get_mut(&AnimationKey::PlayerDown)
                .unwrap()
                .advance(ctx);
        }
        if input::is_key_down(ctx, Key::D) {
            player_body.set_linear_velocity(Vector2::new(PLAYER_SPEED * 100.0, 0.0));
            anim.direction = Direction::Right;
            anim_map
                .get_mut(&AnimationKey::PlayerRight)
                .unwrap()
                .advance(ctx);
        }
        if input::is_key_down(ctx, Key::A) {
            player_body.set_linear_velocity(Vector2::new(-PLAYER_SPEED * 100.0, 0.0));
            anim.direction = Direction::Left;
            anim_map
                .get_mut(&AnimationKey::PlayerLeft)
                .unwrap()
                .advance(ctx);
        }
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
            let map_tileset = tilesets[x].clone();
            let tile_width = map_tileset.tile_width as i32;
            let tile_height = map_tileset.tile_height as i32;
            let tileset_width = &map_tileset.images[0].width;
            let tileset_height = &map_tileset.images[0].height;
            let tileset_sprite_columns = tileset_width / tile_width as i32;
            let tileset_sprite_rows = tileset_height / tile_height as i32;
            let mut object_map: HashMap<u32, Vec<tiled::Object>> = HashMap::new();
            for tile in map_tileset.tiles {
                object_map.insert(tile.id, tile.objectgroup.unwrap().objects);
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

        let size = get_layer_size(tiled_data.layers.clone().remove(0));

        let plane_left_shape = ShapeHandle::new(Plane::new(Vector2::x_axis()));
        let plane_bottom_shape = ShapeHandle::new(Plane::new(-Vector2::y_axis()));
        let plane_right_shape = ShapeHandle::new(Plane::new(-Vector2::x_axis()));
        let plane_top_shape = ShapeHandle::new(Plane::new(Vector2::y_axis()));

        let player_shape = ShapeHandle::new(Ball::new(12.5));

        let player_pos = Isometry2::new(Vector2::new(800.0, 800.0), nalgebra::zero());

        let plane_pos = [
            //left 
            Isometry2::new(Vector2::new(0.0, 0.0), nalgebra::zero()),
            //bottom
            Isometry2::new(
                Vector2::new(0.0, 1600.0 ),
                nalgebra::zero(),
            ),
            //right
            Isometry2::new(
                Vector2::new(1600.0, 0.0),
                nalgebra::zero(),
            ),
            //top
            Isometry2::new(
                Vector2::new(1600.0, 0.0),
                nalgebra::zero(),
            ),
        ];

        let mut player_group = CollisionGroups::new();
        player_group.set_membership(&[1]);

        let mut plane_group = CollisionGroups::new();
        plane_group.set_membership(&[2]);
        plane_group.set_whitelist(&[1]);

        let plane_data = [
            CollisionObjectData::new("left", CollisionType::Left),
            CollisionObjectData::new("bottom", CollisionType::Bottom),
            CollisionObjectData::new("right", CollisionType::Right),
            CollisionObjectData::new("top", CollisionType::Top),
        ];

        let player_data = CollisionObjectData::new("player", CollisionType::Player);

        let mut physics_world: CollisionWorld<f32, CollisionObjectData> = CollisionWorld::new(0.00);
        let geometrical_world: DefaultGeometricalWorld<f32> = DefaultGeometricalWorld::new();
        let mechanical_world: DefaultMechanicalWorld<f32> =
            DefaultMechanicalWorld::new(Vector2::new(0.0, 0.0));
        let mut bodies = DefaultBodySet::new();
        let mut colliders = DefaultColliderSet::new();
        let joint_constraints: DefaultJointConstraintSet<f32> = DefaultJointConstraintSet::new();
        let force_generators: DefaultForceGeneratorSet<f32> = DefaultForceGeneratorSet::new();

        let player_body = RigidBodyDesc::new()
            .position(player_pos)
            .gravity_enabled(false)
            .status(BodyStatus::Dynamic)
            .mass(1.2)
            .build();

        let plane_left = RigidBodyDesc::new()
            .position(plane_pos[0])
            .gravity_enabled(false)
            .status(BodyStatus::Static)
            .build();
        let plane_bottom = RigidBodyDesc::new()
            .position(plane_pos[1])
            .gravity_enabled(false)
            .status(BodyStatus::Static)
            .build();
        let plane_right = RigidBodyDesc::new()
            .position(plane_pos[2])
            .gravity_enabled(false)
            .status(BodyStatus::Static)
            .build();
        let plane_top = RigidBodyDesc::new()
            .position(plane_pos[3])
            .gravity_enabled(false)
            .status(BodyStatus::Static)
            .build();

        let plane_left_handle = bodies.insert(plane_left);
        let plane_right_handle = bodies.insert(plane_right);
        let plane_bottom_handle = bodies.insert(plane_bottom);
        let plane_top_handle = bodies.insert(plane_top);
        let player_handle = bodies.insert(player_body);

        new_player(ctx, &mut world, player_handle.clone());

        let plane_left_collider =
            ColliderDesc::new(plane_left_shape).build(BodyPartHandle(plane_left_handle, 0));
        let plane_right_collider =
            ColliderDesc::new(plane_right_shape).build(BodyPartHandle(plane_right_handle, 0));
        let plane_bottom_collider =
            ColliderDesc::new(plane_bottom_shape).build(BodyPartHandle(plane_bottom_handle, 0));
        let plane_top_collider =
            ColliderDesc::new(plane_top_shape).build(BodyPartHandle(plane_top_handle, 0));

        let player_collider =
            ColliderDesc::new(player_shape).build(BodyPartHandle(player_handle, 0));

        colliders.insert(plane_left_collider);
        colliders.insert(plane_right_collider);
        colliders.insert(plane_bottom_collider);
        colliders.insert(plane_top_collider);
        colliders.insert(player_collider);

        //fs::write("sprite.txt", format!("{:#?}", tile_sprites)).unwrap();

        let layers = tiled_data.layers;

        create_physics_world(&layers, &tile_sprites, &mut colliders, &mut bodies);

        Ok(GameState {
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
        for (_id, camera) in self.world.query::<&Camera>().iter().take(1) {
            graphics::set_transform_matrix(ctx, camera.as_matrix());
        }
        graphics::clear(ctx, Color::rgb(0.0, 0.0, 0.0));
        let mut layers = self.layers.clone();
        let bg_layer: tiled::Layer = layers.remove(0);

        draw_layer(bg_layer.clone(), &self.texture_map, &self.sprite_map, ctx);

        for (_id, (_camera, anim, _player, handle)) in
            &mut self
                .world
                .query::<(&Camera, &EntityAnimation, &Player, &DefaultBodyHandle)>()
        {
            let key = match anim.direction {
                Direction::Up => AnimationKey::PlayerUp,
                Direction::Down => AnimationKey::PlayerDown,
                Direction::Left => AnimationKey::PlayerLeft,
                Direction::Right => AnimationKey::PlayerRight,
            };
            let animation = self.player_anim_map.get(&key).unwrap();
            let player_body = self.body_set.rigid_body(*handle).unwrap();
            let player_pos = Vec2::new(
                player_body.position().translation.vector.x ,
                player_body.position().translation.vector.y ,
            );
            graphics::draw(
                ctx,
                animation,
                DrawParams::new()
                    .position(player_pos)
                    .origin(Vec2::new(12.5, 36.0))
                    .scale(Vec2::new(SCALE, SCALE)),
            );
        }

        //graphics::draw(ctx, &self.player, DrawParams::default());

        for x in layers {
            draw_layer(x, &self.texture_map, &self.sprite_map, ctx);
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
                player_body.position().translation.vector.x ,
                player_body.position().translation.vector.y  ,
            );
            //println!("{:?} Physical Pos", camera.position );
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
    // #[derive(Debug)]
    // println!("{:#?}", tiled_data);
    //fs::write("foo.txt", format!("{:#?}", tiled_data)).unwrap();
    ContextBuilder::new("Neon", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .resizable(true)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
