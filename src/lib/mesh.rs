pub use gfx::Primitive;
use gfx::{Resources, Slice, traits, pso};
use gfx::format::Format;
use gfx::traits::FactoryExt;
use gfx::handle::Buffer;

gfx_defines!{
    vertex VertN {
        pos: [f32; 3] = "a_pos",
        norm: [f32; 3] = "a_norm",
    }

    vertex VertC {
        pos: [f32; 3] = "a_pos",
        color: [f32; 3] = "a_color",
    }
}
pub trait Vertex: traits::Pod + pso::buffer::Structure<Format> { }
impl Vertex for VertN { }
impl Vertex for VertC { }

#[derive(Clone)]
pub enum Indexing {
    Inds(Vec<u32>),
    Range(u32, u32),
    All,
}

#[derive(Clone)]
pub struct MeshSource<V> {
    pub verts: Vec<V>,
    pub inds: Indexing,
    pub prim: Primitive,
}

#[derive(Clone)]
pub struct Mesh<R: Resources, T: Vertex> {
    pub slice: Slice<R>,
    pub buf: Buffer<R, T>,
    pub prim: Primitive,
}

impl<T: Vertex> MeshSource<T> {
    pub fn build<R: Resources, F: FactoryExt<R>>(self, f: &mut F) -> Mesh<R, T> {
        use self::Indexing::*;

        let (buf, slice) = match self.inds {
            All => {
                let buf = f.create_vertex_buffer(&self.verts);
                let slice = Slice::new_match_vertex_buffer(&buf);
                (buf, slice)
            },
            Range(a, b) => {
                let buf = f.create_vertex_buffer(&self.verts);
                let mut slice = Slice::new_match_vertex_buffer(&buf);
                slice.start = a;
                slice.end = b;
                (buf, slice)
            },
            Inds(ref i) => {
                f.create_vertex_buffer_with_slice(&self.verts, &i[..])
            }
        };
        Mesh {
            buf: buf,
            slice: slice,
            prim: self.prim,
        }
    }
}