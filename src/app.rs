use std::path::Path;
use gfx::{self, Factory, Encoder, Rect, PipelineState};
use gfx::traits::FactoryExt;
use gfx::handle::Buffer;
use webvr::{VRDisplay, VRFrameData, VRPose, VRGamepadPtr};
use defines::*;
use shaders;
use cgmath::prelude::*;
use cgmath::*;
use object::*;
use load::*;

pub const NEAR_PLANE: f64 = 0.1;
pub const FAR_PLANE: f64 = 1000.;

pub struct App<R: gfx::Resources> {
    color: TargetRef<R>,
    depth: DepthRef<R>,
    transform: Buffer<R, TransformBlock>,
    unishade: Buffer<R, UnishadeBlock>,
    solid_lines_pso: PipelineState<R, solid::Meta>,
    unishade_tris_pso: PipelineState<R, unishade::Meta>,
    grid: Object<R, VertC>,
    controller_grid: Object<R, VertC>,
    controller: Object<R, VertN>,
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

fn grid_lines(count: u32, size: f32) -> ObjectSource<VertC> {
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
            lines.push(VertC { pos: [-rad, a, b], color: line_color[0] });
            lines.push(VertC { pos: [rad, a, b], color: line_color[0] });
            lines.push(VertC { pos: [a, -rad, b], color: line_color[1] });
            lines.push(VertC { pos: [a, rad, b], color: line_color[1] });
            lines.push(VertC { pos: [a, b, -rad], color: line_color[2] });
            lines.push(VertC { pos: [a, b, rad], color: line_color[2] });
        }
    }
    ObjectSource {
        verts: lines,
        inds: Indexing::All,
        prim: Primitive::LineList,
    }
}

impl<R: gfx::Resources> App<R> {
    pub fn new<F: Factory<R> + FactoryExt<R>>(target: TargetRef<R>, factory: &mut F) -> Self {
        // Load solid shader (shaders/transform.v.glsl + shaders/simple.f.glsl)
        let solid_shader = shaders::simple(factory).unwrap();

        // Load unishade shader (shaders/transform.v.glsl + shaders/unishade.f.glsl)
        let unishade_shader = shaders::unishade(factory).unwrap();

        // Setup pipline state objects
        let solid_lines_pso = factory.create_pipeline_state(
            &solid_shader,
            Primitive::LineList,
            gfx::state::Rasterizer::new_fill(),
            solid::new()
        ).unwrap();
        let unishade_tris_pso = factory.create_pipeline_state(
            &unishade_shader,
            Primitive::TriangleList,
            gfx::state::Rasterizer::new_fill(),
            unishade::new()
        ).unwrap();

        // Create depth buffer
        let (w, h, ..) = target.get_dimensions();
        let (.., depth) = factory.create_depth_stencil(w, h).unwrap();

        // Construct App
        App {
            color: target,
            depth: depth,
            transform: factory.create_constant_buffer(1),
            unishade: factory.create_constant_buffer(1),
            solid_lines_pso: solid_lines_pso,
            unishade_tris_pso: unishade_tris_pso,
            grid: grid_lines(8, 10.).build(factory),
            controller_grid: grid_lines(2, 0.2).build(factory),
            controller: load_wavefront(
                &::wavefront::Obj::load(&Path::new("controller.obj")).unwrap()
            ).build(factory),
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
        enc.clear_depth(&self.depth, FAR_PLANE as f32);
        enc.clear(&self.color, [0.529, 0.808, 0.980, 1.0]);

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
        let mut solid_data = solid::Data { 
            verts: self.app.grid.buf.clone(),
            color: self.app.color.clone(),
            depth: self.app.depth.clone(),
            scissor: scissor, 
            transform: self.app.transform.clone(),
        };

        self.encoder.update_constant_buffer(&self.app.transform, &TransformBlock {
            model: self.stage.into(),
            view: view.into(),
            proj: proj.into(),
            xoffset: offset,
        });
        self.encoder.draw(&self.app.grid.slice, &self.app.solid_lines_pso, &solid_data);

        solid_data.verts = self.app.controller_grid.buf.clone();
        let unishade_data = unishade::Data { 
            verts: self.app.controller.buf.clone(),
            color: self.app.color.clone(),
            depth: self.app.depth.clone(),
            scissor: scissor, 
            transform: self.app.transform.clone(),
            shade: self.app.unishade.clone(),
        };
        self.encoder.update_constant_buffer(&self.app.unishade, &UnishadeBlock {
            light:  [0.467, 0.533, 0.600, 1.0],
            dark: [0.184, 0.310, 0.310, 1.0],
        });
        for cont in &self.controllers {
            self.encoder.update_constant_buffer(&self.app.transform, &TransformBlock {
                model: cont.pose.into(),
                view: view.into(),
                proj: proj.into(),
                xoffset: offset,
            });
            self.encoder.draw(
                &self.app.controller_grid.slice,
                &self.app.solid_lines_pso,
                &solid_data);
            self.encoder.draw(
                &self.app.controller.slice,
                &self.app.unishade_tris_pso,
                &unishade_data);
        }
    }
}