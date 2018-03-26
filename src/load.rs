use wavefront::*;
use image::{self, hdr, GenericImage, RgbaImage, open as open_image, load as load_image};
use gfx;
use gfx::format::*;
use gfx::handle::Sampler;

use fnv::FnvHashMap;
use std::io;
use std::path::Path;
use std::fmt;
use std::mem;

use ::{Error, FlightError, Texture};
use ::mesh::{Mesh, MeshSource, Indexing, VertNT, VertNTT, Primitive};
use ::draw;

/// Load wavefront OBJ data into an internal mesh object
pub fn load_wavefront(obj: &Obj<SimplePolygon>) -> Result<MeshSource<VertNT, ()>, Error> {
    let mut verts = Vec::new();
    let mut ind_look = FnvHashMap::default();
    let mut inds = Vec::new();
    for p in obj.objects.iter().flat_map(|g| &g.groups).flat_map(|g| &g.polys) {
        let poly = p.iter().map(|i| *ind_look.entry((i.0, i.1, i.2)).or_insert_with(|| {
            verts.push(VertNT {
                pos: obj.position[i.0],
                norm: match i.2 { Some(i) => obj.normal[i], None => [0.; 3] },
                tex: match i.1 { Some(i) => obj.texture[i], None => [0.; 2] },
            });
            verts.len() as u32 - 1
        }));
        inds.extend(poly);
    }
    Ok(MeshSource {
        verts: verts,
        inds: Indexing::Inds(inds),
        prim: Primitive::TriangleList,
        mat: (),
    })
}

/// Load a wavefront obj file into an internal mesh object
pub fn open_wavefront<P: AsRef<Path>>(path: P) -> Result<MeshSource<VertNT, ()>, Error> {
    load_wavefront(&Obj::load(path.as_ref())?)
}

pub fn load_integrated_brdf<R, F>(f: &mut F)
    -> Result<Texture<R, (R8_G8, Unorm)>, Error>
    where
        R: gfx::Resources,
        F: gfx::Factory<R>,
{
    let img = load_image(
        io::Cursor::new(&include_bytes!("draw/shaders/brdf_lut.png")[..]),
        image::ImageFormat::PNG)?;
    let (width, height) = img.dimensions();
    let data: Vec<_> = img.to_rgb()
        .pixels()
        .map(|p| [p.data[0], p.data[1]])
        .collect();

    use gfx::texture::*;
    let (_, shader_resource) = f.create_texture_immutable
        ::<(R8_G8, Unorm)>(
        Kind::D2(width as u16, height as u16, AaMode::Single),
        Mipmap::Provided,
        &[&data[..]],
    )?;
    let sampler = f.create_sampler(SamplerInfo::new(
        FilterMethod::Bilinear,
        WrapMode::Border));
    Ok(Texture {
        sampler: sampler,
        buffer: shader_resource,
    })
}

pub fn load_rgba8<R, F, T>(f: &mut F, image: RgbaImage, sampler: Sampler<R>)
    -> Result<Texture<R, (R8_G8_B8_A8, T)>, Error>
    where
        R: gfx::Resources,
        F: gfx::Factory<R>,
        (R8_G8_B8_A8, T): Formatted,
        <(R8_G8_B8_A8, T) as Formatted>::Channel: TextureChannel,
        <(R8_G8_B8_A8, T) as Formatted>::Surface: TextureSurface,
{
    use gfx::texture::*;
    let (width, height) = image.dimensions();
    let (_, shader_resource) = f.create_texture_immutable_u8
        ::<(R8_G8_B8_A8, T)>(
        Kind::D2(width as u16, height as u16, AaMode::Single),
        Mipmap::Provided,
        &[&image.into_raw()[..]],
    )?;
    Ok(Texture {
        sampler: sampler,
        buffer: shader_resource,
    })
}

pub fn open_rgba8<R, F, T, P>(f: &mut F, path: P, sampler: Sampler<R>)
    -> Result<Texture<R, (R8_G8_B8_A8, T)>, Error>
    where
        R: gfx::Resources,
        F: gfx::Factory<R>,
        (R8_G8_B8_A8, T): Formatted,
        <(R8_G8_B8_A8, T) as Formatted>::Channel: TextureChannel,
        <(R8_G8_B8_A8, T) as Formatted>::Surface: TextureSurface,
        P: AsRef<Path>,
{
    load_rgba8(f, open_image(path)?.to_rgba(), sampler)
}

pub fn open_uber_mesh<R, F, P1, P2, P3, P4>(
    f: &mut F,
    wavefront: P1,
    albedo: P2,
    normal: P3,
    knobs: P4,
)
    -> Result<Mesh<R, VertNTT, draw::UberMaterial<R>>, Error>
    where
        R: gfx::Resources,
        F: gfx::Factory<R>,
        P1: AsRef<Path>,
        P2: AsRef<Path>,
        P3: AsRef<Path>,
        P4: AsRef<Path>,
{
    use gfx::texture::*;
    let sampler = f.create_sampler(SamplerInfo::new(
        FilterMethod::Bilinear,
        WrapMode::Tile));
    Ok(open_wavefront(wavefront)?
    .compute_tan()
    .with_material(draw::UberMaterial {
        albedo: open_rgba8(f, albedo, sampler.clone())?,
        normal: open_rgba8(f, normal, sampler.clone())?,
        knobs: open_rgba8(f, knobs, sampler)?,
    }).upload(f))
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CubeSide {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

impl fmt::Display for CubeSide {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CubeSide::*;
        let name = match *self {
            PosX => "posx",
            NegX => "negx",
            PosY => "posy",
            NegY => "negy",
            PosZ => "posz",
            NegZ => "negz",
        };
        write!(f, "{}", name)
    }
}

pub const CUBE_SIDE_ORDER: [CubeSide; 6] = [
    CubeSide::PosX,
    CubeSide::NegX,
    CubeSide::PosY,
    CubeSide::NegY,
    CubeSide::PosZ,
    CubeSide::NegZ,
];

pub fn load_hdr_cubemap<R, F, B, S>(f: &mut F, levels: u8, source: S)
    -> Result<Texture<R, (R32_G32_B32, Float)>, Error>
    where
        R: gfx::Resources,
        F: gfx::Factory<R>,
        B: io::BufRead,
        S: Fn(CubeSide, u8) -> Result<B, Error>,
{
    let mut size = None;
    // image vector
    let mut imgs = Vec::with_capacity(
        CUBE_SIDE_ORDER.len() * levels as usize);
    for &c in &CUBE_SIDE_ORDER {
        for l in 0..levels {
            let img = hdr::HDRDecoder::new(source(c, l)?)?;
            let meta = img.metadata();
            let data = img.read_image_hdr()?;

            // Calculate and verify image size
            let mut size = *size.get_or_insert(meta.width);
            size /= 1 << l;
            ensure!(
                meta.width == size && meta.height == size,
                FlightError::CubemapSizeMismatch { expected: size }
            );

            // Warning! Use of transmute.
            // Make very very sure memory layout is the same.
            assert_eq!(
                mem::size_of::<image::Rgb<f32>>(),
                mem::size_of::<[u32; 3]>(),
            );
            assert_eq!(
                mem::align_of::<image::Rgb<f32>>(),
                mem::align_of::<[u32; 3]>(),
            );
            let img = unsafe { mem::transmute::<
                Vec<image::Rgb<f32>>,
                Vec<[u32; 3]>,
            >(data) };

            imgs.push(img);
        }
    }
    // size must be filled at this point
    let size = size.unwrap();
    // pointer vector
    let refs: Vec<_> = imgs.iter().map(|i| &i[..]).collect();

    use ::gfx::texture::*;
    let sampler = f.create_sampler(SamplerInfo::new(
        FilterMethod::Trilinear, // bilinear + linear between mipmaps
        WrapMode::Border));
    let (_, shader_resource) = f.create_texture_immutable
        ::<(R32_G32_B32, Float)>(
        Kind::Cube(size as u16),
        Mipmap::Provided,
        &refs[..],
    )?;

    Ok(Texture {
        sampler: sampler,
        buffer: shader_resource,
    })
}
