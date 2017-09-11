use gfx::{self, Factory, Encoder, Rect, PipelineState, Slice};
use gfx::traits::FactoryExt;
use webvr::{VRDisplay, VRFrameData, VRPose, VRGamepadPtr};
use defines::*;
use shaders;
use cgmath::prelude::*;
use cgmath::*;

pub const NEAR_PLANE: f64 = 0.1;
pub const FAR_PLANE: f64 = 1000.;

pub struct App<R: gfx::Resources> {
    target: TargetRef<R>,
    depth_target: DepthRef<R>,
    grid_pso: PipelineState<R, simple::Meta>,
    grid_slice: Slice<R>,
    grid_data: simple::Data<R>,
    controller_pso: PipelineState<R, simple::Meta>,
    controller_slice: Slice<R>,
    controller_data: simple::Data<R>,
}

pub fn matrix_from(mat: &[f32; 16]) -> &Matrix4<f32> {
    <&Matrix4<f32>>::from(mat)
}

pub fn pose_transform(ctr: &VRPose) -> Option<Matrix4<f32>> {
    let or = match ctr.orientation { Some(o) => o, None => return None };
    let rot = Quaternion::new(or[3], or[0], or[1], or[2]);
    let pos = Vector3::from(match ctr.position { Some(o) => o, None => return None });
    Some(Matrix4::from(Decomposed {
        scale: 1.,
        rot: rot,
        disp: pos,
    }))
}

fn grid_lines(count: u32, size: f32) -> Vec<Vert> {
    let mut lines = Vec::new();
    let base_color = [0.2, 0.2, 0.2];
    let light_color = [0.8, 0.8, 0.8];
    let mid = count / 2;
    let rad = size / 2.;
    let mult = size / count as f32;
    for a in 0..(count + 1) {
        for b in 0..(count + 1) {
            let line_color = if a == mid && b == mid {
                [[1., 0., 0.],
                 [0., 1., 0.],
                 [0., 0., 1.]]
            } else if a % 2 == 0 && b % 2 == 0 { [base_color; 3] } else { [light_color; 3] };
            let a = a as f32 * mult - rad;
            let b = b as f32 * mult - rad;
            lines.push(Vert { a_pos: [-rad, a, b], a_color: line_color[0] });
            lines.push(Vert { a_pos: [rad, a, b], a_color: line_color[0] });
            lines.push(Vert { a_pos: [a, -rad, b], a_color: line_color[1] });
            lines.push(Vert { a_pos: [a, rad, b], a_color: line_color[1] });
            lines.push(Vert { a_pos: [a, b, -rad], a_color: line_color[2] });
            lines.push(Vert { a_pos: [a, b, rad], a_color: line_color[2] });
        }
    }
    lines
}

impl<R: gfx::Resources> App<R> {
    pub fn new<F: Factory<R> + FactoryExt<R>>(target: TargetRef<R>, factory: &mut F) -> Self {
        // Load simple shader (shaders/transform.v.glsl + shaders/simple.f.glsl)
        let simple_shader = shaders::simple(factory).unwrap();

        // Setup lines pipline state object
        let grid_pso = {
            let shaders = simple_shader.clone();
            factory.create_pipeline_state(
                &shaders,
                gfx::Primitive::LineList,
                gfx::state::Rasterizer::new_fill(),
                simple::new()
            ).unwrap()
        };

        // Setup controller pipline state object
        let controller_pso = {
            let shaders = simple_shader.clone();
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
        let grid = grid_lines(8, 10.);
        let grid_buf = factory.create_vertex_buffer(&grid);
        let grid_slice = Slice::new_match_vertex_buffer(&grid_buf);

        // controller vertex buffer
        let controller = grid_lines(2, 0.2);
        let controller_buf = factory.create_vertex_buffer(&controller);
        let controller_slice = Slice::new_match_vertex_buffer(&controller_buf);

        let transform_buf = factory.create_constant_buffer(1);

        // Setup data for lines pipeline
        let grid_data = simple::Data {
            verts: grid_buf,
            color: target.clone(),
            depth: depth_target.clone(),
            scissor: Rect { x: 0, y: 0, w: 0, h: 0 },
            transform: transform_buf.clone(),
        };

        // Setup data for controller pipeline
        let controller_data = simple::Data {
            verts: controller_buf,
            color: target.clone(),
            depth: depth_target.clone(),
            scissor: Rect { x: 0, y: 0, w: 0, h: 0 },
            transform: transform_buf.clone(),
        };

        // Construct App
        App {
            target: target,
            depth_target: depth_target,
            grid_slice: grid_slice,
            grid_pso: grid_pso,
            grid_data: grid_data,
            controller_slice: controller_slice,
            controller_pso: controller_pso,
            controller_data: controller_data,
        }
    }

    pub fn draw<C: gfx::CommandBuffer<R>>(&self, enc: &mut Encoder<R, C>, display: &VRDisplay, gamepads: Vec<VRGamepadPtr>, vr_frame: VRFrameData, left_clip: Rect, right_clip: Rect) {
        // Get stage transform thing
        let stage = if let Some(ref stage) = display.data().stage_parameters {
            matrix_from(&stage.sitting_to_standing_transform).inverse_transform().unwrap()
        } else {
            Matrix4::identity()
        };

        // Clear targets
        enc.clear_depth(&self.depth_target, FAR_PLANE as f32);
        enc.clear(&self.target, [0.529, 0.808, 0.980, 1.0]);

        // Setup frame
        let mut frame = DrawFrame {
            controllers: gamepads.into_iter().filter_map(|g| Controller::from_gp(g)).collect(),
            app: self,
            encoder: enc,
            stage: stage,
        };

        // Render left eye
        frame.draw(
            *matrix_from(&vr_frame.left_view_matrix),
            *matrix_from(&vr_frame.left_projection_matrix),
            left_clip,
            -0.5);
        // Render right eye
        frame.draw(
            *matrix_from(&vr_frame.right_view_matrix),
            *matrix_from(&vr_frame.right_projection_matrix),
            right_clip,
            0.5);
    }
}

pub struct Controller {
    pose: Matrix4<f32>,
}

impl Controller {
    pub fn from_gp(gp: VRGamepadPtr) -> Option<Self> {
        let gp = gp.borrow();
        let state = gp.state();
        Some(Controller {
            pose: match pose_transform(&state.pose) { Some(p) => p, None => return None },
        })
    }
}

pub struct DrawFrame<'a, R: gfx::Resources, C: gfx::CommandBuffer<R> + 'a> {
    app: &'a App<R>,
    encoder: &'a mut Encoder<R, C>,
    controllers: Vec<Controller>,
    stage: Matrix4<f32>,
}

impl<'a, R: gfx::Resources, C: gfx::CommandBuffer<R>> DrawFrame<'a, R, C> {
    pub fn draw(&mut self, view: Matrix4<f32>, proj: Matrix4<f32>, scissor: Rect, offset: f32) {
        self.encoder.update_constant_buffer(&self.app.grid_data.transform, &TransformBlock {
            model: self.stage.into(),
            view: view.into(),
            proj: proj.into(),
            xoffset: offset,
        });
        self.encoder.draw(&self.app.grid_slice, &self.app.grid_pso, &simple::Data { scissor: scissor, .. self.app.grid_data.clone() });

        let cont_data = simple::Data { scissor: scissor, .. self.app.controller_data.clone() };
        for cont in &self.controllers {
            self.encoder.update_constant_buffer(&self.app.controller_data.transform, &TransformBlock {
                model: cont.pose.into(),
                view: view.into(),
                proj: proj.into(),
                xoffset: offset,
            });
            self.encoder.draw(&self.app.controller_slice, &self.app.controller_pso, &cont_data);
        }
    }
}