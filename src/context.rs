use gfx::{Rect, Encoder, Resources, CommandBuffer};
use cgmath::Matrix4;

use ::{DepthRef, TargetRef};

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