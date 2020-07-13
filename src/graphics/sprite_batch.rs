use indexmap::IndexMap;
use luminance::blending::{Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::pipeline::BoundTexture;
use luminance::pipeline::Pipeline as LuminancePipeline;
use luminance::pipeline::PipelineState;
use luminance::pipeline::ShadingGate;
use luminance::pixel::NormRGBA8UI;
use luminance::pixel::NormUnsigned;
use luminance::render_state::RenderState;
use luminance::shader::program::Program;
use luminance::shader::program::Uniform;
use luminance::tess::{Mode, Tess, TessBuilder};
use luminance::texture::Dim2;
use luminance::texture::Texture;
use luminance_derive::UniformInterface;
use luminance_derive::{Semantics, Vertex};
use nalgebra::base::{Matrix4, Vector2, Vector3};
use std::collections::HashMap;

const VS: &'static str = include_str!("./vertex_sprite_batch.glsl");
const FS: &'static str = include_str!("./fragment_sprite_batch.glsl");

#[derive(UniformInterface)]
struct ShaderInterface {
    u_transform: Uniform<[[f32; 4]; 4]>,

    u_texture_sampler: Uniform<&'static BoundTexture<'static, Dim2, NormUnsigned>>,
}

// *Vertices is param_map v.0, Indicies is param_map v.1, Tess is v.2
pub struct SpriteBatch {
    program: Program<Semantics, (), ShaderInterface>,
    texture_map: IndexMap<String, Texture<Dim2, NormRGBA8UI>>,
    param_map: HashMap<String, (Vec<Vertex>, Vec<u32>, Option<Tess>)>,
}

fn to_4x4(array: &[f32; 16]) -> [[f32; 4]; 4] {
    unsafe { *(array as *const _ as *const _) }
}
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Region {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Region {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Region {
            x,
            y,
            width,
            height,
        }
    }
    pub fn row(x: f32, y: f32, width: f32, height: f32) -> impl Iterator<Item = Region> {
        RegionRow {
            next_region: Region::new(x, y, width, height),
        }
    }
}
#[derive(Copy, Clone, PartialEq)]
struct RegionRow {
    next_region: Region,
}

impl Iterator for RegionRow {
    type Item = Region;

    fn next(&mut self) -> Option<Region> {
        let current_region = self.next_region;
        self.next_region.x += self.next_region.width;
        Some(current_region)
    }
}

#[allow(dead_code)]
impl SpriteBatch {
    pub fn new(texture_map: IndexMap<String, Texture<Dim2, NormRGBA8UI>>) -> Self {
        let program = Program::<Semantics, (), ShaderInterface>::from_strings(None, VS, None, FS)
            .expect("shader failed to compile")
            .program;
        let mut param_map = HashMap::new();
        for (k, _v) in &texture_map {
            param_map.insert(k.clone(), (Vec::new(), Vec::new(), None));
        }
        SpriteBatch {
            texture_map,
            program,
            param_map,
        }
    }

    pub fn queue_sprite(&mut self, texture_name: String, position: Vector3<f32>, region: Region) {
        let region2 = Region {
            x: region.x,
            y: region.y,
            height: region.height,
            width: region.width,
        };

        let texture = self.texture_map.get(&texture_name).unwrap();
        let texture_size = Vector2::new(texture.size()[0] as f32, texture.size()[1] as f32);
        let v0 = Vector3::new(position.x, position.y + region2.height, position.z);
        let v1 = Vector3::new(
            position.x + region2.width,
            position.y + region2.height,
            position.z,
        );
        let v2 = Vector3::new(position.x + region2.width, position.y, position.z);
        let v3 = position;

        let t0 = Vector2::new(
            region.x / texture_size.x,
            (region.y + region.height) / texture_size.y,
        );
        let t1 = Vector2::new(
            (region.x + region.width) / texture_size.x,
            (region.y + region.height) / texture_size.y,
        );
        let t2 = Vector2::new(
            (region.x + region.width) / texture_size.x,
            region.y / texture_size.y,
        );
        let t3 = Vector2::new(region.x / texture_size.x, region.y / texture_size.y);

        let mut indices: Vec<u32> = [0, 1, 2, 2, 3, 0]
            .iter()
            .map(|e| e + self.param_map.get(&texture_name).unwrap().0.len() as u32)
            .collect();

        let mut vertices: Vec<Vertex> = [(v0, t0), (v1, t1), (v2, t2), (v3, t3)]
            .iter()
            .map(|(v, t)| {
                let pos: [f32; 3] = (*v).into();
                let tex: [f32; 2] = (*t).into();
                Vertex {
                    pos: VertexPosition::from(pos),
                    tex: VertexTexCoord::from(tex),
                }
            })
            .collect();

        &mut self
            .param_map
            .get_mut(&texture_name)
            .unwrap()
            .0
            .append(&mut vertices);
        &mut self
            .param_map
            .get_mut(&texture_name)
            .unwrap()
            .1
            .append(&mut indices);
    }
    pub fn draw<'a, C>(
        &mut self,
        pipeline: &mut LuminancePipeline<'a>,
        shading_gate: &mut ShadingGate<'a, C>,
        transform: [f32; 16],
    ) where
        C: GraphicsContext,
    {
        for (k, v) in &self.texture_map {
            let params = &self.param_map.get(k).unwrap();
            if let Some(vao) = &params.2 {
                let bound_texture = pipeline.bind_texture(v);

                // Start shading with our program.
                shading_gate.shade(&self.program, |iface, mut rdr_gate| {
                    iface.u_transform.update(to_4x4(&transform));
                    iface.u_texture_sampler.update(&bound_texture);

                    // Start rendering things with the default render state provided by luminance.
                    rdr_gate.render(
                        &RenderState::default().set_blending((
                            Equation::Additive,
                            Factor::SrcAlpha,
                            Factor::SrcAlphaComplement,
                        )),
                        |mut tess_gate| tess_gate.render(vao),
                    );
                });
            }
        }
    }
    pub fn prepare<C>(&mut self, context: &mut C)
    where
        C: GraphicsContext,
    {
        for (k, v) in &mut self.param_map {
            v.2 = Some(
                TessBuilder::new(context)
                    .add_vertices(v.0.as_slice())
                    .set_indices(v.1.clone())
                    .set_mode(Mode::Triangle)
                    .build()
                    .unwrap(),
            );

            v.0.clear();
            v.1.clear();
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum Semantics {
    #[sem(name = "v_position", repr = "[f32; 3]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "v_tex_coord", repr = "[f32; 2]", wrapper = "VertexTexCoord")]
    TexCoord,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Vertex {
    pos: VertexPosition,
    tex: VertexTexCoord,
}

pub fn orthographic_projection(width: u32, height: u32) -> [f32; 16] {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let mut matrix : [f32; 16] = [0.0; 16];
    matrix.copy_from_slice(
        Matrix4::new_orthographic(0.0, width as f32, height as f32, 0.0, -0.1, -100.0).as_slice(),
    );
    matrix
}

pub fn orthographic_projection_matrix(width: u32, height: u32) -> Matrix4<f32> {
    Matrix4::new_orthographic(0.0, width as f32, height as f32, 0.0, -0.1, -100.0)
}
