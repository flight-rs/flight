use gfx::{Resources, Encoder, Primitive, Rect, CommandBuffer, Slice, ShaderSet, Factory};
use gfx::handle::Buffer;
use gfx::traits::FactoryExt;
use gfx::state::Rasterizer;
use cgmath::Matrix4;
use fnv::FnvHashMap;
use std::cell::RefCell;

use ::{TransformBlock, DepthRef, TargetRef, Error};
use ::context::*;
use ::mesh::{Mesh, Vertex};

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

fn get_eye(m: &Matrix4<f32>) -> [f32; 4] {
    [-m.w.x, -m.w.y, -m.w.z, 1.]
}

impl<R: Resources, E: Style<R>> Styler<R, E> {
    pub fn new<F: Factory<R> + FactoryExt<R>>(f: &mut F) -> Result<Styler<R, E>, Error> {
        Ok(Styler {
            inputs: RefCell::new(E::init(f)?),
            map: Default::default(),
        })
    }

    pub fn setup<F: Factory<R> + FactoryExt<R>>(&mut self, f: &mut F, prim: Primitive) -> Result<(), Error> {
        let mut inputs = self.inputs.borrow_mut();
        use ::std::collections::hash_map::Entry::*;
        match self.map.entry(prim) {
            Vacant(e) => {
                e.insert(E::new(f, &mut *inputs, prim, Rasterizer::new_fill())?);
            },
            _ => (),
        }
        Ok(())
    }

    pub fn try_draw<C>(
        &self,
        ctx: &mut DrawContext<R, C>,
        model: Matrix4<f32>,
        mesh: &Mesh<R, E::Vertex, E::Material>,
    )
        -> Result<(), Error>
        where C: CommandBuffer<R>
    {
        if let Some(ref sty) = self.map.get(&mesh.prim) {
            let mut inputs = self.inputs.borrow_mut();
            let mut trans = TransformBlock {
                model: model.into(),
                view: ctx.left.view.into(),
                proj: ctx.left.proj.into(),
                eye: get_eye(&ctx.left.view),
                clip_offset: ctx.left.clip_offset,
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
            )?;

            trans.view = ctx.right.view.into();
            trans.proj = ctx.right.proj.into();
            trans.eye = get_eye(&ctx.right.view);
            trans.clip_offset = ctx.right.clip_offset;
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
            )?;

            Ok(())
        } else {
            Err(
                Error::invalid_primitive(mesh.prim)
                .context("setup has not been done for this primitive type".to_owned())
            )
        }
    }

    pub fn draw<C>(
        &self,
        ctx: &mut DrawContext<R, C>,
        model: Matrix4<f32>,
        mesh: &Mesh<R, E::Vertex, E::Material>,
    )
        where C: CommandBuffer<R>
    {
        if let Err(e) = self.try_draw(ctx, model, mesh) {
            error!("{}", e);
        }
    }

    pub fn cfg<F: FnOnce(&mut E::Inputs)>(&self, f: F) { 
        f(&mut *self.inputs.borrow_mut())
    }
}

pub trait Style<R: Resources>: Sized {
    type Vertex: Vertex;
    type Inputs: StyleInputs<R>;
    type Material;
    
    fn new<F: Factory<R> + FactoryExt<R>>(
        &mut F,
        &mut Self::Inputs,
        Primitive,
        Rasterizer,
    ) -> Result<Self, Error>;

    fn init<F: Factory<R> + FactoryExt<R>>(
        &mut F,
    ) -> Result<Self::Inputs, Error>;

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
        -> Result<(), Error>
        where C: CommandBuffer<R>;
}

pub trait StyleInputs<R: Resources> {
    fn transform(&mut self, block: TransformBlock);
    fn shader_set(&self) -> &ShaderSet<R>;
}