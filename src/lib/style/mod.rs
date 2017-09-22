use gfx::{Resources, Encoder, Primitive, Rect, CommandBuffer, Slice, ShaderSet, Factory};
use gfx::handle::Buffer;
use gfx::traits::FactoryExt;
use gfx::state::Rasterizer;
use cgmath::Matrix4;
use fnv::FnvHashMap;
use std::cell::RefCell;

use lib::{TransformBlock, DepthRef, TargetRef};
use lib::context::*;
use lib::mesh::{Mesh, Vertex};

#[macro_use]
mod shaders;

mod solid;
pub use self::solid::{SolidStyle, SolidInputs};

mod unishade;
pub use self::unishade::{UnishadeStyle, UnishadeBlock, UnishadeInputs};

mod pbr;
pub use self::pbr::{PbrStyle, PbrBlock, PbrMaterial, PbrInputs};

pub struct Styler<R: Resources, E: Style<R>> {
    inputs: RefCell<E::Inputs>,
    map: FnvHashMap<Primitive, E>,
}

impl<R: Resources, E: Style<R>> Styler<R, E> {
    pub fn new<F: Factory<R> + FactoryExt<R>>(f: &mut F) -> Styler<R, E> {
        Styler {
            inputs: RefCell::new(E::init(f)),
            map: Default::default(),
        }
    }

    pub fn setup<F: Factory<R> + FactoryExt<R>>(&mut self, f: &mut F, prim: Primitive) {
        let mut inputs = self.inputs.borrow_mut();
        self.map.entry(prim).or_insert_with(move ||
            E::new(f, &mut *inputs, prim, Rasterizer::new_fill())
        );
    }

    pub fn draw<C>(
        &self,
        ctx: &mut DrawContext<R, C>,
        model: Matrix4<f32>,
        mesh: &Mesh<R, E::Vertex, E::Material>,
    )
        where C: CommandBuffer<R>
    {
        if let Some(ref sty) = self.map.get(&mesh.prim) {
            let mut inputs = self.inputs.borrow_mut();
            let mut trans = TransformBlock {
                model: model.into(),
                view: ctx.left.view.into(),
                proj: ctx.left.proj.into(),
                eye: (-ctx.left.view.w).into(),
                xoffset: ctx.left.xoffset,
            };
            inputs.transform(trans.clone());
            sty.draw_raw(
                &mut *inputs,
                &mut ctx.encoder,
                ctx.color.clone(),
                ctx.depth.clone(),
                ctx.left.clip,
                &mesh.slice,
                mesh.buf.clone(),
                &mesh.mat,
            );

            trans.view = ctx.right.view.into();
            trans.proj = ctx.right.proj.into();
            trans.eye = (-ctx.right.view.w).into();
            trans.xoffset = ctx.right.xoffset;
            inputs.transform(trans);
            sty.draw_raw(
                &mut *inputs,
                &mut ctx.encoder,
                ctx.color.clone(),
                ctx.depth.clone(),
                ctx.right.clip,
                &mesh.slice,
                mesh.buf.clone(),
                &mesh.mat,
            );
        } else {
            error!("Style is not set up for \"{:?}\"", mesh.prim);
        }
    }

    pub fn cfg<F: FnOnce(&mut E::Inputs)>(&self, f: F) { 
        f(&mut *self.inputs.borrow_mut())
    }
}

pub trait Style<R: Resources> {
    type Vertex: Vertex;
    type Inputs: StyleInputs<R>;
    type Material;
    
    fn new<F: Factory<R> + FactoryExt<R>>(
        &mut F,
        &mut Self::Inputs,
        Primitive,
        Rasterizer,
    ) -> Self;

    fn init<F: Factory<R> + FactoryExt<R>>(
        &mut F,
    ) -> Self::Inputs;

    fn draw_raw<C>(
        &self,
        &mut Self::Inputs,
        &mut Encoder<R, C>,
        TargetRef<R>,
        DepthRef<R>,
        Rect,
        &Slice<R>,
        Buffer<R, Self::Vertex>,
        &Self::Material,
    )
        where C: CommandBuffer<R>;
}

pub trait StyleInputs<R: Resources> {
    fn transform(&mut self, block: TransformBlock);
    fn shader_set(&self) -> &ShaderSet<R>;
}