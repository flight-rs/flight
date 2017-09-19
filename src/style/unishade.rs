use gfx::{Resources, CommandBuffer, ShaderSet};
use gfx::pso::{PipelineState};
use gfx::traits::FactoryExt;
use gfx::handle::Buffer;
use gfx::state::Rasterizer;

use super::*;
use defines::unishade as unishadepso;
use defines::*;
use shaders;

pub struct UnishadeInputs<R: Resources> {
    shaders: ShaderSet<R>,
    transform: Buffer<R, TransformBlock>,
    shade: Option<UnishadeBlock>,
    shade_block: Buffer<R, UnishadeBlock>,
}

impl<R: Resources> UnishadeInputs<R> {
    pub fn colors(&mut self, dark: [f32; 4], light: [f32; 4]) {
        self.shade = Some(UnishadeBlock {
            dark: dark,
            light: light,
        })
    }
}

impl<R: Resources> StyleInputs<R> for UnishadeInputs<R> {
    fn transform_buffer(&self) -> &Buffer<R, TransformBlock> { &self.transform }
    fn shader_set(&self) -> &ShaderSet<R> { &self.shaders }
}

pub struct UnishadeStyle<R: Resources> {
    pso: PipelineState<R, unishadepso::Meta>,
}

impl<R: Resources> Style<R> for UnishadeStyle<R> {
    type Vertex = VertN;
    type Inputs = UnishadeInputs<R>;

    fn new<F: Factory<R> + FactoryExt<R>>(
        f: &mut F,
        i: &mut UnishadeInputs<R>,
        p: Primitive,
        r: Rasterizer,
    ) -> Self {
        UnishadeStyle {
            pso: f.create_pipeline_state(&i.shaders, p, r, unishadepso::new()).unwrap(),
        }
    }

    fn init<F: Factory<R>>(
        f: &mut F,
    ) -> UnishadeInputs<R> {
        UnishadeInputs {
            shaders: shaders::unishade(f).unwrap(),
            transform: f.create_constant_buffer(1),
            shade: None,
            shade_block: f.create_constant_buffer(1),
        }
    }
    
    fn draw_raw<C>(
        &self,
        inputs: &mut UnishadeInputs<R>,
        enc: &mut Encoder<R, C>,
        color: TargetRef<R>,
        depth: DepthRef<R>,
        scissor: Rect,
        slice: &Slice<R>,
        buf: Buffer<R, Self::Vertex>,
    )
        where C: CommandBuffer<R>
    {
        if let Some(shade) = inputs.shade.take() {
            enc.update_constant_buffer(&inputs.shade_block, &shade);
        }
        enc.draw(slice, &self.pso, &unishadepso::Data {
            color: color,
            depth: depth,
            verts: buf,
            scissor: scissor,
            transform: inputs.transform.clone(),
            shade: inputs.shade_block.clone(),
        });
    }
}