use gfx::{Resources, CommandBuffer, ShaderSet};
use gfx::pso::{PipelineState};
use gfx::traits::FactoryExt;
use gfx::handle::Buffer;
use gfx::state::Rasterizer;

use super::*;
use defines::solid as solidpso;
use defines::*;
use shaders;

pub struct SolidInputs<R: Resources> {
    shaders: ShaderSet<R>,
    transform: Buffer<R, TransformBlock>,
}

impl<R: Resources> StyleInputs<R> for SolidInputs<R> {
    fn transform_buffer(&self) -> &Buffer<R, TransformBlock> { &self.transform }
    fn shader_set(&self) -> &ShaderSet<R> { &self.shaders }
}

pub struct SolidStyle<R: Resources> {
    pso: PipelineState<R, solidpso::Meta>,
}

impl<R: Resources> Style<R> for SolidStyle<R> {
    type Vertex = VertC;
    type Inputs = SolidInputs<R>;

    fn new<F: Factory<R> + FactoryExt<R>>(
        f: &mut F,
        i: &mut SolidInputs<R>,
        p: Primitive,
        r: Rasterizer,
    ) -> Self {
        SolidStyle {
            pso: f.create_pipeline_state(&i.shaders, p, r, solidpso::new()).unwrap(),
        }
    }

    fn init<F: Factory<R>>(
        f: &mut F,
    ) -> SolidInputs<R> {
        SolidInputs {
            shaders: shaders::unishade(f).unwrap(),
            transform: f.create_constant_buffer(1),
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
    )
        where C: CommandBuffer<R>
    {
        enc.draw(slice, &self.pso, &solidpso::Data {
            color: color,
            depth: depth,
            verts: buf,
            scissor: scissor,
            transform: inputs.transform.clone(),
        });
    }
}