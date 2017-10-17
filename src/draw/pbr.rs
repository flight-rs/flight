use gfx::{self, Resources, CommandBuffer, ShaderSet, Factory, Rect, Slice, Encoder};
use gfx::pso::PipelineState;
use gfx::traits::FactoryExt;
use gfx::handle::Buffer;
use gfx::state::Rasterizer;
use gfx::format::*;

use super::{StyleInputs, Style, LightBlock, TransformBlock};
use ::mesh::{Primitive, VertNTT};
use ::{Light, Error, ColorFormat, DepthFormat, TargetRef, DepthRef, Texture};

/// The maximum number of point lights that can be simulated
pub const LIGHT_COUNT: usize = 4;

/// The collection of mesh textures used by physically based rendering
#[derive(Clone)]
pub struct PbrMaterial<R: Resources> {
    /// Normal map
    pub normal: Texture<R, (R8_G8_B8_A8, Unorm)>,
    /// Albedo map (base color)
    pub albedo: Texture<R, (R8_G8_B8_A8, Srgb)>,
    /// Metalness map (sets material to metal or dielectric)
    pub metalness: Texture<R, (R8, Unorm)>,
    /// Roughness map (Changes shininess/reflection sharpness)
    pub roughness: Texture<R, (R8, Unorm)>,
}

gfx_defines!{
    pipeline pl {
        verts: gfx::VertexBuffer<VertNTT> = (),
        transform: gfx::ConstantBuffer<TransformBlock> = "transform",
        params: gfx::ConstantBuffer<PbrBlock> = "params",
        lights: gfx::ConstantBuffer<LightBlock> = "lights_layout",
        scissor: gfx::Scissor = (), // TODO: Replace scissoring with viewport
        color: gfx::RenderTarget<ColorFormat> = "f_lum",
        depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
        normal: gfx::TextureSampler<[f32; 4]> = "normal_tex",
        albedo: gfx::TextureSampler<[f32; 4]> = "albedo_tex",
        metalness: gfx::TextureSampler<f32> = "metalness_tex",
        roughness: gfx::TextureSampler<f32> = "roughness_tex",
    }

    constant PbrBlock {
        ambient: [f32; 4] = "ambient",
    }
}

shader!(shader {
    vertex: static_file!("shaders/transform.v.glsl")
        .define("NORM")
        .define("TEX")
        .define("TAN"),
    fragment: static_file!("shaders/pbr.f.glsl")
        .define_to("I_POS", "v_pos")
        .define_to("I_NORM", "v_norm")
        .define_to("I_TEX", "v_tex")
        .define_to("I_TAN", "v_tan")
        .define_to("I_BITAN", "v_bitan")
        .define_to("LIGHT_COUNT", LIGHT_COUNT)
});

/// The configuration for physically based rendering
pub struct PbrInputs<R: Resources> {
    shaders: ShaderSet<R>,
    transform: Option<TransformBlock>,
    transform_block: Buffer<R, TransformBlock>,
    params: Option<PbrBlock>,
    params_block: Buffer<R, PbrBlock>,
    lights: Option<[LightBlock; LIGHT_COUNT]>,
    lights_block: Buffer<R, LightBlock>,
}

impl<R: Resources> PbrInputs<R> {
    /// Sets the point lights present in the scene. Only the first `LIGHT_COUNT` lights will be used.
    pub fn lights(&mut self, lights: &[Light]) {
        let mut all = [LightBlock::from(Light::default()); LIGHT_COUNT];
        for i in 0..lights.len().min(LIGHT_COUNT) {
            all[i] = LightBlock::from(lights[i]);
        }
        self.lights = Some(all);
    }

    /// Sets the ambient light level in the scene
    pub fn ambient(&mut self, c: [f32; 4]) {
        self.params = Some(PbrBlock {
            ambient: c
        });
    }
}

impl<R: Resources> StyleInputs<R> for PbrInputs<R> {
    fn transform(&mut self, block: TransformBlock) {
        self.transform = Some(block);
    }
    fn shader_set(&self) -> &ShaderSet<R> { &self.shaders }
}

/// Draws meshes using a physically based rendering pipeline
pub struct PbrStyle<R: Resources> {
    pso: PipelineState<R, pl::Meta>,
}

impl<R: Resources> Style<R> for PbrStyle<R> {
    type Vertex = VertNTT;
    type Inputs = PbrInputs<R>;
    type Material = PbrMaterial<R>;

    fn new<F: Factory<R> + FactoryExt<R>> (
        f: &mut F,
        i: &mut PbrInputs<R>,
        p: Primitive,
        r: Rasterizer,
    ) -> Result<Self, Error> {
        Ok(PbrStyle {
            pso: f.create_pipeline_state(&i.shaders, p, r, pl::new())?,
        })
    }

    fn init<F: Factory<R>>(
        f: &mut F,
    ) -> Result<PbrInputs<R>, Error> {
        Ok(PbrInputs {
            shaders: shader(f)?,
            transform: None,
            transform_block: f.create_constant_buffer(1),
            params: Some(PbrBlock { ambient: [0.; 4] }),
            params_block: f.create_constant_buffer(1),
            lights: Some([LightBlock::from(Light::default()); 4]),
            lights_block: f.create_constant_buffer(LIGHT_COUNT),
        })
    }

    fn draw_raw<C>(
        &self,
        inputs: &mut PbrInputs<R>,
        enc: &mut Encoder<R, C>,
        color: TargetRef<R>,
        depth: DepthRef<R>,
        scissor: Rect,
        slice: &Slice<R>,
        buf: Buffer<R, Self::Vertex>,
        mat: &PbrMaterial<R>,
    )
        -> Result<(), Error>
        where C: CommandBuffer<R>
    {
        if let Some(t) = inputs.transform.take() {
            enc.update_constant_buffer(&inputs.transform_block, &t);
        }
        if let Some(l) = inputs.lights.take() {
            enc.update_buffer(&inputs.lights_block, &l, 0)?;
        }
        if let Some(p) = inputs.params.take() {
            enc.update_constant_buffer(&inputs.params_block, &p);
        }
        enc.draw(slice, &self.pso, &pl::Data {
            color: color,
            depth: depth,
            verts: buf,
            scissor: scissor,
            transform: inputs.transform_block.clone(),
            params: inputs.params_block.clone(),
            lights: inputs.lights_block.clone(),
            normal: mat.normal.clone().into_tuple(),
            albedo: mat.albedo.clone().into_tuple(),
            metalness: mat.metalness.clone().into_tuple(),
            roughness: mat.roughness.clone().into_tuple(),
        });
        Ok(())
    }
}
