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
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
    xoffset: f32,
    clip: Rect,
}

pub struct DrawContext<R: Resources, C: CommandBuffer<R>> {
    encoder: Encoder<R, C>,
    color: TargetRef<R>,
    depth: DepthRef<R>,
    left: EyeContext,
    right: EyeContext,
}

pub struct StyleGroup<R: Resources, E: Style<R>> {
    inputs: RefCell<E::Inputs>,
    map: FnvHashMap<Primitive, E>,
}

impl<R: Resources, E: Style<R>> StyleGroup<R, E> {
    fn new<F: Factory<R> + FactoryExt<R>>(f: &mut F) -> StyleGroup<R, E> {
        StyleGroup {
            inputs: RefCell::new(E::init(f)),
            map: Default::default(),
        }
    }

    fn draw<C>(
        &mut self,
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
            trans.proj = ctx.left.proj.into();
            trans.xoffset = ctx.left.xoffset;
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

    fn setup<F: Factory<R> + FactoryExt<R>>(&mut self, f: &mut F, prim: Primitive) {
        let mut inputs = self.inputs.borrow_mut();
        self.map.entry(prim).or_insert_with(move ||
            E::new(f, &mut *inputs, prim, Rasterizer::new_fill())
        );
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