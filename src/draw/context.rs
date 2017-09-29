use gfx::{Rect, Encoder, Resources, CommandBuffer};
use nalgebra::{self as na, Transform3, Point3};

use ::{DepthRef, TargetRef};

/// Parameters that control the rendering of an eye
#[derive(Copy, Clone)]
pub struct EyeParams {
    pub eye: Point3<f32>,
    pub view: Transform3<f32>,
    pub proj: Transform3<f32>,
    pub clip_offset: f32,
    pub clip: Rect,
}

impl Default for EyeParams {
    fn default() -> EyeParams {
        EyeParams {
            eye: Point3::origin(),
            view: na::one(),
            proj: na::one(),
            clip_offset: 0.,
            clip: Rect { x: 0, y: 0, w: 0, h: 0 },
        }
    }
}

/// Parameters to the draw system
pub struct DrawParams<R: Resources, C: CommandBuffer<R>> {
    /// The gfx command encoder
    pub encoder: Encoder<R, C>,
    /// The color draw target
    pub color: TargetRef<R>,
    /// The depth draw target
    pub depth: DepthRef<R>,
    /// Left eye parameters
    pub left: EyeParams,
    /// Right eye parameters
    pub right: EyeParams,
}