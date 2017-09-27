use gfx::{Rect, Encoder, Resources, CommandBuffer};
use cgmath::{Matrix4, SquareMatrix};

use ::{DepthRef, TargetRef};

#[derive(Copy, Clone)]
pub struct EyeContext {
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
    pub clip_offset: f32,
    pub clip: Rect,
}

impl Default for EyeContext {
    fn default() -> EyeContext {
        EyeContext {
            view: Matrix4::identity(),
            proj: Matrix4::identity(),
            clip_offset: 0.,
            clip: Rect { x: 0, y: 0, w: 0, h: 0 },
        }
    }
}

pub struct DrawContext<R: Resources, C: CommandBuffer<R>> {
    pub encoder: Encoder<R, C>,
    pub color: TargetRef<R>,
    pub depth: DepthRef<R>,
    pub left: EyeContext,
    pub right: EyeContext,
}