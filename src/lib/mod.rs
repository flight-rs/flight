pub mod style;
pub mod load;
pub mod mesh;
pub mod context;
pub mod volume;

use gfx;
use gfx::shade::core::CreateShaderError;
use gfx::handle::*;
use gfx::format::*;

pub type ColorFormat = (R8_G8_B8_A8, Unorm);
pub type DepthFormat = (D24_S8, Unorm);
pub type TargetRef<R> = RenderTargetView<R, ColorFormat>;
pub type DepthRef<R> = DepthStencilView<R, DepthFormat>;
pub type ShaderResult<R> = Result<gfx::ShaderSet<R>, CreateShaderError>;

// Define GFX rendering stuff and pipelines
gfx_defines!{
    constant TransformBlock {
        model: [[f32; 4]; 4] = "model",
        view: [[f32; 4]; 4] = "view",
        proj: [[f32; 4]; 4] = "proj",
        xoffset: f32 = "xoffset",
    }
}