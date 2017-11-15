pub use gfx::Primitive;
use gfx::{Resources, Slice, traits, pso};
use gfx::format::Format;
use gfx::traits::FactoryExt;
use gfx::handle::Buffer;
use nalgebra::{self as na, Point3, Point2, Vector3};
use ::NativeRepr;
use std::f32::EPSILON;

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
    /// Get the vertex's position
    fn pos(&self) -> &Point3<f32>;
    /// Change the vertex's position
    fn mut_pos(&mut self) -> &mut Point3<f32>;
}

/// A vertex that can have a norm attribute added.
pub trait WithNorm: Vertex {
    type With: HasNorm;
    /// Add a normal vector to this vertex's attributes
    fn with_norm(self, norm: Vector3<f32>) -> Self::With;
}

/// A vertex with a norm component.
pub trait HasNorm: Vertex {
    /// Get the vertex's normal vector
    fn norm(&self) -> &Vector3<f32>;
    /// Change the vertex's normal vector
    fn mut_norm(&mut self) -> &mut Vector3<f32>;
}

/// A vertex that can have a color attribute added.
pub trait WithColor: Vertex {
    type With: HasColor;
    /// Add a color to this vertex's attributes
    fn with_color(self, color: [f32; 3]) -> Self::With;
}

/// A vertex with a color component.
pub trait HasColor: Vertex {
    /// Get the vertex's color
    fn color(&self) -> &[f32; 3];
    /// Change the vertex's color
    fn mut_color(&mut self) -> &mut [f32; 3];
}

/// A vertex that can have a tex attribute added.
pub trait WithTex: Vertex {
    type With: HasTex;
    /// Add a texture or UV coordinate to this vertex's attributes
    fn with_tex(self, tex: Point2<f32>) -> Self::With;
}

/// A vertex with a tex component.
pub trait HasTex: Vertex {
    /// Get the vertex's texture or UV coordinates
    fn tex(&self) -> &Point2<f32>;
    /// Change the vertex's texture or UV coordinates
    fn mut_tex(&mut self) -> &mut Point2<f32>;
}

/// A vertex that can have tan and bitan attributes added.
pub trait WithTan: Vertex {
    type With: HasTan;
    /// Add tangent and bitangent vectors to this vertex's attributes
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
        fn tex(&self) -> &Point2<f32> { NativeRepr::upgrade_ref(&self.tex) }
        fn mut_tex(&mut self) -> &mut Point2<f32> { NativeRepr::upgrade_mut(&mut self.tex) }
    } };
    ($n:ident, norm) => { impl HasNorm for $n {
        fn norm(&self) -> &Vector3<f32> { NativeRepr::upgrade_ref(&self.norm) }
        fn mut_norm(&mut self) -> &mut Vector3<f32> { NativeRepr::upgrade_mut(&mut self.norm) }
    } };
    ($n:ident, color) => { impl HasColor for $n {
        fn color(&self) -> &[f32; 3] { &self.color }
        fn mut_color(&mut self) -> &mut [f32; 3] { &mut self.color }
    } };
    ($n:ident, tan) => { impl HasTan for $n {
        fn tan(&self) -> &Vector3<f32> { NativeRepr::upgrade_ref(&self.tan) }
        fn mut_tan(&mut self) -> &mut Vector3<f32> { NativeRepr::upgrade_mut(&mut self.tan) }
        fn bitan(&self) -> &Vector3<f32> { NativeRepr::upgrade_ref(&self.bitan) }
        fn mut_bitan(&mut self) -> &mut Vector3<f32> { NativeRepr::upgrade_mut(&mut self.bitan) }
    } };
}

macro_rules! impl_vertex {
    ($n:ident { $(&self.$g:ident;)* $($o:ident($s:ident, $($a:ident: $p:ident),*) $c:tt;)* }) => {
        impl Vertex for $n {
            fn pos(&self) -> &Point3<f32> { NativeRepr::upgrade_ref(&self.pos) }
            fn mut_pos(&mut self) -> &mut Point3<f32> { NativeRepr::upgrade_mut(&mut self.pos) }
        }
        $(vertex_component!($n, $g);)*
        $(vertex_fn!($n, $o, $s, $($a: $p),*, $c);)*
    };
}

impl_vertex!(Vert {
    VertN(self, n: norm) {
        pos: self.pos,
        norm: n.downgrade(),
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
        tex: t.downgrade(),
    };
});

impl_vertex!(VertNT {
    &self.tex;
    &self.norm;
    VertNTT(self, t: tan, b: bitan) {
        pos: self.pos,
        norm: self.norm,
        tex: self.tex,
        tan: t.downgrade(),
        bitan: b.downgrade(),
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
        norm: n.downgrade(),
    };
});

impl_vertex!(VertNC {
    &self.norm;
    &self.color;
});

/// A scheme for selecting vertices to combine into primitives.
#[derive(Clone)]
pub enum Indexing {
    /// A list of vertex indices to make primitives out of
    Inds(Vec<u32>),
    /// A range of vertices in sequence to make primitives out of
    Range(u32, u32),
    /// Use all vertices in sequence to make primitives
    All,
}

/// A mesh storage object and builder that sent as drawable geometry to the GPU.
/// Once uploaded, this mesh can only be rendered using a `Painter` whose 
/// associated style supports the given vertex and material type (`<V, M>`).
/// In addition, the painter must be setup for this mesh's primitive type.
#[derive(Clone)]
pub struct MeshSource<V, M> {
    /// Vertices
    pub verts: Vec<V>,
    /// Indexing scheme
    pub inds: Indexing,
    /// Primitive type
    pub prim: Primitive,
    /// Material/texture data
    pub mat: M,
}

/// A reference to a GPU mesh object that can be drawn by a `Painter` supporting the
/// associated resource, vertex, material, and primitive combination. The memory cost
/// of this object is negligible, since  (unlike `MeshSource`) it is a cloneable 
/// reference to GPU allocated resources.
#[derive(Clone)]
pub struct Mesh<R: Resources, T: Vertex, M> {
    /// Reference to slice object (index buffer or range)
    pub slice: Slice<R>,
    /// Reference to VBO
    pub buf: Buffer<R, T>,
    /// Primitive type
    pub prim: Primitive,
    /// Material/texture data
    pub mat: M,
}

impl<R: Resources, T: Vertex, M> Mesh<R, T, M> {
    /// Set the material of this mesh (usually just textures)
    pub fn with_material<N>(self, mat: N) -> Mesh<R, T, N> {
        Mesh {
            slice: self.slice,
            buf: self.buf,
            prim: self.prim,
            mat: mat,
        }
    }
}

impl<T: Vertex, M> MeshSource<T, M> {
    /// Upload this mesh to the GPU.
    pub fn upload<R: Resources, F: FactoryExt<R>>(self, f: &mut F) -> Mesh<R, T, M> {
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

    /// Set the material of this mesh (usually just textures)
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
    /// Adds the given normal vector to each vertex's attributes
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
    /// Adds the given color to each vertex's attributes
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
    /// Adds the given texture or UV coordinates to each vertex's attributes
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

        let f = delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y;
        if f <= EPSILON { return }

        let tan = (Vector3::new(
            delta_uv2.y * edge1.x - delta_uv1.y * edge2.x,
            delta_uv2.y * edge1.y - delta_uv1.y * edge2.y,
            delta_uv2.y * edge1.z - delta_uv1.y * edge2.z,
        ) / f).normalize();

        let bitan = (Vector3::new(
            -delta_uv2.x * edge1.x + delta_uv1.x * edge2.x,
            -delta_uv2.x * edge1.y + delta_uv1.x * edge2.y,
            -delta_uv2.x * edge1.z + delta_uv1.x * edge2.z,
        ) / f).normalize();

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

#[test]
fn compute_tan() {
    use nalgebra::Vector3;
    let dag = 1. / (2f32).sqrt();
    
    let mesh = MeshSource {
        verts: vec![
            VertNT { pos: [0., 0., 0.], norm: [0., 1., 0.], tex: [0., 0.] },
            VertNT { pos: [1., 0., 0.], norm: [0., 1., 0.], tex: [1., 0.] },
            VertNT { pos: [1., 0., 1.], norm: [0., dag, -dag], tex: [1., 1.] },
            VertNT { pos: [0., 0., 1.], norm: [0., dag, -dag], tex: [0., 1.] },
            VertNT { pos: [0., 1., 1.], norm: [0., 0., -1.], tex: [0., 2.] },
            VertNT { pos: [1., 1., 1.], norm: [0., 0., -1.], tex: [1., 2.] },
        ],
        inds: Indexing::All,
        prim: Primitive::TriangleStrip,
        mat: (),
    }.compute_tan();
    for v in &mesh.verts[0..2] {
        relative_eq!(*v.tan(), Vector3::new(1., 0., 0.));
        relative_eq!(*v.bitan(), Vector3::new(0., 0., 1.));
    }
    for v in &mesh.verts[2..4] {
        relative_eq!(*v.tan(), Vector3::new(1., 0., 0.));
        relative_eq!(*v.bitan(), Vector3::new(0., dag, dag));
    }
    for v in &mesh.verts[4..6] {
        relative_eq!(*v.tan(), Vector3::new(1., 0., 0.));
        relative_eq!(*v.bitan(), Vector3::new(0., 1., 0.));
    }
}
