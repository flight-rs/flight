#![allow(dead_code)]

use gfx;
use gfx::format::*;

pub type ColorFormat = (R8_G8_B8_A8, Unorm);
pub type DepthFormat = (D24, Unorm);
pub type TargetRef<R> = gfx::handle::RenderTargetView<R, ColorFormat>;
pub type DepthRef<R> = gfx::handle::DepthStencilView<R, DepthFormat>;

// Define GFX rendering stuff and pipelines
gfx_defines!{
    vertex Vert {
        a_pos: [f32; 3] = "a_pos",
        a_color: [f32; 3] = "a_color",
    }

    constant TransformBlock {
        model: [[f32; 4]; 4] = "model",
        view: [[f32; 4]; 4] = "view",
        proj: [[f32; 4]; 4] = "proj",
        xoffset: f32 = "xoffset",
    }

    pipeline simple {
        verts: gfx::VertexBuffer<Vert> = (),
        transform: gfx::ConstantBuffer<TransformBlock> = "transform",
        scissor: gfx::Scissor = (), // TODO: Replace scissoring with viewport
        color: gfx::RenderTarget<ColorFormat> = "f_color",
        depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}