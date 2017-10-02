use std::path::Path;
use std::time::Instant;
use gfx::{self, Factory};
use gfx::traits::FactoryExt;
use nalgebra::{self as na, Rotation3, SimilarityMatrix3, Translation3, Point3, Point2, Vector3};

use lib::{Texture, Light, PbrMesh, Error};
use lib::mesh::*;
use lib::load;
use lib::draw::{DrawParams, Painter, SolidStyle, PbrStyle, PbrMaterial};
use lib::vr::{primary, secondary, VrMoment, ViveController};

pub const NEAR_PLANE: f64 = 0.1;
pub const FAR_PLANE: f64 = 1000.;
pub const BACKGROUND: [f32; 4] = [0.529, 0.808, 0.980, 1.0];
const PI: f32 = ::std::f32::consts::PI;
const PI2: f32 = 2. * PI;
const DEG: f32 = PI2 / 360.;

pub struct App<R: gfx::Resources> {
    solid: Painter<R, SolidStyle<R>>,
    pbr: Painter<R, PbrStyle<R>>,
    grid: Mesh<R, VertC, ()>,
    controller_grid: Mesh<R, VertC, ()>,
    controller: PbrMesh<R>,
    teapot: PbrMesh<R>,
    start_time: Instant,
    primary: ViveController,
    secondary: ViveController,
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
    }).upload(f))
}

impl<R: gfx::Resources> App<R> {
    pub fn new<F: Factory<R> + FactoryExt<R>>(factory: &mut F) -> Result<Self, Error> {
        // Setup Painters
        let mut solid = Painter::new(factory)?;
        solid.setup(factory, Primitive::LineList)?;
        solid.setup(factory, Primitive::TriangleList)?;

        let mut pbr: Painter<_, PbrStyle<_>> = Painter::new(factory)?;
        pbr.setup(factory, Primitive::TriangleList)?;

        // Construct App
        Ok(App {
            solid: solid,
            pbr: pbr,
            grid: grid_lines(8, 8.).upload(factory),
            controller_grid: grid_lines(2, 0.2).upload(factory),
            controller: load_my_simple_object(factory, "assets/controller.obj", [0x80, 0x80, 0xFF, 0xFF])?,
            teapot: load::object_directory(factory, "assets/teapot_wood/")?,
            start_time: Instant::now(),
            primary: ViveController {
                is: primary(),
                pad: Point2::new(0., 1.),
                .. Default::default()
            },
            secondary: ViveController {
                is: secondary(),
                .. Default::default()
            },
        })
    }

    pub fn draw<C: gfx::CommandBuffer<R>>(
        &mut self,
        ctx: &mut DrawParams<R, C>,
        vrm: &VrMoment,
    ) {
        let elapsed = self.start_time.elapsed();
        let t = elapsed.as_secs() as f32 + (elapsed.subsec_nanos() as f32 * 1e-9);

        match (self.primary.update(vrm), self.secondary.update(vrm)) {
            (Ok(_), Ok(_)) => (),
            _ => warn!("A not vive-like controller is connected"),
        }
        

        // Clear targets
        ctx.encoder.clear_depth(&ctx.depth, FAR_PLANE as f32);
        ctx.encoder.clear(&ctx.color, [BACKGROUND[0].powf(1. / 2.2), BACKGROUND[1].powf(1. / 2.2), BACKGROUND[2].powf(1. / 2.2), BACKGROUND[3]]);

        // Controller light
        let cont_light = if self.secondary.connected {
            Light {
                pos: self.secondary.pose * Point3::new(0., 0., -0.1),
                color: [0.6, 0.6, 0.6, 10. * self.secondary.trigger as f32],
            }
        } else {
            Default::default()
        };

        // Config PBR lights
        self.pbr.cfg(|s| {
            s.ambient(BACKGROUND);
            s.lights(&[
                Light {
                    pos: vrm.stage * Point3::new(4., 0., 0.),
                    color: [0.8, 0.2, 0.2, 100.],
                },
                Light {
                    pos: vrm.stage * Point3::new(0., 4., 0.),
                    color: [0.2, 0.8, 0.2, 100.],
                },
                Light {
                    pos: vrm.stage * Point3::new(0., 0., 4.),
                    color: [0.2, 0.2, 0.8, 100.],
                },
                cont_light,
            ]);
        });

        // Draw grid
        self.solid.draw(ctx, vrm.stage, &self.grid);

        // Draw teapot
        let tearot =
            Rotation3::from_axis_angle(&Vector3::x_axis(), (t * 0.7).sin() * 10. * DEG)
            * Rotation3::from_axis_angle(&Vector3::z_axis(), (t * 0.8).cos() * 15. * DEG)
            * Rotation3::from_axis_angle(&Vector3::y_axis(), t * 60. * DEG);

        let teamat = if self.primary.connected {
            na::convert(self.primary.pose * SimilarityMatrix3::from_parts(
                Translation3::new(0., 0., -0.25),
                tearot,
                0.15 * self.primary.pad_theta().abs() as f32 / PI,		
            ))
        } else {
            vrm.stage * SimilarityMatrix3::from_parts(
                Translation3::new(1., 0., 1.),
                tearot,
                1.,
            )
        };
        self.pbr.draw(ctx, teamat, &self.teapot);

        // Draw controllers
        for cont in vrm.controllers() {
            self.solid.draw(ctx, na::convert(cont.pose), &self.controller_grid);
            self.pbr.draw(ctx, na::convert(cont.pose), &self.controller);
        }
    }
}