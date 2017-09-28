use std::path::Path;
use std::time::Instant;
use gfx::{self, Factory};
use gfx::traits::FactoryExt;
use cgmath::*;

use lib::{Texture, Light, PbrMesh, Error};
use lib::mesh::*;
use lib::context::DrawContext;
use lib::load;
use lib::style::{Styler, SolidStyle, UnishadeStyle, PbrStyle, PbrMaterial};
use lib::vr::{primary, secondary,VrMoment};

pub const NEAR_PLANE: f64 = 0.1;
pub const FAR_PLANE: f64 = 1000.;
pub const BACKGROUND: [f32; 4] = [0.529, 0.808, 0.980, 1.0];
const PI: f32 = ::std::f32::consts::PI;

pub struct App<R: gfx::Resources> {
    solid: Styler<R, SolidStyle<R>>,
    unishade: Styler<R, UnishadeStyle<R>>,
    pbr: Styler<R, PbrStyle<R>>,
    grid: Mesh<R, VertC, ()>,
    controller_grid: Mesh<R, VertC, ()>,
    controller: PbrMesh<R>,
    teapot: PbrMesh<R>,
    start_time: Instant,
    last_pad: (f32, f32),
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

    let frac = 0.125 * mult;
    lines.push(VertC { pos: [rad - frac, 0., -frac], color: [1., 0., 0.] });
    lines.push(VertC { pos: [rad - frac, 0.,  frac], color: [1., 0., 0.] });
    lines.push(VertC { pos: [rad - frac, -frac, 0.], color: [1., 0., 0.] });
    lines.push(VertC { pos: [rad - frac,  frac, 0.], color: [1., 0., 0.] });
    lines.push(VertC { pos: [0., rad - frac, -frac], color: [0., 1., 0.] });
    lines.push(VertC { pos: [0., rad - frac,  frac], color: [0., 1., 0.] });
    lines.push(VertC { pos: [-frac, rad - frac, 0.], color: [0., 1., 0.] });
    lines.push(VertC { pos: [ frac, rad - frac, 0.], color: [0., 1., 0.] });
    lines.push(VertC { pos: [-frac, 0., rad - frac], color: [0., 0., 1.] });
    lines.push(VertC { pos: [ frac, 0., rad - frac], color: [0., 0., 1.] });
    lines.push(VertC { pos: [0., -frac, rad - frac], color: [0., 0., 1.] });
    lines.push(VertC { pos: [0.,  frac, rad - frac], color: [0., 0., 1.] });

    MeshSource {
        verts: lines,
        inds: Indexing::All,
        prim: Primitive::LineList,
        mat: (),
    }
}

fn load_my_simple_object<P, R, F>(f: &mut F, path: P, albedo: [u8; 4])
    -> Result<Mesh<R, VertNTT, PbrMaterial<R>>, Error>
    where P: AsRef<Path>, R: gfx::Resources, F: gfx::Factory<R>
{
    use gfx::format::*;
    Ok(load::wavefront_file(path)?.compute_tan().with_material(PbrMaterial {
        normal: Texture::<_, (R8_G8_B8_A8, Unorm)>::uniform_value(f, albedo)?,
        albedo: Texture::<_, (R8_G8_B8_A8, Srgb)>::uniform_value(f, [0x60, 0x60, 0x60, 0xFF])?,
        metalness: Texture::<_, (R8, Unorm)>::uniform_value(f, 0x00)?,
        roughness: Texture::<_, (R8, Unorm)>::uniform_value(f, 0x20)?,
    }).build(f))
}

impl<R: gfx::Resources> App<R> {
    pub fn new<F: Factory<R> + FactoryExt<R>>(factory: &mut F) -> Result<Self, Error> {
        // Setup stylers
        let mut solid = Styler::new(factory)?;
        solid.setup(factory, Primitive::LineList)?;
        solid.setup(factory, Primitive::TriangleList)?;

        let mut unishade: Styler<_, UnishadeStyle<_>> = Styler::new(factory)?;
        unishade.setup(factory, Primitive::LineList)?;
        unishade.setup(factory, Primitive::TriangleList)?;
        unishade.cfg(|s| s.colors([0.184, 0.310, 0.310, 1.0], [0.467, 0.533, 0.600, 1.0]));

        let mut pbr: Styler<_, PbrStyle<_>> = Styler::new(factory)?;
        pbr.setup(factory, Primitive::TriangleList)?;

        // Construct App
        Ok(App {
            solid: solid,
            unishade: unishade,
            pbr: pbr,
            grid: grid_lines(8, 8.).build(factory),
            controller_grid: grid_lines(2, 0.2).build(factory),
            controller: load_my_simple_object(factory, "assets/controller.obj", [0x80, 0x80, 0xFF, 0xFF])?,
            teapot: load::object_directory(factory, "assets/teapot_wood/")?,
            start_time: Instant::now(),
            last_pad: (1., 0.),
        })
    }

    pub fn draw<C: gfx::CommandBuffer<R>>(
        &mut self,
        ctx: &mut DrawContext<R, C>,
        vrm: &VrMoment,
    ) {
        let elapsed = self.start_time.elapsed();
        let t = elapsed.as_secs() as f32 + (elapsed.subsec_nanos() as f32 * 1e-9);

        // Clear targets
        ctx.encoder.clear_depth(&ctx.depth, FAR_PLANE as f32);
        ctx.encoder.clear(&ctx.color, [BACKGROUND[0].powf(1. / 2.2), BACKGROUND[1].powf(1. / 2.2), BACKGROUND[2].powf(1. / 2.2), BACKGROUND[3]]);

        // Controller light
        let cont_light = if let Some(cont) = vrm.controller(secondary()) {
            Light {
                pos: (cont.pose * Vector4::new(0., 0., -0.1, 1.)).into(),
                color: [0.6, 0.6, 0.6, 10. * cont.axes[2] as f32],
            }
        } else {
            Default::default()
        };

        // Config PBR lights
        self.pbr.cfg(|s| {
            s.ambient(BACKGROUND);
            s.lights(&[
                Light {
                    pos: (vrm.stage * Vector4::new(4., 0., 0., 1.)).into(),
                    color: [0.8, 0.2, 0.2, 100.],
                },
                Light {
                    pos: (vrm.stage * Vector4::new(0., 4., 0., 1.)).into(),
                    color: [0.2, 0.8, 0.2, 100.],
                },
                Light {
                    pos: (vrm.stage * Vector4::new(0., 0., 4., 1.)).into(),
                    color: [0.2, 0.2, 0.8, 100.],
                },
                cont_light,
            ]);
        });
        
        // Draw grid
        self.solid.draw(ctx, vrm.stage, &self.grid);

        // Draw teapot
        let tearot = Quaternion::from(Euler::new(Deg((t * 0.7).sin() * 15.), Deg(t * 60.), Deg((t * 0.8).cos() * 15.)));
        let mat = if let Some(cont) = vrm.controller(primary()) {
            if cont.axes[0] != 0. {
                self.last_pad = (cont.axes[0] as f32, cont.axes[1] as f32);
            }
            cont.pose * Matrix4::from(Decomposed {
                scale: 0.15 * self.last_pad.0.atan2(self.last_pad.1).abs() as f32 / PI,		
                rot: tearot,		
                disp: Vector3::new(0., 0., -0.25),
            })
        } else {
            vrm.stage * Matrix4::from(Decomposed {	
                scale: 1.,		
                rot: tearot,		
                disp: Vector3::new(1., 0., 1.),		
            })
        };
        self.pbr.draw(ctx, mat, &self.teapot);

        // Draw controllers
        for cont in vrm.controllers() {
            self.solid.draw(ctx, cont.pose, &self.controller_grid);
            self.pbr.draw(ctx, cont.pose, &self.controller);
        }
    }
}