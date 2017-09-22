use gfx::{self, Resources, CommandBuffer, ShaderSet, Factory, Rect, Slice, Encoder};
use gfx::pso::PipelineState;
use gfx::traits::FactoryExt;
use gfx::handle::Buffer;
use gfx::state::Rasterizer;

use super::{StyleInputs, Style};
use super::shaders::file;
use lib::mesh::{Primitive, VertC};
use lib::{TransformBlock, ColorFormat, DepthFormat, TargetRef, DepthRef};

gfx_defines!{
    pipeline pl {
        verts: gfx::VertexBuffer<VertC> = (),
        transform: gfx::ConstantBuffer<TransformBlock> = "transform",
        scissor: gfx::Scissor = (), // TODO: Replace scissoring with viewport
        color: gfx::RenderTarget<ColorFormat> = "f_color",
        depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

shader!(shader {
    vertex: file("shaders/transform.v.glsl")
        .define("COLOR"),
    fragment: file("shaders/simple.f.glsl")
        .define_to("I_POS", "v_pos")
        .define_to("I_COLOR", "v_color")
});

pub struct SolidInputs<R: Resources> {
    shaders: ShaderSet<R>,
    transform: Option<TransformBlock>,
    transform_block: Buffer<R, TransformBlock>,
}

impl<R: Resources> StyleInputs<R> for SolidInputs<R> {
    fn transform(&mut self, block: TransformBlock) { self.transform = Some(block); }
    fn shader_set(&self) -> &ShaderSet<R> { &self.shaders }
}

pub struct SolidStyle<R: Resources> {
    pso: PipelineState<R, pl::Meta>,
}

impl<R: Resources> Style<R> for SolidStyle<R> {
    type Vertex = VertC;
    type Inputs = SolidInputs<R>;
    type Material = ();

    fn new<F: Factory<R> + FactoryExt<R>>(
        f: &mut F,
        i: &mut SolidInputs<R>,
        p: Primitive,
        r: Rasterizer,
    ) -> Self {
        SolidStyle {
            pso: f.create_pipeline_state(&i.shaders, p, r, pl::new()).unwrap(),
        }
    }

    fn init<F: Factory<R>>(
        f: &mut F,
    ) -> SolidInputs<R> {
        SolidInputs {
            shaders: shader(f).unwrap(),
            transform: None,
            transform_block: f.create_constant_buffer(1),
        }
    }
    
    fn draw_raw<C>(
        &self,
        inputs: &mut SolidInputs<R>,
        enc: &mut Encoder<R, C>,
        color: TargetRef<R>,
        depth: DepthRef<R>,
        scissor: Rect,
        slice: &Slice<R>,
        buf: Buffer<R, Self::Vertex>,
        _: &(),
    )
        where C: CommandBuffer<R>
    {
        if let Some(t) = inputs.transform.take() { 
            enc.update_constant_buffer(&inputs.transform_block, &t);
        }
        enc.draw(slice, &self.pso, &pl::Data {
            color: color,
            depth: depth,
            verts: buf,
            scissor: scissor,
            transform: inputs.transform_block.clone(),
        });
    }
}