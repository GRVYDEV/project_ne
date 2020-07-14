mod components;
use components::*;
mod prelude;
use prelude::*;
mod world_gen;
use world_gen::*;
mod player;
use player::*;
mod npc;
use npc::*;
mod game;
mod graphics;
use game::{run, Game, GameEvent};
use graphics::load_texture_from_bytes;
use graphics::orthographic_projection;
use graphics::Region;
use graphics::SpriteBatch;
mod camera;
use camera::*;
use glfw::WindowEvent;
use std::collections::HashSet;

const WINDOW_WIDTH: f32 = 1600.0;
const WINDOW_HEIGHT: f32 = 900.0;

pub const SCALE: f32 = 2.0;

const CHAR_HEIGHT: f32 = 32.0;
const CHAR_WIDTH: f32 = 19.0;

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
    (
        "castle",
        include_bytes!("../resources/map/tilesets/castle.png"),
    ),
];

const PLAYER_SHEETS: &[(&usize, &[u8])] = &[
    (&0, include_bytes!("../resources/Wizard-Sheet.png")),
    (&1, include_bytes!("../resources/Viking-Sheet.png")),
    (&2, include_bytes!("../resources/Fire-Man-Sheet.png")),
    (&3, include_bytes!("../resources/Red-Hair-Sheet.png")),
];

const NPC_SHEETS: &[(&usize, &[u8])] = &[
    (&0, include_bytes!("../resources/Wizard-Sheet.png")),
    (&1, include_bytes!("../resources/Viking-Sheet.png")),
    (&2, include_bytes!("../resources/Fire-Man-Sheet.png")),
    (&3, include_bytes!("../resources/NPC01-Sheet.png")),
    (&4, include_bytes!("../resources/NPC02-Sheet.png")),
    (&5, include_bytes!("../resources/Red-Hair-Sheet.png")),
];

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
            let mut origin = Vec2::new(8.0, 8.0);
            if sprite.width == 16.0 && sprite.height == 16.0 {
                origin.x = 8.0;
                origin.y = 8.0;
            }

            let texture = texture_map.get(&sprite.texture).unwrap();
            let mut rotation: f32 = 0.0;
            if tile.flip_h {
                rotation += 180.0;
            }
            if tile.flip_d {
                rotation -= 90.0;
            }
            if tile.flip_v {
                rotation += 0.0;
            }

            tetra_graphics::draw(
                ctx,
                texture,
                DrawParams::new()
                    .position(Vec2::new(
                        (x as f32 * 32.0) + 16.0,
                        (y as f32 * 32.0) + 16.0,
                    ))
                    .origin(origin)
                    .scale(Vec2::new(SCALE, SCALE))
                    //.clip(sprite.rect)
                    .rotation(rotation.to_radians()),
            );
        }
    }
}

fn spawn_ecs_tiles(lyr: &tiled::Layer, world: &mut World, sprite_map: &HashMap<u32, Sprite>) {
    for (y, row) in lyr.tiles.iter().enumerate().clone() {
        for (x, &tile) in row.iter().enumerate() {
            if tile.gid == 0 {
                continue;
            }

            let gid = tile.gid;

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

            let pos = Vec2::new((x as f32 * 32.0), (y as f32 * 32.0) + 32.0);
            world.spawn((Draw {
                y: pos.y,
                draw_type: DrawType::Tile,
                player: None,
                tile: Some(TileDrawData {
                    pos,
                    sprite: sprite_map.get(&gid).unwrap().clone(),
                    rotation,
                }),
            },));
        }
    }
}

fn handle_contact(
    world: &DefaultGeometricalWorld<f32>,
    event: &ContactEvent<DefaultBodyHandle>,
    ecs_world: &mut World,
) {
    if let &ContactEvent::Started(collider1, collider2) = event {
        for (_id, (draw, _npc)) in &mut ecs_world.query::<(&mut Draw, &NPC)>() {
            if draw.player.as_mut().unwrap().handle == collider1
                || draw.player.as_mut().unwrap().handle == collider2
            {
                draw.player.as_mut().unwrap().colliding = true;
            }
        }
    }
}

// impl GameState {
//     fn new(ctx: &mut Context) -> tetra::Result<GameState> {
//         let mut world = World::new();
//         let mut file_to_texture = HashMap::new();
//         for (k, v) in TILESETS {
//             file_to_texture
//                 .entry(k.to_string())
//                 .or_insert(Texture::from_file_data(ctx, v)?);
//         }

//         let mut character_map = HashMap::new();
//         let mut npc_map = HashMap::new();
//         for (k, v) in PLAYER_SHEETS {
//             character_map.insert(**k, Texture::from_file_data(ctx, v)?);
//         }
//         for (k, v) in NPC_SHEETS {
//             npc_map.insert(**k, Texture::from_file_data(ctx, v)?);
//         }

//         let anim_left: Vec<_> = Rectangle::row(0.0, 32.0, CHAR_WIDTH, CHAR_HEIGHT)
//             .take(3)
//             .collect();
//         let anim_right: Vec<_> = Rectangle::row(0.0, 64.0, CHAR_WIDTH, CHAR_HEIGHT)
//             .take(3)
//             .collect();
//         let anim_up: Vec<_> = Rectangle::row(0.0, 96.0, CHAR_WIDTH, CHAR_HEIGHT)
//             .take(3)
//             .collect();
//         let anim_down: Vec<_> = Rectangle::row(0.0, 0.0, CHAR_WIDTH, CHAR_HEIGHT)
//             .take(3)
//             .collect();

//         let anim_data = AnimationData {
//             left: Anim::new(&anim_left, Duration::from_secs_f64(ANIM_SPEED)),
//             right: Anim::new(&anim_right, Duration::from_secs_f64(ANIM_SPEED)),
//             up: Anim::new(&anim_up, Duration::from_secs_f64(ANIM_SPEED)),
//             down: Anim::new(&anim_down, Duration::from_secs_f64(ANIM_SPEED)),
//         };

//         let geometrical_world: DefaultGeometricalWorld<f32> = DefaultGeometricalWorld::new();
//         let mechanical_world: DefaultMechanicalWorld<f32> =
//             DefaultMechanicalWorld::new(Vector2::new(0.0, 0.0));
//         let mut bodies = DefaultBodySet::new();
//         let mut colliders = DefaultColliderSet::new();
//         let joint_constraints: DefaultJointConstraintSet<f32> = DefaultJointConstraintSet::new();
//         let force_generators: DefaultForceGeneratorSet<f32> = DefaultForceGeneratorSet::new();
//         create_map_bounds(&layers[0], &mut colliders, &mut bodies);

//         // spawn(
//         //     &mut colliders,
//         //     &mut bodies,
//         //     &mut world,
//         //     (&(npc_map.len() - 1), &(character_map.len() - 1)),
//         //     &anim_data,
//         //     map,
//         //     ctx,
//         // );

//         let top_layers = &layers[1..];

//         // for layer in top_layers {
//         //     spawn_ecs_tiles(layer, &mut world, &tile_sprites);
//         // }

//         // create_physics_world(&layers, &tile_sprites, &mut colliders, &mut bodies);

//         Ok(GameState {
//             characters: character_map,
//             npcs: npc_map,
//             world,
//             sprite_map: tile_sprites,
//             layers: layers,
//             texture_map: file_to_texture,
//             mechanical_world: mechanical_world,
//             geometrical_world: geometrical_world,
//             body_set: bodies,
//             collider_set: colliders,
//             force_gen_set: force_generators,
//             constraint_set: joint_constraints,
//         })
//     }
// }

// impl State for GameState {
//     fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
//         //&self.texture.set_current_frame_index(1);
//         for (_id, camera) in self.world.query::<&Camera>().iter().take(1) {
//             tetra_graphics::set_transform_matrix(ctx, camera.as_matrix());
//         }
//         tetra_graphics::clear(ctx, Color::rgb(0.0, 0.0, 0.0));

//         let mut layers = self.layers.clone();
//         let bg_layer: tiled::Layer = layers.remove(0);
//         let bg_layer2: tiled::Layer = layers.remove(0);
//         draw_layer(bg_layer.clone(), &self.texture_map, &self.sprite_map, ctx);
//         draw_layer(bg_layer2.clone(), &self.texture_map, &self.sprite_map, ctx);
//         let mut render_vec: Vec<_> = self
//             .world
//             .query::<&Draw>()
//             .iter()
//             .map(|(_, d)| d.clone())
//             .collect();
//         render_vec.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
//         for draw in render_vec {
//             if draw.draw_type == DrawType::Character || draw.draw_type == DrawType::NPC {
//                 draw.draw(
//                     ctx,
//                     &self.texture_map,
//                     (&self.characters, &self.npcs),
//                     &self.body_set,
//                 );
//             }
//         }
//         for layer in layers {
//             draw_layer(layer, &self.texture_map, &self.sprite_map, ctx);
//         }

//         Ok(())
//     }

//     fn update(&mut self, ctx: &mut Context) -> tetra::Result {
//         player_update(&mut self.body_set, ctx, &mut self.world);
//         npc_update(&mut self.body_set, &mut self.world, ctx);
//         for contact in self.geometrical_world.contact_events() {
//             handle_contact(&self.geometrical_world, &contact, &mut self.world)
//         }
//         self.mechanical_world.step(
//             &mut self.geometrical_world,
//             &mut self.body_set,
//             &mut self.collider_set,
//             &mut self.constraint_set,
//             &mut self.force_gen_set,
//         );

//         for (_id, (camera, _player, draw)) in
//             &mut self.world.query::<(&mut Camera, &Player, &Draw)>()
//         {
//             let handle = draw.player.as_ref().unwrap().handle;
//             let player_body = self.body_set.rigid_body_mut(handle).unwrap();
//             player_body.set_linear_velocity(Vector2::new(0.0, 0.0));
//             camera.position = Vec2::new(
//                 player_body.position().translation.vector.x * 2.0,
//                 player_body.position().translation.vector.y * 2.0,
//             );
//             camera.update();
//         }
//         for (_id, draw) in &mut self.world.query::<(&mut Draw)>() {
//             if draw.draw_type == DrawType::Character || draw.draw_type == DrawType::NPC {
//                 let entity = draw.player.as_ref().unwrap();
//                 let handle = entity.handle;
//                 let y = self
//                     .body_set
//                     .rigid_body(handle)
//                     .unwrap()
//                     .position()
//                     .translation
//                     .y;

//                 draw.y = y;
//             }
//         }
//         // for(_id, (_npc, handle)) in &mut self.world.query::<(&NPC, &DefaultBodyHandle)>(){
//         //     let body = self.body_set.rigid_body_mut(*handle).unwrap();
//         //     body.set_linear_velocity(Vector2::new(0.0, 0.0));
//         // }

//         Ok(())
//     }

//     fn event(&mut self, _ctx: &mut Context, event: Event) -> tetra::Result {
//         if let Event::Resized { width, height } = event {
//             for (_id, camera) in self.world.query::<&mut Camera>().iter().take(1) {
//                 camera.set_viewport_size(width as f32, height as f32);
//                 camera.update();
//             }
//         }
//         Ok(())
//     }
// }
fn draw_layer_new(
    batch: &mut SpriteBatch,
    lyr: tiled::Layer,
    sprite_map: &HashMap<u32, Sprite>,
    z_val: &f32,
) {
    for (y, row) in lyr.tiles.iter().enumerate().clone() {
        for (x, &tile) in row.iter().enumerate() {
            if tile.gid == 0 {
                continue;
            }

            let gid = tile.gid;
            let sprite = sprite_map.get(&gid).unwrap();
            let mut origin = Vec2::new(8.0, 8.0);
            if sprite.width == 16.0 && sprite.height == 16.0 {
                origin.x = 8.0;
                origin.y = 8.0;
            }

            

            let mut rotation: f32 = 0.0;
            if tile.flip_h {
                rotation += 180.0;
            }
            if tile.flip_d {
                rotation -= 90.0;
            }
            if tile.flip_v {
                rotation += 0.0;
            }

            batch.queue_sprite(
                sprite.texture.clone(),
                Vector3::new(x as f32 * 16.0, y as f32 * 16.0, *z_val),
                sprite.rect.clone(),
                rotation.to_radians()
            );
        }
    }
}
pub struct MyGameState {
    sprite_map: HashMap<u32, Sprite>,
    layers: Vec<tiled::Layer>,
    batch: SpriteBatch,
    camera: Camera,
}
impl GameState {}

impl Game for MyGameState {
    fn new<C>(context: &mut C) -> Self
    where
        C: GraphicsContext,
    {
        let mut file_to_texture = IndexMap::new();
        for (k, v) in TILESETS {
            file_to_texture
                .entry(k.to_string())
                .or_insert(load_texture_from_bytes(context, v));
        }
        let batch = SpriteBatch::new(file_to_texture);

        let tiled_data = parse(&include_bytes!("../resources/map/map5.tmx")[..]).unwrap();
        //fs::write("map.ron", format!("{:#?}", &tiled_data.clone())).unwrap();

        let map = &tiled_data.clone();
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
            }
            let mut id = 0;
            for y in 0..tileset_sprite_rows {
                for x in 0..tileset_sprite_columns {
                    let sprite_w = tile_width as f32;
                    let sprite_h = tile_height as f32;
                    let pos_x = (x * tile_width) as f32;
                    let pos_y = (y * tile_height) as f32;
                    let objects = object_map.remove(&id).clone();
                    let sprite = Sprite {
                        width: sprite_w,
                        height: sprite_h,
                        rect: Region {
                            x: pos_x,
                            y: pos_y,
                            width: sprite_w,
                            height: sprite_h,
                        },
                        pos: Vector2::new(pos_x, pos_y),
                        texture: map_tileset.name.clone(),
                        collision_objects: objects,
                    };

                    tile_sprites.entry(gid).or_insert(sprite);
                    gid += 1;
                    id += 1;
                }
            }
        }
        let mut camera = Camera::new(1600.0/ 2.0, 900.0 / 2.0);
        camera.set_position(Vector2::new(800.0 / 2.0, 800.0 / 2.0));
        let layers = tiled_data.layers;
        MyGameState {
            sprite_map: tile_sprites,
            layers: layers,
            batch,
            camera,
        }
    }

    fn update(&mut self, key_buffer: &HashSet<glfw::Key>) {
        let mut translate = Vector2::new(0.0, 0.0);
        if key_buffer.contains(&glfw::Key::W) {
            translate.y += 20.0;
        }
        if key_buffer.contains(&glfw::Key::A) {
            translate.x += 20.0;
        }
        if key_buffer.contains(&glfw::Key::D) {
            translate.x -= 20.0;
        }
        if key_buffer.contains(&glfw::Key::S) {
            translate.y -= 20.0;
        }
        self.camera.translate(translate.x, translate.y);
    }

    fn draw<C>(&mut self, context: &mut C, delta_time: Duration, buffer: &Framebuffer<Dim2, (), ()>)
    where
        C: GraphicsContext,
    {
        let mut z: f32 = 50.0;
        for layer in &mut self.layers {
            draw_layer_new(&mut self.batch, layer.clone(), &self.sprite_map, &z);
            z -= 1.0;
        }

        &mut self.batch.prepare(context);

        context.pipeline_builder().pipeline(
            &buffer,
            &PipelineState::default(),
            |mut pipeline, mut shd_gate| {
                &mut self
                    .batch
                    .draw(&mut pipeline, &mut shd_gate, self.camera.as_matrix());
            },
        );
    }

    fn process_event(&mut self, event: GameEvent) {
        if let GameEvent::WindowEvent(WindowEvent::FramebufferSize(width, height)) = event {
            self.camera.set_size(width as u32 / 2, height as u32 / 2);
         
        }
    }
}

fn main() {
    run::<MyGameState>();
    // ContextBuilder::new("Neon", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
    //     .resizable(true)
    //     .quit_on_escape(true)
    //     .build()?
    //     .run(GameState::new)
}
