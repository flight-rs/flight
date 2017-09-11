use gfx::{self, Factory, Encoder, Rect, PipelineState, Slice};
use gfx::traits::FactoryExt;
use webvr::{VRDisplay, VRFrameData};
use defines::*;
use shaders;
use cgmath::prelude::*;
use cgmath::Matrix4;

pub const NEAR_PLANE: f64 = 0.1;
pub const FAR_PLANE: f64 = 1000.;

pub struct App<R: gfx::Resources> {
    target: TargetRef<R>,
    lines_pso: PipelineState<R, simple::Meta>,
    lines_slice: Slice<R>,
    simple_data: simple::Data<R>,
}

pub fn matrix_from(mat: &[f32; 16]) -> &Matrix4<f32> {
    <&Matrix4<f32>>::from(mat)
}

impl<R: gfx::Resources> App<R> {
    pub fn new<F: Factory<R> + FactoryExt<R>>(target: TargetRef<R>, factory: &mut F) -> Self {
        // Load simple shader (shaders/transform.v.glsl + shaders/simple.f.glsl)
        let simple_shader = shaders::simple(factory).unwrap();

        // Setup lines pipline state object
        let lines_pso = {
            let shaders = simple_shader;
            factory.create_pipeline_state(
                &shaders,
                gfx::Primitive::LineList,
                gfx::state::Rasterizer::new_fill(),
                simple::new()
            ).unwrap()
        };

        // Create depth buffer
        let (w, h, ..) = target.get_dimensions();
        let (.., depth_target) = factory.create_depth_stencil(w, h).unwrap();

        // Create grid vertex buffer
        let mut lines = Vec::new();
        let base_color = [0.2, 0.2, 0.2];
        let light_color = [0.8, 0.8, 0.8];
        for a in -4i32..5 {
            for b in -4i32..5 {
                let line_color = if a == 0 && b == 0 {
                    [[1., 0., 0.],
                     [0., 1., 0.],
                     [0., 0., 1.]] 
                } else if a % 2 == 0 && b % 2 == 0 { [base_color; 3] } else { [light_color; 3] };
                let a = a as f32;
                let b = b as f32;
                lines.push(Vert { a_pos: [-4., a, b], a_color: line_color[0] });
                lines.push(Vert { a_pos: [4., a, b], a_color: line_color[0] });
                lines.push(Vert { a_pos: [a, -4., b], a_color: line_color[1] });
                lines.push(Vert { a_pos: [a, 4., b], a_color: line_color[1] });
                lines.push(Vert { a_pos: [a, b, -4.], a_color: line_color[2] });
                lines.push(Vert { a_pos: [a, b, 4.], a_color: line_color[2] });
            }
        }
        let lines_buf = factory.create_vertex_buffer(&lines);
        let lines_slice = Slice::new_match_vertex_buffer(&lines_buf);

        // Load other objects here

        // Setup data for simple shader and lines pipeline
        let simple_data = simple::Data {
            verts: lines_buf,
            color: target.clone(),
            depth: depth_target,
            scissor: Rect { x: 0, y: 0, w: 0, h: 0 },
            transform: factory.create_constant_buffer(1),
        };

        // Construct App
        App {
            target: target,
            lines_slice: lines_slice,
            lines_pso: lines_pso,
            simple_data: simple_data,
        }
    }

    pub fn draw<C: gfx::CommandBuffer<R>>(&self, enc: &mut Encoder<R, C>, display: &VRDisplay, vr_frame: VRFrameData, left_clip: Rect, right_clip: Rect) {
        // Get stage transform thing
        let stage = if let Some(ref stage) = display.data().stage_parameters {
            matrix_from(&stage.sitting_to_standing_transform).inverse_transform().unwrap()
        } else {
            Matrix4::identity()
        };

        // Get view matricies
        let left_view = matrix_from(&vr_frame.left_view_matrix) * stage;
        let right_view = matrix_from(&vr_frame.right_view_matrix) * stage;

        // Clear targets
        enc.clear_depth(&self.simple_data.depth, FAR_PLANE as f32);
        enc.clear(&self.target, [0.529, 0.808, 0.980, 1.0]);

        // Render left eye
        enc.update_constant_buffer(&self.simple_data.transform, &TransformBlock {
            model: Matrix4::identity().into(),
            view: left_view.into(),
            proj: matrix_from(&vr_frame.left_projection_matrix).clone().into(),
            xoffset: -0.5,
        });
        enc.draw(&self.lines_slice, &self.lines_pso, &simple::Data { scissor: left_clip, .. self.simple_data.clone() });

        // Render right eye
        enc.update_constant_buffer(&self.simple_data.transform, &TransformBlock {
            model: Matrix4::identity().into(),
            view: right_view.into(),
            proj: matrix_from(&vr_frame.right_projection_matrix).clone().into(),
            xoffset: 0.5,
        });
        enc.draw(&self.lines_slice, &self.lines_pso, &simple::Data { scissor: right_clip, .. self.simple_data.clone() });
    }
}