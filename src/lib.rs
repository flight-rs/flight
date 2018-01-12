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
#[cfg(test)]
#[macro_use]
extern crate approx;

/// Mesh drawing
pub mod draw;
/// Asset loading
pub mod load;
/// Mesh specification and upload
pub mod mesh;
/// VR hardware interface
pub mod vr;

#[macro_use]
mod error;
pub use self::error::*;

mod util;
pub use self::util::*;

use gfx::shade::core::CreateShaderError;
use gfx::handle::*;
use gfx::format::*;
use nalgebra::{Point3, UnitQuaternion, Point2};

/// The pixel format of color drawing targets
pub type ColorFormat = (R8_G8_B8_A8, Unorm);
/// The pixel format of depth drawing targets
pub type DepthFormat = (D24_S8, Unorm);
/// The pixel format of shadow depth buffers
pub type ShadowDepthFormat = (D32, Float);
/// Reference to a GPU color target
pub type TargetRef<R> = RenderTargetView<R, ColorFormat>;
/// Reference to a GPU depth target
pub type DepthRef<R> = DepthStencilView<R, DepthFormat>;
/// The result of compiling and linking shader programs
pub type ShaderResult<R> = Result<gfx::ShaderSet<R>, CreateShaderError>;
/// A mesh that can be physically (realistically) rendered
pub type PbrMesh<R> = mesh::Mesh<R, mesh::VertNTT, draw::PbrMaterial<R>>;
pub type UberMesh<R> = mesh::Mesh<R, mesh::VertNTT, draw::UberMaterial<R>>;

/// Parameters for a point light source
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

/// Parameters for a sun light source
#[derive(Copy, Debug, Clone)]
pub struct Sun {
    pub view: UnitQuaternion<f32>,
    pub min_corner: Point2<f32>,
    pub max_corner: Point2<f32>,
}

impl Default for Sun {
    fn default() -> Sun {
        Sun {
            view: UnitQuaternion::identity(),
            min_corner: Point2::origin(),
            max_corner: Point2::origin(),
        }
    }
}

/// GPU-allocated texture object. Since this is just a reference to assets stored on the GPU, 
/// its memory footprint is negligible and it can be cloned freely.
#[derive(Clone)]
pub struct Texture<R, T>
    where R: gfx::Resources, T: TextureFormat
{
    pub sampler: Sampler<R>,
    pub buffer: ShaderResourceView<R, <T as Formatted>::View>,
}

impl<R: gfx::Resources, T: TextureFormat> Texture<R, T> {
    /// Convert this texture reference to an internally recognized tuple form
    pub fn into_tuple(self) -> (ShaderResourceView<R, T::View>, Sampler<R>) {
        (self.buffer, self.sampler)
    }

    /// Build a single-pixel (single value) texture. The overhead on this might still be fairly high, even though the memory usage is minimal.
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
            Mipmap::Provided,
            &[&[val]],
        )?;
        let s = f.create_sampler(SamplerInfo::new(FilterMethod::Scale, WrapMode::Tile));
        Ok(Texture {
            buffer: t,
            sampler: s,
        })
    }
}
