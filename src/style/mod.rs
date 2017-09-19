use gfx::{Resources, Encoder, Primitive, Rect, CommandBuffer, Slice, ShaderSet, Factory};
use gfx::handle::Buffer;
use gfx::traits::FactoryExt;
use gfx::state::Rasterizer;
use cgmath::Matrix4;
use fnv::FnvHashMap;
use std::cell::RefCell;

use object::Object;
use defines::{TargetRef, DepthRef, TransformBlock, Vertex};

mod solid;
pub use self::solid::*;

mod unishade;
pub use self::unishade::*;

#[derive(Copy, Clone)]
pub struct EyeContext {
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
    pub xoffset: f32,
    pub clip: Rect,
}

pub struct DrawContext<R: Resources, C: CommandBuffer<R>> {
    pub encoder: Encoder<R, C>,
    pub color: TargetRef<R>,
    pub depth: DepthRef<R>,
    pub left: EyeContext,
    pub right: EyeContext,
}

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
        obj: &Object<R, E::Vertex>,
    )
        where C: CommandBuffer<R>
    {
        if let Some(ref sty) = self.map.get(&obj.prim) {
            let mut inputs = self.inputs.borrow_mut();
            let mut trans = TransformBlock {
                model: model.into(),
                view: ctx.left.view.into(),
                proj: ctx.left.proj.into(),
                xoffset: ctx.left.xoffset,
            };
            ctx.encoder.update_constant_buffer(inputs.transform_buffer(), &trans);
            sty.draw_raw(
                &mut *inputs,
                &mut ctx.encoder,
                ctx.color.clone(),
                ctx.depth.clone(),
                ctx.left.clip,
                &obj.slice,
                obj.buf.clone(),
            );

            trans.view = ctx.right.view.into();
            trans.proj = ctx.right.proj.into();
            trans.xoffset = ctx.right.xoffset;
            ctx.encoder.update_constant_buffer(inputs.transform_buffer(), &trans);
            sty.draw_raw(
                &mut *inputs,
                &mut ctx.encoder,
                ctx.color.clone(),
                ctx.depth.clone(),
                ctx.right.clip,
                &obj.slice,
                obj.buf.clone(),
            );
        } else {
            error!("Style is not set up for \"{:?}\"", obj.prim);
        }
    }

    pub fn cfg<F: FnOnce(&mut E::Inputs)>(&self, f: F) { 
        f(&mut *self.inputs.borrow_mut())
    }
}

pub trait Style<R: Resources> {
    type Vertex: Vertex;
    type Inputs: StyleInputs<R>;
    
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
    )
        where C: CommandBuffer<R>;
}

pub trait StyleInputs<R: Resources> {
    fn transform_buffer(&self) -> &Buffer<R, TransformBlock>;
    fn shader_set(&self) -> &ShaderSet<R>;
}