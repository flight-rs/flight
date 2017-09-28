use gfx::{Rect, Encoder, Resources, CommandBuffer};
use nalgebra::{self as na, Transform3, Point3};

use ::{DepthRef, TargetRef};

#[derive(Copy, Clone)]
pub struct EyeContext {
    pub eye: Point3<f32>,
    pub view: Transform3<f32>,
    pub proj: Transform3<f32>,
    pub clip_offset: f32,
    pub clip: Rect,
}

impl Default for EyeContext {
    fn default() -> EyeContext {
        EyeContext {
            eye: Point3::origin(),
            view: na::one(),
            proj: na::one(),
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