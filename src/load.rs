use wavefront::*;
use image::{DynamicImage, Pixel, ImageBuffer};
use image::open as open_image;
use gfx;
use gfx::format;
use gfx::texture::{Kind, AaMode};
use gfx::handle;

use fnv::FnvHashMap;
use std::ops::Deref;
use std::path::Path;

use ::{Error, Texture};
use ::mesh::{Mesh, MeshSource, Indexing, VertNT, VertNTT, Primitive};
use ::draw::PbrMaterial;

/// Load wavefront OBJ data into an internal mesh object 
pub fn wavefront_data(obj: &Obj<SimplePolygon>) -> Result<MeshSource<VertNT, ()>, Error> {
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
pub fn wavefront_file<P: AsRef<Path>>(path: P) -> Result<MeshSource<VertNT, ()>, Error> {
    wavefront_data(&Obj::load(path.as_ref())?)
}

/// Load some image data into a GPU-allocated internal texture object
pub fn image_data<R, F, T>(f: &mut F, img: DynamicImage, samp: handle::Sampler<R>, aa: AaMode)
    -> Result<Texture<R, T>, Error>
    where
        R: gfx::Resources,
        F: gfx::Factory<R>,
        T: format::TextureFormat,
        <<T as format::Formatted>::Surface as format::SurfaceTyped>::DataType: ImageData,
{
    let data = <T::Surface as format::SurfaceTyped>::DataType::load(&img, aa);
    let (_, t): (
        gfx::handle::Texture<R, <T as format::Formatted>::Surface>,
        _
    ) = f.create_texture_immutable_u8::<T>(
        data.0,
        &[data.1.as_ref()],
    )?;
    Ok(Texture {
        buffer: t,
        sampler: samp,
    })
}

/// A binary pixel format that can be used by the GPU
pub trait ImageData {
    // TODO: make more efficient (currently requires too much iterating and allocating)
    /// Convert the DynamicImage object to a simple array of bytes following this format
    fn load(img: &DynamicImage, aa: AaMode) -> (Kind, Vec<u8>);
}

fn array_data<P, S>(buf: ImageBuffer<P, S>, aa: AaMode) -> (Kind, Vec<u8>)
    where
        P: Pixel<Subpixel=u8> + 'static,
        S: Deref<Target = [u8]>,
{
    (
        Kind::D2(buf.width() as u16, buf.height() as u16, aa),
        buf.into_raw().deref().to_vec(),
    )
}

impl ImageData for [u8; 4] {
    fn load(img: &DynamicImage, aa: AaMode) -> (Kind, Vec<u8>) {
        array_data(img.to_rgba(), aa)
    }
}

impl ImageData for [u8; 3] {
    fn load(img: &DynamicImage, aa: AaMode) -> (Kind, Vec<u8>) {
        array_data(img.to_rgb(), aa)
    }
}

impl ImageData for u8 {
    fn load(img: &DynamicImage, aa: AaMode) -> (Kind, Vec<u8>) {
        array_data(img.to_luma(), aa)
    }
}

/// Load a physically based object out of a directory. Will load
///`normal.png`, `albedo.png`, `metalness.png`, `roughness.png`, and `model.obj`.
pub fn object_directory<R, F, P>(f: &mut F, path: P)
    -> Result<Mesh<R, VertNTT, PbrMaterial<R>>, Error>
    where
        R: gfx::Resources,
        F: gfx::Factory<R>,
        P: AsRef<Path>,
{
    use gfx::texture::*;
    let aa = AaMode::Single;
    let path = path.as_ref();
    info!("Loading object in {:?}", path);

    let sampler = f.create_sampler(SamplerInfo::new(FilterMethod::Bilinear, WrapMode::Tile));

    let normal = image_data(
        f,
        open_image(path.join("normal.png"))?,
        sampler.clone(),
        aa
    )?;
    let albedo = image_data(
        f,
        open_image(path.join("albedo.png"))?,
        sampler.clone(),
        aa
    )?;
    let metalness = image_data(
        f,
        open_image(path.join("metalness.png"))?,
        sampler.clone(),
        aa
    )?;
    let roughness = image_data(
        f,
        open_image(path.join("roughness.png"))?,
        sampler.clone(),
        aa
    )?;
    Ok(wavefront_file(path.join("model.obj"))?
    .compute_tan()
    .with_material(PbrMaterial {
        normal: normal,
        albedo: albedo,
        metalness: metalness,
        roughness: roughness,
    }).upload(f))
}