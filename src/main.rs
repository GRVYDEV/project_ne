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

const WINDOW_WIDTH: f32 = 1600.0;
const WINDOW_HEIGHT: f32 = 900.0;

const SCALE: f32 = 2.0;

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

        let anim_left: Vec<_> = Rectangle::row(0.0, 32.0, CHAR_WIDTH, CHAR_HEIGHT)
            .take(3)
            .collect();
        let anim_right: Vec<_> = Rectangle::row(0.0, 64.0, CHAR_WIDTH, CHAR_HEIGHT)
            .take(3)
            .collect();
        let anim_up: Vec<_> = Rectangle::row(0.0, 96.0, CHAR_WIDTH, CHAR_HEIGHT)
            .take(3)
            .collect();
        let anim_down: Vec<_> = Rectangle::row(0.0, 0.0, CHAR_WIDTH, CHAR_HEIGHT)
            .take(3)
            .collect();

        let anim_data = AnimationData {
            left: Anim::new(&anim_left, Duration::from_secs_f64(ANIM_SPEED)),
            right: Anim::new(&anim_right, Duration::from_secs_f64(ANIM_SPEED)),
            up: Anim::new(&anim_up, Duration::from_secs_f64(ANIM_SPEED)),
            down: Anim::new(&anim_down, Duration::from_secs_f64(ANIM_SPEED)),
        };

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
            anim_data.clone(),
        )
        .expect("Failed to create Player");

        spawn_npcs(
            1000,
            &mut colliders,
            &mut bodies,
            &mut world,
            character_map.len() - 1,
            anim_data.clone(),
        );

        let player_collider =
            ColliderDesc::new(player_shape).build(BodyPartHandle(player_handle, 0));

        colliders.insert(player_collider);

        //fs::write("sprite.txt", format!("{:#?}", tile_sprites)).unwrap();

        create_physics_world(&layers, &tile_sprites, &mut colliders, &mut bodies);

        Ok(GameState {
            characters: character_map,
            world,
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
        for (_id, (camera, _player, _character)) in self
            .world
            .query::<(&Camera, &Player, &Character)>()
            .iter()
            .take(1)
        {
            graphics::set_transform_matrix(ctx, camera.as_matrix());
        }
        graphics::clear(ctx, Color::rgb(0.0, 0.0, 0.0));

        let mut layers = self.layers.clone();
        let bg_layer: tiled::Layer = layers.remove(0);
        let bg_layer_2: tiled::Layer = layers.remove(0);
        draw_layer(bg_layer.clone(), &self.texture_map, &self.sprite_map, ctx);
        draw_layer(bg_layer_2.clone(), &self.texture_map, &self.sprite_map, ctx);
        let mut render_vec: Vec<_> = self
            .world
            .query::<(
                &EntityAnimation,
                &DefaultBodyHandle,
                &AnimationData,
                &Character,
            )>()
            .iter()
            .map(|(_, (&e, &d, a, &c))| (e, d, a.clone(), c))
            .collect();
        render_vec.sort_by(|a, b| {
            let a = self
                .body_set
                .rigid_body(a.1)
                .unwrap()
                .position()
                .translation
                .vector
                .y;
            let b = self
                .body_set
                .rigid_body(b.1)
                .unwrap()
                .position()
                .translation
                .vector
                .y;
            a.partial_cmp(&b).unwrap()
        });
        for (anim, handle, anim_data, character) in render_vec {
            let anim = match anim.direction {
                Direction::Up => &anim_data.up,
                Direction::Down => &anim_data.down,
                Direction::Left => &anim_data.left,
                Direction::Right => &anim_data.right,
            };

            let mut animation = Animation::new(
                self.characters.get(&character.0).unwrap().clone(),
                anim.frames.clone(),
                anim.frame_duration,
            );
            animation.set_current_frame_index(anim.frame_index);
            let body = self.body_set.rigid_body(handle).unwrap();
            let pos = Vec2::new(
                body.position().translation.vector.x,
                body.position().translation.vector.y,
            );
            graphics::draw(
                ctx,
                &animation,
                DrawParams::new()
                    .position(pos)
                    .origin(Vec2::new(9.5, 27.0))
                    .scale(Vec2::new(SCALE, SCALE)),
            );
        }

        for x in layers {
            draw_layer(x.clone(), &self.texture_map, &self.sprite_map, ctx);
        }

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        player_update(&mut self.body_set, ctx, &mut self.world);
        npc_update(&mut self.body_set, &mut self.world, ctx);
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
        // for(_id, (_npc, handle)) in &mut self.world.query::<(&NPC, &DefaultBodyHandle)>(){
        //     let body = self.body_set.rigid_body_mut(*handle).unwrap();
        //     body.set_linear_velocity(Vector2::new(0.0, 0.0));
        // }

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
