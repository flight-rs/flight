use std::path::Path;
use gfx::{self, Factory};
use gfx::traits::FactoryExt;
use webvr::{VRDisplayData, VRFrameData, VRPose, VRGamepadPtr};
use cgmath::prelude::*;
use cgmath::*;

use lib::{Texture, Light, PbrMesh};
use lib::mesh::*;
use lib::context::DrawContext;
use lib::load;
use lib::style::{Styler, SolidStyle, UnishadeStyle, PbrStyle, PbrMaterial};

pub const NEAR_PLANE: f64 = 0.1;
pub const FAR_PLANE: f64 = 1000.;
pub const BACKGROUND: [f32; 4] = [0.529, 0.808, 0.980, 1.0];

pub struct App<R: gfx::Resources> {
    gamepads: Vec<VRGamepadPtr>,
    solid: Styler<R, SolidStyle<R>>,
    unishade: Styler<R, UnishadeStyle<R>>,
    pbr: Styler<R, PbrStyle<R>>,
    grid: Mesh<R, VertC, ()>,
    controller_grid: Mesh<R, VertC, ()>,
    controller: PbrMesh<R>,
    teapot: PbrMesh<R>,
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

fn grid_lines(count: u32, size: f32) -> MeshSource<VertC, ()> {
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
    MeshSource {
        verts: lines,
        inds: Indexing::All,
        prim: Primitive::LineList,
        mat: (),
    }
}

fn load_my_simple_object<P, R, F>(f: &mut F, path: P) -> Mesh<R, VertNTT, PbrMaterial<R>> 
    where P: AsRef<Path>, R: gfx::Resources, F: gfx::Factory<R>
{
    use gfx::format::*;
    load::load_wavefront(
        &::wavefront::Obj::load(path.as_ref()).unwrap()
    ).compute_tan().with_material(PbrMaterial {
        normal: Texture::<_, (R8_G8_B8_A8, Unorm)>::uniform_value(f, [0x80, 0x80, 0xFF, 0xFF]),
        albedo: Texture::<_, (R8_G8_B8_A8, Srgb)>::uniform_value(f, [0xA0, 0xA0, 0xA0, 0xFF]),
        metalness: Texture::<_, (R8, Unorm)>::uniform_value(f, 0x00),
        roughness: Texture::<_, (R8, Unorm)>::uniform_value(f, 0x20),
    }).build(f)
}

impl<R: gfx::Resources> App<R> {
    pub fn new<F: Factory<R> + FactoryExt<R>>(factory: &mut F) -> Self {
        // Setup stylers
        let mut solid = Styler::new(factory);
        solid.setup(factory, Primitive::LineList);
        solid.setup(factory, Primitive::TriangleList);

        let mut unishade: Styler<_, UnishadeStyle<_>> = Styler::new(factory);
        unishade.setup(factory, Primitive::LineList);
        unishade.setup(factory, Primitive::TriangleList);
        unishade.cfg(|s| s.colors([0.184, 0.310, 0.310, 1.0], [0.467, 0.533, 0.600, 1.0]));

        let mut pbr: Styler<_, PbrStyle<_>> = Styler::new(factory);
        pbr.setup(factory, Primitive::TriangleList);
        pbr.cfg(|s| {
            s.ambient(BACKGROUND);
            s.lights(&[
                Light {
                    pos: [4., 0., 0., 1.],
                    color: [0.8, 0.2, 0.2, 100.],
                },
                Light {
                    pos: [0., 4., 0., 1.],
                    color: [0.2, 0.8, 0.2, 100.],
                },
                Light {
                    pos: [0., 0., 4., 1.],
                    color: [0.2, 0.2, 0.8, 100.],
                },
            ]);
        });

        // Construct App
        App {
            gamepads: vec![],
            solid: solid,
            unishade: unishade,
            pbr: pbr,
            grid: grid_lines(8, 10.).build(factory),
            controller_grid: grid_lines(2, 0.2).build(factory),
            controller: load_my_simple_object(factory, "assets/controller.obj"),
            teapot: load::load_object(factory, "assets/teapot_wood/")
        }
    }

    pub fn set_gamepads(&mut self, g: Vec<VRGamepadPtr>) {
        self.gamepads = g;
    }

    pub fn draw<C: gfx::CommandBuffer<R>>(
        &self,
        ctx: &mut DrawContext<R, C>,
        display: &VRDisplayData,
        frame: &VRFrameData,
    ) {
        // Get stage transform thing
        let stage = if let Some(ref stage) = display.stage_parameters {
            <&Matrix4<f32>>::from(&stage.sitting_to_standing_transform).inverse_transform().unwrap()
        } else {
            Matrix4::identity()
        };

        // Clear targets
        ctx.encoder.clear_depth(&ctx.depth, FAR_PLANE as f32);
        ctx.encoder.clear(&ctx.color, [BACKGROUND[0].powf(1. / 2.2), BACKGROUND[1].powf(1. / 2.2), BACKGROUND[2].powf(1. / 2.2), BACKGROUND[3]]);

        // Draw grid
        self.solid.draw(ctx, stage, &self.grid);

        // Draw teapot
        self.pbr.draw(ctx, stage * Matrix4::from(Decomposed {	
            scale: 1.,		
            rot: Quaternion::from(Euler::new(Deg(0.), Deg(0.), Deg(0.))),		
            disp: Vector3::new(3., 1., 3.),		
        }), &self.teapot);

        // Draw controllers
        let controllers = self.gamepads.iter().filter_map(|g| Controller::from_gp(g));
        for cont in controllers {
            self.solid.draw(ctx, cont.pose, &self.controller_grid);
            self.pbr.draw(ctx, cont.pose, &self.controller);
        }
    }
}

pub struct Controller {
    pose: Matrix4<f32>,
}

impl Controller {
    pub fn from_gp(gp: &VRGamepadPtr) -> Option<Self> {
        let gp = gp.borrow();
        let state = gp.state();
        Some(Controller {
            pose: match pose_transform(&state.pose) { Some(p) => p, None => return None },
        })
    }
}