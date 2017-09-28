pub use gfx::Primitive;
use gfx::{Resources, Slice, traits, pso};
use gfx::format::Format;
use gfx::traits::FactoryExt;
use gfx::handle::Buffer;
use nalgebra::{self as na, Point3, Point2, Vector3};
use ::NativeRepr;

gfx_defines!{
    /// A vertex that includes pos only.
    vertex Vert {
        pos: [f32; 3] = "a_pos",
    }

    /// A vertex that includes pos and norm.
    vertex VertN {
        pos: [f32; 3] = "a_pos",
        norm: [f32; 3] = "a_norm",
    }

    /// A vertex that includes pos and color.
    vertex VertC {
        pos: [f32; 3] = "a_pos",
        color: [f32; 3] = "a_color",
    }

    /// A vertex that includes pos, norm, and color.
    vertex VertNC {
        pos: [f32; 3] = "a_pos",
        norm: [f32; 3] = "a_norm",
        color: [f32; 3] = "a_color",
    }

    /// A vertex that includes pos, norm, and tex.
    vertex VertNT {
        pos: [f32; 3] = "a_pos",
        norm: [f32; 3] = "a_norm",
        tex: [f32; 2] = "a_tex",
    }

    /// A vertex that includes pos, norm, tan, bitan, and tex.
    vertex VertNTT {
        pos: [f32; 3] = "a_pos",
        norm: [f32; 3] = "a_norm",
        tan: [f32; 3] = "a_tan",
        bitan: [f32; 3] = "a_bitan",
        tex: [f32; 2] = "a_tex",
    }
}

/// A type that can be used as a vertex.
pub trait Vertex: traits::Pod + pso::buffer::Structure<Format> {
    fn pos(&self) -> &Point3<f32>;
    fn mut_pos(&mut self) -> &mut Point3<f32>;
}

/// A vertex that can have a norm attribute added.
pub trait WithNorm: Vertex {
    type With: HasNorm;
    fn with_norm(self, norm: Vector3<f32>) -> Self::With;
}

/// A vertex with a norm component.
pub trait HasNorm: Vertex {
    fn norm(&self) -> &Vector3<f32>;
    fn mut_norm(&mut self) -> &mut Vector3<f32>;
}

/// A vertex that can have a color attribute added.
pub trait WithColor: Vertex {
    type With: HasColor;
    fn with_color(self, color: [f32; 3]) -> Self::With;
}

/// A vertex with a color component.
pub trait HasColor: Vertex {
    fn color(&self) -> &[f32; 3];
    fn mut_color(&mut self) -> &mut [f32; 3];
}

/// A vertex that can have a tex attribute added.
pub trait WithTex: Vertex {
    type With: HasTex;
    fn with_tex(self, tex: Point2<f32>) -> Self::With;
}

/// A vertex with a tex component.
pub trait HasTex: Vertex {
    fn tex(&self) -> &Point2<f32>;
    fn mut_tex(&mut self) -> &mut Point2<f32>;
}

/// A vertex that can have tan and bitan attributes added.
pub trait WithTan: Vertex {
    type With: HasTan;
    fn with_tan(self, tan: Vector3<f32>, bitan: Vector3<f32>) -> Self::With;
}

/// A vertex with a tan component.
pub trait HasTan: Vertex {
    fn tan(&self) -> &Vector3<f32>;
    fn mut_tan(&mut self) -> &mut Vector3<f32>;
    fn bitan(&self) -> &Vector3<f32>;
    fn mut_bitan(&mut self) -> &mut Vector3<f32>;
}

macro_rules! vertex_fn {
    ($n:ident, $o:ident, $s:ident, $norm:ident: norm, $c:tt) => {
        impl WithNorm for $n {
            type With = $o;
            fn with_norm($s, $norm: Vector3<f32>) -> $o { $o $c }
        }
    };
    ($n:ident, $o:ident, $s:ident, $color:ident: color, $c:tt) => {
        impl WithColor for $n {
            type With = $o;
            fn with_color($s, $color: [f32; 3]) -> $o { $o $c }
        }
    };
    ($n:ident, $o:ident, $s:ident, $tex:ident: tex, $c:tt) => {
        impl WithTex for $n {
            type With = $o;
            fn with_tex($s, $tex: Point2<f32>) -> $o { $o $c }
        }
    };
    ($n:ident, $o:ident, $s:ident, $tan:ident: tan, $bitan:ident: bitan, $c:tt) => {
        impl WithTan for $n {
            type With = $o;
            fn with_tan($s, $tan: Vector3<f32>, $bitan: Vector3<f32>) -> $o { $o $c }
        }
    };
}

macro_rules! vertex_component {
    ($n:ident, tex) => { impl HasTex for $n {
        fn tex(&self) -> &Point2<f32> { NativeRepr::advanced_ref(&self.tex) }
        fn mut_tex(&mut self) -> &mut Point2<f32> { NativeRepr::advanced_mut(&mut self.tex) }
    } };
    ($n:ident, norm) => { impl HasNorm for $n {
        fn norm(&self) -> &Vector3<f32> { NativeRepr::advanced_ref(&self.norm) }
        fn mut_norm(&mut self) -> &mut Vector3<f32> { NativeRepr::advanced_mut(&mut self.norm) }
    } };
    ($n:ident, color) => { impl HasColor for $n {
        fn color(&self) -> &[f32; 3] { &self.color }
        fn mut_color(&mut self) -> &mut [f32; 3] { &mut self.color }
    } };
    ($n:ident, tan) => { impl HasTan for $n {
        fn tan(&self) -> &Vector3<f32> { NativeRepr::advanced_ref(&self.tan) }
        fn mut_tan(&mut self) -> &mut Vector3<f32> { NativeRepr::advanced_mut(&mut self.tan) }
        fn bitan(&self) -> &Vector3<f32> { NativeRepr::advanced_ref(&self.bitan) }
        fn mut_bitan(&mut self) -> &mut Vector3<f32> { NativeRepr::advanced_mut(&mut self.bitan) }
    } };
}

macro_rules! impl_vertex {
    ($n:ident { $(&self.$g:ident;)* $($o:ident($s:ident, $($a:ident: $p:ident),*) $c:tt;)* }) => {
        impl Vertex for $n {
            fn pos(&self) -> &Point3<f32> { NativeRepr::advanced_ref(&self.pos) }
            fn mut_pos(&mut self) -> &mut Point3<f32> { NativeRepr::advanced_mut(&mut self.pos) }
        }
        $(vertex_component!($n, $g);)*
        $(vertex_fn!($n, $o, $s, $($a: $p),*, $c);)*
    };
}

impl_vertex!(Vert {
    VertN(self, n: norm) {
        pos: self.pos,
        norm: n.native(),
    };
    VertC(self, c: color) {
        pos: self.pos,
        color: c,
    };
});

impl_vertex!(VertN {
    &self.norm;
    VertNC(self, c: color) {
        pos: self.pos,
        norm: self.norm,
        color: c,
    };
    VertNT(self, t: tex) {
        pos: self.pos,
        norm: self.norm,
        tex: t.native(),
    };
});

impl_vertex!(VertNT {
    &self.tex;
    &self.norm;
    VertNTT(self, t: tan, b: bitan) {
        pos: self.pos,
        norm: self.norm,
        tex: self.tex,
        tan: t.native(),
        bitan: b.native(),
    };
});

impl_vertex!(VertNTT {
    &self.norm;
    &self.tex;
    &self.tan;
});

impl_vertex!(VertC {
    &self.color;
    VertNC(self, n: norm) {
        pos: self.pos,
        color: self.color,
        norm: n.native(),
    };
});

impl_vertex!(VertNC {
    &self.norm;
    &self.color;
});

/// A scheme for selecting vertices to combine into primitives.
#[derive(Clone)]
pub enum Indexing {
    Inds(Vec<u32>),
    Range(u32, u32),
    All,
}

/// A mesh description that must be sent to the GPU before it can be drawn.
#[derive(Clone)]
pub struct MeshSource<V, M> {
    /// Vertices
    pub verts: Vec<V>,
    /// Indexing scheme
    pub inds: Indexing,
    /// Primitive type
    pub prim: Primitive,
    /// Material data
    pub mat: M,
}

/// A reference to a GPU mesh object that can be drawn.
#[derive(Clone)]
pub struct Mesh<R: Resources, T: Vertex, M> {
    /// Reference to slice object (index buffer or range)
    pub slice: Slice<R>,
    /// Reference to VBO
    pub buf: Buffer<R, T>,
    /// Primitive type
    pub prim: Primitive,
        /// Material data
    pub mat: M,
}

impl<T: Vertex, M> MeshSource<T, M> {
    /// Transfer this mesh to the GPU.
    pub fn build<R: Resources, F: FactoryExt<R>>(self, f: &mut F) -> Mesh<R, T, M> {
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
            mat: self.mat,
        }
    }

    pub fn with_material<N>(self, mat: N) -> MeshSource<T, N> {
        MeshSource {
            verts: self.verts,
            inds: self.inds,
            prim: self.prim,
            mat: mat,
        }
    }
}

impl<V: WithNorm, M> MeshSource<V, M> {
    pub fn with_normal(self, n: Vector3<f32>) -> MeshSource<V::With, M> {
        MeshSource {
            verts: self.verts.into_iter().map(|v| v.with_norm(n.into())).collect(),
            inds: self.inds,
            prim: self.prim,
            mat: self.mat,
        }
    }
}

impl<V: WithColor, M> MeshSource<V, M> {
    pub fn with_color(self, c: [f32; 3]) -> MeshSource<V::With, M> {
        MeshSource {
            verts: self.verts.into_iter().map(|v| v.with_color(c)).collect(),
            inds: self.inds,
            prim: self.prim,
            mat: self.mat,
        }
    }
}

impl<V: WithTex, M> MeshSource<V, M> {
    pub fn with_tex(self, c: Point2<f32>) -> MeshSource<V::With, M> {
        MeshSource {
            verts: self.verts.into_iter().map(|v| v.with_tex(c)).collect(),
            inds: self.inds,
            prim: self.prim,
            mat: self.mat,
        }
    }
}

fn add_tri_tan<T: HasTan + HasTex>(a: &mut T, b: &mut T, c: &mut T) {
    let (tan, bitan) = {
        // positions
        let pos1 = a.pos();
        let pos2 = b.pos();
        let pos3 = c.pos();
        // texture coordinates
        let uv1 = a.tex();
        let uv2 = b.tex();
        let uv3 = c.tex();

        // deltas
        let edge1 = pos2 - pos1;
        let edge2 = pos3 - pos1;
        let delta_uv1 = uv2 - uv1;
        let delta_uv2 = uv3 - uv1;

        let f = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

        let tan = Vector3::new(
            f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x),
            f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y),
            f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z),
        ).normalize();

        let bitan = Vector3::new(
            f * (-delta_uv2.x * edge1.x + delta_uv1.x * edge2.x),
            f * (-delta_uv2.x * edge1.y + delta_uv1.x * edge2.y),
            f * (-delta_uv2.x * edge1.z + delta_uv1.x * edge2.z),
        ).normalize();

        (tan, bitan)
    };

    *a.mut_tan() += tan;
    *b.mut_tan() += tan;
    *c.mut_tan() += tan;

    *a.mut_bitan() += bitan;
    *b.mut_bitan() += bitan;
    *c.mut_bitan() += bitan;
}

unsafe fn mut_ind<T>(arr: &[T], i: usize) -> &mut T {
    let ptr: *mut T = ::std::mem::transmute(&arr[i] as *const T);
    ptr.as_mut().unwrap()
}

fn add_tans<I, V>(mut inds: I, tris: &mut Vec<V>, p: Primitive)
    where I: Iterator<Item=usize>, V: HasTan + HasTex
{
    use self::Primitive::*;
    match p {
        TriangleList => {
            while let (Some(a), Some(b), Some(c)) = (inds.next(), inds.next(), inds.next()) {
                assert!(a != b && b != c && a != c);
                unsafe { add_tri_tan(mut_ind(tris, a), mut_ind(tris, b), mut_ind(tris, c)) };
            }
        },
        TriangleStrip => {
            let mut a = match inds.next() { Some(i) => i, None => return };
            let mut b = match inds.next() { Some(i) => i, None => return };
            for c in inds {
                assert!(a != b && b != c && a != c);
                unsafe { add_tri_tan(mut_ind(tris, a), mut_ind(tris, b), mut_ind(tris, c)) };
                a = b;
                b = c;
            }
        },
        _ => (),
    }
}

impl<V, M> MeshSource<V, M>
    where V: WithTan + HasTex, V::With: HasTex
{
    /// Computes tangents and bitangents for a textured mesh so that normal mapping can be used.
    /// The calculated vectors will be 0 if the primitive type is not `TriangleList` or `TriangleStrip`.
    pub fn compute_tan(self) -> MeshSource<V::With, M> {
        let mut new: Vec<_> = self.verts.into_iter()
            .map(|v| v.with_tan(na::zero(), na::zero()))
            .collect();
        use self::Indexing::*;
        match self.inds {
            Inds(ref inds) => add_tans(inds.iter().map(|&i| i as usize), &mut new, self.prim),
            All => add_tans(0..new.len(), &mut new, self.prim),
            Range(a, b) => add_tans(a as usize..b as usize, &mut new, self.prim),
        };
        for v in new.iter_mut() {
            use std::f32::EPSILON;
            v.mut_tan().try_normalize_mut(EPSILON);
            v.mut_bitan().try_normalize_mut(EPSILON);
        }
        MeshSource {
            verts: new,
            inds: self.inds,
            prim: self.prim,
            mat: self.mat,
        }
    }
}