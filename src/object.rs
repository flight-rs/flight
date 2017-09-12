pub use gfx::Primitive;
use gfx::{Resources, Slice};
use gfx::traits::FactoryExt;
use gfx::handle::Buffer;
use defines::Vertex;

#[derive(Clone)]
pub enum Indexing {
    Inds(Vec<u32>),
    Range(u32, u32),
    All,
}

#[derive(Clone)]
pub struct ObjectSource<V> {
    pub verts: Vec<V>,
    pub inds: Indexing,
    pub prim: Primitive,
}

pub struct Object<R: Resources, T: Vertex> {
    pub slice: Slice<R>,
    pub buf: Buffer<R, T>,
    pub prim: Primitive,
}

impl<T: Vertex> ObjectSource<T> {
    pub fn build<R: Resources, F: FactoryExt<R>>(self, f: &mut F) -> Object<R, T> {
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
        Object {
            buf: buf,
            slice: slice,
            prim: self.prim,
        }
    }
}