// Crates
#[macro_use]
extern crate log;
#[macro_use]
extern crate gfx;
extern crate nalgebra;
extern crate obj as wavefront;
extern crate fnv;
extern crate image;
extern crate rust_webvr as webvr;

pub mod style;
pub mod load;
pub mod mesh;
pub mod context;
pub mod vr;

#[macro_use]
mod error;
pub use self::error::*;

mod util;
pub use self::util::*;

use gfx::shade::core::CreateShaderError;
use gfx::handle::*;
use gfx::format::*;
use nalgebra::{Point3};

pub type ColorFormat = (R8_G8_B8_A8, Unorm);
pub type DepthFormat = (D24_S8, Unorm);
pub type TargetRef<R> = RenderTargetView<R, ColorFormat>;
pub type DepthRef<R> = DepthStencilView<R, DepthFormat>;
pub type ShaderResult<R> = Result<gfx::ShaderSet<R>, CreateShaderError>;
pub type PbrMesh<R> = mesh::Mesh<R, mesh::VertNTT, style::PbrMaterial<R>>;

#[derive(Copy, Debug, Clone)]
pub struct Light {
    pub pos: Point3<f32>,
    pub color: [f32; 4],
}

impl Default for Light {
    fn default() -> Light {
        Light {
            pos: Point3::origin(),
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
    pub fn uniform_value<F>(f: &mut F, val: <<T as Formatted>::Surface as SurfaceTyped>::DataType)
        -> Result<Self, Error>
        where F: gfx::Factory<R>
    {
        use gfx::texture::*;
        let (_, t): (
            gfx::handle::Texture<R, <T as Formatted>::Surface>,
            _
        ) = f.create_texture_immutable::<T>(
            Kind::D2(1, 1, AaMode::Single),
            &[&[val]],
        )?;
        let s = f.create_sampler(SamplerInfo::new(FilterMethod::Scale, WrapMode::Tile));
        Ok(Texture {
            buffer: t,
            sampler: s,
        })
    }
}