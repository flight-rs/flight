#![allow(dead_code)]

use gfx::{self, traits, handle, pso};
use gfx::format::*;

pub type ColorFormat = (R8_G8_B8_A8, Unorm);
pub type DepthFormat = (D24, Unorm);
pub type TargetRef<R> = handle::RenderTargetView<R, ColorFormat>;
pub type DepthRef<R> = handle::DepthStencilView<R, DepthFormat>;

// Define GFX rendering stuff and pipelines
gfx_defines!{
    vertex VertN {
        pos: [f32; 3] = "a_pos",
        norm: [f32; 3] = "a_norm",
    }

    vertex VertC {
        pos: [f32; 3] = "a_pos",
        color: [f32; 3] = "a_color",
    }

    constant TransformBlock {
        model: [[f32; 4]; 4] = "model",
        view: [[f32; 4]; 4] = "view",
        proj: [[f32; 4]; 4] = "proj",
        xoffset: f32 = "xoffset",
    }

    constant UnishadeBlock {
        dark: [f32; 4] = "dark",
        light: [f32; 4] = "light",
    }

    pipeline solid {
        verts: gfx::VertexBuffer<VertC> = (),
        transform: gfx::ConstantBuffer<TransformBlock> = "transform",
        scissor: gfx::Scissor = (), // TODO: Replace scissoring with viewport
        color: gfx::RenderTarget<ColorFormat> = "f_color",
        depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }

    pipeline unishade {
        verts: gfx::VertexBuffer<VertN> = (),
        transform: gfx::ConstantBuffer<TransformBlock> = "transform",
        shade: gfx::ConstantBuffer<UnishadeBlock> = "shade",
        scissor: gfx::Scissor = (), // TODO: Replace scissoring with viewport
        color: gfx::RenderTarget<ColorFormat> = "f_color",
        depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

pub trait Vertex: traits::Pod + pso::buffer::Structure<Format> { }
impl Vertex for VertN { }
impl Vertex for VertC { }