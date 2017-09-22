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
pub type PbrMesh<R> = mesh::Mesh<R, mesh::VertNTT, style::PbrMaterial<R>>;

// Define GFX rendering stuff and pipelines
gfx_defines!{
    constant TransformBlock {
        model: [[f32; 4]; 4] = "model",
        view: [[f32; 4]; 4] = "view",
        proj: [[f32; 4]; 4] = "proj",
        eye: [f32; 4] = "eye_pos",
        xoffset: f32 = "xoffset",
    }

    constant Light {
        pos: [f32; 4] = "pos",
        color: [f32; 4] = "color",
    }
}

impl Default for Light {
    fn default() -> Light {
        Light {
            pos: [0.; 4],
            color: [0.; 4],
        }
    }
}

#[derive(Clone)]
pub struct Texture<R, T>
    where R: gfx::Resources, T: TextureFormat
{
    pub sampler: Sampler<R>,
    pub buffer: ShaderResourceView<R, <T as Formatted>::View>,
}

impl<R: gfx::Resources, T: TextureFormat> Texture<R, T> {
    pub fn into_tuple(self) -> (ShaderResourceView<R, T::View>, Sampler<R>) {
        (self.buffer, self.sampler)
    }

    /// Build a single-pixel (single value) texture
    pub fn uniform_value<F>(f: &mut F, val: <<T as Formatted>::Surface as SurfaceTyped>::DataType) -> Self
        where F: gfx::Factory<R>
    {
        use gfx::texture::*;
        let (_, t): (
            gfx::handle::Texture<R, <T as Formatted>::Surface>,
            _
        ) = f.create_texture_immutable::<T>(
            Kind::D2(1, 1, AaMode::Single),
            &[&[val]],
        ).unwrap();
        let s = f.create_sampler(SamplerInfo::new(FilterMethod::Scale, WrapMode::Tile));
        Texture {
            buffer: t,
            sampler: s,
        }
    }
}