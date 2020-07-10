use luminance::blending::{Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::pipeline::BoundTexture;
use luminance::pipeline::Pipeline as LuminancePipeline;
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
use nalgebra::base::Vector2;

const VS: &'static str = include_str!("./vertex_sprite_batch.glsl");
const FS: &'static str = include_str!("./fragment_sprite_batch.glsl");

#[derive(UniformInterface)]
struct ShaderInterface {
    u_transform: Uniform<[[f32; 4]; 4]>,

    u_texture_sampler: Uniform<&'static BoundTexture<'static, Dim2, NormUnsigned>>,
}

pub struct SpriteBatch {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    tess: Option<Tess>,
    program: Program<Semantics, (), ShaderInterface>,
    texture: Texture<Dim2, NormRGBA8UI>,
}

fn to_4x4(array: &[f32; 16]) -> [[f32; 4]; 4] {
    unsafe { *(array as *const _ as *const _) }
}

pub struct Region {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[allow(dead_code)]
impl SpriteBatch {
    pub fn new(texture: Texture<Dim2, NormRGBA8UI>) -> Self {
        let program = Program::<Semantics, (), ShaderInterface>::from_strings(None, VS, None, FS)
            .expect("shader failed to compile")
            .program;
        SpriteBatch {
            texture,
            program,
            vertices: Vec::new(),
            indices: Vec::new(),
            tess: None,
        }
    }
    pub fn queue_sprite(&mut self, position: Vector2<f32>, region: Region) {
        let region2 = Region {
            x: region.x,
            y: region.y,
            height: region.height * 2.0,
            width: region.width * 2.0,
        };
        let texture_size =
            Vector2::new(self.texture.size()[0] as f32, self.texture.size()[1] as f32);
        let v0 = Vector2::new(position.x, position.y + region2.height);
        let v1 = Vector2::new(position.x + region2.width, position.y + region2.height);
        let v2 = Vector2::new(position.x + region2.width, position.y);
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
            .map(|e| e + self.vertices.len() as u32)
            .collect();

        self.indices.append(&mut indices);

        let mut vertices: Vec<Vertex> = [(v0, t0), (v1, t1), (v2, t2), (v3, t3)]
            .iter()
            .map(|(v, t)| {
                let pos: [f32; 2] = (*v).into();
                let tex: [f32; 2] = (*t).into();
                Vertex {
                    pos: VertexPosition::from(pos),
                    tex: VertexTexCoord::from(tex),
                }
            })
            .collect();

        self.vertices.append(&mut vertices);
    }
    pub fn draw<'a, C>(
        &mut self,
        pipeline: &mut LuminancePipeline<'a>,
        shading_gate: &mut ShadingGate<'a, C>,
        transform: [f32; 16],
    ) where
        C: GraphicsContext,
    {
        if let Some(vao) = &self.tess {
            let bound_texture = pipeline.bind_texture(&self.texture);

            // Start shading with our program.
            shading_gate.shade(&self.program, |iface, mut rdr_gate| {
                iface.u_transform.update(to_4x4(&transform));
                iface.u_texture_sampler.update(&bound_texture);

                // Start rendering things with the default render state provided by luminance.
                rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                    tess_gate.render(vao);
                });
            });
        }
    }
    pub fn prepare<C>(&mut self, context: &mut C)
    where
        C: GraphicsContext,
    {
        self.tess = Some(
            TessBuilder::new(context)
                .add_vertices(self.vertices.as_slice())
                .set_indices(self.indices.clone())
                .set_mode(Mode::Triangle)
                .build()
                .unwrap(),
        );

        self.vertices.clear();
        self.indices.clear();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum Semantics {
    #[sem(name = "v_position", repr = "[f32; 2]", wrapper = "VertexPosition")]
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
    [
        2.0 / width as f32, 0.0, 0.0, 0.0,
        0.0, -2.0 / height as f32, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        -1.0, 1.0, 0.0, 1.0,
    ]
}
