use gfx::{self, Resources, CommandBuffer, ShaderSet, Factory, Rect, Slice, Encoder};
use gfx::pso::PipelineState;
use gfx::traits::FactoryExt;
use gfx::handle::Buffer;
use gfx::state::Rasterizer;

use super::{StyleInputs, Style};
use super::shaders::file;
use lib::mesh::{Primitive, VertN};
use lib::{Error, TransformBlock, ColorFormat, DepthFormat, TargetRef, DepthRef};

gfx_defines!{
    constant UnishadeBlock {
        dark: [f32; 4] = "dark",
        light: [f32; 4] = "light",
    }

    pipeline pl {
        verts: gfx::VertexBuffer<VertN> = (),
        transform: gfx::ConstantBuffer<TransformBlock> = "transform",
        shade: gfx::ConstantBuffer<UnishadeBlock> = "shade",
        scissor: gfx::Scissor = (), // TODO: Replace scissoring with viewport
        color: gfx::RenderTarget<ColorFormat> = "f_color",
        depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

shader!(shader {
    vertex: file("shaders/transform.v.glsl")?
        .define("NORM"),
    fragment: file("shaders/unishade.f.glsl")?
        .define_to("I_POS", "v_pos")
        .define_to("I_NORM", "v_norm")
});

pub struct UnishadeInputs<R: Resources> {
    shaders: ShaderSet<R>,
    transform: Option<TransformBlock>,
    transform_block: Buffer<R, TransformBlock>,
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
    fn transform(&mut self, block: TransformBlock) { self.transform = Some(block); }
    fn shader_set(&self) -> &ShaderSet<R> { &self.shaders }
}

pub struct UnishadeStyle<R: Resources> {
    pso: PipelineState<R, pl::Meta>,
}

impl<R: Resources> Style<R> for UnishadeStyle<R> {
    type Vertex = VertN;
    type Inputs = UnishadeInputs<R>;
    type Material = ();

    fn new<F: Factory<R> + FactoryExt<R>>(
        f: &mut F,
        i: &mut UnishadeInputs<R>,
        p: Primitive,
        r: Rasterizer,
    ) -> Result<Self, Error> {
        Ok(UnishadeStyle {
            pso: f.create_pipeline_state(&i.shaders, p, r, pl::new())?,
        })
    }

    fn init<F: Factory<R>>(
        f: &mut F,
    ) -> Result<UnishadeInputs<R>, Error> {
        Ok(UnishadeInputs {
            shaders: shader(f)?,
            transform: None,
            transform_block: f.create_constant_buffer(1),
            shade: None,
            shade_block: f.create_constant_buffer(1),
        })
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
        _: &(),
    )
        -> Result<(), Error>
        where C: CommandBuffer<R>
    {
        if let Some(t) = inputs.transform.take() { 
            enc.update_constant_buffer(&inputs.transform_block, &t);
        }
        if let Some(shade) = inputs.shade.take() {
            enc.update_constant_buffer(&inputs.shade_block, &shade);
        }
        enc.draw(slice, &self.pso, &pl::Data {
            color: color,
            depth: depth,
            verts: buf,
            scissor: scissor,
            transform: inputs.transform_block.clone(),
            shade: inputs.shade_block.clone(),
        });
        Ok(())
    }
}